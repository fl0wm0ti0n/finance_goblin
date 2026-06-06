//! BUG-0009 / Q0016 — Grafana provisioning SQL + JSON contract tests (DEC-0068).
//! SQL fixtures skip when DATABASE_URL is not set.

use serde_json::Value;
use sqlx::PgPool;
use std::fs;
use std::path::{Path, PathBuf};

const BREAKDOWN_SQL: &str = r#"
SELECT elem->>'name' AS name
FROM (
  SELECT payload
  FROM net_worth_snapshots
  ORDER BY snapshot_date DESC
  LIMIT 1
) latest
CROSS JOIN LATERAL jsonb_array_elements(latest.payload->'accounts') AS elem
ORDER BY ABS((elem->>'balance')::float) DESC
"#;

const ACCOUNT_VARIABLE_SQL: &str = r#"
SELECT a.firefly_id AS __value, a.name AS __text
FROM accounts a
WHERE a.type = 'asset'
ORDER BY ABS(COALESCE(a.balance, 0)) DESC, a.name ASC
"#;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn read_dashboard(relative: &str) -> Value {
    let path = repo_root().join(relative);
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {relative}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {relative}: {e}"))
}

fn panel_raw_sql(dashboard: &Value, panel_id: i64) -> Option<String> {
    dashboard["panels"]
        .as_array()?
        .iter()
        .find(|p| p["id"].as_i64() == Some(panel_id))?
        .get("targets")?
        .as_array()?
        .first()?
        .get("rawSql")?
        .as_str()
        .map(str::to_string)
}

fn templating_query(dashboard: &Value, name: &str) -> Option<String> {
    dashboard["templating"]["list"]
        .as_array()?
        .iter()
        .find(|v| v["name"].as_str() == Some(name))?
        .get("query")?
        .as_str()
        .map(str::to_string)
}

fn has_text_panel_content(dashboard: &Value, needle: &str) -> bool {
    dashboard["panels"]
        .as_array()
        .map(|panels| {
            panels.iter().any(|p| {
                p["type"].as_str() == Some("text")
                    && p["options"]["content"]
                        .as_str()
                        .is_some_and(|c| c.contains(needle))
            })
        })
        .unwrap_or(false)
}

fn ml_panel_ids() -> [i64; 4] {
    [7, 8, 9, 10]
}

fn panel_no_value(dashboard: &Value, panel_id: i64) -> Option<String> {
    dashboard["panels"]
        .as_array()?
        .iter()
        .find(|p| p["id"].as_i64() == Some(panel_id))?
        .pointer("/fieldConfig/defaults/noValue")?
        .as_str()
        .map(str::to_string)
}

#[test]
fn portfolio_breakdown_sql_uses_latest_snapshot_lateral() {
    let dashboard = read_dashboard("grafana/provisioning/dashboards/analytics/portfolio.json");
    let sql = panel_raw_sql(&dashboard, 5).expect("panel 5 rawSql");
    assert!(
        sql.contains("CROSS JOIN LATERAL jsonb_array_elements"),
        "expected LATERAL unnest in breakdown SQL"
    );
    assert!(
        !sql.contains("jsonb_array_elements(payload->'accounts') AS elem ORDER BY snapshot_date DESC LIMIT 1"),
        "reject global LIMIT 1 on cross-join"
    );
    assert!(sql.contains("ORDER BY snapshot_date DESC LIMIT 1) latest"));
}

#[test]
fn portfolio_overview_table_title_and_grid() {
    let dashboard = read_dashboard("grafana/provisioning/dashboards/analytics/portfolio.json");
    let panel = dashboard["panels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_i64() == Some(5))
        .expect("panel 5");
    assert_eq!(
        panel["title"].as_str(),
        Some("All accounts (latest snapshot)")
    );
    assert_eq!(panel["gridPos"]["y"].as_i64(), Some(4));
}

#[test]
fn account_id_variable_uses_abs_balance_sort() {
    for path in [
        "grafana/provisioning/dashboards/analytics/cashflow.json",
        "grafana/provisioning/dashboards/analytics/forecast-horizons.json",
    ] {
        let dashboard = read_dashboard(path);
        let query = templating_query(&dashboard, "account_id").expect("account_id query");
        assert!(
            query.contains("ABS(COALESCE(a.balance, 0)) DESC"),
            "{path}: expected ABS(balance) sort"
        );
        assert!(
            !query.contains("ORDER BY name"),
            "{path}: reject alphabetical-only sort"
        );
        let vars = dashboard["templating"]["list"].as_array().unwrap();
        let account_var = vars
            .iter()
            .find(|v| v["name"].as_str() == Some("account_id"))
            .expect("account_id variable");
        assert!(
            account_var.get("current").is_none(),
            "{path}: omit saved current on account_id"
        );
    }
}

#[test]
fn forecast_horizons_ml_banner_and_no_value() {
    let dashboard =
        read_dashboard("grafana/provisioning/dashboards/analytics/forecast-horizons.json");
    assert!(has_text_panel_content(
        &dashboard,
        "ML forecast not enabled on this deployment"
    ));
    for id in ml_panel_ids() {
        assert_eq!(
            panel_no_value(&dashboard, id).as_deref(),
            Some("ML unavailable"),
            "panel {id} noValue"
        );
    }
}

async fn setup_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(pool)
}

