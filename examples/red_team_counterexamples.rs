//! Demonstrates the DfCM counterexample check concepts to teach the red-team methodology.
//!
//! Run with: `cargo run --example red_team_counterexamples`

#![allow(dead_code)]

fn parse_valid_treated_as_trusted() {
    println!("[Counterexample 1] parse_valid_treated_as_trusted");
    println!("  - Attack: Treating basic TOML syntax parse as trusted config.");
    println!("  - Defense: loader::load_admitted() forces Validate + config bounds check.");
}

fn unknown_field_accepted_in_trusted_mode() {
    println!("[Counterexample 8] unknown_field_accepted_in_trusted_mode");
    println!("  - Attack: Silently accepting undeclared fields in production config.");
    println!("  - Defense: load_admitted() is strict by default; rejects unknown fields.");
}

fn validation_error_without_path() {
    println!("[Counterexample 10] validation_error_without_path");
    println!("  - Attack: Obfuscating config errors by returning root-level errors without paths.");
    println!("  - Defense: Errors carry path-precise loc segments.");
}

fn path_traversal_accepted() {
    println!("[Counterexample 12] path_traversal_accepted");
    println!("  - Attack: Using parent traversals (../) to escape sandbox roots.");
    println!("  - Defense: clean_path and resolve_and_validate reject parent directory components.");
}

fn witness_missing_source_digest() {
    println!("[Counterexample 18] witness_missing_source_digest");
    println!("  - Attack: Witness succeeds without matching file source digest.");
    println!("  - Defense: Source digests must exist and participate in the BLAKE3 witness hash.");
}

fn ocel_treated_as_standing_authority() {
    println!("[Counterexample 23] ocel_treated_as_standing_authority");
    println!("  - Attack: Using process-mining logs (OCEL) to grant configuration standing.");
    println!("  - Defense: OCEL is history-only; ConfigWitness and q_config reside solely in loader.");
}

fn main() {
    println!("--- Red Team DfCM Counterexamples Example ---");
    parse_valid_treated_as_trusted();
    unknown_field_accepted_in_trusted_mode();
    validation_error_without_path();
    path_traversal_accepted();
    witness_missing_source_digest();
    ocel_treated_as_standing_authority();
}
