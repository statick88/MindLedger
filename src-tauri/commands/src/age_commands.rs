use crate::error::{AppError, AppResult};
use soft_mindledger_domain::{Age, AgeBreakdown};
use tauri::command;
use chrono::NaiveDate;

/// Calculate age from a birth date to today.
#[command]
pub async fn calculate_age(birth_date: String) -> AppResult<Age> {
    let birth = NaiveDate::parse_from_str(&birth_date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;
    let today = chrono::Local::now().date_naive();
    Ok(Age::from_birth_date(birth, today))
}

/// Calculate age at a specific date.
#[command]
pub async fn calculate_age_at(birth_date: String, at_date: String) -> AppResult<Age> {
    let birth = NaiveDate::parse_from_str(&birth_date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid birth date: {}", e)))?;
    let at = NaiveDate::parse_from_str(&at_date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid at date: {}", e)))?;
    Ok(Age::from_birth_date(birth, at))
}

/// Calculate full age breakdown with totals, minority check, and formatted strings.
#[command]
pub async fn calculate_age_breakdown(
    birth_date: String,
    age_of_majority: Option<u32>,
) -> AppResult<AgeBreakdown> {
    let birth = NaiveDate::parse_from_str(&birth_date, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;
    let today = chrono::Local::now().date_naive();
    let age = Age::from_birth_date(birth, today);
    let majority = age_of_majority.unwrap_or(18);
    Ok(AgeBreakdown::from_age(age, birth, majority))
}
