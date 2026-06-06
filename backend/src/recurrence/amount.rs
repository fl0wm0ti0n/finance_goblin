pub fn median_amount(amounts: &[f64]) -> f64 {
    if amounts.is_empty() {
        return 0.0;
    }
    let mut sorted = amounts.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted[sorted.len() / 2]
}

pub fn amounts_within_tolerance(amounts: &[f64], median: f64, tolerance_pct: f64) -> bool {
    if median.abs() < 0.01 {
        return false;
    }
    let tolerance = median.abs() * (tolerance_pct / 100.0);
    amounts.iter().all(|a| (a - median).abs() <= tolerance)
}

pub fn coefficient_of_variation(amounts: &[f64]) -> f64 {
    if amounts.is_empty() {
        return 0.0;
    }
    let mean = amounts.iter().sum::<f64>() / amounts.len() as f64;
    if mean.abs() < 0.01 {
        return 0.0;
    }
    let variance = amounts
        .iter()
        .map(|a| {
            let d = a - mean;
            d * d
        })
        .sum::<f64>()
        / amounts.len() as f64;
    variance.sqrt() / mean.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cv_near_zero_for_identical_amounts() {
        let amounts = vec![-100.0, -100.0, -100.0];
        assert!(coefficient_of_variation(&amounts) < 0.02);
    }
}
