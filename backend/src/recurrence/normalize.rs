pub fn payee_key(description: &str) -> String {
    let mut s = description.trim().to_lowercase();
    s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    strip_trailing_tokens(&mut s);
    normalize_inflow_payroll_prefix(&mut s);
    s
}

/// Collapse variable payroll period suffixes (e.g. `lohn/gehalt 00133205/202604`).
fn normalize_inflow_payroll_prefix(s: &mut String) {
    if s.starts_with("lohn/gehalt") {
        *s = "lohn/gehalt".into();
    }
}

fn strip_trailing_tokens(s: &mut String) {
    for _ in 0..3 {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }
        let last = parts[parts.len() - 1];
        if last.len() >= 4
            && last.chars().any(|c| c.is_ascii_digit())
            && last.chars().all(|c| c.is_ascii_alphanumeric())
        {
            *s = parts[..parts.len() - 1].join(" ");
        } else {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_reference_codes() {
        assert_eq!(payee_key("Netflix P3E460"), "netflix");
        assert_eq!(payee_key("  Rent   Payment  "), "rent payment");
    }

    #[test]
    fn collapses_payroll_period_suffixes() {
        assert_eq!(
            payee_key("Lohn/Gehalt 00133205/202604"),
            payee_key("Lohn/Gehalt 00133205/202603")
        );
        assert_eq!(payee_key("Lohn/Gehalt 00133205/202604"), "lohn/gehalt");
    }
}
