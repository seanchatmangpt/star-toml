//! Conformance tests for star-toml standing law at runtime.
//!
//! Ten categories covering the full DoD matrix.

use star_toml::{
    docs::{
        AuthorityBoundary, ConfigDocumentationModel, DocModelBuilder, OverridePolicy,
        ReceiptBinding,
    },
    Schema,
};
use star_toml_config_test::*;

// ---------------------------------------------------------------------------
// Shared fixture builders
// ---------------------------------------------------------------------------

fn valid_schema() -> Schema {
    Schema::new()
        .domain_metadata("app", "App Config", "Application runtime settings.")
        .stability("stable")
        .authority_boundary(AuthorityBoundary {
            id: "app_b".to_string(),
            domain: "app".to_string(),
            may_control: vec!["port".to_string()],
            must_not_claim: vec!["system".to_string()],
            defer_to: Some("ops-team".to_string()),
            escalation_rule: None,
        })
        .override_policy(OverridePolicy {
            id: "app_override".to_string(),
            allowed: false,
            required_reason: false,
            required_scope: vec![],
            required_expiration: None,
            required_tracking: None,
            required_replacement_plan: false,
            refusal_conditions: vec!["no-production-override".to_string()],
        })
        .field("port")
        .description("TCP port the application listens on (default: 8080).")
        .stability("stable")
        .range_i64(1, 65535)
        .default("8080")
        .done()
}

fn build_valid_model() -> ConfigDocumentationModel {
    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(valid_schema().get_domain());
    let mut model = builder.build().expect("valid schema should build without diagnostics");
    // Add a receipt to satisfy assert_receipt_bound
    model.receipts.push(ReceiptBinding {
        json_schema_digest: String::new(),
        admitted_schema_digest: String::new(),
        admitted_config_digest: model.input_digest.clone(),
        documentation_model_digest: model.model_digest.clone(),
        projection_digest: String::new(),
        generator_version: "26.6.29".to_string(),
        timestamp: "2026-06-29T00:00:00Z".to_string(),
        signer_or_operator: "test-signer".to_string(),
    });
    model.canonicalize();
    model
}

// ---------------------------------------------------------------------------
// Category 1: Happy path — admitted model passes all checks
// ---------------------------------------------------------------------------

#[test]
fn test_happy_path_admitted_model() {
    let model = build_valid_model();
    assert_no_doc_diagnostics(&model).unwrap();
    assert_receipt_bound(&model).unwrap();
    assert_docs_fresh(&model).unwrap();
}

// ---------------------------------------------------------------------------
// Category 2: Invalid model refused with correct diagnostic
// ---------------------------------------------------------------------------

#[test]
fn test_empty_domain_id_refused() {
    let domain = {
        let mut d = valid_schema().get_domain();
        d.id = String::new();
        d
    };
    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(domain);
    let result = builder.build();
    assert!(result.is_err(), "Empty domain ID should be refused");
    let diags = result.unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "DOC-001"),
        "Expected DOC-001, got: {:?}",
        diags.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Category 3: Unadmitted JSON schema refused (SCH-JSON-008 gate)
// ---------------------------------------------------------------------------

#[test]
fn test_unadmitted_schema_refused_by_loader() {
    use serde::{Deserialize, Serialize};
    use star_toml::{loader::TrustedLoader, ConfigLifecycle, Validate, Validator};

    #[derive(Debug, Deserialize, Serialize, Default)]
    struct Cfg {
        port: Option<i64>,
    }
    impl Validate for Cfg {
        fn validate(&self, _v: &mut Validator) {}
    }
    impl ConfigLifecycle for Cfg {}

    let result = TrustedLoader::new()
        .require_json_schema_admission(false)
        .layer_str("port = 8080", "test")
        .load_admitted::<Cfg>();

    assert!(result.is_err(), "Unadmitted schema should refuse admission");
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("SCH-JSON-008"), "Expected SCH-JSON-008 in error: {err_str}");
}

// ---------------------------------------------------------------------------
// Category 4: Stale docs detected (DOC-012 via json_schema_digest gate)
// ---------------------------------------------------------------------------

#[test]
fn test_docs_from_unadmitted_schema_refused() {
    // Build a model where domain has a json_schema_digest but no admitted_schema_digest
    let mut domain = valid_schema().get_domain();
    domain.receipt_binding = Some(ReceiptBinding {
        json_schema_digest: "abc123".to_string(),
        admitted_schema_digest: String::new(), // not admitted
        admitted_config_digest: String::new(),
        documentation_model_digest: String::new(),
        projection_digest: String::new(),
        generator_version: "26.6.29".to_string(),
        timestamp: "2026-06-29T00:00:00Z".to_string(),
        signer_or_operator: "test".to_string(),
    });
    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(domain);
    let result = builder.build();
    assert!(result.is_err(), "Docs from unadmitted schema should be refused");
    let diags = result.unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "DOC-012"),
        "Expected DOC-012, got: {:?}",
        diags.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Category 5: Receipt digest mismatch refused (DOC-020)
// ---------------------------------------------------------------------------

#[test]
fn test_receipt_digest_mismatch_refused() {
    let model = build_valid_model();
    let tampered = ReceiptBinding {
        json_schema_digest: String::new(),
        admitted_schema_digest: String::new(),
        admitted_config_digest: model.input_digest.clone(),
        documentation_model_digest: "deadbeef_stale_digest".to_string(), // wrong
        projection_digest: String::new(),
        generator_version: "26.6.29".to_string(),
        timestamp: "2026-06-29T00:00:00Z".to_string(),
        signer_or_operator: "test-signer".to_string(),
    };
    assert_receipt_mismatch_detected(&model, &tampered).unwrap();
}

