use std::collections::HashMap;

use chrono::{NaiveDate, Utc};

use crate::forecast::types::TransactionRow;
use crate::recurrence::{
    amount::median_amount, cadence::median_interval_days, detect_recurrence_groups,
    group::transaction_payee_key, RecurrenceConfig,
};

use super::repository::{
    find_confirmed_payee_interval, is_rejected_payee_interval, SubscriptionRepository,
    TransactionSearchDbRow, TransactionSearchParams,
};
use super::types::{
    PreviewGroupResponse, RecurringHint, TransactionSearchItem, TransactionSearchMeta,
    TransactionSearchResponse,
};

pub const SEARCH_PAGE_LIMIT: u32 = 100;
pub const HINT_SCAN_CAP: i64 = 500;

pub struct TransactionSearchQuery<'a> {
    pub account_id: &'a str,
    pub payee: Option<&'a str>,
    pub category_id: Option<&'a str>,
    pub account_role: Option<&'a str>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub recurring_hint: bool,
    pub page: u32,
    pub limit: u32,
}

pub async fn run_transaction_search(
    repo: &SubscriptionRepository,
    query: TransactionSearchQuery<'_>,
) -> Result<TransactionSearchResponse, sqlx::Error> {
    let config = repo.config();
    let window_days = config.detection_window_days;
    let window_cutoff = Utc::now().date_naive() - chrono::Duration::days(window_days);

    let date_from = query
        .date_from
        .map(|d| d.max(window_cutoff))
        .or(Some(window_cutoff));
    let date_to = query.date_to;

    let params = TransactionSearchParams {
        account_id: query.account_id,
        payee: query.payee,
        category_id: query.category_id,
        account_role: query.account_role,
        date_from,
        date_to,
        window_cutoff,
    };

    let limit = query.limit.clamp(1, SEARCH_PAGE_LIMIT);
    let page = query.page.max(1);
    let offset = ((page - 1) as i64) * (limit as i64);

    let total_count = repo.count_transactions(&params).await?;
    let rows = repo.search_transactions(&params, limit as i64, offset).await?;

    let mut hint_map: HashMap<String, RecurringHint> = HashMap::new();
    let mut truncated_hint_scan = false;

    if query.recurring_hint {
        let hint_rows = repo
            .search_transactions(&params, HINT_SCAN_CAP, 0)
            .await?;
        truncated_hint_scan = total_count > HINT_SCAN_CAP;
        hint_map = attach_recurring_hints(repo, &hint_rows).await?;
    }

    let has_more = (page as i64) * (limit as i64) < total_count;
    let truncated = total_count > limit as i64 && page == 1;

    let transactions = rows
        .into_iter()
        .map(|row| {
            let hint = hint_map.get(&row.firefly_id).cloned();
            map_search_row(row, hint)
        })
        .collect();

    Ok(TransactionSearchResponse {
        transactions,
        meta: TransactionSearchMeta {
            page,
            limit,
            total_count,
            has_more,
            truncated,
            truncated_hint_scan,
            window_days,
        },
    })
}

async fn attach_recurring_hints(
    repo: &SubscriptionRepository,
    rows: &[TransactionSearchDbRow],
) -> Result<HashMap<String, RecurringHint>, sqlx::Error> {
    if rows.is_empty() {
        return Ok(HashMap::new());
    }

    let txs: Vec<TransactionRow> = rows
        .iter()
        .map(|r| TransactionRow {
            firefly_id: r.firefly_id.clone(),
            account_id: Some(r.account_id.clone()),
            date: r.date,
            amount: r.amount,
            description: r.description.clone(),
            category_id: r.category_id.clone(),
            payload: serde_json::json!({}),
        })
        .collect();

    let config = repo.config();
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

    let mut hint_map: HashMap<String, RecurringHint> = HashMap::new();

    for group in &groups {
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
        if find_confirmed_payee_interval(
            &confirmed_payee_intervals,
            &group.payee_key,
            interval_days,
        )
        .is_some()
        {
            continue;
        }

        let hint = RecurringHint {
            interval_days,
            confidence_pct: group.confidence_pct,
            payee_key: group.payee_key.clone(),
            group_transaction_ids: group.transaction_ids.clone(),
        };

        for tx_id in &group.transaction_ids {
            hint_map.entry(tx_id.clone()).or_insert_with(|| hint.clone());
        }
    }

    Ok(hint_map)
}

