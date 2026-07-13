pub mod error;
pub mod patient_commands;
pub mod accounting_commands;
pub mod diagnostics_commands;
pub mod age_commands;

#[cfg(test)]
mod e2e_integration;

pub use error::{AppError, AppResult};
pub use patient_commands::*;
pub use accounting_commands::*;
pub use diagnostics_commands::*;
pub use age_commands::*;
