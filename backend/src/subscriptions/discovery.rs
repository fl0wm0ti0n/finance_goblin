use std::collections::HashSet;

use crate::recurrence::{detect_recurrence_groups, RecurrenceConfig, RecurrenceGroup};

use super::repository::{
    find_confirmed_payee_interval, interval_matches, is_rejected_payee_interval,
    SubscriptionRepository,
};
use super::types::{DiscoverCandidate, DiscoverMeta, DiscoverResponse};

pub struct DiscoverQuery<'a> {
    pub account_id: Option<&'a str>,
    pub payee: Option<&'a str>,
    pub interval_days: Option<i32>,
    pub limit: usize,
}

pub async fn run_discover(
    repo: &SubscriptionRepository,
    query: DiscoverQuery<'_>,
) -> Result<DiscoverResponse, sqlx::Error> {
    let config = repo.config();
    let window_days = config.detection_window_days;
    let account_id = query.account_id;
    let txs = repo
        .load_expense_transactions(window_days, account_id)
        .await?;

    let recurrence_config = RecurrenceConfig {
        high_tolerance_pct: config.confidence_tolerance_pct.high,
        medium_tolerance_pct: config.confidence_tolerance_pct.medium,
        low_tolerance_pct: config.confidence_tolerance_pct.low,
        min_emit_confidence: 60,
    };

    let groups = detect_recurrence_groups(&txs, &recurrence_config);
    let rejections = repo.load_rejection_fingerprints().await?;
    let confirmed_fps = repo.load_confirmed_fingerprints().await?;
    let confirmed_payee_intervals = repo.load_confirmed_payee_intervals().await?;
    let rejected_payee_intervals = repo.load_rejected_payee_intervals().await?;

    let payee_filter = query.payee.map(|p| p.trim().to_lowercase()).filter(|p| !p.is_empty());
    let interval_filter = query.interval_days;
    let limit = query.limit.min(50).max(1);

    let mut candidates: Vec<DiscoverCandidate> = Vec::new();

    for group in &groups {
        if !group_matches_filters(group, payee_filter.as_deref(), interval_filter) {
            continue;
        }

        let fingerprint = crate::recurrence::compute_fingerprint(
            &group.payee_key,
            group.interval_days,
            group.median_amount,
        );
        if rejections.contains(&fingerprint) || confirmed_fps.contains(&fingerprint) {
            continue;
        }

        let interval_days = group.interval_days as i32;
        if is_rejected_payee_interval(&rejected_payee_intervals, &group.payee_key, interval_days) {
            continue;
        }
        if find_confirmed_payee_interval(&confirmed_payee_intervals, &group.payee_key, interval_days)
            .is_some()
        {
            continue;
        }

        let account_ids = collect_account_ids(&txs, &group.transaction_ids);
        if let Some(req_account) = account_id {
            if !account_ids.iter().any(|a| a == req_account) {
                continue;
            }
        }

        candidates.push(DiscoverCandidate {
            payee_key: group.payee_key.clone(),
            display_name: group.display_name.clone(),
            interval_days,
            median_amount: group.median_amount,
            confidence_pct: group.confidence_pct,
            transaction_count: group.transaction_ids.len(),
            transaction_ids: group.transaction_ids.clone(),
            account_ids,
        });
    }

    candidates.sort_by(|a, b| {
        b.confidence_pct
            .cmp(&a.confidence_pct)
            .then_with(|| b.transaction_count.cmp(&a.transaction_count))
    });

    let truncated = candidates.len() > limit;
    candidates.truncate(limit);

    Ok(DiscoverResponse {
        candidates,
        meta: DiscoverMeta {
            limit,
            truncated,
            window_days,
        },
    })
}

fn group_matches_filters(
    group: &RecurrenceGroup,
    payee: Option<&str>,
    interval_days: Option<i32>,
) -> bool {
    if let Some(p) = payee {
        let haystack = format!("{} {}", group.payee_key, group.display_name).to_lowercase();
        if !haystack.contains(p) {
            return false;
        }
    }
    if let Some(target) = interval_days {
        if !interval_matches(target, group.interval_days as i32) {
            return false;
        }
    }
    true
}

fn collect_account_ids(
    txs: &[crate::forecast::types::TransactionRow],
    transaction_ids: &[String],
) -> Vec<String> {
    let wanted: HashSet<&str> = transaction_ids.iter().map(String::as_str).collect();
    let mut ids: HashSet<String> = HashSet::new();
    for tx in txs {
        if wanted.contains(tx.firefly_id.as_str()) {
            if let Some(ref account_id) = tx.account_id {
                ids.insert(account_id.clone());
            }
        }
    }
    let mut out: Vec<String> = ids.into_iter().collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_group(payee: &str, interval: i32) -> RecurrenceGroup {
        RecurrenceGroup {
            payee_key: payee.into(),
            display_name: payee.into(),
            interval_days: interval as i64,
            median_amount: -12.99,
            confidence_pct: 95,
            transaction_ids: vec!["a".into(), "b".into(), "c".into()],
            transaction_dates: vec![
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            ],
            category_ids: vec![],
        }
    }

    #[test]
    fn payee_filter_matches_substring() {
        let g = sample_group("netflix", 30);
        assert!(group_matches_filters(&g, Some("netf"), Some(30)));
        assert!(!group_matches_filters(&g, Some("spotify"), Some(30)));
    }

    #[test]
    fn interval_filter_uses_tolerance() {
        let g = sample_group("netflix", 30);
        assert!(group_matches_filters(&g, None, Some(31)));
        assert!(!group_matches_filters(&g, None, Some(7)));
    }
}
