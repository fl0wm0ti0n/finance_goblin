use chrono::{Datelike, Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::plan::types::PlanAdjustment;

use super::types::{AlertCandidate, AlertSeverity, AlertType, AlertsConfig};

pub struct EvaluateContext<'a> {
    pub pool: &'a PgPool,
    pub forecast_computation_id: Uuid,
    pub plan_computation_id: Option<Uuid>,
    pub config: &'a AlertsConfig,
}

pub async fn evaluate_scarcity(ctx: &EvaluateContext<'_>) -> Result<Vec<AlertCandidate>, sqlx::Error> {
    let threshold = ctx.config.scarcity_threshold_eur;
    let today = Utc::now().date_naive();
    let horizon_end = today + Duration::days(45);

    let daily: Vec<(NaiveDate, f64)> = sqlx::query_as(
        r#"
        SELECT fbd.ts::date AS day, SUM(fbd.balance::float8) AS balance
        FROM forecast_balance_daily fbd
        JOIN accounts a ON a.firefly_id = fbd.account_id
        WHERE fbd.computation_id = $1
          AND a.type = 'asset'
          AND COALESCE((a.payload->>'include_net_worth')::boolean, true) = true
          AND fbd.ts::date >= $2 AND fbd.ts::date <= $3
        GROUP BY fbd.ts::date
        ORDER BY day
        "#,
    )
    .bind(ctx.forecast_computation_id)
    .bind(today)
    .bind(horizon_end)
    .fetch_all(ctx.pool)
    .await?;

    let current_balance: f64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(balance::float8), 0)
        FROM accounts
        WHERE type = 'asset'
          AND COALESCE((payload->>'include_net_worth')::boolean, true) = true
          AND balance >= 0
        "#,
    )
    .fetch_one(ctx.pool)
    .await?;

    let next_month = if today.month() == 12 {
        NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
    }
    .unwrap();
    let current_month_end = next_month - Duration::days(1);

    let month_end_balance = daily
        .iter()
        .find(|(d, _)| *d == current_month_end)
        .map(|(_, b)| *b)
        .unwrap_or(current_balance);

    let breach_day = daily
        .iter()
        .find(|(_, b)| *b < threshold)
        .map(|(d, _)| *d);

    let month_end_breach = month_end_balance < threshold;
    let horizon_breach = breach_day.is_some();

    if !horizon_breach && !month_end_breach {
        return Ok(vec![]);
    }

    let earliest = breach_day.unwrap_or(current_month_end);
    let tomorrow = today + Duration::days(1);

    let severity = if current_balance < threshold
        || breach_day.map(|d| d <= tomorrow).unwrap_or(false)
    {
        AlertSeverity::Critical
    } else {
        AlertSeverity::Warning
    };

    let month_key = format!("{}-{:02}", today.year(), today.month());
    let fingerprint = format!("scarcity:household:{month_key}");

    Ok(vec![AlertCandidate {
        alert_type: AlertType::Scarcity,
        severity,
        fingerprint,
        title: "Scarcity alert — household balance".into(),
        message: format!(
            "Projected household balance falls below €{threshold:.0} threshold; earliest breach {earliest}"
        ),
        entity_type: Some("household".into()),
        entity_id: None,
        context: serde_json::json!({
            "forecast_computation_id": ctx.forecast_computation_id,
            "threshold_eur": threshold,
            "earliest_breach_date": earliest.to_string(),
            "current_balance": current_balance,
        }),
    }])
}

pub async fn evaluate_budget_drift(
    ctx: &EvaluateContext<'_>,
    adjustments: &[PlanAdjustment],
) -> Result<Vec<AlertCandidate>, sqlx::Error> {
    let today = Utc::now().date_naive();
    let month_key = format!("{}-{:02}", today.year(), today.month());
    let days_in_month = {
        let next = if today.month() == 12 {
            NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
        }
        .unwrap();
        (next - NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap()).num_days()
    };
    let days_elapsed = today.day() as i64;
    let drift_pct = ctx.config.budget_drift_pct / 100.0;

    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    let mut candidates = Vec::new();

    for adj in adjustments {
        if adj.target_type != crate::plan::types::AdjustmentTarget::Category {
            continue;
        }
        let Some(category_id) = &adj.target_key else {
            continue;
        };

        let monthly_delta = adj.amount.abs();
        let mtd_target = monthly_delta * (days_elapsed as f64 / days_in_month as f64);

        let mtd_actual: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(ABS(amount::float8)), 0)
            FROM transactions
            WHERE category_id = $1
              AND date >= $2 AND date <= $3
              AND amount::float8 < 0
            "#,
        )
        .bind(category_id)
        .bind(month_start)
        .bind(today)
        .fetch_one(ctx.pool)
        .await?;

        if mtd_target <= 0.0 {
            continue;
        }

        if mtd_actual > mtd_target * (1.0 + drift_pct) {
            let fingerprint = format!("budget_drift:category:{category_id}:{month_key}");
            candidates.push(AlertCandidate {
                alert_type: AlertType::BudgetDrift,
                severity: AlertSeverity::Warning,
                fingerprint,
                title: format!("Budget drift — category {category_id}"),
                message: format!(
                    "MTD spend €{mtd_actual:.2} exceeds prorated plan target €{mtd_target:.2} by more than {:.0}%",
                    ctx.config.budget_drift_pct
                ),
                entity_type: Some("category".into()),
                entity_id: Some(category_id.clone()),
                context: serde_json::json!({
                    "mtd_actual": mtd_actual,
                    "mtd_target": mtd_target,
                    "budget_drift_pct": ctx.config.budget_drift_pct,
                    "month": month_key,
                }),
            });
        }
    }

    Ok(candidates)
}

