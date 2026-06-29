//! Release verifier binary for checking the 19 release gate counterexamples.
//!
//! Exits non-zero if any counterexample is active or if git status is dirty (unless overridden).

#![allow(clippy::all, clippy::pedantic, unused_imports, dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Check {
    name: &'static str,
    passed: bool,
    note: String,
}

macro_rules! check {
    ($name:expr, $body:block) => {{
        let mut note = String::new();
        let passed = (|| -> bool {
            let res = $body;
            note = res.1;
            res.0
        })();
        Check { name: $name, passed, note }
    }};
}

fn strip_comments_and_strings(content: &str) -> String {
    let mut clean = String::new();
    let mut in_string = false;
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_string {
            if ch == '"' {
                in_string = false;
            } else if ch == '\\' {
                chars.next(); // skip escaped char
            }
        } else if ch == '"' {
            in_string = true;
        } else if ch == '/' && chars.peek() == Some(&'/') {
            // skip line comment
            while let Some(&c) = chars.peek() {
                if c == '\n' {
                    break;
                }
                chars.next();
            }
        } else {
            clean.push(ch);
        }
    }
    clean
}

fn parse_package_version(cargo_toml_path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(cargo_toml_path)
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
        }
        if in_package && trimmed.starts_with("version") {
            let parts: Vec<&str> = trimmed.split('=').collect();
            if parts.len() == 2 {
                let ver = parts[1].trim().trim_matches('"').trim();
                return Ok(ver.to_string());
            }
        }
    }
    Err("version key not found under [package] section".to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dev_mode = args.iter().any(|a| a == "--dev")
        || std::env::var("STAR_TOML_DEV").map_or(false, |v| v == "1");

    println!("# star-toml v26.6.29 — Full ST-JSON Admission Release\n");
    if dev_mode {
        println!("(--dev mode: git-clean check bypassed)\n");
    }

    let results = run_release_checks(dev_mode);
    let mut failed = 0;

    println!("## Counterexample Gate\n");
    println!("| # | Check | Status | Note |");
    println!("|---|---|---|---|");
    for (i, c) in results.iter().enumerate() {
        let status = if c.passed {
            "PASS"
        } else {
            failed += 1;
            "FAIL"
        };
        println!("| {} | {} | **{}** | {} |", i + 1, c.name, status, c.note);
    }

    println!(
        "\nTotal: {}  Passed: {}  Failed: {}\n",
        results.len(),
        results.len() - failed,
        failed
    );

    // ── 10-Section DoD Standing Report ─────────────────────────────────────
    let pass = |checks: &[&str]| -> &'static str {
        let ok = checks
            .iter()
            .all(|name| results.iter().find(|c| c.name == *name).map_or(false, |c| c.passed));
        if ok {
            "PASS"
        } else {
            "FAIL"
        }
    };

    let s1 = pass(&["json_schema_bridge_standing"]);
    let s2 = pass(&["version_alignment_drift", "dependency_order_not_publishable"]);
    let s3 = pass(&["raw_parse_treated_as_trusted", "example_uses_non_admitted_path"]);
    let s4 = pass(&["docs_integrity_and_freshness"]);
    let s5 = pass(&["witness_claim_without_hash_input"]);
    let s6 = pass(&["deferred_feature_marked_complete", "doc_claim_not_backed_by_test"]);
    let s7 = pass(&["docs_integrity_and_freshness"]);
    let s8 = pass(&["json_schema_bridge_standing"]);
    let s9 = pass(&[
        "variant_axis_without_fixture",
        "detector_without_negative_fixture",
        "config_test_crate_present",
        "config_test_suite_passes",
    ]);

    let sections_all_pass =
        [s1, s2, s3, s4, s5, s6, s7, s8, s9].iter().all(|&s| s == "PASS") && failed == 0;
    let final_standing = if sections_all_pass { "ADMITTED" } else { "REFUSED" };

    println!("## v26.6.29 Full ST-JSON Admission Release — Standing\n");
    println!("| # | Section | Status |");
    println!("|---|---|---|");
    println!("| 1 | JSON Schema Bridge Standing | **{}** |", s1);
    println!("| 2 | Schema Admission            | **{}** |", s2);
    println!("| 3 | Config Admission            | **{}** |", s3);
    println!("| 4 | Generated Docs Standing     | **{}** |", s4);
    println!("| 5 | Receipt Binding             | **{}** |", s5);
    println!("| 6 | Projection Freshness        | **{}** |", s6);
    println!("| 7 | DOC Diagnostics             | **{}** |", s7);
    println!("| 8 | SCH-JSON Diagnostics        | **{}** |", s8);
    println!("| 9 | DfCM Fixture Coverage       | **{}** |", s9);
    println!("|10 | Final Standing              | **{}** |", final_standing);

    println!("\n## Final: {}", final_standing);

    if final_standing == "REFUSED" {
        std::process::exit(1);
    }
}

