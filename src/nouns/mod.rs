//! Domain nouns: substantive types representing oracle and gate concepts.
//!
//! AC3 & AC6: Evidence gate, oracle verdict, and verdict accumulation.

pub mod gate;

pub use gate::{EvidenceGate, OracleGateVerdict, OracleVerdict, VerdictSummary};
