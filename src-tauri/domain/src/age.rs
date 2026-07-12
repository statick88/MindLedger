use chrono::{DateTime, Utc, NaiveDate, Datelike, Months};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Returns the number of days in a given month/year.
fn days_in_month(year: i32, month: u32) -> i32 {
    // Go to the 1st of the month, advance to next month, go back 1 day
    NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|d| d.checked_add_months(Months::new(1)))
        .and_then(|d| d.pred_opt())
        .map(|d| d.day() as i32)
        .unwrap_or(30)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Age {
    pub years: u32,
    pub months: u32,
    pub days: u32,
}

impl Age {
    pub fn new(years: u32, months: u32, days: u32) -> Self {
        Self { years, months, days }
    }
    
    pub fn from_birth_date(birth_date: NaiveDate, at: NaiveDate) -> Self {
        let mut years = at.year() - birth_date.year();
        let mut months = at.month() as i32 - birth_date.month() as i32;

        // Clamp birth day to the "at" month's dimension so that e.g.
        // born Jan 31 → Feb 29 treats the birth day as 29 (Feb's last day).
        let days_in_at_month = days_in_month(at.year(), at.month());
        let effective_birth_day = std::cmp::min(birth_date.day() as i32, days_in_at_month);
        let mut days = at.day() as i32 - effective_birth_day;

        if days < 0 {
            // Borrow one month. The effective birth day for the *previous* month
            // must also be clamped (e.g. born Jan 31 borrowing from Feb → clamp to 29).
            months -= 1;
            let prev_month = if at.month() == 1 { 12 } else { at.month() - 1 };
            let prev_year = if at.month() == 1 { at.year() - 1 } else { at.year() };
            let prev_month_dim = days_in_month(prev_year, prev_month);
            let effective_birth_day_prev = std::cmp::min(birth_date.day() as i32, prev_month_dim);
            days = at.day() as i32 + prev_month_dim - effective_birth_day_prev;
        }

        if months < 0 {
            years -= 1;
            months += 12;
        }

        Self::new(years as u32, months as u32, days as u32)
    }
    
    pub fn from_birth_datetime(birth_datetime: DateTime<Utc>, at: DateTime<Utc>) -> Self {
        let birth_date = birth_datetime.date_naive();
        let at_date = at.date_naive();
        
        let mut age = Self::from_birth_date(birth_date, at_date);
        
        if at.time() < birth_datetime.time() {
            if age.days > 0 {
                age.days -= 1;
            } else if age.months > 0 {
                age.months -= 1;
                let prev_month = if at.month() == 1 { 12 } else { at.month() - 1 };
                let prev_year = if at.month() == 1 { at.year() - 1 } else { at.year() };
                age.days = days_in_month(prev_year, prev_month) as u32 - 1;
            } else if age.years > 0 {
                age.years -= 1;
                age.months = 11;
                age.days = days_in_month(at.year() - 1, 12) as u32 - 1;
            }
        }
        
        age
    }
    
    /// Normalize age components so that days < 30 and months < 12.
    /// This is NOT called automatically — use it when you want to carry over
    /// excess days into months or months into years (e.g. for display purposes).
    pub fn normalize(&mut self) {
        if self.days >= 30 {
            self.months += self.days / 30;
            self.days %= 30;
        }
        if self.months >= 12 {
            self.years += self.months / 12;
            self.months %= 12;
        }
    }
    
    pub fn total_months(&self) -> u32 {
        self.years * 12 + self.months
    }
    
    pub fn total_days(&self, from: NaiveDate) -> u64 {
        let mut days = 0u64;
        let mut current = from;
        
        for _ in 0..self.years {
            days += if NaiveDate::from_ymd_opt(current.year(), 2, 29).is_some() { 366 } else { 365 };
            current = NaiveDate::from_ymd_opt(current.year() + 1, current.month(), current.day())
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(current.year() + 1, current.month(), 28).unwrap());
        }
        
        for _ in 0..self.months {
            let dim = days_in_month(current.year(), current.month()) as u64;
            days += dim;
            current = current.checked_add_months(Months::new(1)).unwrap_or(current);
        }
        
        days += self.days as u64;
        days
    }
    
    pub fn is_minor(&self, age_of_majority: u32) -> bool {
        self.years < age_of_majority
    }
    
    pub fn format_short(&self) -> String {
        if self.years > 0 {
            if self.months > 0 {
                format!("{}y {}m", self.years, self.months)
            } else {
                format!("{}y", self.years)
            }
        } else if self.months > 0 {
            if self.days > 0 {
                format!("{}m {}d", self.months, self.days)
            } else {
                format!("{}m", self.months)
            }
        } else {
            format!("{}d", self.days)
        }
    }
    
    pub fn format_long(&self) -> String {
        let mut parts = Vec::new();
        if self.years > 0 {
            parts.push(format!("{} year{}", self.years, if self.years == 1 { "" } else { "s" }));
        }
        if self.months > 0 {
            parts.push(format!("{} month{}", self.months, if self.months == 1 { "" } else { "s" }));
        }
        if self.days > 0 || parts.is_empty() {
            parts.push(format!("{} day{}", self.days, if self.days == 1 { "" } else { "s" }));
        }
        parts.join(", ")
    }
}

