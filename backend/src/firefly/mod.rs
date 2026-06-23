use std::time::Duration;

use reqwest::{Client, Method, StatusCode, Url};
use serde_json::Value;
use sqlx::PgPool;

use crate::audit;
use crate::config::FireflyConfig;

const ALLOWED_PATHS: &[&str] = &[
    "/api/v1/accounts",
    "/api/v1/transactions",
    "/api/v1/categories",
    "/api/v1/budgets",
    "/api/v1/tags",
    "/api/v1/piggy_banks",
];

#[derive(Debug, thiserror::Error)]
pub enum FireflyError {
    #[error("only GET requests are permitted to Firefly API, attempted {0}")]
    MethodNotAllowed(String),
    #[error("path not in allowlist: {0}")]
    PathNotAllowed(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("unexpected status {0}")]
    UnexpectedStatus(StatusCode),
    #[error(
        "firefly_personal_access_token_missing: set non-empty FIREFLY_PERSONAL_ACCESS_TOKEN (see docs/engineering/runbook.md § Omniflow external deploy)"
    )]
    PersonalAccessTokenMissing,
    #[error(
        "firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"
    )]
    Unauthorized,
}

pub struct FireflyClient {
    http: Client,
    base_url: String,
    token: String,
    page_limit: u32,
    audit_enabled: bool,
    pool: PgPool,
}

impl FireflyClient {
    pub fn new(config: &FireflyConfig, pool: PgPool) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("reqwest client");
        Self {
            http,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            token: config.personal_access_token.clone(),
            page_limit: config.page_limit,
            audit_enabled: config.audit_enabled,
            pool,
        }
    }

    pub async fn get_json(&self, path: &str, query: &[(&str, String)]) -> Result<Value, FireflyError> {
        self.request(Method::GET, path, query).await
    }

    pub async fn get_paginated(&self, path: &str, extra_query: &[(&str, String)]) -> Result<Vec<Value>, FireflyError> {
        let mut page = 1u32;
        let mut items = Vec::new();

        loop {
            let mut query = vec![
                ("page", page.to_string()),
                ("limit", self.page_limit.to_string()),
            ];
            query.extend_from_slice(extra_query);

            let body = self.get_json(path, &query).await?;
            let data = body["data"]
                .as_array()
                .cloned()
                .unwrap_or_default();

            if data.is_empty() {
                break;
            }

            items.extend(data);

            let has_next = body["meta"]["pagination"]["total_pages"]
                .as_u64()
                .map(|total| page < total as u32)
                .unwrap_or(false);

            if !has_next {
                break;
            }
            page += 1;
        }

        Ok(items)
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value, FireflyError> {
        if method != Method::GET {
            return Err(FireflyError::MethodNotAllowed(method.to_string()));
        }

        if !ALLOWED_PATHS.iter().any(|p| path.starts_with(p)) {
            return Err(FireflyError::PathNotAllowed(path.to_string()));
        }

        let mut url = Url::parse(&format!("{}{}", self.base_url, path))
            .map_err(|e| FireflyError::PathNotAllowed(e.to_string()))?;
        for (k, v) in query {
            url.query_pairs_mut().append_pair(k, v);
        }

        let mut delay = Duration::from_millis(500);
        let max_retries = 5;

        for attempt in 0..=max_retries {
            let response = self
                .http
                .request(Method::GET, url.clone())
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/json")
                .send()
                .await?;

            let status = response.status();
            if self.audit_enabled {
                let _ = audit::record_request(
                    &self.pool,
                    "GET",
                    path,
                    Some(status.as_u16() as i32),
                )
                .await;
            }

            if status.is_success() {
                return Ok(response.json().await?);
            }

            if status == StatusCode::UNAUTHORIZED {
                return Err(FireflyError::Unauthorized);
            }

            if (status.is_server_error() || status.as_u16() == 429) && attempt < max_retries {
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(Duration::from_secs(30));
                continue;
            }

            return Err(FireflyError::UnexpectedStatus(status));
        }

        Err(FireflyError::UnexpectedStatus(StatusCode::SERVICE_UNAVAILABLE))
    }

    /// Runtime guard for non-GET — used by tests and explicit rejection API.
    pub fn reject_non_get(method: &str) -> Result<(), FireflyError> {
        if method.eq_ignore_ascii_case("GET") {
            Ok(())
        } else {
            Err(FireflyError::MethodNotAllowed(method.to_string()))
        }
    }
}

