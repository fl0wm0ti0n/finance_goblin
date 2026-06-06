use crate::config::SubscriptionsConfig;

#[derive(Debug, Clone)]
pub struct PriceChangeConfig {
    pub min_eur: f64,
    pub min_pct: f64,
}

impl From<&SubscriptionsConfig> for PriceChangeConfig {
    fn from(c: &SubscriptionsConfig) -> Self {
        Self {
            min_eur: c.price_change_min_eur,
            min_pct: c.price_change_min_pct,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriceChangeKind {
    None,
    Increase,
    Decrease,
    IntervalChange,
}

pub fn is_material_change(previous: f64, current: f64, config: &PriceChangeConfig) -> bool {
    let delta = (current - previous).abs();
    if delta < config.min_eur {
        return false;
    }
    if previous.abs() < 0.01 {
        return false;
    }
    let pct = delta / previous.abs() * 100.0;
    pct >= config.min_pct
}

pub fn classify_price_change(
    previous: f64,
    current: f64,
    config: &PriceChangeConfig,
) -> PriceChangeKind {
    if !is_material_change(previous, current, config) {
        return PriceChangeKind::None;
    }
    if current > previous {
        PriceChangeKind::Increase
    } else {
        PriceChangeKind::Decrease
    }
}

pub fn delta_pct(previous: f64, current: f64) -> f64 {
    if previous.abs() < 0.01 {
        return 0.0;
    }
    (current - previous) / previous.abs() * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> PriceChangeConfig {
        PriceChangeConfig {
            min_eur: 1.0,
            min_pct: 5.0,
        }
    }

    #[test]
    fn dual_threshold_requires_both() {
        assert_eq!(
            classify_price_change(-10.0, -10.50, &cfg()),
            PriceChangeKind::None
        );
        assert_eq!(
            classify_price_change(-10.0, -12.0, &cfg()),
            PriceChangeKind::Decrease
        );
    }

    #[test]
    fn small_absolute_change_ignored() {
        assert_eq!(
            classify_price_change(-5.0, -5.50, &cfg()),
            PriceChangeKind::None
        );
    }
}
