use std::collections::HashMap;

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

pub fn accumulate_bucket(entry: &mut super::types::MonthlyCashflow, bucket: Bucket, amount: f64) {
    match (bucket, amount) {
        (Bucket::Income, a) if a > 0.0 => entry.income += a,
        (Bucket::Fixed, a) if a < 0.0 => entry.fixed_costs += a.abs(),
        (Bucket::Variable, a) if a < 0.0 => entry.variable_costs += a.abs(),
        (Bucket::Income, a) if a < 0.0 => entry.variable_costs += a.abs(),
        _ => {}
    }
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
        };
        let mut names = HashMap::new();
        names.insert("42".into(), "Salary".into());
        assert_eq!(resolve_bucket(Some("42"), &names, &config), Bucket::Income);
    }
}
