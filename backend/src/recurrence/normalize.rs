pub fn payee_key(description: &str) -> String {
    let mut s = description.trim().to_lowercase();
    s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    strip_sepa_reference_tokens(&mut s);
    strip_trailing_tokens(&mut s);
    collapse_legal_entity_suffixes(&mut s);
    normalize_inflow_payroll_prefix(&mut s);
    apply_card_billing_rules(&mut s);
    s
}

/// DEC-0084 card/PSP billing descriptor rules — after DEC-0072 SEPA passes.
fn apply_card_billing_rules(s: &mut String) {
    if let Some(idx) = s.find('*') {
        *s = s[..idx].trim().to_string();
    }
    if let Some(idx) = s.find(',') {
        *s = s[..idx].trim().to_string();
    }
    if collapse_billing_root_alias(s) {
        return;
    }
    strip_domain_tail(s);
}

fn collapse_billing_root_alias(s: &mut String) -> bool {
    const APPLE_ROOTS: &[&str] = &["apple.com/bill", "itunes.com", "apple.com/bill itunes"];
    if APPLE_ROOTS.iter().any(|root| s.as_str() == *root) || s.starts_with("apple.com/bill") {
        *s = "apple".into();
        return true;
    }
    false
}

fn strip_domain_tail(s: &mut String) {
    if s.ends_with("/bill") {
        *s = s.strip_suffix("/bill").unwrap_or(s).trim().to_string();
    }
    if let Some(stripped) = s.strip_suffix(".com") {
        if !stripped.is_empty()
            && stripped
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            *s = stripped.to_string();
        }
    }
}

/// Strip SEPA reference prefixes (SVWZ+, EREF+, etc.) and leading reference tokens.
fn strip_sepa_reference_tokens(s: &mut String) {
    for prefix in ["svwz+", "svwz ", "eref+", "eref ", "kref+", "kref ", "cred+", "cred "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            *s = rest.trim().to_string();
            break;
        }
    }
    if s.starts_with("svwz") && s.len() > 4 {
        let rest = s[4..].trim_start_matches(['+', ' ', '-']);
        if !rest.is_empty() {
            *s = rest.to_string();
        }
    }
    strip_leading_reference_tokens(s);
    strip_sepa_noise_tokens(s);
}

fn strip_sepa_noise_tokens(s: &mut String) {
    const NOISE: &[&str] = &[
        "ueberweisung",
        "überweisung",
        "lastschrift",
        "monatlich",
        "monat",
        "dauerauftrag",
        "sepa",
    ];
    let parts: Vec<&str> = s
        .split_whitespace()
        .filter(|p| !NOISE.contains(p))
        .collect();
    *s = parts.join(" ");
}

fn strip_leading_reference_tokens(s: &mut String) {
    loop {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        let first = parts[0];
        let is_reference = first.starts_with("ref")
            || (first.len() >= 4
                && first.chars().any(|c| c.is_ascii_digit())
                && first.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '/')));
        if is_reference && parts.len() > 1 {
            *s = parts[1..].join(" ");
        } else {
            return;
        }
    }
}

fn collapse_legal_entity_suffixes(s: &mut String) {
    const SUFFIXES: &[&str] = &["gmbh", "ab", "ag", "kg", "ug", "llc", "inc", "ltd", "co"];
    loop {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }
        let last = parts[parts.len() - 1];
        if SUFFIXES.contains(&last) {
            *s = parts[..parts.len() - 1].join(" ");
        } else {
            return;
        }
    }
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

    #[test]
    fn sepa_reference_tokens_merge_to_single_payee_key() {
        assert_eq!(
            payee_key("SVWZ+REF123456 Netflix Streaming"),
            payee_key("Netflix Streaming")
        );
        assert_eq!(
            payee_key("SVWZ+MONATlich Spotify Premium"),
            payee_key("Spotify Premium")
        );
    }

    #[test]
    fn legal_entity_suffixes_collapse() {
        assert_eq!(payee_key("Spotify AB"), "spotify");
        assert_eq!(payee_key("Acme Services GmbH"), "acme services");
    }

    #[test]
    fn card_billing_asterisk_split() {
        assert_eq!(payee_key("DBA*Plan"), "dba");
        assert_eq!(payee_key("AcmeInc*Gold Plan"), "acmeinc");
    }

    #[test]
    fn card_billing_comma_memo_left_segment() {
        assert_eq!(
            payee_key("CURSOR, AI POWERED IDE, CURSOR.COM"),
            payee_key("CURSOR")
        );
        assert_eq!(payee_key("Netflix, Streaming"), "netflix");
    }

    #[test]
    fn card_billing_apple_roots_collapse() {
        assert_eq!(payee_key("APPLE.COM/BILL"), "apple");
        assert_eq!(payee_key("ITUNES.COM"), "apple");
        assert_eq!(payee_key("apple.com/bill itunes"), "apple");
    }

    #[test]
    fn card_billing_domain_tail_strips_com_and_bill() {
        assert_eq!(payee_key("CURSOR.COM"), "cursor");
        assert_eq!(payee_key("cursor.com/bill"), "cursor");
    }

    #[test]
    fn sepa_rules_unchanged_for_non_card_paths() {
        assert_eq!(payee_key("Spotify AB"), "spotify");
        assert_eq!(
            payee_key("SVWZ+REF123456 Netflix Streaming"),
            payee_key("Netflix Streaming")
        );
    }
}
