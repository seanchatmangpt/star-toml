use serde_json::{Map, Value};
use star_toml::docs::ConfigDomain;

use crate::diagnostics::SchJsonDiagnostic;

pub fn export_to_json_schema(
    domain: &ConfigDomain,
) -> Result<(String, Vec<SchJsonDiagnostic>), Vec<SchJsonDiagnostic>> {
    let mut diags = Vec::new();
    let mut root = Map::new();

    root.insert(
        "$schema".to_string(),
        Value::String("http://json-schema.org/draft-07/schema#".to_string()),
    );
    root.insert("title".to_string(), Value::String(domain.id.clone()));
    root.insert("description".to_string(), Value::String(domain.description.clone()));
    root.insert("type".to_string(), Value::String("object".to_string()));
    root.insert("additionalProperties".to_string(), Value::Bool(false));

    // Custom metadata preservation
    let mut boundary_val = Map::new();
    boundary_val.insert("id".to_string(), Value::String(domain.authority_boundary.id.clone()));
    boundary_val.insert(
        "may_control".to_string(),
        Value::Array(
            domain
                .authority_boundary
                .may_control
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    boundary_val.insert(
        "must_not_claim".to_string(),
        Value::Array(
            domain
                .authority_boundary
                .must_not_claim
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    if let Some(ref d) = domain.authority_boundary.defer_to {
        boundary_val.insert("defer_to".to_string(), Value::String(d.clone()));
    }
    if let Some(ref e) = domain.authority_boundary.escalation_rule {
        boundary_val.insert("escalation_rule".to_string(), Value::String(e.clone()));
    }
    root.insert("x-authority-boundary".to_string(), Value::Object(boundary_val));

    let mut override_val = Map::new();
    override_val.insert("id".to_string(), Value::String(domain.override_policy.id.clone()));
    override_val.insert("allowed".to_string(), Value::Bool(domain.override_policy.allowed));
    override_val
        .insert("required_reason".to_string(), Value::Bool(domain.override_policy.required_reason));
    override_val.insert(
        "required_scope".to_string(),
        Value::Array(
            domain
                .override_policy
                .required_scope
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    if let Some(ref ex) = domain.override_policy.required_expiration {
        override_val.insert("required_expiration".to_string(), Value::String(ex.clone()));
    }
    if let Some(ref t) = domain.override_policy.required_tracking {
        override_val.insert("required_tracking".to_string(), Value::String(t.clone()));
    }
    override_val.insert(
        "required_replacement_plan".to_string(),
        Value::Bool(domain.override_policy.required_replacement_plan),
    );
    override_val.insert(
        "refusal_conditions".to_string(),
        Value::Array(
            domain
                .override_policy
                .refusal_conditions
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect(),
        ),
    );
    root.insert("x-override-policy".to_string(), Value::Object(override_val));

    let mut properties = Map::new();
    let mut required = Vec::new();

    // Iterate in sorted field order (P0.11 / Deterministic)
    let mut fields = domain.fields.clone();
    fields.sort_by(|a, b| a.id.cmp(&b.id));

    for field in fields {
        let mut prop = Map::new();
        let prop_type = match field.r#type.as_str() {
            "String" => "string",
            "integer" => "integer",
            "boolean" => "boolean",
            "float" => "number",
            _ => "string",
        };

        prop.insert("type".to_string(), Value::String(prop_type.to_string()));
        prop.insert("description".to_string(), Value::String(field.description.clone()));

        if let Some(ref def) = field.default {
            let parsed_default = match prop_type {
                "integer" => {
                    def.parse::<i64>().map(Value::from).unwrap_or(Value::String(def.clone()))
                }
                "number" => {
                    def.parse::<f64>().map(Value::from).unwrap_or(Value::String(def.clone()))
                }
                "boolean" => {
                    def.parse::<bool>().map(Value::from).unwrap_or(Value::String(def.clone()))
                }
                _ => Value::String(def.clone()),
            };
            prop.insert("default".to_string(), parsed_default);
        }

        if !field.allowed_values.is_empty() {
            prop.insert(
                "enum".to_string(),
                Value::Array(
                    field.allowed_values.iter().map(|s| Value::String(s.clone())).collect(),
                ),
            );
        }

        if field.required {
            required.push(Value::String(field.name.clone()));
        }

        // Try mapping validators to standard JSON Schema constraints where possible
        for val_ref in &field.validator_refs {
            if let Some(validator) = domain.validators.iter().find(|v| v.id == *val_ref) {
                // If it is a min limit, map it to "minimum"
                if validator.predicate.starts_with("val >= ") {
                    if let Ok(limit) = validator.predicate["val >= ".len()..].parse::<i64>() {
                        prop.insert("minimum".to_string(), Value::from(limit));

                        // Map counterexample back
                        if let Some(ce_code) = validator.failure_counterexamples.first() {
                            let mut map_ce = Map::new();
                            map_ce.insert("minimum".to_string(), Value::String(ce_code.clone()));
                            prop.insert("x-counterexamples".to_string(), Value::Object(map_ce));
                        }
                    } else {
                        // Flag lost validation rule as a warning (SCH-JSON-011)
                        diags.push(SchJsonDiagnostic::export_loses_standing_fact(&validator.name));
                    }
                } else {
                    // Custom non-mappable predicate loses standing facts (SCH-JSON-011)
                    diags.push(SchJsonDiagnostic::export_loses_standing_fact(&validator.name));
                }
            }
        }

        properties.insert(field.name, Value::Object(prop));
    }

    root.insert("properties".to_string(), Value::Object(properties));

    if !required.is_empty() {
        root.insert("required".to_string(), Value::Array(required));
    }

    let json_str = serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_default();
    Ok((json_str, diags))
}
