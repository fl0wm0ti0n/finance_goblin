use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;

use crate::forecast::types::{DailyPoint, DailyPointWithBands, MonthlyCashflow};

use super::sidecar::ForecastPoint;

/// Integrate monthly ML net-cashflow forecasts onto baseline daily balance path.
pub fn overlay_monthly_onto_baseline(
    baseline_daily: &[DailyPoint],
    baseline_monthly: &[MonthlyCashflow],
    ml_forecasts: &[ForecastPoint],
) -> Vec<DailyPointWithBands> {
    let mut baseline_by_month: BTreeMap<NaiveDate, f64> = BTreeMap::new();
    for m in baseline_monthly {
        let key = month_key(m.month);
        baseline_by_month.insert(key, m.free_cashflow);
    }

    let mut cum_delta = 0.0;
    let mut cum_lo = 0.0;
    let mut cum_hi = 0.0;
    let mut month_deltas: BTreeMap<NaiveDate, (f64, f64, f64)> = BTreeMap::new();

    for fc in ml_forecasts {
        if let Ok(d) = NaiveDate::parse_from_str(&fc.ds, "%Y-%m-%d") {
            let key = month_key(d);
            let base_fc = baseline_by_month.get(&key).copied().unwrap_or(0.0);
            let delta = fc.y - base_fc;
            let delta_lo = fc.y_lo - base_fc;
            let delta_hi = fc.y_hi - base_fc;
            cum_delta += delta;
            cum_lo += delta_lo;
            cum_hi += delta_hi;
            month_deltas.insert(key, (cum_delta, cum_lo, cum_hi));
        }
    }

    baseline_daily
        .iter()
        .map(|p| {
            let key = month_key(p.date);
            let (d, lo, hi) = month_deltas
                .range(..=key)
                .next_back()
                .map(|(_, v)| *v)
                .unwrap_or((0.0, 0.0, 0.0));
            DailyPointWithBands {
                date: p.date,
                balance: p.balance + d,
                balance_p10: Some(p.balance + lo),
                balance_p90: Some(p.balance + hi),
            }
        })
        .collect()
}

fn month_key(d: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn overlay_applies_cumulative_delta() {
        let baseline_daily = vec![
            DailyPoint {
                date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                balance: 1000.0,
            },
            DailyPoint {
                date: NaiveDate::from_ymd_opt(2025, 2, 15).unwrap(),
                balance: 1100.0,
            },
        ];
        let baseline_monthly = vec![
            MonthlyCashflow {
                month: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                income: 0.0,
                fixed_costs: 0.0,
                variable_costs: 0.0,
                free_cashflow: 100.0,
                bucket_sources: None,
                ai_mapped: false,
            },
            MonthlyCashflow {
                month: NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
                income: 0.0,
                fixed_costs: 0.0,
                variable_costs: 0.0,
                free_cashflow: 100.0,
                bucket_sources: None,
                ai_mapped: false,
            },
        ];
        let ml = vec![
            ForecastPoint {
                ds: "2025-01-01".into(),
                y: 150.0,
                y_lo: 120.0,
                y_hi: 180.0,
            },
            ForecastPoint {
                ds: "2025-02-01".into(),
                y: 100.0,
                y_lo: 80.0,
                y_hi: 120.0,
            },
        ];
        let out = overlay_monthly_onto_baseline(&baseline_daily, &baseline_monthly, &ml);
        assert_eq!(out.len(), 2);
        assert!((out[0].balance - 1050.0).abs() < 0.01);
        assert!((out[1].balance - 1150.0).abs() < 0.01);
        assert!(out[0].balance_p10.is_some());
    }
}