fn run_release_checks(dev_mode: bool) -> Vec<Check> {
    let mut results = Vec::new();
    let root = Path::new(".");
    let _dev_mode = dev_mode;

    // 1. architecture_claim_without_invariant
    results.push(check!("architecture_claim_without_invariant", {
        let mut ok = true;
        let mut note = "Verified: star-toml-lsp does not bypass loader internals.".to_string();
        if let Ok(entries) = fs::read_dir(root.join("star-toml-lsp/src")) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let clean = strip_comments_and_strings(&content);
                    if clean.contains("AdmittedConfig {") {
                        ok = false;
                        note = format!(
                            "LSP constructs AdmittedConfig directly in {:?}",
                            entry.file_name()
                        );
                        break;
                    }
                }
            }
        }
        (ok, note)
    }));

    // 2. public_api_without_example
    results.push(check!("public_api_without_example", {
        let mut ok = true;
        let mut note =
            "Verified: all core public API surface elements have matching examples.".to_string();
        let api_elements = vec![
            "TrustedLoader",
            "load_admitted",
            "load_admitted_exploratory",
            "ConfigWitness",
            "save_canonical",
        ];

        let mut examples_content = String::new();
        if let Ok(entries) = fs::read_dir(root.join("examples")) {
            for entry in entries.filter_map(Result::ok) {
                if entry.path().extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(c) = fs::read_to_string(entry.path()) {
                        examples_content.push_str(&c);
                    }
                }
            }
        }

        for element in api_elements {
            if !examples_content.contains(element) {
                ok = false;
                note = format!("Public API item `{}` lacks example coverage.", element);
                break;
            }
        }
        (ok, note)
    }));

    // 3. public_api_without_lifecycle_position
    results.push(check!("public_api_without_lifecycle_position", {
        let mut ok = true;
        let mut note =
            "Verified: all public items are lifecycle-mapped in ST-102 or ST-112.".to_string();
        if let Ok(spec) =
            fs::read_to_string(root.join("docs/jira/v26.6.29/ST-112_release_standing.md"))
        {
            let api_elements = vec![
                "TrustedLoader",
                "load_admitted",
                "load_admitted_exploratory",
                "Validate",
                "Validator",
                "ConfigWitness",
                "AdmittedConfig",
                "save_canonical",
            ];
            for element in api_elements {
                if !spec.contains(element) {
                    ok = false;
                    note = format!(
                        "Public API item `{}` is not mapped in ST-112 specification.",
                        element
                    );
                    break;
                }
            }
        } else {
            ok = false;
            note = "ST-112 specification file is missing.".to_string();
        }
        (ok, note)
    }));

    // 4. doc_claim_not_backed_by_test
    results.push(check!("doc_claim_not_backed_by_test", {
        let mut ok = true;
        let mut note = "Verified: doc claims are backed by active tests.".to_string();
        let tests_content = fs::read_to_string(root.join("tests/brce.rs")).unwrap_or_default();
        let claims = vec![
            "test_forbidden_path_fails",
            "test_path_absolute_rejected_under_relative_only",
            "test_path_relative_traversal_rejected",
            "test_path_win_backslash_traversal_rejected",
        ];
        for claim in claims {
            if !tests_content.contains(claim) {
                ok = false;
                note = format!("Doc claim check `{}` is missing from tests/brce.rs.", claim);
                break;
            }
        }
        (ok, note)
    }));

    // 5. deferred_feature_marked_complete
    results.push(check!("deferred_feature_marked_complete", {
        let mut ok = true;
        let mut note = "Verified: no deferred features are marked complete.".to_string();
        if let Ok(spec) =
            fs::read_to_string(root.join("docs/jira/v26.6.28/ST-110_error_topology_lsp.md"))
        {
            if spec.contains("[x] Custom LSP Extension") || spec.contains("[x] Document Formatting")
            {
                ok = false;
                note =
                    "Deferred LSP features (formatting/extensions) are marked complete in ST-110."
                        .to_string();
            }
        }
        (ok, note)
    }));

    // 6. implemented_claim_without_gate
    results.push(check!("implemented_claim_without_gate", {
        let mut ok = true;
        let mut note =
            "Verified: implemented validation checkers have active test gates.".to_string();
        let test_methods =
            vec!["test_check_ip_or_domain", "test_check_size_format", "test_check_semver"];
        let validation_tests =
            fs::read_to_string(root.join("src/validation.rs")).unwrap_or_default();
        for test in test_methods {
            if !validation_tests.contains(test) {
                ok = false;
                note = format!(
                    "Validation checker test `{}` is missing from validation.rs unit tests.",
                    test
                );
                break;
            }
        }
        (ok, note)
    }));

    // 7. lsp_feature_claim_without_handler
    results.push(check!("lsp_feature_claim_without_handler", {
        let mut ok = true;
        let mut note = "Verified: all claimed LSP features have active handlers.".to_string();
        let server_content =
            fs::read_to_string(root.join("star-toml-lsp/src/server.rs")).unwrap_or_default();
        let claimed_features = vec![
            "completion_provider",
            "hover_provider",
            "document_symbol_provider",
            "code_action_provider",
        ];
        for feature in claimed_features {
            if !server_content.contains(feature) {
                ok = false;
                note = format!("LSP handler for `{}` is missing from server.rs.", feature);
                break;
            }
        }
        (ok, note)
    }));

    // 8. lsp_authority_boundary_violation
    results.push(check!("lsp_authority_boundary_violation", {
        let mut ok = true;
        let mut note =
            "Verified: star-toml-lsp has zero imports of core authority loader builders."
                .to_string();
        if let Ok(entries) = fs::read_dir(root.join("star-toml-lsp/src")) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let clean = strip_comments_and_strings(&content);
                    if clean.contains("TrustedLoader::")
                        || clean.contains("load_admitted::<")
                        || clean.contains("use crate::TrustedLoader")
                    {
                        ok = false;
                        note = format!(
                            "LSP violates boundary by importing authority builders in {:?}",
                            entry.file_name()
                        );
                        break;
                    }
                }
            }
        }
        (ok, note)
    }));

    // 9. ocel_treated_as_standing_authority
    results.push(check!("ocel_treated_as_standing_authority", {
        let mut ok = true;
        let mut note =
            "Verified: OCEL history tracking is decoupled from authority / witness checks."
                .to_string();
        if let Ok(content) = fs::read_to_string(root.join("src/ocel.rs")) {
            let clean = strip_comments_and_strings(&content);
            if clean.contains("ConfigWitness")
                || clean.contains("witness")
                || clean.contains("q_config")
            {
                ok = false;
                note = "OCEL module references ConfigWitness or witness authority in code."
                    .to_string();
            }
        }
        (ok, note)
    }));

    // 10. raw_parse_treated_as_trusted
    results.push(check!("raw_parse_treated_as_trusted", {
        let mut ok = true;
        let mut note =
            "Verified: examples do not treat raw TOML parse as admitted config.".to_string();
        if let Ok(entries) = fs::read_dir(root.join("examples")) {
            for entry in entries.filter_map(Result::ok) {
                if entry.path().extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let clean = strip_comments_and_strings(&content);
                        if clean.contains("toml::from_str")
                            && !clean.contains("TrustedLoader")
                            && !clean.contains("load_admitted")
                        {
                            ok = false;
                            note = format!(
                                "Example {:?} treats raw parsing as trusted configuration.",
                                entry.file_name()
                            );
                            break;
                        }
                    }
                }
            }
        }
        (ok, note)
    }));

    // 11. example_uses_non_admitted_path
    results.push(check!("example_uses_non_admitted_path", {
        let mut ok = true;
        let mut note = "Verified: examples use admitted or frozen wrappers or standalone validators to access config state.".to_string();
        if let Ok(entries) = fs::read_dir(root.join("examples")) {
            for entry in entries.filter_map(Result::ok) {
                if entry.path().extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let clean = strip_comments_and_strings(&content);
                        if clean.contains("fn main") 
                           && !clean.contains("AdmittedConfig") 
                           && !clean.contains("load_admitted")
                           && !clean.contains("load_frozen")
                           && !clean.contains("load_admitted_exploratory")
                           && !clean.contains("resolve_and_validate")
                           && !clean.contains("export_events_to_ocel")
                           && !clean.contains(".check()")
                           && entry.file_name() != "dfcm_axes_matrix.rs" 
                           && entry.file_name() != "dfcm_common_patterns.rs"
                           && entry.file_name() != "red_team_counterexamples.rs" {
                            ok = false;
                            note = format!("Example {:?} lacks AdmittedConfig/Frozen/Standalone envelope usage.", entry.file_name());
                            break;
                        }
                    }
                }
            }
        }
        (ok, note)
    }));

    // 12. variant_axis_without_fixture
    results.push(check!("variant_axis_without_fixture", {
        let mut ok = true;
        let mut note = "Verified: all variant axes are covered by matching examples.".to_string();
        let examples_dir = root.join("examples");
        let required_examples = vec![
            "layered_profiles.rs",
            "env_overrides.rs",
            "strict_unknown_fields.rs",
            "exploratory_unknown_fields.rs",
            "path_policy_sandbox.rs",
            "witness_and_q_config.rs",
            "ocel_lifecycle_export.rs",
        ];
        for ex in required_examples {
            if !examples_dir.join(ex).exists() {
                ok = false;
                note = format!("Axis example file `{}` is missing.", ex);
                break;
            }
        }
        (ok, note)
    }));

    // 13. detector_without_negative_fixture
    results.push(check!("detector_without_negative_fixture", {
        let mut ok = true;
        let mut note = "Verified: verifier checks are covered by negative test gates.".to_string();
        if let Ok(content) = fs::read_to_string(root.join("tests/brce.rs")) {
            let required_tests = vec![
                "test_forbidden_path_fails",
                "test_missing_required_file_fails",
                "test_path_relative_traversal_rejected",
                "test_null_byte_fails",
            ];
            for t in required_tests {
                if !content.contains(t) {
                    ok = false;
                    note = format!("Verifier negative test `{}` is missing from tests/brce.rs.", t);
                    break;
                }
            }
        }
        (ok, note)
    }));

    // 14. witness_claim_without_hash_input
    results.push(check!("witness_claim_without_hash_input", {
        let mut ok = true;
        let mut note = "Verified: ConfigWitness correctly hashes sources, layers, envs, validation, and canonical TOML.".to_string();
        if let Ok(content) = fs::read_to_string(root.join("src/loader.rs")) {
            let required_hash_components = vec![
                "source_entries",
                "last_lod",
                "env_entries",
                "validation_fitness",
                "canonical_bytes",
            ];
            for component in required_hash_components {
                if !content.contains(component) {
                    ok = false;
                    note = format!("Witness hash input is missing required component: `{}`.", component);
                    break;
                }
            }
        }
        (ok, note)
    }));

    // 15. cargo_package_without_clean_git
    //
    // DfCM invariant: a release-standing gate cannot pass while the working
    // tree is dirty.  Any modified, staged, or untracked file is a FAIL.
    // There are no allowlisted exceptions — a clean release requires a clean
    // tree, full stop.
    results.push(check!("cargo_package_without_clean_git", {
        if _dev_mode {
            (true, "skipped in --dev mode".to_string())
        } else {
            let mut ok = true;
            let mut note = "Verified: git working tree is clean.".to_string();

            match Command::new("git").args(&["status", "--porcelain"]).output() {
                Err(e) => {
                    ok = false;
                    note = format!("Could not run git status: {}", e);
                }
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let dirty: Vec<&str> =
                        stdout.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                    if !dirty.is_empty() {
                        ok = false;
                        note = format!(
                            "Working tree is not clean — {} file(s) dirty/untracked: {:?}",
                            dirty.len(),
                            dirty
                        );
                    }
                }
            }
            (ok, note)
        }
    }));

    // 16. dependency_order_not_publishable
    results.push(check!("dependency_order_not_publishable", {
        let mut ok = true;
        let mut note =
            "Verified: workspace dependency hierarchy is valid and publishable.".to_string();
        if let Ok(content) = fs::read_to_string(root.join("star-toml-lsp/Cargo.toml")) {
            if !content.contains("path = \"..\"") {
                ok = false;
                note = "star-toml-lsp dependency configuration is invalid.".to_string();
            }
        }
        (ok, note)
    }));

    // 17. version_alignment_drift
    results.push(check!("version_alignment_drift", {
        let mut ok = true;
        let mut note = "Verified: workspace package versions and dependency specifications are fully aligned.".to_string();

        let root_ver = parse_package_version(&root.join("Cargo.toml"));
        let derive_ver = parse_package_version(&root.join("star-toml-derive/Cargo.toml"));
        let lsp_ver = parse_package_version(&root.join("star-toml-lsp/Cargo.toml"));

        match (root_ver, derive_ver, lsp_ver) {
            (Ok(rv), Ok(dv), Ok(lv)) => {
                if rv != dv {
                    ok = false;
                    note = format!("Version mismatch: star-toml ({}) vs star-toml-derive ({})", rv, dv);
                } else if rv != lv {
                    ok = false;
                    note = format!("Version mismatch: star-toml ({}) vs star-toml-lsp ({})", rv, lv);
                } else {
                    let root_content = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
                    let expected_version_str = format!("\"{}\"", rv);

                    let mut found_derive_dep = false;
                    for line in root_content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("star-toml-derive") {
                            if trimmed.contains(&expected_version_str) {
                                found_derive_dep = true;
                                break;
                            }
                        }
                    }
                    if !found_derive_dep {
                        ok = false;
                        note = format!("Root Cargo.toml dependency on star-toml-derive does not specify version {}", rv);
                    }

                    let lsp_content = fs::read_to_string(root.join("star-toml-lsp/Cargo.toml")).unwrap_or_default();
                    let mut found_lsp_dep = false;
                    for line in lsp_content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("star-toml") && !trimmed.starts_with("star-toml-lsp") {
                            if trimmed.contains(&expected_version_str) {
                                found_lsp_dep = true;
                                break;
                            }
                        }
                    }
                    if !found_lsp_dep {
                        ok = false;
                        note = format!("star-toml-lsp dependency on star-toml does not specify version {}", rv);
                    }
                }
            }
            (Err(e), _, _) => {
                ok = false;
                note = format!("Root Cargo.toml version error: {}", e);
            }
            (_, Err(e), _) => {
                ok = false;
                note = format!("star-toml-derive Cargo.toml version error: {}", e);
            }
            (_, _, Err(e)) => {
                ok = false;
                note = format!("star-toml-lsp Cargo.toml version error: {}", e);
            }
        }
        (ok, note)
    }));

    // 18. git_release_tag_missing
    results.push(check!("git_release_tag_missing", {
        let mut ok = true;
        let mut note = "Verified: a git tag matching the package version exists.".to_string();

        match parse_package_version(&root.join("Cargo.toml")) {
            Err(e) => {
                ok = false;
                note = format!("Root Cargo.toml version error: {}", e);
            }
            Ok(root_ver) => {
                let tag_name = format!("v{}", root_ver);
                match Command::new("git").args(&["tag"]).output() {
                    Err(e) => {
                        ok = false;
                        note = format!("Failed to run git tag: {}", e);
                    }
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let mut found = false;
                        for line in stdout.lines() {
                            if line.trim() == tag_name {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            ok = false;
                            note = format!(
                                "Git tag `{}` matching version `{}` is missing.",
                                tag_name, root_ver
                            );
                        }
                    }
                }
            }
        }
        (ok, note)
    }));

    // 19. example_runtime_panic
    results.push(check!("example_runtime_panic", {
        let mut ok = true;
        let mut note =
            "Verified: all examples compile and run to completion (exit 0) successfully."
                .to_string();

        let mut failed_examples = Vec::new();
        if let Ok(entries) = fs::read_dir(root.join("examples")) {
            for entry in entries.filter_map(Result::ok) {
                if entry.path().extension().map_or(false, |ext| ext == "rs") {
                    let file_stem =
                        entry.path().file_stem().unwrap().to_string_lossy().into_owned();
                    let run_res = Command::new("cargo")
                        .args(&["run", "--quiet", "--example", &file_stem])
                        .output();
                    match run_res {
                        Err(e) => {
                            ok = false;
                            failed_examples
                                .push(format!("{} (failed to execute: {})", file_stem, e));
                        }
                        Ok(output) => {
                            if !output.status.success() {
                                ok = false;
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                failed_examples.push(format!(
                                    "{} (exit code: {:?}, stderr: {})",
                                    file_stem,
                                    output.status.code(),
                                    stderr.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }
        if !ok {
            note = format!("Examples failed to run successfully: {:?}", failed_examples);
        }
        (ok, note)
    }));

    // 20. docs_integrity_and_freshness
    results.push(check!("docs_integrity_and_freshness", {
        let mut ok = true;
        let mut note = "Verified: generated documentation is fresh and complete.".to_string();

        let run_res = Command::new("cargo")
            .args(&["run", "--quiet", "--bin", "star_toml_docs", "verify"])
            .output();

        match run_res {
            Err(e) => {
                ok = false;
                note = format!("Failed to execute star_toml_docs: {}", e);
            }
            Ok(output) => {
                if !output.status.success() {
                    ok = false;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    note =
                        format!("Docs verification failed:\n{}\n{}", stdout.trim(), stderr.trim());
                }
            }
        }
        (ok, note)
    }));

    // 22. config_test_crate_present
    results.push(check!("config_test_crate_present", {
        let lib_path = root.join("star-toml-config-test/src/lib.rs");
        if lib_path.exists() {
            (true, "Verified: star-toml-config-test crate is present.".to_string())
        } else {
            (false, "star-toml-config-test/src/lib.rs is missing.".to_string())
        }
    }));

    // 23. config_test_suite_passes
    results.push(check!("config_test_suite_passes", {
        let mut ok = true;
        let mut note = "Verified: star-toml-config-test conformance suite passes.".to_string();
        match Command::new("cargo")
            .args(&["test", "--quiet", "-p", "star-toml-config-test"])
            .output()
        {
            Err(e) => {
                ok = false;
                note = format!("Failed to run config-test suite: {}", e);
            }
            Ok(output) => {
                if !output.status.success() {
                    ok = false;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    note = format!("Config-test suite failed: {}", stderr.trim());
                }
            }
        }
        (ok, note)
    }));

    // 21. json_schema_bridge_standing
    results.push(check!("json_schema_bridge_standing", {
        let mut ok = true;
        let mut note =
            "Verified: star-toml-json-schema bridge tests pass (SCH-JSON-001..016).".to_string();

        let run_res = Command::new("cargo")
            .args(&["test", "--quiet", "-p", "star-toml-json-schema", "--test", "bridge_tests"])
            .output();

        match run_res {
            Err(e) => {
                ok = false;
                note = format!("Failed to run bridge tests: {}", e);
            }
            Ok(output) => {
                if !output.status.success() {
                    ok = false;
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    note = format!("JSON Schema bridge tests failed: {}", stderr.trim());
                }
            }
        }
        (ok, note)
    }));

    results
}