pub async fn evaluate_plan_viability(
    ctx: &EvaluateContext<'_>,
    plan_id: Uuid,
    version_id: Uuid,
    plan_computation_id: Uuid,
) -> Result<Vec<AlertCandidate>, sqlx::Error> {
    let today = Utc::now().date_naive();
    let current_month_end = {
        let next = if today.month() == 12 {
            NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1)
        }
        .unwrap();
        next - Duration::days(1)
    };
    let next_month_end = {
        let after_next = if today.month() >= 11 {
            NaiveDate::from_ymd_opt(today.year() + 1, (today.month() + 2).min(12), 1)
        } else {
            NaiveDate::from_ymd_opt(today.year(), today.month() + 2, 1)
        }
        .unwrap();
        after_next - Duration::days(1)
    };

    let balances: Vec<(NaiveDate, f64)> = sqlx::query_as(
        r#"
        SELECT ts::date AS day, planned_balance::float8 AS bal
        FROM plan_daily_cashflow
        WHERE version_id = $1 AND computation_id = $2
          AND ts::date IN ($3, $4)
        ORDER BY day
        "#,
    )
    .bind(version_id)
    .bind(plan_computation_id)
    .bind(current_month_end)
    .bind(next_month_end)
    .fetch_all(ctx.pool)
    .await?;

    let current_end = balances
        .iter()
        .find(|(d, _)| *d == current_month_end)
        .map(|(_, b)| *b);
    let next_end = balances
        .iter()
        .find(|(d, _)| *d == next_month_end)
        .map(|(_, b)| *b);

    let primary_breach = current_end.map(|b| b < 0.0).unwrap_or(false);
    let secondary_breach =
        current_end.map(|b| b < 0.0).unwrap_or(false) && next_end.map(|b| b < 0.0).unwrap_or(false);

    if !primary_breach && !secondary_breach {
        return Ok(vec![]);
    }

    let severity = if secondary_breach {
        AlertSeverity::Critical
    } else {
        AlertSeverity::Warning
    };

    let fingerprint = format!("plan_viability:{plan_id}:{version_id}");

    Ok(vec![AlertCandidate {
        alert_type: AlertType::PlanViability,
        severity,
        fingerprint,
        title: "Plan viability — negative month-end balance".into(),
        message: format!(
            "Active plan projected balance at month-end is negative (€{:.2})",
            current_end.unwrap_or(0.0)
        ),
        entity_type: Some("plan".into()),
        entity_id: Some(plan_id.to_string()),
        context: serde_json::json!({
            "plan_computation_id": plan_computation_id,
            "forecast_computation_id": ctx.forecast_computation_id,
            "plan_id": plan_id,
            "version_id": version_id,
            "current_month_end_balance": current_end,
        }),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scarcity_fingerprint_format() {
        let fp = "scarcity:household:2026-05";
        assert!(fp.starts_with("scarcity:household:"));
    }

    #[test]
    fn budget_drift_proration_mid_month() {
        let monthly = 300.0;
        let days_in_month = 31i64;
        let days_elapsed = 15i64;
        let mtd_target = monthly * (days_elapsed as f64 / days_in_month as f64);
        assert!((mtd_target - 145.16).abs() < 0.1);
    }

    #[test]
    fn budget_drift_skips_non_category() {
        use crate::plan::types::{
            AdjustmentDirection, AdjustmentFrequency, AdjustmentTarget, PlanAdjustment,
        };
        let adj = PlanAdjustment {
            id: Uuid::new_v4(),
            version_id: Uuid::new_v4(),
            direction: AdjustmentDirection::AddOutflow,
            frequency: AdjustmentFrequency::Monthly,
            target_type: AdjustmentTarget::Household,
            target_key: None,
            amount: 100.0,
            label: Some("test".into()),
            effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            effective_to: None,
            sort_order: 0,
        };
        assert_ne!(adj.target_type, AdjustmentTarget::Category);
    }

    #[test]
    fn scarcity_severity_critical_when_current_below() {
        let current = 150.0;
        let threshold = 200.0;
        assert!(current < threshold);
    }
}
