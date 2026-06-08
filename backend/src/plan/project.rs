use std::collections::HashMap;

use chrono::NaiveDate;

use super::overlay::{build_overlay_deltas, CategoryRemoveCaps};
use super::types::{ConfirmedSubscription, DailyNetPoint, PlanAdjustment};

/// Merge baseline household daily net with overlay deltas.
pub fn project_plan_series(
    baseline_net: &HashMap<NaiveDate, f64>,
    adjustments: &[PlanAdjustment],
    confirmed_subs: &[ConfirmedSubscription],
    start: NaiveDate,
    end: NaiveDate,
    starting_balance: f64,
    category_remove_caps: &CategoryRemoveCaps,
) -> Vec<DailyNetPoint> {
    let overlay = build_overlay_deltas(adjustments, confirmed_subs, start, end, category_remove_caps);
    let mut dates: Vec<NaiveDate> = baseline_net.keys().chain(overlay.keys()).copied().collect();
    dates.sort();
    dates.dedup();
    dates.retain(|d| *d >= start && *d <= end);

    let mut balance = starting_balance;
    let mut series = Vec::with_capacity(dates.len());

    for date in dates {
        let base = baseline_net.get(&date).copied().unwrap_or(0.0);
        let overlay_delta = overlay.get(&date).copied().unwrap_or(0.0);
        let planned_net = base + overlay_delta;
        balance += planned_net;
        series.push(DailyNetPoint {
            date,
            planned_net,
            planned_balance: Some(balance),
        });
    }

    series
}

/// Derive daily net cashflow from consecutive aggregate balance points.
pub fn balances_to_daily_net(balances: &[(NaiveDate, f64)]) -> HashMap<NaiveDate, f64> {
    let mut net = HashMap::new();
    if balances.is_empty() {
        return net;
    }

    let mut sorted = balances.to_vec();
    sorted.sort_by_key(|(d, _)| *d);

    for window in sorted.windows(2) {
        let (prev_date, prev_bal) = window[0];
        let (date, bal) = window[1];
        if date > prev_date {
            net.insert(date, bal - prev_bal);
        }
    }

    net
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::plan::types::{
        AdjustmentDirection, AdjustmentFrequency, AdjustmentTarget, PlanAdjustment,
    };

    #[test]
    fn merges_baseline_and_overlay_without_mutating_input() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 3).unwrap();
        let mut baseline = HashMap::new();
        baseline.insert(start, 0.0);
        baseline.insert(start + chrono::Duration::days(1), -100.0);
        baseline.insert(start + chrono::Duration::days(2), -50.0);

        let adjustments = vec![PlanAdjustment {
            id: Uuid::new_v4(),
            version_id: Uuid::new_v4(),
            direction: AdjustmentDirection::AddOutflow,
            amount: 300.0,
            frequency: AdjustmentFrequency::OneTime,
            target_type: AdjustmentTarget::Household,
            target_key: None,
            label: Some("Leasing".into()),
            effective_from: start + chrono::Duration::days(1),
            effective_to: None,
            sort_order: 0,
        }];

        let series = project_plan_series(
            &baseline,
            &adjustments,
            &[],
            start,
            end,
            5000.0,
            &CategoryRemoveCaps::new(),
        );
        assert_eq!(series.len(), 3);
        let day2 = series.iter().find(|p| p.date == start + chrono::Duration::days(1)).unwrap();
        assert!(day2.planned_net < -350.0);
    }
}
