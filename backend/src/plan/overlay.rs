use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use super::types::{
    AdjustmentDirection, AdjustmentFrequency, AdjustmentTarget, ConfirmedSubscription, PlanAdjustment,
};

const HORIZON_DAYS: i64 = 730;

/// Per-category average monthly outflow cap for `remove_outflow` (DEC-0093).
pub type CategoryRemoveCaps = HashMap<String, f64>;

/// Map plan adjustments to daily net-cashflow deltas keyed by calendar date.
pub fn build_overlay_deltas(
    adjustments: &[PlanAdjustment],
    confirmed_subs: &[ConfirmedSubscription],
    start: NaiveDate,
    end: NaiveDate,
    category_remove_caps: &CategoryRemoveCaps,
) -> HashMap<NaiveDate, f64> {
    let mut deltas: HashMap<NaiveDate, f64> = HashMap::new();
    let mut sorted: Vec<_> = adjustments.to_vec();
    sorted.sort_by_key(|a| (a.sort_order, a.id));

    for adj in &sorted {
        if adj.target_type == AdjustmentTarget::Subscription
            && adj.direction == AdjustmentDirection::RemoveOutflow
        {
            if let Some(payee) = &adj.target_key {
                apply_subscription_removal(&mut deltas, payee, confirmed_subs, start, end);
            }
            continue;
        }

        let mut effective_amount = adj.amount;
        if adj.target_type == AdjustmentTarget::Category
            && adj.direction == AdjustmentDirection::RemoveOutflow
        {
            if let Some(cat) = &adj.target_key {
                let cap = category_remove_caps.get(cat).copied().unwrap_or(0.0);
                effective_amount = effective_amount.min(cap);
            } else {
                effective_amount = 0.0;
            }
            if effective_amount <= 0.0 {
                continue;
            }
        }

        let signed_amount = effective_amount * adj.direction.signed_multiplier();
        let effective_end = adj.effective_to.unwrap_or(end);

        let mut date = start;
        while date <= end {
            if date >= adj.effective_from && date <= effective_end && is_due(adj, date) {
                *deltas.entry(date).or_insert(0.0) += signed_amount;
            }
            date += chrono::Duration::days(1);
        }
    }

    deltas
}

fn apply_subscription_removal(
    deltas: &mut HashMap<NaiveDate, f64>,
    payee_key: &str,
    confirmed_subs: &[ConfirmedSubscription],
    start: NaiveDate,
    end: NaiveDate,
) {
    let Some(sub) = confirmed_subs.iter().find(|s| s.payee_key == payee_key) else {
        return;
    };

    if sub.interval_days <= 0 {
        return;
    }

    let mut date = start;
    while date <= end {
        if is_interval_due(date, sub.interval_days) {
            *deltas.entry(date).or_insert(0.0) += sub.amount.abs();
        }
        date += chrono::Duration::days(1);
    }
}

fn is_due(adj: &PlanAdjustment, date: NaiveDate) -> bool {
    if date < adj.effective_from {
        return false;
    }
    if let Some(to) = adj.effective_to {
        if date > to {
            return false;
        }
    }

    match adj.frequency {
        AdjustmentFrequency::OneTime => date == adj.effective_from,
        AdjustmentFrequency::Weekly => {
            (date - adj.effective_from).num_days() % 7 == 0
        }
        AdjustmentFrequency::Monthly => {
            date.day() == adj.effective_from.day()
                || (adj.effective_from.day() > 28
                    && date.day() == last_day_of_month(date).day())
        }
        AdjustmentFrequency::Quarterly => {
            let months = months_between(adj.effective_from, date);
            months % 3 == 0 && date.day() == adj.effective_from.day()
        }
    }
}

fn is_interval_due(date: NaiveDate, interval_days: i64) -> bool {
    let epoch = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    (date - epoch).num_days() % interval_days == 0
}

fn months_between(from: NaiveDate, to: NaiveDate) -> i32 {
    (to.year() - from.year()) * 12 + (to.month() as i32 - from.month() as i32)
}

fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    let (y, m) = (date.year(), date.month());
    if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    }
}

pub fn overlay_horizon_end(start: NaiveDate) -> NaiveDate {
    start + chrono::Duration::days(HORIZON_DAYS)
}

fn calendar_month_end(month_start: NaiveDate) -> NaiveDate {
    let (y, m) = (month_start.year(), month_start.month());
    if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    }
}

/// Sum overlay deltas from `month_start` through `min(today, calendar month end)`.
/// Empty adjustments always return **0.00** (DEC-0073).
pub fn monthly_overlay_delta_sum(
    adjustments: &[PlanAdjustment],
    confirmed_subs: &[ConfirmedSubscription],
    month_start: NaiveDate,
    today: NaiveDate,
    category_remove_caps: &CategoryRemoveCaps,
) -> f64 {
    if adjustments.is_empty() {
        return 0.0;
    }
    let month_end = calendar_month_end(month_start);
    let effective_end = if today < month_end { today } else { month_end };
    if effective_end < month_start {
        return 0.0;
    }
    let overlay = build_overlay_deltas(
        adjustments,
        confirmed_subs,
        month_start,
        effective_end,
        category_remove_caps,
    );
    overlay.values().sum()
}

