use std::collections::BTreeMap;

use serde_json::Value;
use star_toml::{
    docs::{
        AuthorityBoundary, ConfigDomain, ConfigField, ConfigValidator, Counterexample,
        OverridePolicy,
    },
    Schema,
};

use crate::diagnostics::SchJsonDiagnostic;

#[derive(Debug, Clone)]
pub struct SchemaCandidate {
    pub domain_id: String,
    pub title: String,
    pub description: String,
    pub schema: Schema,
    pub domain: ConfigDomain,
    pub digest: String,
    pub admitted: bool,
}

pub fn import_json_schema(
    schema_json: &str,
    feature_enabled: bool,
) -> Result<SchemaCandidate, Vec<SchJsonDiagnostic>> {
    let mut diags = Vec::new();

    if !feature_enabled {
        diags.push(SchJsonDiagnostic::feature_disabled());
        return Err(diags);
    }

    let root_val: Value = match serde_json::from_str(schema_json) {
        Ok(v) => v,
        Err(e) => {
            diags.push(SchJsonDiagnostic::parse_failed(&e.to_string()));
            return Err(diags);
        }
    };

    // Check $schema dialect (SCH-JSON-016)
    if let Some(schema_uri) = root_val.get("$schema") {
        if let Some(uri_str) = schema_uri.as_str() {
            if !uri_str.contains("draft-07")
                && !uri_str.contains("draft-06")
                && !uri_str.contains("draft/2019-09")
                && !uri_str.contains("draft/2020-12")
            {
                diags.push(SchJsonDiagnostic::unsupported_dialect(uri_str));
            }
        }
    }

    // Check unresolved references (SCH-JSON-004)
    check_references(&root_val, &mut diags);

    // Check ambiguous composition (SCH-JSON-012)
    check_composition(&root_val, &mut diags);

    // Check unsupported keywords (SCH-JSON-003)
    check_unsupported_keywords(&root_val, &mut diags);

    // Assert type is object (SCH-JSON-003)
    if let Some(t) = root_val.get("type") {
        if t.as_str() != Some("object") {
            diags.push(SchJsonDiagnostic::unsupported_keyword("non-object root type"));
        }
    }

    // Check additionalProperties (SCH-JSON-007)
    let additional_props = root_val.get("additionalProperties");
    if additional_props.is_none() || additional_props.and_then(|v| v.as_bool()) != Some(false) {
        diags.push(SchJsonDiagnostic::allows_unbounded_additional_properties());
    }

    let title = root_val.get("title").and_then(|t| t.as_str()).unwrap_or("system").to_string();

    let description =
        root_val.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();

    // SCH-JSON-006 Check root description
    if description.trim().is_empty() {
        diags.push(SchJsonDiagnostic::missing_descriptions(&title));
    }

    // Extract authority boundary (SCH-JSON-015)
    let boundary = if let Some(b_val) = root_val.get("x-authority-boundary") {
        let b_id = b_val.get("id").and_then(|id| id.as_str()).unwrap_or("boundary").to_string();
        let may_control = parse_str_list(b_val.get("may_control"));
        let must_not_claim = parse_str_list(b_val.get("must_not_claim"));

        AuthorityBoundary {
            id: b_id,
            domain: title.clone(),
            may_control,
            must_not_claim,
            defer_to: b_val.get("defer_to").and_then(|d| d.as_str().map(|s| s.to_string())),
            escalation_rule: b_val
                .get("escalation_rule")
                .and_then(|e| e.as_str().map(|s| s.to_string())),
        }
    } else {
        diags.push(SchJsonDiagnostic::authority_boundary_missing(&title));
        // Placeholder to proceed
        AuthorityBoundary {
            id: "missing".to_string(),
            domain: title.clone(),
            may_control: vec![],
            must_not_claim: vec![],
            defer_to: None,
            escalation_rule: None,
        }
    };

    // Extract override policy
    let override_policy = if let Some(o_val) = root_val.get("x-override-policy") {
        let o_id = o_val.get("id").and_then(|id| id.as_str()).unwrap_or("override").to_string();
        let allowed = o_val.get("allowed").and_then(|a| a.as_bool()).unwrap_or(true);
        let required_reason =
            o_val.get("required_reason").and_then(|r| r.as_bool()).unwrap_or(true);
        let required_scope = parse_str_list(o_val.get("required_scope"));

        OverridePolicy {
            id: o_id,
            allowed,
            required_reason,
            required_scope,
            required_expiration: o_val
                .get("required_expiration")
                .and_then(|x| x.as_str().map(|s| s.to_string())),
            required_tracking: o_val
                .get("required_tracking")
                .and_then(|t| t.as_str().map(|s| s.to_string())),
            required_replacement_plan: o_val
                .get("required_replacement_plan")
                .and_then(|p| p.as_bool())
                .unwrap_or(false),
            refusal_conditions: parse_str_list(o_val.get("refusal_conditions")),
        }
    } else {
        OverridePolicy {
            id: "default_override".to_string(),
            allowed: true,
            required_reason: true,
            required_scope: vec!["env".to_string()],
            required_expiration: None,
            required_tracking: None,
            required_replacement_plan: false,
            refusal_conditions: vec![],
        }
    };

    let required_fields: Vec<String> = root_val
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let mut properties = BTreeMap::new();
    if let Some(props_val) = root_val.get("properties") {
        if let Some(obj) = props_val.as_object() {
            // Read into BTreeMap to enforce alphabetical sorting (SCH-JSON-005 / determinism)
            for (k, v) in obj {
                properties.insert(k.clone(), v.clone());
            }
        }
    }

    let mut schema_builder = Schema::new();
    schema_builder = schema_builder
        .domain_metadata(&title, &title, &description)
        .stability("stable")
        .authority_boundary(boundary.clone());

    let mut fields = Vec::new();
    let mut validators = Vec::new();
    let mut counterexamples = Vec::new();

    for (prop_name, prop_val) in &properties {
        let is_required = required_fields.contains(prop_name);
        let prop_desc =
            prop_val.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();

        // SCH-JSON-006 Missing property description
        if prop_desc.trim().is_empty() {
            diags.push(SchJsonDiagnostic::missing_descriptions(prop_name));
        }

        let prop_type_str = prop_val.get("type").and_then(|t| t.as_str()).unwrap_or("string");
        let rust_type = match prop_type_str {
            "string" => "String",
            "integer" => "integer",
            "boolean" => "boolean",
            "number" => "float",
            other => {
                diags.push(SchJsonDiagnostic::unsupported_keyword(other));
                "String"
            }
        };

        let default_val = prop_val.get("default").map(|d| match d {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        });

        // SCH-JSON-013 Check default undocumented in description
        if default_val.is_some() {
            if !prop_desc.to_lowercase().contains("default") {
                diags.push(SchJsonDiagnostic::default_undocumented(prop_name));
            }
        }

        let allowed_values = if let Some(enum_val) = prop_val.get("enum") {
            enum_val
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Constraint checking
        let mut validator_refs = Vec::new();
        let mut counterexample_refs = Vec::new();

        if let Some(min_val) = prop_val.get("minimum") {
            let limit = min_val.as_i64().unwrap_or(0);

            // Check SCH-JSON-014 (Constraint without counterexample)
            let has_counterexample =
                prop_val.get("x-counterexamples").and_then(|c| c.get("minimum")).is_some();
            if !has_counterexample {
                diags.push(SchJsonDiagnostic::constraint_without_counterexample(prop_name));
            } else {
                let code = prop_val
                    .get("x-counterexamples")
                    .and_then(|c| c.get("minimum"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("SCH-ERR-MIN");

                let val_id = format!("{}_min_val", prop_name);
                validator_refs.push(val_id.clone());
                counterexample_refs.push(code.to_string());

                validators.push(ConfigValidator {
                    id: val_id.clone(),
                    name: format!("{} minimum limit", prop_name),
                    predicate: format!("val >= {}", limit),
                    description: format!("Ensures {} is at least {}", prop_name, limit),
                    applies_to: vec![prop_name.clone()],
                    failure_counterexamples: vec![code.to_string()],
                    severity: "Error".to_string(),
                });

                counterexamples.push(Counterexample {
                    code: code.to_string(),
                    name: format!("{}BelowMinimum", prop_name),
                    description: format!(
                        "Field {} is below the configured minimum of {}",
                        prop_name, limit
                    ),
                    severity: "Error".to_string(),
                    detection_rule: format!("val < {}", limit),
                    repair_hint: format!("Increase {} to at least {}", prop_name, limit),
                });
            }
        }

        let field_id = format!("{}.{}", title, prop_name);
        fields.push(ConfigField {
            id: field_id,
            domain_id: title.clone(),
            name: prop_name.clone(),
            r#type: rust_type.to_string(),
            required: is_required,
            default: default_val,
            allowed_values,
            forbidden_values: vec![],
            description: prop_desc,
            validator_refs,
            counterexample_refs,
            override_policy_ref: Some(override_policy.id.clone()),
            source_binding: "json-schema".to_string(),
            stability: "stable".to_string(),
        });
    }

    let domain = ConfigDomain {
        id: title.clone(),
        name: title.clone(),
        description: description.clone(),
        authority_boundary: boundary,
        fields,
        validators,
        counterexamples,
        override_policy,
        receipt_binding: None,
        stability: "stable".to_string(),
    };

    if !diags.is_empty() {
        return Err(diags);
    }

    let serialized = serde_json::to_string(&domain).unwrap_or_default();
    let digest = star_toml::reports::blake3_hex(serialized.as_bytes());

    Ok(SchemaCandidate {
        domain_id: title.clone(),
        title,
        description,
        schema: schema_builder,
        domain,
        digest,
        admitted: false,
    })
}

fn check_references(val: &Value, diags: &mut Vec<SchJsonDiagnostic>) {
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            if k == "$ref" {
                if let Some(ref_str) = v.as_str() {
                    // Check unresolved pointers
                    if !ref_str.starts_with("#/") && !ref_str.starts_with("http") {
                        diags.push(SchJsonDiagnostic::reference_unresolved(ref_str));
                    }
                }
            }
            check_references(v, diags);
        }
    } else if let Some(arr) = val.as_array() {
        for v in arr {
            check_references(v, diags);
        }
    }
}

fn check_composition(val: &Value, diags: &mut Vec<SchJsonDiagnostic>) {
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            if k == "anyOf" || k == "oneOf" {
                diags.push(SchJsonDiagnostic::composition_ambiguous(k));
            }
            check_composition(v, diags);
        }
    } else if let Some(arr) = val.as_array() {
        for v in arr {
            check_composition(v, diags);
        }
    }
}

fn check_unsupported_keywords(val: &Value, diags: &mut Vec<SchJsonDiagnostic>) {
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            if k == "patternProperties" || k == "dependencies" {
                diags.push(SchJsonDiagnostic::unsupported_keyword(k));
            }
            check_unsupported_keywords(v, diags);
        }
    } else if let Some(arr) = val.as_array() {
        for v in arr {
            check_unsupported_keywords(v, diags);
        }
    }
}

fn parse_str_list(val: Option<&Value>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}
