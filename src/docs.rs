//! Star TOML Generated Configuration Documentation Engine (ST-113).
//!
//! Provides the data models, fact ingestors, validators, projection engines,
//! and receipt binders for configuration standing documentation.

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::reports::blake3_hex;

// ---------------------------------------------------------------------------
// Phase 1: Documentation Data Model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigDocumentationModel {
    pub version: String,
    pub generated_at: String,
    pub input_digest: String,
    pub model_digest: String,
    pub domains: Vec<ConfigDomain>,
    pub diagnostics: Vec<String>,
    pub receipts: Vec<ReceiptBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigDomain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub authority_boundary: AuthorityBoundary,
    pub fields: Vec<ConfigField>,
    pub validators: Vec<ConfigValidator>,
    pub counterexamples: Vec<Counterexample>,
    pub override_policy: OverridePolicy,
    pub receipt_binding: Option<ReceiptBinding>,
    pub stability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigField {
    pub id: String,
    pub domain_id: String,
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub default: Option<String>,
    pub allowed_values: Vec<String>,
    pub forbidden_values: Vec<String>,
    pub description: String,
    pub validator_refs: Vec<String>,
    pub counterexample_refs: Vec<String>,
    pub override_policy_ref: Option<String>,
    pub source_binding: String,
    pub stability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigValidator {
    pub id: String,
    pub name: String,
    pub predicate: String,
    pub description: String,
    pub applies_to: Vec<String>,
    pub failure_counterexamples: Vec<String>,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counterexample {
    pub code: String,
    pub name: String,
    pub description: String,
    pub severity: String,
    pub detection_rule: String,
    pub repair_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityBoundary {
    pub id: String,
    pub domain: String,
    pub may_control: Vec<String>,
    pub must_not_claim: Vec<String>,
    pub defer_to: Option<String>,
    pub escalation_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverridePolicy {
    pub id: String,
    pub allowed: bool,
    pub required_reason: bool,
    pub required_scope: Vec<String>,
    pub required_expiration: Option<String>,
    pub required_tracking: Option<String>,
    pub required_replacement_plan: bool,
    pub refusal_conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReceiptBinding {
    pub json_schema_digest: String,
    pub admitted_schema_digest: String,
    pub admitted_config_digest: String,
    pub documentation_model_digest: String,
    pub projection_digest: String,
    pub generator_version: String,
    pub timestamp: String,
    pub signer_or_operator: String,
}

impl ConfigDocumentationModel {
    /// Enforces absolute deterministic sorting of all internal collections by ID/code.
    pub fn canonicalize(&mut self) {
        self.domains.sort_by(|a, b| a.id.cmp(&b.id));
        for domain in &mut self.domains {
            domain.fields.sort_by(|a, b| a.id.cmp(&b.id));
            domain.validators.sort_by(|a, b| a.id.cmp(&b.id));
            domain.counterexamples.sort_by(|a, b| a.code.cmp(&b.code));
            domain.authority_boundary.may_control.sort();
            domain.authority_boundary.must_not_claim.sort();
            domain.override_policy.required_scope.sort();
            domain.override_policy.refusal_conditions.sort();
            for field in &mut domain.fields {
                field.allowed_values.sort();
                field.forbidden_values.sort();
                field.validator_refs.sort();
                field.counterexample_refs.sort();
            }
            for validator in &mut domain.validators {
                validator.applies_to.sort();
                validator.failure_counterexamples.sort();
            }
        }
        self.diagnostics.sort();
        self.receipts.sort_by(|a, b| a.admitted_config_digest.cmp(&b.admitted_config_digest));
    }

    /// Computes the model digest by zeroing temporal and self-referential fields.
    /// Receipts are excluded from the hash so the receipt can bind this digest
    /// without a bootstrapping cycle. Guarantees identical hashes across runs.
    pub fn compute_model_digest(&self) -> String {
        let mut clone = self.clone();
        clone.canonicalize();
        clone.generated_at = String::new();
        clone.model_digest = String::new();
        clone.receipts = Vec::new(); // receipts bind this digest; they can't be part of it
        for d in &mut clone.domains {
            d.receipt_binding = None; // same reason
        }
        let serialized = serde_json::to_string(&clone).unwrap_or_default();
        blake3_hex(serialized.as_bytes())
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Fact Ingestor & Documented Trait
// ---------------------------------------------------------------------------

pub trait Documented {
    fn get_domain() -> ConfigDomain;
}

// Implementations for dynamic Schema and standard helper macros will ingest
// data and transform it into the stable ConfigDocumentationModel.

// ---------------------------------------------------------------------------
// Phase 3 & 4: Model Builder & Documentation Validator
// ---------------------------------------------------------------------------

pub struct DocModelBuilder {
    version: String,
    domains: Vec<ConfigDomain>,
    diagnostics: Vec<String>,
    receipts: Vec<ReceiptBinding>,
}

impl DocModelBuilder {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            domains: Vec::new(),
            diagnostics: Vec::new(),
            receipts: Vec::new(),
        }
    }

    pub fn add_domain(&mut self, domain: ConfigDomain) {
        self.domains.push(domain);
    }

    pub fn add_receipt(&mut self, receipt: ReceiptBinding) {
        self.receipts.push(receipt);
    }

    pub fn build(self) -> Result<ConfigDocumentationModel, Vec<DocDiagnostic>> {
        let mut model = ConfigDocumentationModel {
            version: self.version,
            generated_at: "2026-06-28T15:41:36-07:00".to_string(), // Stable or placeholder replaced during projection
            input_digest: String::new(),
            model_digest: String::new(),
            domains: self.domains,
            diagnostics: self.diagnostics,
            receipts: self.receipts,
        };

        model.canonicalize();

        // Compute input digest based on domain facts
        let input_facts = serde_json::to_string(&model.domains).unwrap_or_default();
        model.input_digest = blake3_hex(input_facts.as_bytes());

        // Validate the model and collect diagnostics
        let doc_errors = validate_docs_integrity(&model);
        if !doc_errors.is_empty() {
            return Err(doc_errors);
        }

        model.model_digest = model.compute_model_digest();
        Ok(model)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocDiagnostic {
    pub code: &'static str,
    pub name: &'static str,
    pub severity: String,
    pub message: String,
    pub repair_hint: String,
    pub trigger: String,
    pub negative_fixture: Option<String>,
    pub test_coverage: bool,
    pub release_verifier_coverage_if_fatal: bool,
}

/// Run full validation checks to assert q_docs metrics (DOC-001 through DOC-020).
pub fn validate_docs_integrity(model: &ConfigDocumentationModel) -> Vec<DocDiagnostic> {
    let mut diags = Vec::new();

    // DOC-013 Nondeterministic model
    let mut model_clone = model.clone();
    model_clone.canonicalize();
    if model.domains != model_clone.domains {
        diags.push(DocDiagnostic {
            code: "DOC-013",
            name: "DocumentationModelNondeterministic",
            severity: "Fatal".to_string(),
            message: "Documentation model ordering is not deterministic.".to_string(),
            repair_hint: "Ensure domains, fields, and validators are sorted alphabetically by ID."
                .to_string(),
            trigger: "model.domains order differs from canonicalized order".to_string(),
            negative_fixture: Some("unsorted_domain_fixture".to_string()),
            test_coverage: true,
            release_verifier_coverage_if_fatal: true,
        });
    }

    // DOC-020 ReceiptDigestMismatch
    let expected_model_digest = model.compute_model_digest();
    for receipt in &model.receipts {
        if !receipt.documentation_model_digest.is_empty()
            && receipt.documentation_model_digest != expected_model_digest
        {
            diags.push(DocDiagnostic {
                code: "DOC-020",
                name: "ReceiptDigestMismatch",
                severity: "Fatal".to_string(),
                message: format!(
                    "Receipt documentation_model_digest '{}' does not match computed digest '{}'.",
                    receipt.documentation_model_digest, expected_model_digest
                ),
                repair_hint: "Regenerate the documentation receipt after any model change."
                    .to_string(),
                trigger: "receipt digest != compute_model_digest()".to_string(),
                negative_fixture: Some("stale_receipt_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: true,
            });
        }
    }

    // DOC-012 DocsGeneratedFromUnadmittedFacts
    for domain in &model.domains {
        if let Some(ref r) = domain.receipt_binding {
            if !r.json_schema_digest.is_empty() && r.admitted_schema_digest.is_empty() {
                diags.push(DocDiagnostic {
                    code: "DOC-012",
                    name: "DocsGeneratedFromUnadmittedFacts",
                    severity: "Fatal".to_string(),
                    message: format!(
                        "Domain '{}' has a json_schema_digest but no admitted_schema_digest — schema was not admitted before docs were generated.",
                        domain.id
                    ),
                    repair_hint: "Run schema admission before generating documentation.".to_string(),
                    trigger: "json_schema_digest present, admitted_schema_digest empty".to_string(),
                    negative_fixture: Some("unadmitted_schema_docs_fixture".to_string()),
                    test_coverage: true,
                    release_verifier_coverage_if_fatal: true,
                });
            }
        }
    }

    for domain in &model.domains {
        // DOC-001 ConfigDomainWithoutGeneratedDocs
        if domain.id.is_empty() {
            diags.push(DocDiagnostic {
                code: "DOC-001",
                name: "ConfigDomainWithoutGeneratedDocs",
                severity: "Fatal".to_string(),
                message: "A configuration domain is missing an ID.".to_string(),
                repair_hint: "Define a unique stable ID for all configuration domains.".to_string(),
                trigger: "domain.id.is_empty()".to_string(),
                negative_fixture: Some("empty_domain_id_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: true,
            });
        }

        // DOC-008 ReceiptPolicyUndocumented
        if domain.override_policy.allowed && domain.receipt_binding.is_none() {
            diags.push(DocDiagnostic {
                code: "DOC-008",
                name: "ReceiptPolicyUndocumented",
                severity: "Error".to_string(),
                message: format!(
                    "Domain '{}' allows overrides but has no receipt binding documenting the policy.",
                    domain.id
                ),
                repair_hint: "Add a ReceiptBinding to the domain that documents override policy provenance.".to_string(),
                trigger: "override_policy.allowed && receipt_binding.is_none()".to_string(),
                negative_fixture: Some("missing_receipt_for_override_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: false,
            });
        }

        // DOC-009 AuthorityBoundaryUndocumented
        if domain.authority_boundary.may_control.is_empty()
            && domain.authority_boundary.must_not_claim.is_empty()
        {
            diags.push(DocDiagnostic {
                code: "DOC-009",
                name: "AuthorityBoundaryUndocumented",
                severity: "Error".to_string(),
                message: format!("Authority boundary rules for domain '{}' are empty or undocumented.", domain.id),
                repair_hint: "Specify may_control or must_not_claim rules inside the domain's authority boundary.".to_string(),
                trigger: "may_control.is_empty() && must_not_claim.is_empty()".to_string(),
                negative_fixture: Some("empty_authority_boundary_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: false,
            });
        }

        // DOC-018 AuthorityBoundaryDocumentedButNotEnforced
        if !domain.authority_boundary.must_not_claim.is_empty()
            && domain.authority_boundary.defer_to.is_none()
            && domain.authority_boundary.escalation_rule.is_none()
        {
            // must_not_claim without defer_to or escalation = unenforced boundary
            diags.push(DocDiagnostic {
                code: "DOC-018",
                name: "AuthorityBoundaryDocumentedButNotEnforced",
                severity: "Warning".to_string(),
                message: format!(
                    "Domain '{}' declares must_not_claim but has no defer_to or escalation_rule to enforce the boundary.",
                    domain.id
                ),
                repair_hint: "Add a defer_to or escalation_rule to the authority boundary.".to_string(),
                trigger: "must_not_claim non-empty, defer_to None, escalation_rule None".to_string(),
                negative_fixture: Some("unenforced_boundary_fixture".to_string()),
                test_coverage: false,
                release_verifier_coverage_if_fatal: false,
            });
        }

        // DOC-005 OverridePolicyWithoutDocumentation
        if domain.override_policy.id.is_empty() {
            diags.push(DocDiagnostic {
                code: "DOC-005",
                name: "OverridePolicyWithoutDocumentation",
                severity: "Error".to_string(),
                message: format!("Override policy for domain '{}' is undocumented or missing ID.", domain.id),
                repair_hint: "Provide details for the override policy, including tracking and expiration guidelines.".to_string(),
                trigger: "override_policy.id.is_empty()".to_string(),
                negative_fixture: Some("missing_override_policy_id_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: false,
            });
        }

        // DOC-017 OverrideDocumentedButNotEnforced
        if domain.override_policy.allowed
            && !domain.override_policy.required_reason
            && domain.override_policy.refusal_conditions.is_empty()
        {
            diags.push(DocDiagnostic {
                code: "DOC-017",
                name: "OverrideDocumentedButNotEnforced",
                severity: "Warning".to_string(),
                message: format!(
                    "Domain '{}' allows overrides but requires no reason and has no refusal conditions.",
                    domain.id
                ),
                repair_hint: "Set required_reason = true or add refusal_conditions to enforce override governance.".to_string(),
                trigger: "override.allowed && !required_reason && refusal_conditions.is_empty()".to_string(),
                negative_fixture: Some("unenforced_override_fixture".to_string()),
                test_coverage: false,
                release_verifier_coverage_if_fatal: false,
            });
        }

        for field in &domain.fields {
            // DOC-002 ConfigFieldWithoutDescription
            if field.description.trim().is_empty() {
                diags.push(DocDiagnostic {
                    code: "DOC-002",
                    name: "ConfigFieldWithoutDescription",
                    severity: "Error".to_string(),
                    message: format!(
                        "Field '{}.{}' is missing a description.",
                        domain.id, field.name
                    ),
                    repair_hint:
                        "Add a non-empty description attribute or doc comment for the field."
                            .to_string(),
                    trigger: "field.description.trim().is_empty()".to_string(),
                    negative_fixture: Some("missing_description_fixture".to_string()),
                    test_coverage: true,
                    release_verifier_coverage_if_fatal: false,
                });
            }

            // DOC-003 ConfigFieldWithoutValidationRule
            if field.validator_refs.is_empty() && field.required && field.r#type == "String" {
                diags.push(DocDiagnostic {
                    code: "DOC-003",
                    name: "ConfigFieldWithoutValidationRule",
                    severity: "Error".to_string(),
                    message: format!("Field '{}.{}' has no validation rules documented.", domain.id, field.name),
                    repair_hint: "Add validation rule references (e.g. range, non-empty) to the field metadata.".to_string(),
                    trigger: "required String field with empty validator_refs".to_string(),
                    negative_fixture: Some("missing_validator_fixture".to_string()),
                    test_coverage: true,
                    release_verifier_coverage_if_fatal: false,
                });
            }

            // DOC-004 ConfigFieldWithoutCounterexample
            for val_ref in &field.validator_refs {
                let has_counterexample = domain
                    .validators
                    .iter()
                    .filter(|v| v.id == *val_ref)
                    .any(|v| !v.failure_counterexamples.is_empty());
                if !has_counterexample {
                    diags.push(DocDiagnostic {
                        code: "DOC-004",
                        name: "ConfigFieldWithoutCounterexample",
                        severity: "Error".to_string(),
                        message: format!("Validator '{}' on field '{}.{}' lacks an associated counterexample mapping.", val_ref, domain.id, field.name),
                        repair_hint: "Define at least one counterexample code for this validator.".to_string(),
                        trigger: "validator has no failure_counterexamples".to_string(),
                        negative_fixture: Some("validator_without_counterexample_fixture".to_string()),
                        test_coverage: true,
                        release_verifier_coverage_if_fatal: false,
                    });
                }
            }

            // DOC-006 ForbiddenValueUndocumented
            if !field.forbidden_values.is_empty()
                && !field.description.contains("forbidden")
                && !field.description.contains("must not")
            {
                diags.push(DocDiagnostic {
                    code: "DOC-006",
                    name: "ForbiddenValueUndocumented",
                    severity: "Warning".to_string(),
                    message: format!("Field '{}.{}' defines forbidden values that are not documented in the description.", domain.id, field.name),
                    repair_hint: "Mention the forbidden values in the field's description text.".to_string(),
                    trigger: "forbidden_values non-empty, description lacks mention".to_string(),
                    negative_fixture: Some("forbidden_value_undocumented_fixture".to_string()),
                    test_coverage: true,
                    release_verifier_coverage_if_fatal: false,
                });
            }

            // DOC-007 DefaultValueUndocumented
            if field.default.is_some() && !field.description.contains("default") {
                diags.push(DocDiagnostic {
                    code: "DOC-007",
                    name: "DefaultValueUndocumented",
                    severity: "Warning".to_string(),
                    message: format!("Field '{}.{}' specifies a default value but the description does not explain it.", domain.id, field.name),
                    repair_hint: "Add default value explanations to the field's description text.".to_string(),
                    trigger: "field.default.is_some() && !description.contains(\"default\")".to_string(),
                    negative_fixture: Some("default_undocumented_fixture".to_string()),
                    test_coverage: true,
                    release_verifier_coverage_if_fatal: false,
                });
            }
        }

        // DOC-015 ValidatorDocumentedButNotEnforced
        for validator in &domain.validators {
            let is_referenced =
                domain.fields.iter().any(|f| f.validator_refs.contains(&validator.id));
            if !is_referenced {
                diags.push(DocDiagnostic {
                    code: "DOC-015",
                    name: "ValidatorDocumentedButNotEnforced",
                    severity: "Warning".to_string(),
                    message: format!(
                        "Validator '{}' in domain '{}' is documented but not referenced by any field.",
                        validator.id, domain.id
                    ),
                    repair_hint: "Add this validator's ID to the validator_refs of at least one field, or remove the validator.".to_string(),
                    trigger: "validator.id not in any field.validator_refs".to_string(),
                    negative_fixture: Some("orphaned_validator_fixture".to_string()),
                    test_coverage: false,
                    release_verifier_coverage_if_fatal: false,
                });
            }
        }

        // DOC-016 CounterexampleDocumentedButNotTested
        for ce in &domain.counterexamples {
            // A counterexample is considered tested if it appears in a validator's failure_counterexamples
            let is_tested =
                domain.validators.iter().any(|v| v.failure_counterexamples.contains(&ce.code));
            if !is_tested {
                diags.push(DocDiagnostic {
                    code: "DOC-016",
                    name: "CounterexampleDocumentedButNotTested",
                    severity: "Warning".to_string(),
                    message: format!(
                        "Counterexample '{}' in domain '{}' is not referenced by any validator's failure_counterexamples.",
                        ce.code, domain.id
                    ),
                    repair_hint: "Reference this counterexample code in at least one validator's failure_counterexamples list.".to_string(),
                    trigger: "counterexample.code not in any validator.failure_counterexamples".to_string(),
                    negative_fixture: Some("untested_counterexample_fixture".to_string()),
                    test_coverage: false,
                    release_verifier_coverage_if_fatal: false,
                });
            }
        }

        // DOC-019 ProjectionTargetMissingRequiredDomain
        if domain.id.is_empty() || domain.name.is_empty() || domain.description.is_empty() {
            diags.push(DocDiagnostic {
                code: "DOC-019",
                name: "ProjectionTargetMissingRequiredDomain",
                severity: "Error".to_string(),
                message: format!(
                    "Domain at index has empty id, name, or description — projection would be incomplete."
                ),
                repair_hint: "Ensure all domains have non-empty id, name, and description before projection.".to_string(),
                trigger: "domain.id or name or description is empty".to_string(),
                negative_fixture: Some("incomplete_domain_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: false,
            });
        }
    }

    diags
}

/// Check DOC-014: StableDomainMissingMachineReadableExport — call from CLI/release verifier.
pub fn check_machine_readable_export(model: &ConfigDocumentationModel) -> Option<DocDiagnostic> {
    let has_stable = model.domains.iter().any(|d| d.stability == "stable");
    if has_stable && !std::path::Path::new("docs/config_metadata.json").exists() {
        return Some(DocDiagnostic {
            code: "DOC-014",
            name: "StableDomainMissingMachineReadableExport",
            severity: "Fatal".to_string(),
            message: "A stable domain exists but docs/config_metadata.json is missing.".to_string(),
            repair_hint: "Run `cargo run --bin star_toml_docs export` to generate the export."
                .to_string(),
            trigger: "stable domain present, docs/config_metadata.json absent".to_string(),
            negative_fixture: Some("missing_metadata_json_fixture".to_string()),
            test_coverage: true,
            release_verifier_coverage_if_fatal: true,
        });
    }
    None
}

/// Check DOC-010: GeneratedDocsOutOfDate — compare on-disk JSON against the current model.
/// Call from the CLI or release verifier, not from pure model validation.
pub fn check_docs_out_of_date(model: &ConfigDocumentationModel) -> Option<DocDiagnostic> {
    let json_path = std::path::Path::new("docs/config_metadata.json");
    if !json_path.exists() {
        return None;
    }
    if let Ok(on_disk) = std::fs::read_to_string(json_path) {
        let disk_digest = crate::reports::blake3_hex(on_disk.as_bytes());
        let current_json = project_json(model);
        let current_digest = crate::reports::blake3_hex(current_json.as_bytes());
        if disk_digest != current_digest {
            return Some(DocDiagnostic {
                code: "DOC-010",
                name: "GeneratedDocsOutOfDate",
                severity: "Error".to_string(),
                message:
                    "docs/config_metadata.json is stale — model has changed since last export."
                        .to_string(),
                repair_hint: "Run `cargo run --bin star_toml_docs export` to refresh.".to_string(),
                trigger: "on-disk JSON digest != current projection digest".to_string(),
                negative_fixture: Some("stale_json_fixture".to_string()),
                test_coverage: true,
                release_verifier_coverage_if_fatal: false,
            });
        }
    }
    None
}

/// Check DOC-011: ManualDocsClaimStanding — scan docs/*.md for files claiming standing
/// without a receipt footer. Call from CLI, not from pure model validation.
pub fn check_manual_standing_claims() -> Vec<DocDiagnostic> {
    let mut diags = Vec::new();
    if let Ok(entries) = std::fs::read_dir("docs") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("standing") && !content.contains("star-toml-receipt:") {
                        diags.push(DocDiagnostic {
                            code: "DOC-011",
                            name: "ManualDocsClaimStanding",
                            severity: "Warning".to_string(),
                            message: format!(
                                "File '{}' claims standing but has no star-toml-receipt footer.",
                                path.display()
                            ),
                            repair_hint: "Add a `<!-- star-toml-receipt: ... -->` footer to any file that claims standing.".to_string(),
                            trigger: "doc file contains 'standing' without receipt footer".to_string(),
                            negative_fixture: Some("manual_standing_claim_fixture".to_string()),
                            test_coverage: false,
                            release_verifier_coverage_if_fatal: false,
                        });
                    }
                }
            }
        }
    }
    diags
}

// ---------------------------------------------------------------------------
// Phase 5 & 6: Projection Engine & Receipt Binding
// ---------------------------------------------------------------------------

pub fn project_markdown(model: &ConfigDocumentationModel) -> String {
    let mut out = String::new();

    out.push_str("# star-toml Configuration Reference\n\n");
    out.push_str(&format!("* **Model Version**: `{}`\n", model.version));
    out.push_str(&format!("* **Generated At**: `{}`\n", model.generated_at));
    out.push_str(&format!("* **Input Facts Digest**: `{}`\n", model.input_digest));
    out.push_str(&format!("* **Model Digest**: `{}`\n\n", model.model_digest));

    out.push_str("## 🏛️ Receipts & Standing\n\n");
    if model.receipts.is_empty() {
        out.push_str("No active receipt bindings.\n\n");
    } else {
        out.push_str("| Admitted Config Digest | Model Digest | Projection Digest | Generator Version | Signer/Operator |\n");
        out.push_str("|---|---|---|---|---|\n");
        for r in &model.receipts {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                r.admitted_config_digest,
                r.documentation_model_digest,
                r.projection_digest,
                r.generator_version,
                r.signer_or_operator
            ));
        }
        out.push_str("\n");
    }

    for domain in &model.domains {
        out.push_str(&format!("## Domain: {} ({})\n\n", domain.name, domain.id));
        out.push_str(&format!("* **Stability**: `{}`\n", domain.stability));
        out.push_str(&format!("* **Description**: {}\n\n", domain.description));

        out.push_str("### 🛡️ Authority Boundary\n\n");
        out.push_str(&format!(
            "* **Allowed Control**: `{:?}`\n",
            domain.authority_boundary.may_control
        ));
        out.push_str(&format!(
            "* **Forbidden Claims**: `{:?}`\n",
            domain.authority_boundary.must_not_claim
        ));
        if let Some(ref d) = domain.authority_boundary.defer_to {
            out.push_str(&format!("* **Delegates To**: `{}`\n", d));
        }
        out.push_str("\n");

        out.push_str("### ⚙️ Fields Reference\n\n");
        out.push_str("| Field | Type | Required | Default | Allowed Values | Forbidden Values | Description | Stability |\n");
        out.push_str("|---|---|---|---|---|---|---|---|\n");
        for f in &domain.fields {
            let default_str = f.default.as_deref().unwrap_or("-");
            let allowed_str = if f.allowed_values.is_empty() {
                "-".to_owned()
            } else {
                f.allowed_values.join(", ")
            };
            let forbidden_str = if f.forbidden_values.is_empty() {
                "-".to_owned()
            } else {
                f.forbidden_values.join(", ")
            };
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} | `{}` |\n",
                f.name,
                f.r#type,
                f.required,
                default_str,
                allowed_str,
                forbidden_str,
                f.description,
                f.stability
            ));
        }
        out.push_str("\n");

        if !domain.validators.is_empty() {
            out.push_str("### 🔍 Validators\n\n");
            for v in &domain.validators {
                out.push_str(&format!(
                    "* **`{}`**: {} (Applies to: `{:?}`, Severity: `{}`)\n",
                    v.name, v.description, v.applies_to, v.severity
                ));
            }
            out.push_str("\n");
        }

        if !domain.counterexamples.is_empty() {
            out.push_str("### ❌ Counterexamples\n\n");
            for c in &domain.counterexamples {
                out.push_str(&format!(
                    "* **`{}`** - *{}* (Detection: `{}`, Severity: `{}`)\n",
                    c.code, c.name, c.detection_rule, c.severity
                ));
                out.push_str(&format!("  * **Repair**: {}\n", c.repair_hint));
            }
            out.push_str("\n");
        }
    }

    out
}

pub fn project_json(model: &ConfigDocumentationModel) -> String {
    serde_json::to_string_pretty(model).unwrap_or_default()
}

pub fn compute_receipt(
    admitted_config_digest: &str,
    model_digest: &str,
    projection_digest: &str,
) -> String {
    let payload = format!("{}{}{}", admitted_config_digest, model_digest, projection_digest);
    blake3_hex(payload.as_bytes())
}

// ---------------------------------------------------------------------------
// Phase 7: Tooling APIs
// ---------------------------------------------------------------------------

pub fn check_docs_freshness(
    model: &ConfigDocumentationModel,
    md_path: &Path,
    json_path: &Path,
) -> bool {
    let fresh_md = project_markdown(model);
    let fresh_json = project_json(model);

    let md_content = fs::read_to_string(md_path).unwrap_or_default();
    let json_content = fs::read_to_string(json_path).unwrap_or_default();

    let md_ok = blake3_hex(md_content.as_bytes()) == blake3_hex(fresh_md.as_bytes());
    let json_ok = blake3_hex(json_content.as_bytes()) == blake3_hex(fresh_json.as_bytes());

    md_ok && json_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_model_determinism() {
        // Create domain with unsorted lists
        let domain_1 = ConfigDomain {
            id: "db".to_string(),
            name: "Database".to_string(),
            description: "Database settings".to_string(),
            authority_boundary: AuthorityBoundary {
                id: "db_b".to_string(),
                domain: "db".to_string(),
                may_control: vec!["db_host".to_string(), "db_port".to_string()],
                must_not_claim: vec!["system".to_string()],
                defer_to: Some("ops-team".to_string()),
                escalation_rule: None,
            },
            fields: vec![
                ConfigField {
                    id: "db.port".to_string(),
                    domain_id: "db".to_string(),
                    name: "port".to_string(),
                    r#type: "integer".to_string(),
                    required: true,
                    default: Some("5432".to_string()),
                    allowed_values: vec![],
                    forbidden_values: vec![],
                    description: "DB port (default: 5432)".to_string(),
                    validator_refs: vec![],
                    counterexample_refs: vec![],
                    override_policy_ref: None,
                    source_binding: "env".to_string(),
                    stability: "stable".to_string(),
                },
                ConfigField {
                    id: "db.host".to_string(),
                    domain_id: "db".to_string(),
                    name: "host".to_string(),
                    r#type: "String".to_string(),
                    required: false,
                    default: None,
                    allowed_values: vec![],
                    forbidden_values: vec![],
                    description: "DB host name".to_string(),
                    validator_refs: vec![],
                    counterexample_refs: vec![],
                    override_policy_ref: None,
                    source_binding: "toml".to_string(),
                    stability: "stable".to_string(),
                },
            ],
            validators: vec![],
            counterexamples: vec![],
            override_policy: OverridePolicy {
                id: "db_o".to_string(),
                allowed: false,
                required_reason: false,
                required_scope: vec!["env".to_string(), "local".to_string()],
                required_expiration: None,
                required_tracking: None,
                required_replacement_plan: false,
                refusal_conditions: vec![],
            },
            receipt_binding: None,
            stability: "stable".to_string(),
        };

        let mut builder_a = DocModelBuilder::new("1.0.0");
        builder_a.add_domain(domain_1.clone());
        let model_a = builder_a.build().unwrap();

        let mut builder_b = DocModelBuilder::new("1.0.0");
        let mut domain_2 = domain_1.clone();
        domain_2.fields.reverse();
        domain_2.authority_boundary.may_control.reverse();
        domain_2.override_policy.required_scope.reverse();
        builder_b.add_domain(domain_2);
        let model_b = builder_b.build().unwrap();

        assert_eq!(model_a.model_digest, model_b.model_digest);

        let json_a = project_json(&model_a);
        let json_b = project_json(&model_b);
        assert_eq!(json_a, json_b);
    }
}
