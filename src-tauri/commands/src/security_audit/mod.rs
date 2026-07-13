//! Security audit test suite for MindLedger.
//!
//! Covers: IPC injection fuzzing, DOCX parser resilience,
//! Tauri allowlist compliance, and business logic abuse tests.

pub mod ipc_fuzz_tests;
pub mod business_logic_tests;

use serde::Serialize;

/// Severity level for security findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// A single security finding from an audit test.
#[derive(Debug, Clone, Serialize)]
pub struct SecurityFinding {
    pub id: String,
    pub severity: Severity,
    pub category: String,
    pub description: String,
    pub evidence: String,
    pub remediation: String,
    pub cvss_score: Option<f64>,
}

/// Trait for security test modules.
pub trait SecurityTest {
    fn run(&self) -> Vec<SecurityFinding>;
}
