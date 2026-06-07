use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};

use super::bucket_inference::{BucketAssignment, BucketSource, collect_ambiguous_features};
use super::categories::{
    accumulate_bucket_with_source, resolve_bucket_with_ai, MonthlyProvenanceTracker, Bucket,
};
use super::recurring::detect_patterns;
use super::rolling::variable_residual;
use super::types::{
    is_transfer, DailyPoint, Milestones, MonthlyCashflow, ProjectionResult, RecurringPattern,
    TransactionRow,
};
use crate::config::ForecastConfig;
use crate::recurrence::compute_fingerprint;
use crate::recurrence::group::transaction_payee_key;
use crate::subscriptions::DetectionResult;

const HORIZON_DAYS: i64 = 730;

pub struct HouseholdIncomeContext<'a> {
    pub patterns: &'a [RecurringPattern],
    /// Mirror rows used for due-date matching (revenue-account salary txs).
    pub reference_transactions: &'a [TransactionRow],
}

pub fn project_account(
    starting_balance: f64,
    transactions: &[TransactionRow],
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
    subscription_context: Option<&DetectionResult>,
    household_income: Option<&HouseholdIncomeContext<'_>>,
    bucket_assignments: &HashMap<String, BucketAssignment>,
) -> ProjectionResult {
    let non_transfer: Vec<TransactionRow> = transactions
        .iter()
        .filter(|t| !is_transfer(&t.payload))
        .cloned()
        .collect();

    let mut recurring = detect_patterns(&non_transfer, config.recurring_amount_tolerance_pct);
    if let Some(ctx) = subscription_context {
        recurring = apply_subscription_override(recurring, &ctx.confirmed_recurring, &non_transfer);
        recurring = exclude_rejected(recurring, &ctx.forecast_excluded_rejections);
    }
    let (rolling, low_confidence) = variable_residual(
        &non_transfer,
        &recurring,
        config.rolling_window_days,
        config.sparse_history_days,
    );

    let today = Utc::now().date_naive();
    let mut balance = starting_balance;
    let mut daily = Vec::with_capacity(HORIZON_DAYS as usize + 1);

    daily.push(DailyPoint {
        date: today,
        balance,
    });

    let mut monthly_map: HashMap<NaiveDate, MonthlyCashflow> = HashMap::new();
    let mut monthly_provenance: HashMap<NaiveDate, MonthlyProvenanceTracker> = HashMap::new();

    for day_offset in 1..=HORIZON_DAYS {
        let date = today + chrono::Duration::days(day_offset);
        let mut delta = rolling.daily_rate;
        let mut recurring_dues: Vec<&RecurringPattern> = Vec::new();

        for pattern in &recurring {
            if is_recurring_due(pattern, &non_transfer, date) {
                delta += pattern.amount;
                recurring_dues.push(pattern);
            }
        }

        balance += delta;
        daily.push(DailyPoint { date, balance });

        let month_start = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
        let entry = monthly_map.entry(month_start).or_insert_with(|| MonthlyCashflow {
            month: month_start,
            income: 0.0,
            fixed_costs: 0.0,
            variable_costs: 0.0,
            free_cashflow: 0.0,
            bucket_sources: None,
            ai_mapped: false,
        });
        let prov = monthly_provenance
            .entry(month_start)
            .or_insert_with(MonthlyProvenanceTracker::default);

        accumulate_bucket_with_source(
            entry,
            Bucket::Variable,
            rolling.daily_rate,
            BucketSource::Default,
            prov,
        );
        for pattern in recurring_dues {
            let (bucket, source) =
                resolve_bucket_with_ai(pattern, category_names, config, bucket_assignments);
            accumulate_bucket_with_source(entry, bucket, pattern.amount, source, prov);
        }
        if let Some(ctx) = household_income {
            for pattern in ctx.patterns {
                if is_recurring_due_for(
                    pattern,
                    ctx.reference_transactions,
                    date,
                ) {
                    let (bucket, source) =
                        resolve_bucket_with_ai(pattern, category_names, config, bucket_assignments);
                    accumulate_bucket_with_source(entry, bucket, pattern.amount, source, prov);
                }
            }
        }
        entry.free_cashflow = entry.income - entry.fixed_costs - entry.variable_costs;
    }

    for (month, prov) in monthly_provenance {
        if let Some(entry) = monthly_map.get_mut(&month) {
            entry.bucket_sources = Some(prov.finalize());
            entry.ai_mapped = prov.ai_mapped;
        }
    }

    let mut monthly: Vec<MonthlyCashflow> = monthly_map.into_values().collect();
    monthly.sort_by_key(|m| m.month);

    let milestones = compute_milestones(&daily, today);
    let horizon_balances = compute_horizon_balances(&daily, today);

    ProjectionResult {
        daily,
        monthly,
        milestones,
        low_confidence,
        horizon_balances,
    }
}

