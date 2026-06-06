use chrono::NaiveDate;
use std::collections::HashMap;

use crate::forecast::types::{RecurringPattern, TransactionRow};
use crate::recurrence::{
    detect_recurrence_groups, detect_recurrence_inflow_groups, RecurrenceConfig, RecurrenceGroup,
};

fn recurrence_config(tolerance_pct: f64) -> RecurrenceConfig {
    RecurrenceConfig {
        high_tolerance_pct: tolerance_pct,
        medium_tolerance_pct: tolerance_pct.max(10.0),
        low_tolerance_pct: tolerance_pct.max(15.0),
        min_emit_confidence: 60,
    }
}

fn groups_to_patterns(groups: Vec<RecurrenceGroup>) -> Vec<RecurringPattern> {
    groups
        .into_iter()
        .map(|g| RecurringPattern {
            description: g.payee_key.clone(),
            amount: g.median_amount,
            interval_days: g.interval_days,
            category_id: mode_category_id(&g),
        })
        .collect()
}

pub fn detect_patterns(
    transactions: &[TransactionRow],
    tolerance_pct: f64,
) -> Vec<RecurringPattern> {
    groups_to_patterns(detect_recurrence_groups(
        transactions,
        &recurrence_config(tolerance_pct),
    ))
}

/// Recurring inflows on revenue (or asset) accounts — salary, regular refunds.
pub fn detect_inflow_patterns(
    transactions: &[TransactionRow],
    tolerance_pct: f64,
) -> Vec<RecurringPattern> {
    groups_to_patterns(detect_recurrence_inflow_groups(
        transactions,
        &recurrence_config(tolerance_pct),
    ))
}

fn mode_category_id(group: &RecurrenceGroup) -> Option<String> {
    let mut tallies: HashMap<String, (usize, NaiveDate)> = HashMap::new();
    for (cat, date) in group
        .category_ids
        .iter()
        .zip(group.transaction_dates.iter())
    {
        if let Some(id) = cat {
            let entry = tallies.entry(id.clone()).or_insert((0, *date));
            entry.0 += 1;
            if *date > entry.1 {
                entry.1 = *date;
            }
        }
    }
    tallies
        .into_iter()
        .max_by(|a, b| a.1.0.cmp(&b.1.0).then_with(|| a.1.1.cmp(&b.1.1)))
        .map(|(id, _)| id)
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
            payload: json!({}),
        }
    }

    #[test]
    fn detects_monthly_recurring_rent() {
        let txs = vec![
            tx("2026-01-01", -1000.0, "Rent Payment"),
            tx("2026-02-01", -1000.0, "Rent Payment"),
            tx("2026-03-01", -990.0, "Rent Payment"),
        ];
        let patterns = detect_patterns(&txs, 5.0);
        assert_eq!(patterns.len(), 1);
        assert!((patterns[0].amount - (-1000.0)).abs() < 1.0);
        assert!((28..=32).contains(&patterns[0].interval_days));
    }

    #[test]
    fn carries_mode_category_id_from_group() {
        fn tx_cat(date: &str, amount: f64, desc: &str, cat: &str) -> TransactionRow {
            TransactionRow {
                firefly_id: format!("{date}-{amount}"),
                account_id: Some("1".into()),
                date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
                amount,
                description: Some(desc.into()),
                category_id: Some(cat.into()),
                payload: json!({}),
            }
        }
        let txs = vec![
            tx_cat("2026-01-01", -1000.0, "Rent Payment", "cat-old"),
            tx_cat("2026-02-01", -1000.0, "Rent Payment", "cat-rent"),
            tx_cat("2026-03-01", -1000.0, "Rent Payment", "cat-rent"),
        ];
        let patterns = detect_patterns(&txs, 5.0);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].category_id.as_deref(), Some("cat-rent"));
    }

    #[test]
    fn ignores_irregular_transactions() {
        let txs = vec![
            tx("2026-01-01", -50.0, "Coffee"),
            tx("2026-01-15", -60.0, "Coffee"),
        ];
        assert!(detect_patterns(&txs, 5.0).is_empty());
    }
}
