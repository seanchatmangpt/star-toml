use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchJsonDiagnostic {
    pub code: &'static str,
    pub name: &'static str,
    pub severity: String, // "Fatal", "Error", "Warning"
    pub trigger: String,
    pub repair_guidance: String,
}

impl SchJsonDiagnostic {
    pub fn feature_disabled() -> Self {
        Self {
            code: "SCH-JSON-001",
            name: "JsonSchemaFeatureDisabled",
            severity: "Fatal".to_string(),
            trigger:
                "JSON Schema bridge feature is disabled but a JSON Schema source is attempted."
                    .to_string(),
            repair_guidance: "Enable the JSON Schema feature or bridge interface.".to_string(),
        }
    }

    pub fn parse_failed(err: &str) -> Self {
        Self {
            code: "SCH-JSON-002",
            name: "JsonSchemaParseFailed",
            severity: "Fatal".to_string(),
            trigger: format!("Failed to parse JSON Schema syntax: {}", err),
            repair_guidance: "Check JSON syntax and schema structure.".to_string(),
        }
    }

    pub fn unsupported_keyword(keyword: &str) -> Self {
        Self {
            code: "SCH-JSON-003",
            name: "JsonSchemaUnsupportedKeyword",
            severity: "Error".to_string(),
            trigger: format!("Unsupported standing-critical keyword present: '{}'.", keyword),
            repair_guidance: "Remove unsupported keywords or restrict mapping.".to_string(),
        }
    }

    pub fn reference_unresolved(reference: &str) -> Self {
        Self {
            code: "SCH-JSON-004",
            name: "JsonSchemaReferenceUnresolved",
            severity: "Error".to_string(),
            trigger: format!("Reference '$ref' cannot be resolved: '{}'.", reference),
            repair_guidance: "Ensure the referenced file or pointer exists.".to_string(),
        }
    }

    pub fn nondeterministic_import() -> Self {
        Self {
            code: "SCH-JSON-005",
            name: "JsonSchemaNondeterministicImport",
            severity: "Error".to_string(),
            trigger: "Keys or fields are imported in a nondeterministic order.".to_string(),
            repair_guidance: "Ensure keys/properties are processed in sorted order.".to_string(),
        }
    }

    pub fn missing_descriptions(domain_or_field: &str) -> Self {
        Self {
            code: "SCH-JSON-006",
            name: "JsonSchemaMissingDescriptionsForStableDomain",
            severity: "Error".to_string(),
            trigger: format!(
                "Stable domain or field lacks description metadata: '{}'.",
                domain_or_field
            ),
            repair_guidance: "Add description property to the schema element.".to_string(),
        }
    }

    pub fn allows_unbounded_additional_properties() -> Self {
        Self {
            code: "SCH-JSON-007",
            name: "JsonSchemaAllowsUnboundedAdditionalProperties",
            severity: "Error".to_string(),
            trigger:
                "Schema allows unbounded expansion (additionalProperties is not false or missing)."
                    .to_string(),
            repair_guidance:
                "Set additionalProperties to false or declare an explicit boundary policy."
                    .to_string(),
        }
    }

    pub fn config_validated_against_unadmitted_schema() -> Self {
        Self {
            code: "SCH-JSON-008",
            name: "ConfigValidatedAgainstUnadmittedJsonSchema",
            severity: "Fatal".to_string(),
            trigger: "Configuration validated against unadmitted JSON Schema.".to_string(),
            repair_guidance: "Admit the schema facts before running config validation.".to_string(),
        }
    }

    pub fn docs_generated_from_unadmitted_schema() -> Self {
        Self {
            code: "SCH-JSON-009",
            name: "DocsGeneratedFromUnadmittedJsonSchema",
            severity: "Fatal".to_string(),
            trigger: "Generated documentation references unadmitted JSON Schema facts.".to_string(),
            repair_guidance: "Pass the schema candidate through star-toml admission.".to_string(),
        }
    }

    pub fn digest_mismatch() -> Self {
        Self {
            code: "SCH-JSON-010",
            name: "JsonSchemaDigestMismatch",
            severity: "Fatal".to_string(),
            trigger: "Schema digest changed without updating documentation or receipts."
                .to_string(),
            repair_guidance: "Re-generate documentation and receipt binding.".to_string(),
        }
    }

    pub fn export_loses_standing_fact(fact: &str) -> Self {
        Self {
            code: "SCH-JSON-011",
            name: "JsonSchemaExportLosesRequiredStandingFact",
            severity: "Warning".to_string(),
            trigger: format!("Exporting schema loses required standing facts: '{}'.", fact),
            repair_guidance: "Acknowledge lossy export or restrict the schema features exported."
                .to_string(),
        }
    }

    pub fn composition_ambiguous(keyword: &str) -> Self {
        Self {
            code: "SCH-JSON-012",
            name: "JsonSchemaCompositionAmbiguous",
            severity: "Error".to_string(),
            trigger: format!("Composition keyword '{}' results in ambiguous mapping.", keyword),
            repair_guidance: "Clarify composition rules to avoid overlap.".to_string(),
        }
    }

    pub fn default_undocumented(field: &str) -> Self {
        Self {
            code: "SCH-JSON-013",
            name: "JsonSchemaDefaultUndocumented",
            severity: "Warning".to_string(),
            trigger: format!(
                "Default value specified but description does not document it for field: '{}'.",
                field
            ),
            repair_guidance: "Document default value in the description field.".to_string(),
        }
    }

    pub fn constraint_without_counterexample(constraint: &str) -> Self {
        Self {
            code: "SCH-JSON-014",
            name: "JsonSchemaConstraintWithoutCounterexample",
            severity: "Error".to_string(),
            trigger: format!(
                "Constraint maps to validator but lacks a counterexample candidate for '{}'.",
                constraint
            ),
            repair_guidance: "Associate the validator/constraint with a counterexample name."
                .to_string(),
        }
    }

    pub fn authority_boundary_missing(domain: &str) -> Self {
        Self {
            code: "SCH-JSON-015",
            name: "JsonSchemaAuthorityBoundaryMissing",
            severity: "Error".to_string(),
            trigger: format!(
                "Stable schema domain lacks authority boundary metadata: '{}'.",
                domain
            ),
            repair_guidance: "Add x-authority-boundary metadata or provide it during admission."
                .to_string(),
        }
    }

    pub fn unsupported_dialect(dialect: &str) -> Self {
        Self {
            code: "SCH-JSON-016",
            name: "JsonSchemaUnsupportedDialect",
            severity: "Fatal".to_string(),
            trigger: format!("The schema version/$schema dialect is unsupported: '{}'.", dialect),
            repair_guidance: "Use a supported JSON Schema version (e.g. Draft 7).".to_string(),
        }
    }
}
