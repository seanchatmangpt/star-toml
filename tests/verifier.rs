//! Integration test: run the verifier_report binary and confirm all 23
//! counterexamples pass. This prevents verifier drift during future changes.

#[test]
fn verifier_report_all_pass() {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_verifier_report"))
        .status()
        .expect("failed to run verifier_report binary");
    assert!(status.success(), "verifier_report found active counterexamples");
}