impl fmt::Display for Age {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_long())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgeBreakdown {
    pub years: u32,
    pub months: u32,
    pub days: u32,
    pub total_days: u64,
    pub total_months: u32,
    pub is_minor: bool,
    pub age_of_majority: u32,
}

impl AgeBreakdown {
    pub fn from_age(age: Age, birth_date: NaiveDate, age_of_majority: u32) -> Self {
        let _today = Utc::now().date_naive();
        Self {
            years: age.years,
            months: age.months,
            days: age.days,
            total_days: age.total_days(birth_date),
            total_months: age.total_months(),
            is_minor: age.is_minor(age_of_majority),
            age_of_majority,
        }
    }
    
    pub fn to_typescript_object(&self) -> serde_json::Value {
        serde_json::json!({
            "years": self.years,
            "months": self.months,
            "days": self.days,
            "totalDays": self.total_days,
            "totalMonths": self.total_months,
            "isMinor": self.is_minor,
            "ageOfMajority": self.age_of_majority,
            "formattedShort": Age::new(self.years, self.months, self.days).format_short(),
            "formattedLong": Age::new(self.years, self.months, self.days).format_long(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};
    
    #[test]
    fn test_age_calculation_basic() {
        let birth = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }
    
    #[test]
    fn test_age_calculation_with_months() {
        let birth = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 6);
        assert_eq!(age.days, 0);
    }
    
    #[test]
    fn test_age_calculation_with_days() {
        let birth = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 5);
    }
    
    #[test]
    fn test_age_calculation_cross_month_boundary() {
        let birth = NaiveDate::from_ymd_opt(2000, 1, 31).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 1);
        assert_eq!(age.days, 1);
    }
    
    #[test]
    fn test_age_calculation_leap_year() {
        let birth = NaiveDate::from_ymd_opt(2000, 2, 29).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 23);
        assert_eq!(age.months, 11);
        assert_eq!(age.days, 30);
        
        let at = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }
    
    #[test]
    fn test_age_calculation_end_of_month() {
        let birth = NaiveDate::from_ymd_opt(2000, 1, 31).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 1);
        assert_eq!(age.days, 0);
    }
    
    #[test]
    fn test_age_calculation_newborn() {
        let birth = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let at = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 0);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 14);
    }
    
    #[test]
    fn test_age_calculation_same_day() {
        let birth = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        let at = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        let age = Age::from_birth_date(birth, at);
        
        assert_eq!(age.years, 0);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }
    
    #[test]
    fn test_age_datetime_precision() {
        let birth = Utc.with_ymd_and_hms(2000, 6, 15, 14, 30, 0).single().unwrap();
        let at = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).single().unwrap();
        let age = Age::from_birth_datetime(birth, at);
        
        assert_eq!(age.years, 23);
        assert_eq!(age.months, 11);
        assert_eq!(age.days, 30);
        
        let at = Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 0).single().unwrap();
        let age = Age::from_birth_datetime(birth, at);
        
        assert_eq!(age.years, 24);
        assert_eq!(age.months, 0);
        assert_eq!(age.days, 0);
    }
    
    #[test]
    fn test_age_minor_check() {
        let age = Age::new(17, 11, 30);
        assert!(age.is_minor(18));
        
        let age = Age::new(18, 0, 0);
        assert!(!age.is_minor(18));
        
        let age = Age::new(18, 0, 1);
        assert!(!age.is_minor(18));
    }
    
    #[test]
    fn test_age_format() {
        let age = Age::new(25, 3, 10);
        assert_eq!(age.format_short(), "25y 3m");
        assert_eq!(age.format_long(), "25 years, 3 months, 10 days");
        
        let age = Age::new(0, 6, 15);
        assert_eq!(age.format_short(), "6m 15d");
        assert_eq!(age.format_long(), "6 months, 15 days");
        
        let age = Age::new(0, 0, 45);
        assert_eq!(age.format_short(), "45d");
        assert_eq!(age.format_long(), "45 days");
    }
    
    #[test]
    fn test_age_breakdown() {
        let birth = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
        let age = Age::from_birth_date(birth, Utc::now().date_naive());
        let breakdown = AgeBreakdown::from_age(age, birth, 18);
        
        assert_eq!(breakdown.years, age.years);
        assert_eq!(breakdown.months, age.months);
        assert_eq!(breakdown.days, age.days);
        assert!(breakdown.total_days > 0);
        assert!(breakdown.total_months > 0);
    }
}