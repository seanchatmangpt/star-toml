//! CLI Tool for star-toml Generated Configuration Documentation.

use std::{env, fs, path::Path, process};

use star_toml::{
    docs::{
        check_docs_freshness, project_json, project_markdown, validate_docs_integrity,
        DocDiagnostic, DocModelBuilder, ReceiptBinding,
    },
    Schema,
};

fn system_config_schema() -> Schema {
    Schema::new()
        .domain_metadata(
            "system",
            "Star TOML System Config",
            "Configuration schema governing star-toml sandbox policies, log paths, and cache sizes.",
        )
        .stability("stable")
        .authority_boundary(star_toml::docs::AuthorityBoundary {
            id: "system_boundary".to_string(),
            domain: "system".to_string(),
            may_control: vec!["log_path".to_string(), "cache_size".to_string()],
            must_not_claim: vec!["root_authority".to_string()],
            defer_to: Some("platform-ops".to_string()),
            escalation_rule: None,
        })
        .override_policy(star_toml::docs::OverridePolicy {
            id: "system_override".to_string(),
            allowed: false,
            required_reason: false,
            required_scope: vec![],
            required_expiration: None,
            required_tracking: None,
            required_replacement_plan: false,
            refusal_conditions: vec!["no-root-override".to_string()],
        })
        .field("log_path")
        .description("Absolute or relative path to write lifecycle events (required).")
        .stability("stable")
        .non_empty()
        .required()
        .done()
        .field("cache_size")
        .description(
            "Maximum memory cached configuration instances in bytes (default: 10485760).",
        )
        .stability("stable")
        .range_i64(1024, 1024 * 1024 * 1024)
        .default("10485760")
        .done()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: star_toml_docs [export | check | verify]");
        process::exit(1);
    }

    let cmd = &args[1];

    // Build the dynamic Schema and ingest into ConfigDomain
    let schema = system_config_schema();
    let domain = schema.get_domain();

    let mut builder = DocModelBuilder::new("26.6.29");
    builder.add_domain(domain);

    // Compute digests and build the model
    let mut model = match builder.build() {
        Ok(m) => m,
        Err(diags) => {
            eprintln!("Documentation Model Ingestion Failed with diagnostics:");
            print_diagnostics(&diags);
            process::exit(1);
        }
    };

    let md_path = Path::new("docs/config_reference.md");
    let json_path = Path::new("docs/config_metadata.json");

    // Add a receipt binding
    let md_proj = project_markdown(&model);
    let json_proj = project_json(&model);
    let proj_digest = star_toml::reports::blake3_hex(md_proj.as_bytes());

    let timestamp = env::var("STAR_TOML_DOCS_TIMESTAMP").unwrap_or_else(|_| "UNKNOWN".to_string());
    let signer =
        env::var("STAR_TOML_DOCS_SIGNER").unwrap_or_else(|_| "release-verifier-signer".to_string());
    let json_schema_digest = env::var("STAR_TOML_JSON_SCHEMA_DIGEST").unwrap_or_default();
    let admitted_schema_digest = env::var("STAR_TOML_ADMITTED_SCHEMA_DIGEST").unwrap_or_default();

    let receipt = ReceiptBinding {
        json_schema_digest,
        admitted_schema_digest,
        admitted_config_digest: model.input_digest.clone(),
        documentation_model_digest: model.model_digest.clone(),
        projection_digest: proj_digest,
        generator_version: "26.6.29".to_string(),
        timestamp,
        signer_or_operator: signer,
    };
    let _ = json_proj;
    model.receipts.push(receipt);
    model.canonicalize();

    match cmd.as_str() {
        "export" => {
            // Write projections to workspace docs directory
            if let Some(parent) = md_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let final_md = project_markdown(&model);
            let final_json = project_json(&model);
            let schema_path = Path::new("docs/schema.json");

            if let Err(e) = fs::write(md_path, &final_md) {
                eprintln!("Failed to write Markdown projection: {}", e);
                process::exit(1);
            }
            if let Err(e) = fs::write(json_path, &final_json) {
                eprintln!("Failed to write JSON projection: {}", e);
                process::exit(1);
            }
            // Write JSON Schema export (machine-readable schema layout)
            if let Err(e) = fs::write(schema_path, &final_json) {
                eprintln!("Failed to write schema.json: {}", e);
                process::exit(1);
            }
            println!("Exported generated documentation and metadata successfully.");
            println!("MD path: {}", md_path.display());
            println!("JSON path: {}", json_path.display());
            println!("Schema path: {}", schema_path.display());
        }
        "check" => {
            if !md_path.exists() || !json_path.exists() {
                eprintln!("Generated documentation files are missing!");
                process::exit(1);
            }
            let is_fresh = check_docs_freshness(&model, md_path, json_path);
            if !is_fresh {
                eprintln!("Generated documentation is stale or modified!");
                process::exit(1);
            }
            println!("Generated documentation is fresh.");
        }
        "verify" => {
            // Validate completeness of the model and run release verifier check
            let diags = validate_docs_integrity(&model);
            if !diags.is_empty() {
                eprintln!("Release verification REFUSED. Documentation errors:");
                print_diagnostics(&diags);
                process::exit(1);
            }

            if !md_path.exists() || !json_path.exists() {
                eprintln!(
                    "Release verification REFUSED: Generated documentation files are missing!"
                );
                process::exit(1);
            }

            let is_fresh = check_docs_freshness(&model, md_path, json_path);
            if !is_fresh {
                eprintln!("Release verification REFUSED: Documentation is stale!");
                process::exit(1);
            }

            println!("Release verification ADMITTED: Documentation is complete and fresh.");
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            process::exit(1);
        }
    }
}

fn print_diagnostics(diags: &[DocDiagnostic]) {
    println!("\n| Code | Name | Severity | Message | Repair Hint |");
    println!("|---|---|---|---|---|");
    for d in diags {
        println!(
            "| {} | {} | {} | {} | {} |",
            d.code, d.name, d.severity, d.message, d.repair_hint
        );
    }
}
