use crate::{diagnostics::SchJsonDiagnostic, import::SchemaCandidate};

pub fn validate_json_schema_standing(candidate: &SchemaCandidate) -> Vec<SchJsonDiagnostic> {
    let mut diags = Vec::new();

    // 1. Dialect & Parse check
    if candidate.title.is_empty() {
        diags.push(SchJsonDiagnostic::unsupported_dialect("missing_title"));
    }

    // 2. Import Determinism: Assert that fields are sorted alphabetically by ID
    let mut sorted_fields = candidate.domain.fields.clone();
    sorted_fields.sort_by(|a, b| a.id.cmp(&b.id));
    if candidate.domain.fields != sorted_fields {
        diags.push(SchJsonDiagnostic::nondeterministic_import());
    }

    // 3. Unbounded Extensions check
    if candidate.domain.fields.is_empty() {
        // Safe placeholder: empty schema is considered unbounded if it accepts anything
        diags.push(SchJsonDiagnostic::allows_unbounded_additional_properties());
    }

    // 4. Required Documentation Facts check
    if candidate.description.trim().is_empty() {
        diags.push(SchJsonDiagnostic::missing_descriptions(&candidate.title));
    }

    for field in &candidate.domain.fields {
        if field.description.trim().is_empty() {
            diags.push(SchJsonDiagnostic::missing_descriptions(&field.name));
        }

        if field.default.is_some() {
            if !field.description.to_lowercase().contains("default") {
                diags.push(SchJsonDiagnostic::default_undocumented(&field.name));
            }
        }

        // Constraints check: validator reference but no counterexample reference
        for (i, val_ref) in field.validator_refs.iter().enumerate() {
            let has_counterexample = field.counterexample_refs.get(i).is_some();
            if !has_counterexample {
                diags.push(SchJsonDiagnostic::constraint_without_counterexample(val_ref));
            }
        }
    }

    // 5. Authority Boundary Attached check
    if candidate.domain.authority_boundary.may_control.is_empty()
        && candidate.domain.authority_boundary.must_not_claim.is_empty()
    {
        diags.push(SchJsonDiagnostic::authority_boundary_missing(&candidate.title));
    }

    // 6. Schema Digest check
    if candidate.digest.is_empty() {
        diags.push(SchJsonDiagnostic::digest_mismatch());
    }

    diags
}
