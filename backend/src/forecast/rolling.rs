use chrono::{NaiveDate, Utc};

use super::types::{RecurringPattern, TransactionRow};

pub struct RollingResult {
    pub daily_rate: f64,
}

pub fn variable_residual(
    transactions: &[TransactionRow],
    recurring: &[RecurringPattern],
    window_days: i64,
    sparse_history_days: i64,
) -> (RollingResult, bool) {
    let today = Utc::now().date_naive();
    let earliest = transactions
        .iter()
        .map(|t| t.date)
        .min()
        .unwrap_or(today);
    let history_days = (today - earliest).num_days().max(0);
    let low_confidence = history_days < sparse_history_days;

    let effective_window = if low_confidence {
        history_days.max(1)
    } else {
        window_days
    };

    let window_start = today - chrono::Duration::days(effective_window);

    let recurring_descs: std::collections::HashSet<String> = recurring
        .iter()
        .map(|p| p.description.clone())
        .collect();

    let mut daily_totals: Vec<f64> = Vec::new();
    let mut day_map: std::collections::HashMap<NaiveDate, f64> = std::collections::HashMap::new();

    for tx in transactions {
        if tx.date < window_start || tx.date > today {
            continue;
        }
        let desc = tx
            .description
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        if recurring_descs.contains(&desc) {
            continue;
        }
        *day_map.entry(tx.date).or_insert(0.0) += tx.amount;
    }

    for total in day_map.values() {
        daily_totals.push(*total);
    }

    if daily_totals.is_empty() {
        return (RollingResult { daily_rate: 0.0 }, low_confidence);
    }

    let cap = percentile_95(&daily_totals);
    let capped: Vec<f64> = daily_totals.iter().map(|t| t.min(cap)).collect();
    let sum: f64 = capped.iter().sum();
    let days = effective_window.max(1) as f64;
    let daily_rate = sum / days;

    (RollingResult { daily_rate }, low_confidence)
}

fn percentile_95(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forecast::types::TransactionRow;
    use chrono::{NaiveDate, Utc};
    use serde_json::json;

    fn tx(date: &str, amount: f64) -> TransactionRow {
        TransactionRow {
            firefly_id: date.into(),
            account_id: Some("1".into()),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount,
            description: Some("misc".into()),
            category_id: None,
            payload: json!({}),
        }
    }

    #[test]
    fn computes_daily_rate_with_cap() {
        let today = Utc::now().date_naive();
        let mut txs = Vec::new();
        for i in 0..30 {
            let d = today - chrono::Duration::days(30 - i);
            txs.push(tx(&d.format("%Y-%m-%d").to_string(), -10.0));
        }
        txs.push(tx(
            &(today - chrono::Duration::days(15)).format("%Y-%m-%d").to_string(),
            -500.0,
        ));
        let (result, _) = variable_residual(&txs, &[], 90, 90);
        assert!(result.daily_rate < 0.0);
    }
}