pub mod sync {
    use chrono::{DateTime, Duration, NaiveDate, Utc};
    use reqwest::StatusCode;
    use serde_json::Value;

    use super::FireflyClient;
    use crate::db::repositories;

    /// Parse Firefly split date: strict YYYY-MM-DD → RFC3339 → first-10-char prefix.
    pub fn parse_sync_date(date_str: &str) -> Option<NaiveDate> {
        if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(d);
        }
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Some(dt.date_naive());
        }
        if date_str.len() >= 10 {
            if let Ok(d) = NaiveDate::parse_from_str(&date_str[..10], "%Y-%m-%d") {
                return Some(d);
            }
        }
        tracing::warn!(date = date_str, "failed to parse Firefly transaction date");
        None
    }

    /// Parse Firefly split amount (JSON number or string).
    pub fn parse_split_amount(value: &Value) -> Option<f64> {
        if let Some(n) = value.as_f64() {
            return Some(n);
        }
        value
            .as_str()
            .and_then(|s| s.trim().parse::<f64>().ok())
    }

    /// Normalize signed amount from split `type` per DEC-0059.
    pub fn normalize_split_amount(raw: Option<f64>, split_type: Option<&str>) -> Option<f64> {
        let raw = raw?;
        let abs = raw.abs();
        match split_type.map(str::to_ascii_lowercase).as_deref() {
            Some("withdrawal") => Some(-abs),
            Some("deposit") => Some(abs),
            Some("transfer") => Some(-abs),
            _ => {
                if raw > 0.0 {
                    tracing::debug!(
                        split_type = ?split_type,
                        amount = raw,
                        "normalizing unknown split type as outflow"
                    );
                    Some(-abs)
                } else {
                    Some(raw)
                }
            }
        }
    }

    pub fn extract_category_id(first: &Value) -> Option<String> {
        first
            .get("category_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    pub async fn sync_reference_entities(client: &FireflyClient, pool: &sqlx::PgPool) -> Result<(), super::FireflyError> {
        sync_accounts(client, pool).await?;
        sync_categories(client, pool).await?;
        sync_budgets(client, pool).await?;
        sync_tags(client, pool).await?;
        sync_piggy_banks(client, pool).await?;
        Ok(())
    }

    async fn sync_accounts(client: &FireflyClient, pool: &sqlx::PgPool) -> Result<i64, super::FireflyError> {
        let items = client.get_paginated("/api/v1/accounts", &[]).await?;
        let count = items.len() as i64;
        for item in &items {
            let id = item["id"].as_str().unwrap_or_default();
            let attrs = &item["attributes"];
            let raw_balance = &attrs["current_balance"];
            let parsed_balance = parse_split_amount(raw_balance);
            let account_role = attrs["account_role"].as_str();

            tracing::info!(
                event = "balance_ingest",
                firefly_id = %id,
                name = attrs["name"].as_str().unwrap_or_default(),
                raw_current_balance = ?raw_balance,
                parsed_balance = ?parsed_balance,
                account_role = ?account_role,
            );

            if parsed_balance.is_none() && !raw_balance.is_null() {
                tracing::warn!(
                    event = "balance_ingest",
                    firefly_id = %id,
                    name = attrs["name"].as_str().unwrap_or_default(),
                    raw_current_balance = ?raw_balance,
                    "failed to parse Firefly account current_balance"
                );
            }

            repositories::upsert_account(
                pool,
                id,
                attrs["type"].as_str(),
                attrs["name"].as_str(),
                attrs["iban"].as_str(),
                attrs["currency_code"].as_str(),
                parsed_balance,
                item,
            )
            .await?;
        }
        repositories::upsert_cursor(pool, "accounts", Utc::now(), count).await?;
        Ok(count)
    }

    async fn sync_categories(client: &FireflyClient, pool: &sqlx::PgPool) -> Result<(), super::FireflyError> {
        let items = client.get_paginated("/api/v1/categories", &[]).await?;
        for item in &items {
            repositories::upsert_category(
                pool,
                item["id"].as_str().unwrap_or_default(),
                item["attributes"]["name"].as_str(),
                item,
            )
            .await?;
        }
        repositories::upsert_cursor(pool, "categories", Utc::now(), items.len() as i64).await?;
        Ok(())
    }

    async fn sync_budgets(client: &FireflyClient, pool: &sqlx::PgPool) -> Result<(), super::FireflyError> {
        let items = client.get_paginated("/api/v1/budgets", &[]).await?;
        for item in &items {
            repositories::upsert_budget(
                pool,
                item["id"].as_str().unwrap_or_default(),
                item["attributes"]["name"].as_str(),
                item["attributes"]["amount"].as_f64(),
                item["attributes"]["period"].as_str(),
                item,
            )
            .await?;
        }
        repositories::upsert_cursor(pool, "budgets", Utc::now(), items.len() as i64).await?;
        Ok(())
    }

    async fn sync_tags(client: &FireflyClient, pool: &sqlx::PgPool) -> Result<(), super::FireflyError> {
        let items = client.get_paginated("/api/v1/tags", &[]).await?;
        for item in &items {
            repositories::upsert_tag(
                pool,
                item["id"].as_str().unwrap_or_default(),
                item["attributes"]["tag"].as_str(),
                item,
            )
            .await?;
        }
        repositories::upsert_cursor(pool, "tags", Utc::now(), items.len() as i64).await?;
        Ok(())
    }

    async fn sync_piggy_banks(client: &FireflyClient, pool: &sqlx::PgPool) -> Result<(), super::FireflyError> {
        let items = match client.get_paginated("/api/v1/piggy_banks", &[]).await {
            Ok(items) => items,
            Err(super::FireflyError::UnexpectedStatus(status)) if status == StatusCode::NOT_FOUND => {
                tracing::warn!(
                    "Firefly piggy_banks endpoint unavailable (404); skipping optional entity sync"
                );
                repositories::upsert_cursor(pool, "piggy_banks", Utc::now(), 0).await?;
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        for item in &items {
            repositories::upsert_piggy_bank(
                pool,
                item["id"].as_str().unwrap_or_default(),
                item["attributes"]["name"].as_str(),
                item["attributes"]["target_amount"].as_f64(),
                item["attributes"]["current_amount"].as_f64(),
                item,
            )
            .await?;
        }
        repositories::upsert_cursor(pool, "piggy_banks", Utc::now(), items.len() as i64).await?;
        Ok(())
    }

    const MANUAL_LOOKBACK_DAYS: i64 = 365;

    pub async fn sync_transactions(
        client: &FireflyClient,
        pool: &sqlx::PgPool,
        overlap_days: i64,
        trigger: &str,
    ) -> Result<i64, super::FireflyError> {
        let watermark = last_watermark(pool).await;
        let start = if trigger == "manual" {
            Utc::now() - Duration::days(MANUAL_LOOKBACK_DAYS)
        } else if let Some(w) = watermark {
            w - Duration::days(overlap_days)
        } else {
            Utc::now() - Duration::days(MANUAL_LOOKBACK_DAYS)
        };
        let start_date = start.format("%Y-%m-%d").to_string();

        let items = client
            .get_paginated("/api/v1/transactions", &[("start", start_date)])
            .await?;

        for item in &items {
            let attrs = &item["attributes"];
            let tx_list = attrs["transactions"].as_array();
            let first = tx_list.and_then(|l| l.first());
            let date_str = first
                .and_then(|t| t["date"].as_str())
                .or_else(|| attrs["date"].as_str());
            let date = date_str.and_then(parse_sync_date);
            let amount = first.and_then(|t| {
                normalize_split_amount(parse_split_amount(&t["amount"]), t["type"].as_str())
            });
            let category_id = first.map(extract_category_id).flatten();
            let description = first
                .and_then(|t| t["description"].as_str())
                .or_else(|| attrs["description"].as_str());

            repositories::upsert_transaction(
                pool,
                item["id"].as_str().unwrap_or_default(),
                first.and_then(|t| t["source_id"].as_str()),
                date,
                amount,
                description,
                category_id.as_deref(),
                item,
            )
            .await?;
        }

        let count = items.len() as i64;
        repositories::upsert_cursor(pool, "transactions", Utc::now(), count).await?;
        Ok(count)
    }

    async fn last_watermark(pool: &sqlx::PgPool) -> Option<DateTime<Utc>> {
        sqlx::query_scalar(
            "SELECT last_successful_sync_at FROM sync_cursors WHERE entity_type = 'transactions'",
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn parse_sync_date_accepts_iso_datetime() {
            let d = parse_sync_date("2025-06-01T12:00:00+02:00").unwrap();
            assert_eq!(d, NaiveDate::from_ymd_opt(2025, 6, 1).unwrap());
        }

        #[test]
        fn parse_sync_date_accepts_plain_date() {
            let d = parse_sync_date("2025-06-01").unwrap();
            assert_eq!(d, NaiveDate::from_ymd_opt(2025, 6, 1).unwrap());
        }

        #[test]
        fn parse_sync_date_invalid_returns_none() {
            assert!(parse_sync_date("not-a-date").is_none());
        }

        #[test]
        fn normalize_split_amount_withdrawal_deposit_transfer() {
            assert_eq!(normalize_split_amount(Some(42.0), Some("withdrawal")), Some(-42.0));
            assert_eq!(normalize_split_amount(Some(42.0), Some("deposit")), Some(42.0));
            assert_eq!(normalize_split_amount(Some(42.0), Some("transfer")), Some(-42.0));
        }

        #[test]
        fn normalize_split_amount_unknown_positive_is_outflow() {
            assert_eq!(normalize_split_amount(Some(10.0), Some("reconciliation")), Some(-10.0));
            assert_eq!(normalize_split_amount(Some(10.0), None), Some(-10.0));
        }

        #[test]
        fn normalize_split_amount_unknown_non_positive_keeps_raw() {
            assert_eq!(normalize_split_amount(Some(-5.0), None), Some(-5.0));
            assert_eq!(normalize_split_amount(Some(0.0), None), Some(0.0));
        }

        #[test]
        fn extract_category_id_from_first_split() {
            let split = json!({
                "category_id": "cat-123",
                "amount": "50.00",
                "type": "withdrawal"
            });
            assert_eq!(extract_category_id(&split), Some("cat-123".into()));
        }

        #[test]
        fn extract_category_id_absent_is_none() {
            let split = json!({ "amount": "50.00", "type": "withdrawal" });
            assert!(extract_category_id(&split).is_none());
        }

        #[test]
        fn journal_fixture_extracts_category_date_and_signed_amount() {
            let item = json!({
                "id": "tx-1",
                "attributes": {
                    "transactions": [{
                        "date": "2025-06-01T12:00:00+02:00",
                        "amount": "25.50",
                        "type": "withdrawal",
                        "category_id": "cat-groceries",
                        "source_id": "acct-1",
                        "description": "Groceries"
                    }]
                }
            });
            let first = item["attributes"]["transactions"][0].clone();
            assert_eq!(
                parse_sync_date(first["date"].as_str().unwrap()),
                Some(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap())
            );
            assert_eq!(extract_category_id(&first), Some("cat-groceries".into()));
            assert_eq!(
                normalize_split_amount(parse_split_amount(&first["amount"]), first["type"].as_str()),
                Some(-25.50)
            );
        }

        #[test]
        fn parse_split_amount_accepts_string_or_number() {
            assert_eq!(parse_split_amount(&json!("12.34")), Some(12.34));
            assert_eq!(parse_split_amount(&json!(99.0)), Some(99.0));
            assert!(parse_split_amount(&json!("")).is_none());
        }

        #[test]
        fn account_current_balance_parses_string_or_number() {
            let string_balance = json!({ "attributes": { "current_balance": "1234.56" } });
            let number_balance = json!({ "attributes": { "current_balance": 1234.56 } });
            assert_eq!(
                parse_split_amount(&string_balance["attributes"]["current_balance"]),
                Some(1234.56)
            );
            assert_eq!(
                parse_split_amount(&number_balance["attributes"]["current_balance"]),
                Some(1234.56)
            );
        }

        #[test]
        fn account_current_balance_parse_failure_returns_none() {
            assert!(parse_split_amount(&json!("not-a-number")).is_none());
        }
    }
}
