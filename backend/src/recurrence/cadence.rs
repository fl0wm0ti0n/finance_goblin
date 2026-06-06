#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenceKind {
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
    Annual,
    Unknown,
}

pub fn median_interval_days(intervals: &[i64]) -> i64 {
    if intervals.is_empty() {
        return 0;
    }
    let mut sorted = intervals.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

pub fn classify_cadence(median: i64) -> CadenceKind {
    match median {
        d if (6..=8).contains(&d) => CadenceKind::Weekly,
        d if (13..=16).contains(&d) => CadenceKind::Biweekly,
        d if (27..=32).contains(&d) => CadenceKind::Monthly,
        d if (85..=95).contains(&d) => CadenceKind::Quarterly,
        d if (350..=380).contains(&d) => CadenceKind::Annual,
        _ => CadenceKind::Unknown,
    }
}

pub fn cadence_tolerance(median: i64) -> i64 {
    match median {
        d if (6..=8).contains(&d) => 2,
        d if (13..=16).contains(&d) => 3,
        d if (27..=32).contains(&d) => 4,
        d if (85..=95).contains(&d) => 7,
        d if (350..=380).contains(&d) => 14,
        _ => 0,
    }
}

pub fn is_stable_cadence(median: i64, intervals: &[i64]) -> bool {
    if classify_cadence(median) == CadenceKind::Unknown {
        return false;
    }
    let tolerance = cadence_tolerance(median);
    intervals.iter().all(|i| (i - median).abs() <= tolerance)
}

pub fn interval_outlier_count(median: i64, intervals: &[i64]) -> usize {
    let tolerance = cadence_tolerance(median);
    intervals
        .iter()
        .filter(|i| (*i - median).abs() > tolerance)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_monthly_cadence() {
        let intervals = vec![28, 30, 31, 29];
        let median = median_interval_days(&intervals);
        assert!(is_stable_cadence(median, &intervals));
        assert_eq!(classify_cadence(median), CadenceKind::Monthly);
    }
}