fn is_recurring_due(pattern: &RecurringPattern, txs: &[TransactionRow], date: NaiveDate) -> bool {
    is_recurring_due_for(pattern, txs, date)
}

fn is_recurring_due_for(
    pattern: &RecurringPattern,
    txs: &[TransactionRow],
    date: NaiveDate,
) -> bool {
    let matching: Vec<NaiveDate> = txs
        .iter()
        .filter(|t| transaction_matches_pattern(t, &pattern.description))
        .map(|t| t.date)
        .collect();

    let Some(last) = matching.iter().max().copied() else {
        return false;
    };

    if date <= last {
        return false;
    }

    let days_since = (date - last).num_days();
    if pattern.interval_days <= 0 {
        return false;
    }
    days_since % pattern.interval_days == 0
        || (days_since - pattern.interval_days).abs() <= 2
}

fn transaction_matches_pattern(tx: &TransactionRow, pattern_payee_key: &str) -> bool {
    transaction_payee_key(tx).as_deref() == Some(pattern_payee_key)
}

fn compute_milestones(daily: &[DailyPoint], today: NaiveDate) -> Milestones {
    let tomorrow = balance_on(daily, today + chrono::Duration::days(1));
    let next_week = balance_on(daily, today + chrono::Duration::days(7));
    let month_end = last_day_of_month(today);
    Milestones {
        tomorrow,
        next_week,
        month_end: balance_on(daily, month_end),
    }
}

fn compute_horizon_balances(
    daily: &[DailyPoint],
    today: NaiveDate,
) -> HashMap<i64, f64> {
    let mut horizons = HashMap::new();
    for months in [3i64, 6, 12, 24] {
        let target = add_months(today, months);
        let end_of_month = last_day_of_month(target);
        horizons.insert(months, balance_on(daily, end_of_month));
    }
    horizons
}

fn balance_on(daily: &[DailyPoint], date: NaiveDate) -> f64 {
    daily
        .iter()
        .find(|p| p.date == date)
        .or_else(|| daily.iter().filter(|p| p.date <= date).last())
        .map(|p| p.balance)
        .unwrap_or(0.0)
}

fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    let (y, m) = (date.year(), date.month());
    if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap() - chrono::Duration::days(1)
    }
}

fn add_months(date: NaiveDate, months: i64) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months as i32;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) + 1;
    let day = date.day().min(days_in_month(y, m as u32));
    NaiveDate::from_ymd_opt(y, m as u32, day).unwrap()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

fn apply_subscription_override(
    mut recurring: Vec<RecurringPattern>,
    confirmed: &[crate::subscriptions::ConfirmedRecurring],
    txs: &[TransactionRow],
) -> Vec<RecurringPattern> {
    for c in confirmed {
        let inherited = recurring
            .iter()
            .find(|p| p.description == c.payee_key)
            .and_then(|p| p.category_id.clone())
            .or_else(|| lookup_category_id(&c.payee_key, txs));
        recurring.retain(|p| p.description != c.payee_key);
        recurring.push(RecurringPattern {
            description: c.payee_key.clone(),
            amount: c.amount,
            interval_days: c.interval_days,
            category_id: inherited,
        });
    }
    recurring
}

fn lookup_category_id(payee_key: &str, txs: &[TransactionRow]) -> Option<String> {
    txs
        .iter()
        .filter(|t| transaction_matches_pattern(t, payee_key))
        .max_by_key(|t| t.date)
        .and_then(|t| t.category_id.clone())
}

