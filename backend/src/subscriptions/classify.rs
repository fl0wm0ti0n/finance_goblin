use crate::config::SubscriptionsConfig;
use crate::recurrence::RecurrenceGroup;
use crate::recurrence::amount::coefficient_of_variation;
use crate::recurrence::cadence::classify_cadence;

pub fn classify_kind(group: &RecurrenceGroup, config: &SubscriptionsConfig) -> &'static str {
    let amounts: Vec<f64> = std::iter::repeat(group.median_amount)
        .take(group.transaction_ids.len())
        .collect();
    let cv = coefficient_of_variation(&amounts);

    if cv < 0.02
        && amounts.iter().all(|a| (a - group.median_amount).abs() <= 0.01)
        && group.median_amount <= -200.0
    {
        return "standing_order";
    }

    let cadence = classify_cadence(group.interval_days);
    let monthly_or_quarterly = matches!(cadence, crate::recurrence::cadence::CadenceKind::Monthly
        | crate::recurrence::cadence::CadenceKind::Quarterly);

    if monthly_or_quarterly && cv < 0.01 && group.median_amount <= -200.0 {
        return "standing_order";
    }

    let category_match_ratio = group
        .category_ids
        .iter()
        .filter(|cid| {
            cid.as_ref().map_or(false, |id| {
                config
                    .standing_order_category_patterns
                    .iter()
                    .any(|p| id.to_lowercase().contains(p))
            })
        })
        .count();
    if !group.category_ids.is_empty()
        && category_match_ratio * 2 >= group.category_ids.len()
    {
        return "standing_order";
    }

    if group.median_amount <= -200.0 && cv < 0.01 {
        return "standing_order";
    }

    for pattern in &config.standing_order_payee_patterns {
        if group.payee_key.contains(pattern) || group.display_name.to_lowercase().contains(pattern)
        {
            return "standing_order";
        }
    }

    "subscription"
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn group(amount: f64, interval: i64, categories: Vec<Option<String>>) -> RecurrenceGroup {
        RecurrenceGroup {
            payee_key: "rent payment".into(),
            display_name: "Rent Payment".into(),
            interval_days: interval,
            median_amount: amount,
            confidence_pct: 95,
            transaction_ids: vec!["1".into(), "2".into(), "3".into()],
            transaction_dates: vec![
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            ],
            category_ids: categories,
        }
    }

    fn config() -> SubscriptionsConfig {
        SubscriptionsConfig::default()
    }

    #[test]
    fn large_fixed_monthly_is_standing_order() {
        let g = group(-850.0, 30, vec![None, None, None]);
        assert_eq!(classify_kind(&g, &config()), "standing_order");
    }

    #[test]
    fn small_variable_is_subscription() {
        let g = group(-9.99, 30, vec![None, None, None]);
        assert_eq!(classify_kind(&g, &config()), "subscription");
    }
}
