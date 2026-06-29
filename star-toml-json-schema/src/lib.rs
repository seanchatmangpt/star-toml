pub mod diagnostics;
pub mod export;
pub mod import;
pub mod standing;

pub use diagnostics::SchJsonDiagnostic;
pub use export::export_to_json_schema;
pub use import::{import_json_schema, SchemaCandidate};
pub use standing::validate_json_schema_standing;

pub fn validate_config_against_json_schema(
    schema_json: &str,
    config_json: &serde_json::Value,
) -> Result<(), Vec<String>> {
    let schema_val = serde_json::from_str(schema_json).map_err(|e| vec![e.to_string()])?;
    let compiled = jsonschema::JSONSchema::compile(&schema_val).map_err(|e| vec![e.to_string()])?;
    if let Err(errors) = compiled.validate(config_json) {
        let mut err_msgs = Vec::new();
        for err in errors {
            err_msgs.push(err.to_string());
        }
        return Err(err_msgs);
    }
    Ok(())
}