fn exclude_rejected(
    recurring: Vec<RecurringPattern>,
    rejected: &std::collections::HashSet<String>,
) -> Vec<RecurringPattern> {
    recurring
        .into_iter()
        .filter(|p| {
            let fp = compute_fingerprint(&p.description, p.interval_days, p.amount);
            !rejected.contains(&fp)
        })
        .collect()
}

/// Prepare recurring patterns the same way as `project_account` (for async AI batching).
pub fn prepare_recurring_patterns(
    transactions: &[TransactionRow],
    config: &ForecastConfig,
    subscription_context: Option<&DetectionResult>,
) -> Vec<RecurringPattern> {
    let non_transfer: Vec<TransactionRow> = transactions
        .iter()
        .filter(|t| !is_transfer(&t.payload))
        .cloned()
        .collect();
    let mut recurring = detect_patterns(&non_transfer, config.recurring_amount_tolerance_pct);
    if let Some(ctx) = subscription_context {
        recurring = apply_subscription_override(recurring, &ctx.confirmed_recurring, &non_transfer);
        recurring = exclude_rejected(recurring, &ctx.forecast_excluded_rejections);
    }
    recurring
}

pub fn ambiguous_bucket_features(
    recurring: &[RecurringPattern],
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
) -> Vec<crate::ai::privacy::RawBucketFeatureInput> {
    collect_ambiguous_features(recurring, category_names, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forecast::categories::default_category_buckets;
    use crate::forecast::types::TransactionRow;
    use chrono::NaiveDate;
    use serde_json::json;

    fn config() -> ForecastConfig {
        ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: default_category_buckets(),
            ai_bucket_min_confidence: 0.75,
        }
    }

    fn tx(date: &str, amount: f64, desc: &str) -> TransactionRow {
        TransactionRow {
            firefly_id: date.into(),
            account_id: Some("1".into()),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount,
            description: Some(desc.into()),
            category_id: None,
            payload: json!({"attributes": {"type": "withdrawal"}}),
        }
    }

    fn tx_on(
        date: NaiveDate,
        amount: f64,
        desc: &str,
        category_id: Option<&str>,
    ) -> TransactionRow {
        TransactionRow {
            firefly_id: format!("{date}-{amount}"),
            account_id: Some("1".into()),
            date,
            amount,
            description: Some(desc.into()),
            category_id: category_id.map(str::to_string),
            payload: json!({"attributes": {"type": "withdrawal"}}),
        }
    }

    fn monthly_recurring_history(
        desc: &str,
        amount: f64,
        category_id: Option<&str>,
    ) -> Vec<TransactionRow> {
        let today = Utc::now().date_naive();
        [90i64, 60, 30]
            .iter()
            .map(|days| {
                tx_on(
                    today - chrono::Duration::days(*days),
                    amount,
                    desc,
                    category_id,
                )
            })
            .collect()
    }

    fn category_names() -> HashMap<String, String> {
        let mut names = HashMap::new();
        names.insert("cat-salary".into(), "Salary".into());
        names.insert("cat-rent".into(), "Rent".into());
        names.insert("cat-coffee".into(), "Groceries".into());
        names
    }

    fn first_month(result: &ProjectionResult) -> &MonthlyCashflow {
        result.monthly.first().expect("monthly series")
    }

    #[test]
    fn excludes_transfers_from_projection() {
        let mut txs = vec![tx("2026-01-01", -100.0, "Spend")];
        txs.push(TransactionRow {
            payload: json!({"attributes": {"type": "transfer"}}),
            ..tx("2026-01-02", -500.0, "Transfer")
        });
        let result = project_account(1000.0, &txs, &HashMap::new(), &config(), None, None, &HashMap::new());
        assert!(result.daily.len() > 1);
    }

    #[test]
    fn confirmed_subscription_overrides_heuristic() {
        use std::collections::HashSet;
        use crate::subscriptions::{ConfirmedRecurring, DetectionResult};

        let txs = vec![
            tx("2026-01-01", -1000.0, "Rent Payment"),
            tx("2026-02-01", -1000.0, "Rent Payment"),
            tx("2026-03-01", -1000.0, "Rent Payment"),
        ];
        let ctx = DetectionResult {
            confirmed_recurring: vec![ConfirmedRecurring {
                payee_key: "rent payment".into(),
                amount: -1200.0,
                interval_days: 30,
                fingerprint: "abc".into(),
            }],
            rejected_fingerprints: HashSet::new(),
            forecast_excluded_rejections: HashSet::new(),
        };
        let result = project_account(5000.0, &txs, &HashMap::new(), &config(), Some(&ctx), None, &HashMap::new());
        assert!(result.daily.len() > 1);
    }

    #[test]
    fn rejected_fingerprint_excluded_from_projection() {
        use std::collections::HashSet;
        use crate::forecast::recurring::detect_patterns;
        use crate::subscriptions::DetectionResult;

        let txs = vec![
            tx("2026-01-01", -1000.0, "Rent Payment"),
            tx("2026-02-01", -1000.0, "Rent Payment"),
            tx("2026-03-01", -1000.0, "Rent Payment"),
        ];
        let patterns = detect_patterns(&txs, 5.0);
        assert_eq!(patterns.len(), 1);
        let p = &patterns[0];
        let fp = compute_fingerprint(&p.description, p.interval_days, p.amount);
        let mut rejected = HashSet::new();
        rejected.insert(fp);
        let ctx = DetectionResult {
            confirmed_recurring: vec![],
            rejected_fingerprints: rejected.clone(),
            forecast_excluded_rejections: rejected,
        };
        let without = project_account(5000.0, &txs, &HashMap::new(), &config(), None, None, &HashMap::new());
        let with_reject = project_account(5000.0, &txs, &HashMap::new(), &config(), Some(&ctx), None, &HashMap::new());
        assert_ne!(
            without.daily.last().map(|p| p.balance),
            with_reject.daily.last().map(|p| p.balance)
        );
    }

    #[test]
    fn sparse_history_flags_low_confidence() {
        let txs = vec![tx("2026-05-01", -20.0, "Coffee")];
        let result = project_account(100.0, &txs, &HashMap::new(), &config(), None, None, &HashMap::new());
        assert!(result.low_confidence);
    }

    #[test]
    fn salary_recurring_maps_to_income_bucket() {
        use std::collections::HashSet;
        use crate::subscriptions::{ConfirmedRecurring, DetectionResult};

        let txs = monthly_recurring_history("Employer Payroll", 5000.0, Some("cat-salary"));
        let ctx = DetectionResult {
            confirmed_recurring: vec![ConfirmedRecurring {
                payee_key: "employer payroll".into(),
                amount: 5000.0,
                interval_days: 30,
                fingerprint: "salary".into(),
            }],
            rejected_fingerprints: HashSet::new(),
            forecast_excluded_rejections: HashSet::new(),
        };
        let result = project_account(1000.0, &txs, &category_names(), &config(), Some(&ctx), None, &HashMap::new());
        let month = first_month(&result);
        assert!(month.income > 0.0, "expected income > 0, got {}", month.income);
    }

    #[test]
    fn rent_recurring_maps_to_fixed_bucket() {
        let txs = monthly_recurring_history("Rent Payment", -1000.0, Some("cat-rent"));
        let result = project_account(5000.0, &txs, &category_names(), &config(), None, None, &HashMap::new());
        let month = first_month(&result);
        assert!(
            month.fixed_costs > 0.0,
            "expected fixed_costs > 0, got {}",
            month.fixed_costs
        );
    }

    #[test]
    fn same_day_salary_and_rent_both_buckets_nonzero() {
        use std::collections::HashSet;
        use crate::subscriptions::{ConfirmedRecurring, DetectionResult};

        let mut txs = monthly_recurring_history("Employer Payroll", 5000.0, Some("cat-salary"));
        txs.extend(monthly_recurring_history(
            "Rent Payment",
            -1000.0,
            Some("cat-rent"),
        ));
        let ctx = DetectionResult {
            confirmed_recurring: vec![ConfirmedRecurring {
                payee_key: "employer payroll".into(),
                amount: 5000.0,
                interval_days: 30,
                fingerprint: "salary".into(),
            }],
            rejected_fingerprints: HashSet::new(),
            forecast_excluded_rejections: HashSet::new(),
        };
        let result = project_account(1000.0, &txs, &category_names(), &config(), Some(&ctx), None, &HashMap::new());
        let month = first_month(&result);
        assert!(month.income > 0.0);
        assert!(month.fixed_costs > 0.0);
        assert!(result.daily.len() > 1);
    }

    #[test]
    fn discretionary_coffee_recurring_stays_variable() {
        let txs = monthly_recurring_history("Coffee Shop", -5.0, Some("cat-coffee"));
        let result = project_account(100.0, &txs, &category_names(), &config(), None, None, &HashMap::new());
        let month = first_month(&result);
        assert_eq!(month.fixed_costs, 0.0);
        assert!(month.variable_costs > 0.0);
    }

    #[test]
    fn rent_moves_from_variable_to_fixed_bucket() {
        let txs = monthly_recurring_history("Rent Payment", -1000.0, Some("cat-rent"));
        let with_categories = project_account(5000.0, &txs, &category_names(), &config(), None, None, &HashMap::new());
        let without_categories =
            project_account(5000.0, &txs, &HashMap::new(), &config(), None, None, &HashMap::new());
        let fixed_month = first_month(&with_categories);
        let unmapped_month = first_month(&without_categories);
        assert!(fixed_month.fixed_costs > 0.0);
        assert_eq!(unmapped_month.fixed_costs, 0.0);
        assert!(unmapped_month.variable_costs >= fixed_month.variable_costs);
    }

    #[test]
    fn subscription_override_inherits_category_id() {
        use std::collections::HashSet;
        use crate::subscriptions::{ConfirmedRecurring, DetectionResult};

        let txs = monthly_recurring_history("Rent Payment", -1000.0, Some("cat-rent"));
        let ctx = DetectionResult {
            confirmed_recurring: vec![ConfirmedRecurring {
                payee_key: "rent payment".into(),
                amount: -1200.0,
                interval_days: 30,
                fingerprint: "abc".into(),
            }],
            rejected_fingerprints: HashSet::new(),
            forecast_excluded_rejections: HashSet::new(),
        };
        let result = project_account(
            5000.0,
            &txs,
            &category_names(),
            &config(),
            Some(&ctx),
            None,
            &HashMap::new(),
        );
        let month = first_month(&result);
        assert!(
            month.fixed_costs > 0.0,
            "override should inherit category_id → Fixed bucket"
        );
    }

    #[test]
    fn payee_key_matching_finds_recurring_with_reference_suffix() {
        let desc = "220003055316 Strom Teilbetrag NETTO298.00 +USt 59.60 /VST 3400, Birago gasse 18/1";
        let txs = monthly_recurring_history(desc, -357.6, Some("cat-strom"));
        let mut names = category_names();
        names.insert("cat-strom".into(), "Wohnen - Stromkosten".into());
        let mut buckets = default_category_buckets();
        buckets.insert("wohnen - stromkosten".into(), "fixed".into());
        let cfg = ForecastConfig {
            category_buckets: buckets,
            ..config()
        };
        let result = project_account(5000.0, &txs, &names, &cfg, None, None, &HashMap::new());
        let month = first_month(&result);
        assert!(
            month.fixed_costs > 100.0,
            "strom recurring should map to fixed, got {}",
            month.fixed_costs
        );
    }

    #[test]
    fn config_mapped_salary_never_uses_ai_assignment() {
        use std::collections::HashSet;
        use std::collections::HashMap as Hm;
        use crate::forecast::bucket_inference::{BucketAssignment, BucketSource, feature_id_for_pattern};
        use crate::subscriptions::{ConfirmedRecurring, DetectionResult};

        let txs = monthly_recurring_history("Employer Payroll", 5000.0, Some("cat-salary"));
        let pattern = RecurringPattern {
            description: "employer payroll".into(),
            amount: 5000.0,
            interval_days: 30,
            category_id: Some("cat-salary".into()),
        };
        let fid = feature_id_for_pattern(&pattern);
        let mut fake_ai = Hm::new();
        fake_ai.insert(
            fid.clone(),
            BucketAssignment {
                feature_id: fid,
                bucket: Bucket::Variable,
                confidence: 0.99,
                source: BucketSource::Ai,
                rationale_code: "should_not_apply".into(),
            },
        );
        let ctx = DetectionResult {
            confirmed_recurring: vec![ConfirmedRecurring {
                payee_key: "employer payroll".into(),
                amount: 5000.0,
                interval_days: 30,
                fingerprint: "salary".into(),
            }],
            rejected_fingerprints: HashSet::new(),
            forecast_excluded_rejections: HashSet::new(),
        };
        let result = project_account(
            1000.0,
            &txs,
            &category_names(),
            &config(),
            Some(&ctx),
            None,
            &fake_ai,
        );
        let month = first_month(&result);
        assert!(month.income > 0.0);
        assert_eq!(
            month.bucket_sources.as_ref().map(|s| s.income.as_str()),
            Some("config")
        );
        assert!(!month.ai_mapped);
    }

    #[test]
    fn ai_assignment_applies_on_ambiguous_recurring() {
        use std::collections::HashSet;
        use std::collections::HashMap as Hm;
        use crate::forecast::bucket_inference::{BucketAssignment, BucketSource, feature_id_for_pattern};
        use crate::subscriptions::{ConfirmedRecurring, DetectionResult};

        let txs = monthly_recurring_history("Unknown Merchant", -80.0, None);
        let pattern = RecurringPattern {
            description: "unknown merchant".into(),
            amount: -80.0,
            interval_days: 30,
            category_id: None,
        };
        let fid = feature_id_for_pattern(&pattern);
        let mut assignments = Hm::new();
        assignments.insert(
            fid.clone(),
            BucketAssignment {
                feature_id: fid,
                bucket: Bucket::Fixed,
                confidence: 0.9,
                source: BucketSource::Ai,
                rationale_code: "llm".into(),
            },
        );
        let ctx = DetectionResult {
            confirmed_recurring: vec![ConfirmedRecurring {
                payee_key: "unknown merchant".into(),
                amount: -80.0,
                interval_days: 30,
                fingerprint: "unk".into(),
            }],
            rejected_fingerprints: HashSet::new(),
            forecast_excluded_rejections: HashSet::new(),
        };
        let result = project_account(
            1000.0,
            &txs,
            &category_names(),
            &config(),
            Some(&ctx),
            None,
            &assignments,
        );
        let month = first_month(&result);
        assert!(month.fixed_costs > 0.0);
        assert!(month.ai_mapped);
    }

    #[test]
    fn household_revenue_income_merged_into_asset_monthly() {
        let asset_txs = monthly_recurring_history("Rent Payment", -1000.0, Some("cat-rent"));
        let salary_desc = "Lohn/Gehalt 00133205/202604";
        let salary_txs = monthly_recurring_history(salary_desc, 3200.0, Some("cat-salary"));
        use crate::forecast::recurring::detect_inflow_patterns;
        let patterns = detect_inflow_patterns(&salary_txs, 5.0);
        assert!(!patterns.is_empty(), "salary inflow pattern detected");
        let mut names = category_names();
        names.insert(
            "cat-salary".into(),
            "Einnahmen - regelmäßige Bezüge".into(),
        );
        let mut buckets = default_category_buckets();
        buckets.insert("einnahmen - regelmäßige bezüge".into(), "income".into());
        let cfg = ForecastConfig {
            category_buckets: buckets,
            ..config()
        };
        let household = HouseholdIncomeContext {
            patterns: &patterns,
            reference_transactions: &salary_txs,
        };
        let result = project_account(
            1000.0,
            &asset_txs,
            &names,
            &cfg,
            None,
            Some(&household),
            &HashMap::new(),
        );
        let month = first_month(&result);
        assert!(month.income > 0.0, "household salary income expected");
        assert!(month.fixed_costs > 0.0);
    }
}
