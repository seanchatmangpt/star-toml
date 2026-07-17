//! AC5: Diagnostics for oracle integration with error codes and counterexamples.
//!
//! Provides comprehensive diagnostic reporting for oracle verdicts,
//! OCEL validation, and gate decisions.

pub mod code;

pub use code::{
    OracleDiagnostic, OracleDiagnosticCode, OracleDiagnosticReport, OracleDiagnostics,
    DiagnosticSummary,
};