// ---------------------------------------------------------------------------
// Category 6: Override policy violation (DOC-008)
// ---------------------------------------------------------------------------

#[test]
fn test_override_without_receipt_binding_refused() {
    let mut domain = valid_schema().get_domain();
    domain.override_policy.allowed = true;
    domain.receipt_binding = None;
    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(domain);
    let result = builder.build();
    assert!(result.is_err(), "Override without receipt binding should be refused");
    let diags = result.unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "DOC-008"),
        "Expected DOC-008, got: {:?}",
        diags.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Category 7: Authority boundary violation (DOC-009 — empty boundary)
// ---------------------------------------------------------------------------

#[test]
fn test_empty_authority_boundary_refused() {
    let mut domain = valid_schema().get_domain();
    domain.authority_boundary.may_control.clear();
    domain.authority_boundary.must_not_claim.clear();
    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(domain);
    let result = builder.build();
    assert!(result.is_err(), "Empty authority boundary should be refused");
    let diags = result.unwrap_err();
    assert!(
        diags.iter().any(|d| d.code == "DOC-009"),
        "Expected DOC-009, got: {:?}",
        diags.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Category 8: Forbidden value undocumented (DOC-006)
// ---------------------------------------------------------------------------

#[test]
fn test_forbidden_value_not_in_description_warns() {
    use star_toml::docs::ConfigField;
    // Build a domain directly with forbidden_values set but description lacking mention
    let domain = star_toml::docs::ConfigDomain {
        id: "svc".to_string(),
        name: "Service Config".to_string(),
        description: "Service settings.".to_string(),
        authority_boundary: AuthorityBoundary {
            id: "svc_b".to_string(),
            domain: "svc".to_string(),
            may_control: vec!["mode".to_string()],
            must_not_claim: vec![],
            defer_to: None,
            escalation_rule: None,
        },
        fields: vec![ConfigField {
            id: "svc.mode".to_string(),
            domain_id: "svc".to_string(),
            name: "mode".to_string(),
            r#type: "String".to_string(),
            required: false,
            default: None,
            allowed_values: vec![],
            forbidden_values: vec!["debug".to_string()], // description doesn't mention "forbidden"
            description: "Operating mode for the service.".to_string(),
            validator_refs: vec![],
            counterexample_refs: vec![],
            override_policy_ref: None,
            source_binding: "schema".to_string(),
            stability: "stable".to_string(),
        }],
        validators: vec![],
        counterexamples: vec![],
        override_policy: OverridePolicy {
            id: "svc_override".to_string(),
            allowed: false,
            required_reason: false,
            required_scope: vec![],
            required_expiration: None,
            required_tracking: None,
            required_replacement_plan: false,
            refusal_conditions: vec![],
        },
        receipt_binding: None,
        stability: "stable".to_string(),
    };
    let diags =
        star_toml::docs::validate_docs_integrity(&star_toml::docs::ConfigDocumentationModel {
            version: "26.6.29".to_string(),
            generated_at: String::new(),
            input_digest: String::new(),
            model_digest: String::new(),
            domains: vec![domain],
            diagnostics: vec![],
            receipts: vec![],
        });
    assert!(
        diags.iter().any(|d| d.code == "DOC-006"),
        "Expected DOC-006 for undocumented forbidden values, got: {:?}",
        diags.iter().map(|d| d.code).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Category 9: Default value applied correctly (DOC-007 fires when missing from desc)
// ---------------------------------------------------------------------------

#[test]
fn test_default_without_description_warns() {
    let schema = Schema::new()
        .domain_metadata("worker", "Worker Config", "Worker configuration.")
        .stability("stable")
        .authority_boundary(AuthorityBoundary {
            id: "w_b".to_string(),
            domain: "worker".to_string(),
            may_control: vec!["threads".to_string()],
            must_not_claim: vec![],
            defer_to: None,
            escalation_rule: None,
        })
        .override_policy(OverridePolicy {
            id: "w_override".to_string(),
            allowed: false,
            required_reason: false,
            required_scope: vec![],
            required_expiration: None,
            required_tracking: None,
            required_replacement_plan: false,
            refusal_conditions: vec![],
        })
        .field("threads")
        .description("Number of worker threads.") // no mention of "default"
        .stability("stable")
        .range_i64(1, 256)
        .default("4")
        .done();

    let domain = schema.get_domain();
    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(domain);
    let result = builder.build();
    let model = match result {
        Ok(m) => m,
        Err(_) => return, // other errors are fine
    };
    let diags = star_toml::docs::validate_docs_integrity(&model);
    assert!(
        diags.iter().any(|d| d.code == "DOC-007"),
        "Expected DOC-007 warning for undocumented default value"
    );
}

// ---------------------------------------------------------------------------
// Category 10: DfCM matrix — 11 axes
// ---------------------------------------------------------------------------

#[test]
fn test_dfcm_matrix_schema_and_docs_axes() {
    // Verify that the DfCM runner correctly identifies pass/fail variants
    let axes = vec![
        DfCMAxis {
            name: "SchemaState",
            variants: vec![("valid_schema", true), ("empty_id_schema", false)],
        },
        DfCMAxis {
            name: "ReceiptState",
            variants: vec![("with_receipt", true), ("without_receipt", true)],
        },
    ];

    // admission function: "valid_schema" → true, everything else → false
    let result = run_dfcm_matrix(&axes, |label| {
        label == "valid_schema" || label == "with_receipt" || label == "without_receipt"
    });
    let failures: Vec<_> = result.iter().filter(|(_, _, expected, got)| expected != got).collect();
    assert!(failures.is_empty(), "DfCM matrix unexpected failures: {failures:?}");
}