#[tokio::test]
async fn breakdown_query_returns_all_accounts_from_latest_snapshot() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for breakdown SQL fixture");
            return;
        }
    };

    let older_date = chrono::Utc::now().date_naive() - chrono::Duration::days(2);
    let latest_date = chrono::Utc::now().date_naive() - chrono::Duration::days(1);

    let older_payload = serde_json::json!({
        "accounts": [
            {"name": "Old only", "account_role": "asset", "currency": "EUR", "balance": "1.0"}
        ]
    });
    let latest_payload = serde_json::json!({
        "accounts": [
            {"name": "Giro", "account_role": "asset", "currency": "EUR", "balance": "-1200.50"},
            {"name": "Savings", "account_role": "asset", "currency": "EUR", "balance": "5000.00"},
            {"name": "Cash wallet", "account_role": "asset", "currency": "EUR", "balance": "0.00"}
        ]
    });

    sqlx::query(
        r#"
        INSERT INTO net_worth_snapshots (snapshot_date, total_eur, mixed_currency, account_count, payload)
        VALUES ($1, 0, false, 1, $2), ($3, -3395.75, false, 3, $4)
        ON CONFLICT (snapshot_date) DO UPDATE SET
          account_count = EXCLUDED.account_count,
          payload = EXCLUDED.payload,
          total_eur = EXCLUDED.total_eur
        "#,
    )
    .bind(older_date)
    .bind(older_payload)
    .bind(latest_date)
    .bind(latest_payload)
    .execute(&pool)
    .await
    .expect("seed snapshots");

    let rows: Vec<(String,)> = sqlx::query_as(BREAKDOWN_SQL)
        .fetch_all(&pool)
        .await
        .expect("breakdown query");

    assert_eq!(rows.len(), 3, "latest snapshot should unnest 3 accounts");
    assert_eq!(rows[0].0, "Savings");
}

#[tokio::test]
async fn account_variable_query_prefers_funded_account_over_zero_wallet() {
    let pool = match setup_db().await {
        Some(p) => p,
        None => {
            eprintln!("SKIP: DATABASE_URL not set for account variable SQL fixture");
            return;
        }
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let funded_id = format!("114-{suffix}");
    let zero_id = format!("116-{suffix}");

    for (firefly_id, name, balance) in [
        (&funded_id, "Funded Giro", -2536.78_f64),
        (&zero_id, "Cash wallet", 0.0_f64),
        (&format!("115-{suffix}"), "Savings", 1200.0_f64),
    ] {
        sqlx::query(
            r#"
            INSERT INTO accounts (firefly_id, type, name, currency, balance, payload)
            VALUES ($1, 'asset', $2, 'EUR', $3, '{}')
            ON CONFLICT (firefly_id) DO UPDATE SET balance = EXCLUDED.balance, name = EXCLUDED.name
            "#,
        )
        .bind(firefly_id)
        .bind(name)
        .bind(balance)
        .execute(&pool)
        .await
        .expect("seed account");
    }

    let rows: Vec<(String, String)> = sqlx::query_as(ACCOUNT_VARIABLE_SQL)
        .fetch_all(&pool)
        .await
        .expect("account variable query");

    assert!(
        rows.first().is_some_and(|(id, _)| id == &funded_id || id == &format!("115-{suffix}")),
        "ABS(balance) sort should pick funded account before zero wallet; got {:?}",
        rows.iter().take(3).collect::<Vec<_>>()
    );
    let zero_pos = rows.iter().position(|(id, _)| id == &zero_id);
    let funded_pos = rows.iter().position(|(id, _)| id == &funded_id);
    if let (Some(z), Some(f)) = (zero_pos, funded_pos) {
        assert!(f < z, "funded account {funded_id} should sort before zero wallet {zero_id}");
    }
}
