use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::NaiveDate;

use crate::forecast::types::TransactionRow;

use super::amount::{amounts_within_tolerance, median_amount};
use super::cadence::{
    classify_cadence, interval_outlier_count, is_stable_cadence, median_interval_days, CadenceKind,
};
use super::group::{by_payee, by_payee_inflow};

#[derive(Debug, Clone)]
pub struct RecurrenceConfig {
    pub high_tolerance_pct: f64,
    pub medium_tolerance_pct: f64,
    pub low_tolerance_pct: f64,
    pub min_emit_confidence: i16,
}

impl Default for RecurrenceConfig {
    fn default() -> Self {
        Self {
            high_tolerance_pct: 5.0,
            medium_tolerance_pct: 10.0,
            low_tolerance_pct: 15.0,
            min_emit_confidence: 60,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecurrenceGroup {
    pub payee_key: String,
    pub display_name: String,
    pub interval_days: i64,
    pub median_amount: f64,
    pub confidence_pct: i16,
    pub transaction_ids: Vec<String>,
    pub transaction_dates: Vec<NaiveDate>,
    pub category_ids: Vec<Option<String>>,
}

pub fn compute_fingerprint(payee_key: &str, interval_days: i64, amount: f64) -> String {
    let rounded = (amount * 100.0).round() / 100.0;
    let mut hasher = DefaultHasher::new();
    format!("{payee_key}:{interval_days}:{rounded:.2}").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn detect_recurrence_groups(
    transactions: &[TransactionRow],
    config: &RecurrenceConfig,
) -> Vec<RecurrenceGroup> {
    detect_recurrence_groups_from_map(by_payee(transactions), config)
}

pub fn detect_recurrence_inflow_groups(
    transactions: &[TransactionRow],
    config: &RecurrenceConfig,
) -> Vec<RecurrenceGroup> {
    detect_recurrence_groups_from_map(by_payee_inflow(transactions), config)
}

fn detect_recurrence_groups_from_map(
    groups: std::collections::HashMap<String, Vec<&TransactionRow>>,
    config: &RecurrenceConfig,
) -> Vec<RecurrenceGroup> {
    let mut results = Vec::new();

    for (payee_key, txs) in groups {
        if txs.len() < 3 {
            continue;
        }

        let mut dates: Vec<NaiveDate> = txs.iter().map(|t| t.date).collect();
        dates.sort();
        let mut intervals = Vec::new();
        for w in dates.windows(2) {
            intervals.push((w[1] - w[0]).num_days());
        }
        if intervals.is_empty() {
            continue;
        }

        let interval_days = median_interval_days(&intervals);
        if !is_stable_cadence(interval_days, &intervals) {
            continue;
        }

        let mut dated: Vec<_> = txs.iter().map(|t| (t.date, t.amount)).collect();
        dated.sort_by_key(|(d, _)| *d);
        let amounts: Vec<f64> = dated.iter().map(|(_, a)| *a).collect();
        let recent_len = 6.min(amounts.len());
        let recent_amounts: Vec<f64> = amounts.iter().rev().take(recent_len).copied().collect();
        let median = median_amount(&recent_amounts);
        let confidence =
            score_confidence(txs.len(), &intervals, interval_days, &recent_amounts, median, config);
        if confidence < config.min_emit_confidence {
            continue;
        }

        let display_name = txs
            .last()
            .and_then(|t| t.description.clone())
            .unwrap_or_else(|| payee_key.clone());

        results.push(RecurrenceGroup {
            payee_key: payee_key.clone(),
            display_name,
            interval_days,
            median_amount: median,
            confidence_pct: confidence,
            transaction_ids: txs.iter().map(|t| t.firefly_id.clone()).collect(),
            transaction_dates: dates,
            category_ids: txs.iter().map(|t| t.category_id.clone()).collect(),
        });
    }

    results
}

fn score_confidence(
    count: usize,
    intervals: &[i64],
    median_interval: i64,
    amounts: &[f64],
    median_amount: f64,
    config: &RecurrenceConfig,
) -> i16 {
    let outliers = interval_outlier_count(median_interval, intervals);
    let cadence = classify_cadence(median_interval);

    if count >= 4
        && outliers == 0
        && cadence != CadenceKind::Unknown
        && amounts_within_tolerance(amounts, median_amount, config.high_tolerance_pct)
    {
        return 95;
    }

    if count >= 3
        && outliers == 0
        && cadence != CadenceKind::Unknown
        && amounts_within_tolerance(amounts, median_amount, config.medium_tolerance_pct)
    {
        return 80;
    }

    if count >= 3
        && cadence != CadenceKind::Unknown
        && (outliers <= 1 || amounts_within_tolerance(amounts, median_amount, config.low_tolerance_pct))
    {
        return 60;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forecast::types::TransactionRow;
    use chrono::NaiveDate;
    use serde_json::json;

    fn tx(date: &str, amount: f64, desc: &str) -> TransactionRow {
        TransactionRow {
            firefly_id: format!("{date}-{amount}"),
            account_id: Some("1".into()),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount,
            description: Some(desc.into()),
            category_id: None,
            payload: json!({"attributes": {"type": "withdrawal"}}),
        }
    }

    #[test]
    fn confidence_tiers_per_dec_0014() {
        let txs = vec![
            tx("2026-01-01", -9.99, "Netflix"),
            tx("2026-02-01", -9.99, "Netflix"),
            tx("2026-03-01", -9.99, "Netflix"),
            tx("2026-04-01", -9.99, "Netflix"),
        ];
        let groups = detect_recurrence_groups(&txs, &RecurrenceConfig::default());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].confidence_pct, 95);
    }

    #[test]
    fn emits_only_at_least_sixty_percent() {
        let txs = vec![
            tx("2026-01-01", -50.0, "Coffee"),
            tx("2026-01-15", -60.0, "Coffee"),
            tx("2026-02-01", -55.0, "Coffee"),
        ];
        let groups = detect_recurrence_groups(&txs, &RecurrenceConfig::default());
        assert!(groups.is_empty());
    }
}
