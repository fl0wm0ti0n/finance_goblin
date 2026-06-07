use std::collections::HashMap;

use super::bucket_inference::{BucketAssignment, BucketSource, feature_id_for_pattern, is_ai_eligible};
use super::types::{BucketSourceMass, MonthlyBucketSources, RecurringPattern};
use crate::config::ForecastConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bucket {
    Income,
    Fixed,
    Variable,
}

impl Bucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Bucket::Income => "income",
            Bucket::Fixed => "fixed",
            Bucket::Variable => "variable",
        }
    }
}

pub fn map_category(category_name: Option<&str>, config: &ForecastConfig) -> Bucket {
    let name = category_name.unwrap_or("").trim().to_lowercase();
    if name.is_empty() {
        return Bucket::Variable;
    }
    match config
        .category_buckets
        .get(&name)
        .map(|s| s.as_str())
    {
        Some("income") => Bucket::Income,
        Some("fixed") => Bucket::Fixed,
        _ => Bucket::Variable,
    }
}

pub fn resolve_bucket(
    category_id: Option<&str>,
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
) -> Bucket {
    let name = category_id
        .and_then(|id| category_names.get(id))
        .map(|s| s.as_str());
    map_category(name, config)
}

/// DEC-0078: config precedence first; AI only on config-map miss for ambiguous rows.
pub fn resolve_bucket_with_ai(
    pattern: &RecurringPattern,
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
    assignments: &HashMap<String, BucketAssignment>,
) -> (Bucket, BucketSource) {
    if !is_ai_eligible(pattern.category_id.as_deref(), category_names, config) {
        let bucket = resolve_bucket(pattern.category_id.as_deref(), category_names, config);
        return (bucket, BucketSource::Config);
    }

    let fid = feature_id_for_pattern(pattern);
    if let Some(assignment) = assignments.get(&fid) {
        return (assignment.bucket, assignment.source);
    }

    (Bucket::Variable, BucketSource::Default)
}

pub fn accumulate_bucket(entry: &mut super::types::MonthlyCashflow, bucket: Bucket, amount: f64) {
    match (bucket, amount) {
        (Bucket::Income, a) if a > 0.0 => entry.income += a,
        (Bucket::Fixed, a) if a < 0.0 => entry.fixed_costs += a.abs(),
        (Bucket::Variable, a) if a < 0.0 => entry.variable_costs += a.abs(),
        (Bucket::Income, a) if a < 0.0 => entry.variable_costs += a.abs(),
        _ => {}
    }
}

#[derive(Debug, Clone, Default)]
pub struct MonthlyProvenanceTracker {
    pub income: BucketSourceMass,
    pub fixed: BucketSourceMass,
    pub variable: BucketSourceMass,
    pub ai_mapped: bool,
}

impl MonthlyProvenanceTracker {
    pub fn track(&mut self, bucket: Bucket, mass: f64, source: &str) {
        let target = match bucket {
            Bucket::Income => &mut self.income,
            Bucket::Fixed => &mut self.fixed,
            Bucket::Variable => &mut self.variable,
        };
        match source {
            "config" => target.config += mass,
            "ai" => target.ai += mass,
            _ => target.default += mass,
        }
    }

    pub fn finalize(&self) -> MonthlyBucketSources {
        MonthlyBucketSources {
            income: dominant_label(&self.income),
            fixed_costs: dominant_label(&self.fixed),
            variable_costs: dominant_label(&self.variable),
        }
    }
}

fn dominant_label(mass: &BucketSourceMass) -> String {
    if mass.config > 0.0 {
        "config".into()
    } else if mass.ai > 0.0 {
        "ai".into()
    } else {
        "default".into()
    }
}

pub fn accumulate_bucket_with_source(
    entry: &mut super::types::MonthlyCashflow,
    bucket: Bucket,
    amount: f64,
    source: BucketSource,
    provenance: &mut MonthlyProvenanceTracker,
) {
    let mass = amount.abs();
    if mass <= 0.0 {
        return;
    }
    match source {
        BucketSource::Config => provenance.track(bucket, mass, "config"),
        BucketSource::Ai => provenance.track(bucket, mass, "ai"),
        BucketSource::Default => provenance.track(bucket, mass, "default"),
    }
    if source == BucketSource::Ai {
        provenance.ai_mapped = true;
    }
    accumulate_bucket(entry, bucket, amount);
}

pub fn default_category_buckets() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("salary".into(), "income".into());
    m.insert("payroll".into(), "income".into());
    m.insert("rent".into(), "fixed".into());
    m.insert("mortgage".into(), "fixed".into());
    m.insert("insurance".into(), "fixed".into());
    m.insert("utilities".into(), "fixed".into());
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ForecastConfig;

    #[test]
    fn maps_known_categories() {
        let config = ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: default_category_buckets(),
            ai_bucket_min_confidence: 0.75,
        };
        assert_eq!(map_category(Some("Salary"), &config), Bucket::Income);
        assert_eq!(map_category(Some("Rent"), &config), Bucket::Fixed);
        assert_eq!(map_category(Some("Groceries"), &config), Bucket::Variable);
    }

    #[test]
    fn resolve_bucket_uses_category_name_map() {
        let config = ForecastConfig {
            rolling_window_days: 90,
            sparse_history_days: 90,
            retention_count: 5,
            recurring_amount_tolerance_pct: 5.0,
            category_buckets: default_category_buckets(),
            ai_bucket_min_confidence: 0.75,
        };
        let mut names = HashMap::new();
        names.insert("42".into(), "Salary".into());
        assert_eq!(resolve_bucket(Some("42"), &names, &config), Bucket::Income);
    }
}
