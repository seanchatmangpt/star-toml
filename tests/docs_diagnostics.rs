//! Integration tests for Star TOML configuration documentation diagnostics (DOC-001 - DOC-020).

use star_toml::{
    docs::{AuthorityBoundary, DocModelBuilder},
    Schema,
};

#[test]
fn test_doc_diagnostics_coverage() {
    // 1. DOC-002: ConfigFieldWithoutDescription
    let schema_missing_desc = Schema::new()
        .domain_metadata("sys", "System", "A stable domain description.")
        .stability("stable")
        .authority_boundary(AuthorityBoundary {
            id: "sys_b".to_string(),
            domain: "sys".to_string(),
            may_control: vec!["name".to_string()],
            must_not_claim: vec![],
            defer_to: None,
            escalation_rule: None,
        })
        .field("name")
            // missing description
            .non_empty()
            .done();

    let domain = schema_missing_desc.get_domain();
    let mut builder = DocModelBuilder::new("1.0.0");
    builder.add_domain(domain);
    let model = builder.build();
    assert!(model.is_err());
    let errs = model.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "DOC-002"));

    // 2. DOC-009: AuthorityBoundaryUndocumented
    let schema_no_boundary = Schema::new()
        .domain_metadata("sys", "System", "A stable domain description.")
        .stability("stable")
        .field("name")
        .description("System name")
        .non_empty()
        .done();

    let domain = schema_no_boundary.get_domain();
    let mut builder = DocModelBuilder::new("1.0.0");
    builder.add_domain(domain);
    let model = builder.build();
    assert!(model.is_err());
    let errs = model.unwrap_err();
    assert!(errs.iter().any(|e| e.code == "DOC-009"));
}
