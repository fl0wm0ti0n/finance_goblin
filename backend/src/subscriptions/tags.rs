use uuid::Uuid;

pub fn normalize_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

pub fn validate_tag_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("name is required");
    }
    if trimmed.len() > 64 {
        return Err("name too long");
    }
    Ok(())
}

pub fn parse_tag_ids(ids: &[String]) -> Result<Vec<Uuid>, uuid::Error> {
    ids.iter().map(|s| Uuid::parse_str(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_normalization() {
        assert_eq!(normalize_slug("  Luxus Tag "), "luxus-tag");
        assert_eq!(normalize_slug("important"), "important");
    }

    #[test]
    fn validate_tag_name_rejects_empty() {
        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("  ").is_err());
        assert!(validate_tag_name("ok").is_ok());
    }
}
