//! OCEL 2.0 schema validation for star-toml.
//!
//! Performs lightweight structural validation of OCEL 2.0 JSON documents
//! before passing them to the wpm oracle.

/// Validate that a JSON string conforms to the OCEL 2.0 schema.
///
/// Checks:
/// - `"ocel:version"` field is present and equals `"2.0"`
/// - `"ocel:events"` field is present
/// - At least one event has `"ocel:activity"` and `"ocel:timestamp"`
///
/// # Errors
///
/// Returns an `Err(String)` describing the first schema violation found.
pub fn validate_ocel_schema(json: &str) -> Result<(), String> {
    // Check ocel:version == "2.0"
    if !json.contains("\"ocel:version\"") {
        return Err("missing required field: \"ocel:version\"".to_string());
    }

    // Verify version value is "2.0"
    // Look for "ocel:version" followed by "2.0" (allowing whitespace and colon)
    let has_correct_version =
        find_string_value(json, "ocel:version").map(|v| v == "2.0").unwrap_or(false);
    if !has_correct_version {
        return Err("\"ocel:version\" must be \"2.0\"".to_string());
    }

    // Check ocel:events exists
    if !json.contains("\"ocel:events\"") {
        return Err("missing required field: \"ocel:events\"".to_string());
    }

    // Check at least one event has ocel:activity and ocel:timestamp
    if !json.contains("\"ocel:activity\"") {
        return Err("no events found with required field \"ocel:activity\"".to_string());
    }
    if !json.contains("\"ocel:timestamp\"") {
        return Err("no events found with required field \"ocel:timestamp\"".to_string());
    }

    Ok(())
}

/// Naively extract a JSON string value for a given key.
///
/// Looks for `"key": "value"` patterns (with optional whitespace).
fn find_string_value<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let key_pos = json.find(&needle)?;
    let after_key = &json[key_pos + needle.len()..];

    // Skip whitespace and colon
    let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();

    // Expect a quoted string
    let inner = after_colon.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(&inner[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ocel_passes() {
        let json = r#"{"ocel:version":"2.0","ocel:events":[{"ocel:activity":"load","ocel:timestamp":"2024-01-01T00:00:00Z"}]}"#;
        assert!(validate_ocel_schema(json).is_ok());
    }

    #[test]
    fn missing_version_fails() {
        let json =
            r#"{"ocel:events":[{"ocel:activity":"load","ocel:timestamp":"2024-01-01T00:00:00Z"}]}"#;
        let err = validate_ocel_schema(json).unwrap_err();
        assert!(err.contains("ocel:version"));
    }

    #[test]
    fn wrong_version_fails() {
        let json = r#"{"ocel:version":"1.0","ocel:events":[{"ocel:activity":"load","ocel:timestamp":"2024-01-01T00:00:00Z"}]}"#;
        let err = validate_ocel_schema(json).unwrap_err();
        assert!(err.contains("2.0"));
    }

    #[test]
    fn missing_events_fails() {
        let json = r#"{"ocel:version":"2.0"}"#;
        let err = validate_ocel_schema(json).unwrap_err();
        assert!(err.contains("ocel:events"));
    }
}
