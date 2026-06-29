use star_toml_json_schema::{
    export_to_json_schema, import_json_schema, validate_json_schema_standing,
};

#[test]
fn test_valid_schema_import_and_export() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "Star TOML System Config schema.",
      "type": "object",
      "properties": {
        "log_path": {
          "type": "string",
          "description": "Absolute or relative path to write lifecycle events."
        },
        "cache_size": {
          "type": "integer",
          "minimum": 1024,
          "default": 10485760,
          "description": "Maximum memory cached config in bytes (default: 10485760).",
          "x-counterexamples": {
            "minimum": "SCH-ERR-MIN-CACHE"
          }
        }
      },
      "required": ["log_path"],
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_boundary",
        "may_control": ["log_path", "cache_size"],
        "must_not_claim": ["root_authority"]
      },
      "x-override-policy": {
        "id": "sys_override",
        "allowed": true,
        "required_reason": true,
        "required_scope": ["local"]
      }
    }"#;

    let candidate = import_json_schema(schema_json, true).unwrap();
    assert_eq!(candidate.title, "system");
    assert_eq!(candidate.domain.fields.len(), 2);

    let diags = validate_json_schema_standing(&candidate);
    assert!(diags.is_empty(), "Expected no standing diagnostics: {:?}", diags);

    // Test export
    let (exported_json, export_diags) = export_to_json_schema(&candidate.domain).unwrap();
    assert!(export_diags.is_empty());
    assert!(exported_json.contains("\"title\": \"system\""));
    assert!(exported_json.contains("\"minimum\": 1024"));
}

#[test]
fn test_feature_disabled() {
    let schema_json = "{}";
    let res = import_json_schema(schema_json, false);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-001"));
}

#[test]
fn test_invalid_schema_parse() {
    let schema_json = "{ invalid: json }";
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-002"));
}

#[test]
fn test_unsupported_dialect() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-04/schema#",
      "title": "system",
      "description": "System config",
      "type": "object",
      "properties": {},
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-016"));
}

#[test]
fn test_allows_unbounded_additional_properties() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System config",
      "type": "object",
      "properties": {},
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-007"));
}

#[test]
fn test_missing_description() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "type": "object",
      "properties": {
        "log_path": {
          "type": "string"
        }
      },
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-006"));
}

#[test]
fn test_authority_boundary_missing() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System Config",
      "type": "object",
      "properties": {},
      "additionalProperties": false
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-015"));
}

#[test]
fn test_default_undocumented() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System Config",
      "type": "object",
      "properties": {
        "log_path": {
          "type": "string",
          "default": "/var/log",
          "description": "Log file path without mentioning the val."
        }
      },
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-013"));
}

#[test]
fn test_constraint_without_counterexample() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System Config",
      "type": "object",
      "properties": {
        "cache_size": {
          "type": "integer",
          "minimum": 1024,
          "description": "Cache size in bytes (default: 10485760)."
        }
      },
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-014"));
}

#[test]
fn test_unsupported_keyword() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System Config",
      "type": "object",
      "properties": {},
      "patternProperties": {
        "^foo": { "type": "string" }
      },
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-003"));
}

#[test]
fn test_reference_unresolved() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System Config",
      "type": "object",
      "properties": {
        "log_path": {
          "$ref": "missing_ref.json"
        }
      },
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-004"));
}

#[test]
fn test_composition_ambiguous() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "description": "System Config",
      "type": "object",
      "properties": {
        "log_path": {
          "oneOf": [
            { "type": "string" },
            { "type": "integer" }
          ]
        }
      },
      "additionalProperties": false,
      "x-authority-boundary": {
        "id": "sys_b",
        "may_control": ["a"]
      }
    }"#;
    let res = import_json_schema(schema_json, true);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "SCH-JSON-012"));
}

#[test]
fn test_validate_config_against_json_schema() {
    let schema_json = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "system",
      "type": "object",
      "properties": {
        "port": {
          "type": "integer",
          "minimum": 1024
        }
      },
      "additionalProperties": false
    }"#;

    let valid_config = serde_json::json!({
        "port": 8080
    });
    assert!(star_toml_json_schema::validate_config_against_json_schema(schema_json, &valid_config)
        .is_ok());

    let invalid_config = serde_json::json!({
        "port": 80
    });
    let res =
        star_toml_json_schema::validate_config_against_json_schema(schema_json, &invalid_config);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs
        .iter()
        .any(|e| e.contains("is less than the minimum of 1024") || e.contains("1024")));
}