/// Build 3-month average outflow caps for category remove_outflow lines.
pub fn build_category_remove_caps<F>(adjustments: &[PlanAdjustment], avg_for: F) -> CategoryRemoveCaps
where
    F: Fn(&str) -> f64,
{
    let mut caps = CategoryRemoveCaps::new();
    for adj in adjustments {
        if adj.target_type == AdjustmentTarget::Category
            && adj.direction == AdjustmentDirection::RemoveOutflow
        {
            if let Some(cat) = &adj.target_key {
                caps.entry(cat.clone()).or_insert_with(|| avg_for(cat));
            }
        }
    }
    caps
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn adj(
        direction: AdjustmentDirection,
        amount: f64,
        frequency: AdjustmentFrequency,
        from: NaiveDate,
    ) -> PlanAdjustment {
        PlanAdjustment {
            id: Uuid::new_v4(),
            version_id: Uuid::new_v4(),
            direction,
            amount,
            frequency,
            target_type: AdjustmentTarget::Household,
            target_key: None,
            label: None,
            effective_from: from,
            effective_to: None,
            sort_order: 0,
        }
    }

    #[test]
    fn monthly_outflow_applies_on_schedule() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let adjustments = vec![adj(
            AdjustmentDirection::AddOutflow,
            300.0,
            AdjustmentFrequency::Monthly,
            start,
        )];

        let deltas = build_overlay_deltas(&adjustments, &[], start, end, &CategoryRemoveCaps::new());
        assert!(deltas.get(&start).copied().unwrap_or(0.0).abs() > 299.0);
        assert!(deltas.get(&NaiveDate::from_ymd_opt(2026, 2, 15).unwrap()).is_some());
    }

    #[test]
    fn weekly_adjustment_repeats_every_seven_days() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 21).unwrap();
        let adjustments = vec![adj(
            AdjustmentDirection::AddInflow,
            50.0,
            AdjustmentFrequency::Weekly,
            start,
        )];

        let deltas = build_overlay_deltas(&adjustments, &[], start, end, &CategoryRemoveCaps::new());
        assert_eq!(deltas.get(&start).copied(), Some(50.0));
        assert_eq!(
            deltas.get(&NaiveDate::from_ymd_opt(2026, 1, 8).unwrap()).copied(),
            Some(50.0)
        );
    }

    #[test]
    fn one_time_applies_only_on_effective_from() {
        let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let adjustments = vec![adj(
            AdjustmentDirection::AddOutflow,
            1000.0,
            AdjustmentFrequency::OneTime,
            start,
        )];

        let deltas = build_overlay_deltas(&adjustments, &[], start, end, &CategoryRemoveCaps::new());
        assert_eq!(deltas.len(), 1);
        assert!(deltas.get(&start).copied().unwrap_or(0.0) < -999.0);
    }

    #[test]
    fn monthly_overlay_delta_sum_zero_when_no_adjustments() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let month_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let sum = monthly_overlay_delta_sum(&[], &[], month_start, today, &CategoryRemoveCaps::new());
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn monthly_overlay_delta_sum_leasing_template_approx_minus_300() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let month_start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let adjustments = vec![adj(
            AdjustmentDirection::AddOutflow,
            300.0,
            AdjustmentFrequency::Monthly,
            month_start,
        )];
        let sum = monthly_overlay_delta_sum(&adjustments, &[], month_start, today, &CategoryRemoveCaps::new());
        assert!(
            sum < -299.0 && sum > -301.0,
            "leasing overlay delta expected ~-300, got {sum}"
        );
    }

    #[test]
    fn category_remove_outflow_clamped_to_cap() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let mut adjustment = adj(
            AdjustmentDirection::RemoveOutflow,
            200.0,
            AdjustmentFrequency::Monthly,
            start,
        );
        adjustment.target_type = AdjustmentTarget::Category;
        adjustment.target_key = Some("entertainment".into());

        let mut caps = CategoryRemoveCaps::new();
        caps.insert("entertainment".into(), 85.5);

        let deltas = build_overlay_deltas(&[adjustment], &[], start, end, &caps);
        let total: f64 = deltas.values().sum();
        assert!(total > 84.0 && total < 86.0, "expected ~85.5 cap, got {total}");
    }

    #[test]
    fn category_remove_zero_cap_produces_no_overlay() {
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let mut adjustment = adj(
            AdjustmentDirection::RemoveOutflow,
            100.0,
            AdjustmentFrequency::Monthly,
            start,
        );
        adjustment.target_type = AdjustmentTarget::Category;
        adjustment.target_key = Some("new_cat".into());

        let deltas = build_overlay_deltas(&[adjustment], &[], start, end, &CategoryRemoveCaps::new());
        assert!(deltas.is_empty());
    }

    #[test]
    fn subscription_removal_matches_payee_key() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let mut adjustment = adj(
            AdjustmentDirection::RemoveOutflow,
            0.0,
            AdjustmentFrequency::Monthly,
            start,
        );
        adjustment.target_type = AdjustmentTarget::Subscription;
        adjustment.target_key = Some("netflix".into());

        let subs = vec![ConfirmedSubscription {
            payee_key: "netflix".into(),
            amount: -12.99,
            interval_days: 30,
        }];

        let deltas = build_overlay_deltas(&[adjustment], &subs, start, end, &CategoryRemoveCaps::new());
        assert!(!deltas.is_empty());
        assert!(deltas.values().any(|v| *v > 12.0));
    }
}
