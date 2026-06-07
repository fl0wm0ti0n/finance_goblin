use std::collections::HashMap;

use crate::forecast::types::{is_transfer, TransactionRow};
use serde_json::Value;

use super::normalize::payee_key;

fn first_split_field(payload: &Value, field: &str) -> Option<String> {
    payload
        .get("attributes")?
        .get("transactions")?
        .as_array()?
        .first()?
        .get(field)?
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

fn is_transfer_shaped_description(desc: &str) -> bool {
    let upper = desc.to_uppercase();
    upper.contains("SVWZ")
        || upper.contains("UEBERWEISUNG")
        || upper.contains("LASTSCHRIFT")
}

/// Payee source per DEC-0061 with DEC-0072 transfer guard:
/// description → counterparty_name → destination_name, except transfer-shaped
/// memos prefer counterparty_name first.
pub fn extract_payee_source(tx: &TransactionRow) -> Option<String> {
    if let Some(desc) = tx.description.as_deref().filter(|s| !s.trim().is_empty()) {
        if is_transfer_shaped_description(desc) {
            if let Some(name) = first_split_field(&tx.payload, "counterparty_name") {
                return Some(name);
            }
        }
        return Some(desc.to_string());
    }
    if let Some(name) = first_split_field(&tx.payload, "counterparty_name") {
        return Some(name);
    }
    first_split_field(&tx.payload, "destination_name")
}

fn group_by_payee_sign(
    transactions: &[TransactionRow],
    outflow: bool,
) -> HashMap<String, Vec<&TransactionRow>> {
    let mut groups: HashMap<String, Vec<&TransactionRow>> = HashMap::new();

    for tx in transactions {
        if tx.amount.abs() < 0.01 {
            continue;
        }
        if outflow {
            if tx.amount >= 0.0 {
                continue;
            }
        } else if tx.amount <= 0.0 {
            continue;
        }
        if is_transfer(&tx.payload) {
            continue;
        }
        let Some(source) = extract_payee_source(tx) else {
            continue;
        };
        let key = payee_key(&source);
        if key.is_empty() {
            continue;
        }
        groups.entry(key).or_default().push(tx);
    }

    groups
}

pub fn by_payee(transactions: &[TransactionRow]) -> HashMap<String, Vec<&TransactionRow>> {
    group_by_payee_sign(transactions, true)
}

/// Recurring inflows (salary, refunds) — mirror of `by_payee` for positive amounts.
pub fn by_payee_inflow(transactions: &[TransactionRow]) -> HashMap<String, Vec<&TransactionRow>> {
    group_by_payee_sign(transactions, false)
}

/// Normalized payee key for matching recurrence patterns to mirror rows.
pub fn transaction_payee_key(tx: &TransactionRow) -> Option<String> {
    let source = extract_payee_source(tx)?;
    let key = payee_key(&source);
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use serde_json::json;

    use super::*;

    fn expense_tx(description: Option<&str>, payload: Value) -> TransactionRow {
        TransactionRow {
            firefly_id: "tx-1".into(),
            account_id: None,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            amount: -9.99,
            description: description.map(str::to_string),
            category_id: None,
            payload,
        }
    }

    #[test]
    fn counterparty_only_fixture_groups_by_payee_key() {
        let payload = json!({
            "attributes": {
                "type": "withdrawal",
                "transactions": [{
                    "counterparty_name": "Netflix",
                    "amount": "-9.99",
                    "type": "withdrawal"
                }]
            }
        });
        let tx = expense_tx(None, payload);
        let txs = [tx];
        let groups = by_payee(&txs);
        assert_eq!(groups.len(), 1);
        assert!(groups.contains_key("netflix"));
    }

    #[test]
    fn empty_description_uses_counterparty_fallback() {
        let payload = json!({
            "attributes": {
                "type": "withdrawal",
                "transactions": [{
                    "counterparty_name": "Spotify AB",
                    "destination_name": "Checking",
                    "amount": "-4.99",
                    "type": "withdrawal"
                }]
            }
        });
        let tx = expense_tx(Some(""), payload);
        let source = extract_payee_source(&tx).unwrap();
        assert_eq!(payee_key(&source), "spotify");
    }

    #[test]
    fn description_takes_priority_over_counterparty() {
        let payload = json!({
            "attributes": {
                "type": "withdrawal",
                "transactions": [{
                    "counterparty_name": "Ignored Merchant",
                    "amount": "-9.99",
                    "type": "withdrawal"
                }]
            }
        });
        let tx = expense_tx(Some("Netflix P3E460"), payload);
        let txs = [tx];
        let groups = by_payee(&txs);
        assert!(groups.contains_key("netflix"));
        assert!(!groups.contains_key("ignored merchant"));
    }

    #[test]
    fn transfer_shaped_description_prefers_counterparty() {
        let payload = json!({
            "attributes": {
                "type": "withdrawal",
                "transactions": [{
                    "counterparty_name": "Netflix",
                    "amount": "-12.99",
                    "type": "withdrawal"
                }]
            }
        });
        let tx = expense_tx(
            Some("SVWZ+REF123456 UEBERWEISUNG Netflix Streaming Monat"),
            payload,
        );
        let source = extract_payee_source(&tx).unwrap();
        assert_eq!(payee_key(&source), "netflix");
        let txs = [tx];
        let groups = by_payee(&txs);
        assert!(groups.contains_key("netflix"));
    }

    #[test]
    fn non_transfer_description_unchanged_priority() {
        let payload = json!({
            "attributes": {
                "type": "withdrawal",
                "transactions": [{
                    "counterparty_name": "Ignored",
                    "amount": "-9.99",
                    "type": "withdrawal"
                }]
            }
        });
        let tx = expense_tx(Some("Amazon Prime Video"), payload);
        let source = extract_payee_source(&tx).unwrap();
        assert_eq!(source, "Amazon Prime Video");
    }
}
