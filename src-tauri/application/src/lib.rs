//! Application layer — use cases and repository traits.
//!
//! This crate will contain the business logic use cases.
//! For now, it re-exports domain types for the commands layer.

pub mod docx_parser;

pub use soft_mindledger_domain::*;