fn map_search_row(row: TransactionSearchDbRow, hint: Option<RecurringHint>) -> TransactionSearchItem {
    TransactionSearchItem {
        firefly_id: row.firefly_id,
        account_id: row.account_id,
        account_role: row.account_role,
        date: row.date,
        amount: row.amount,
        description: row.description,
        category_id: row.category_id,
        category_name: row.category_name,
        recurring_hint: hint,
    }
}

pub async fn preview_transaction_group(
    repo: &SubscriptionRepository,
    transaction_ids: &[String],
) -> Result<Result<PreviewGroupResponse, PreviewGroupError>, sqlx::Error> {
    if transaction_ids.len() < 2 {
        return Ok(Err(PreviewGroupError::TooFewTransactions));
    }

    let txs = repo.load_transactions_by_ids(transaction_ids).await?;
    if txs.len() != transaction_ids.len() {
        return Ok(Err(PreviewGroupError::InvalidTransactions));
    }

    let mut payee_keys: Vec<String> = Vec::new();
    for tx in &txs {
        let Some(key) = transaction_payee_key(tx) else {
            return Ok(Err(PreviewGroupError::InvalidTransactions));
        };
        payee_keys.push(key);
    }
    let payee_key = payee_keys.first().cloned().unwrap_or_default();
    if !payee_keys.iter().all(|k| k == &payee_key) {
        return Ok(Err(PreviewGroupError::PayeeMismatch));
    }

    let config = repo.config();
    let recurrence_config = RecurrenceConfig {
        high_tolerance_pct: config.confidence_tolerance_pct.high,
        medium_tolerance_pct: config.confidence_tolerance_pct.medium,
        low_tolerance_pct: config.confidence_tolerance_pct.low,
        min_emit_confidence: 60,
    };

    let groups = detect_recurrence_groups(&txs, &recurrence_config);

    if let Some(group) = groups.into_iter().find(|g| {
        g.payee_key == payee_key
            && transaction_ids
                .iter()
                .all(|id| g.transaction_ids.iter().any(|tid| tid == id))
    }) {
        return Ok(Ok(PreviewGroupResponse {
            payee_key,
            interval_days: group.interval_days as i32,
            median_amount: group.median_amount,
            transaction_ids: transaction_ids.to_vec(),
        }));
    }

    let mut dates: Vec<NaiveDate> = txs.iter().map(|t| t.date).collect();
    dates.sort();
    let mut intervals = Vec::new();
    for w in dates.windows(2) {
        intervals.push((w[1] - w[0]).num_days());
    }
    let interval_days = if intervals.is_empty() {
        30
    } else {
        median_interval_days(&intervals) as i32
    };

    let amounts: Vec<f64> = txs.iter().map(|t| t.amount).collect();
    let median_amount = median_amount(&amounts);

    Ok(Ok(PreviewGroupResponse {
        payee_key,
        interval_days,
        median_amount,
        transaction_ids: transaction_ids.to_vec(),
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewGroupError {
    TooFewTransactions,
    InvalidTransactions,
    PayeeMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn search_page_limit_is_100() {
        assert_eq!(SEARCH_PAGE_LIMIT, 100);
    }

    #[test]
    fn hint_scan_cap_is_500() {
        assert_eq!(HINT_SCAN_CAP, 500);
    }

    #[test]
    fn map_search_row_attaches_hint() {
        let row = TransactionSearchDbRow {
            firefly_id: "tx-1".into(),
            account_id: "acct".into(),
            account_role: Some("defaultAsset".into()),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            amount: -9.99,
            description: Some("Netflix".into()),
            category_id: Some("18".into()),
            category_name: Some("Entertainment".into()),
        };
        let hint = RecurringHint {
            interval_days: 30,
            confidence_pct: 95,
            payee_key: "netflix".into(),
            group_transaction_ids: vec!["tx-1".into()],
        };
        let item = map_search_row(row, Some(hint.clone()));
        assert_eq!(item.recurring_hint.as_ref(), Some(&hint));
    }
}
