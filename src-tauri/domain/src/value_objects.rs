use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
            Gender::Other => write!(f, "Other"),
            Gender::PreferNotToSay => write!(f, "Prefer not to say"),
        }
    }
}

impl FromStr for Gender {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Male" => Ok(Gender::Male),
            "Female" => Ok(Gender::Female),
            "Other" => Ok(Gender::Other),
            "Prefer not to say" => Ok(Gender::PreferNotToSay),
            _ => Err(format!("Invalid gender: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    NationalId,
    Passport,
    DriversLicense,
    ForeignId,
    BirthCertificate,
    Other,
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentType::NationalId => write!(f, "DNI"),
            DocumentType::Passport => write!(f, "PASSPORT"),
            DocumentType::DriversLicense => write!(f, "LICENSE"),
            DocumentType::ForeignId => write!(f, "CE"),
            DocumentType::BirthCertificate => write!(f, "BIRTH"),
            DocumentType::Other => write!(f, "OTHER"),
        }
    }
}

impl FromStr for DocumentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DNI" => Ok(DocumentType::NationalId),
            "PASSPORT" => Ok(DocumentType::Passport),
            "LICENSE" => Ok(DocumentType::DriversLicense),
            "CE" => Ok(DocumentType::ForeignId),
            "BIRTH" => Ok(DocumentType::BirthCertificate),
            "OTHER" => Ok(DocumentType::Other),
            _ => Err(format!("Invalid document type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentNumber {
    pub number: String,
    pub document_type: DocumentType,
    pub country_code: String,
}

impl DocumentNumber {
    pub fn new(number: String, document_type: DocumentType, country_code: String) -> Result<Self, DocumentNumberError> {
        let cleaned = number.trim().to_string();
        let cleaned_country = country_code.trim().to_uppercase();
        
        if cleaned.is_empty() {
            return Err(DocumentNumberError::EmptyNumber);
        }
        if cleaned_country.is_empty() || cleaned_country.len() != 2 {
            return Err(DocumentNumberError::InvalidCountryCode);
        }
        
        Ok(Self {
            number: cleaned,
            document_type,
            country_code: cleaned_country,
        })
    }
    
    pub fn formatted(&self) -> String {
        format!("{}:{}", self.document_type, self.number)
    }
}

impl fmt::Display for DocumentNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.document_type, self.number)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentNumberError {
    #[error("Document number cannot be empty")]
    EmptyNumber,
    #[error("Country code must be 2 characters (ISO 3166-1 alpha-2)")]
    InvalidCountryCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email {
    pub address: String,
}

impl Email {
    pub fn new(address: String) -> Result<Self, EmailError> {
        let cleaned = address.trim().to_lowercase();
        
        if cleaned.is_empty() {
            return Err(EmailError::Empty);
        }
        
        if !Self::is_valid(&cleaned) {
            return Err(EmailError::InvalidFormat);
        }
        
        Ok(Self { address: cleaned })
    }
    
    fn is_valid(email: &str) -> bool {
        email.contains('@')
            && email.split('@').count() == 2
            && email.split('@').nth(1).map_or(false, |domain| {
                domain.contains('.')
                    && domain.len() > 2
                    && !domain.starts_with('.')
                    && !domain.starts_with('-')
            })
            && !email.starts_with('@')
            && !email.ends_with('@')
            && !email.starts_with('.')
            && !email.ends_with('.')
    }
    
    pub fn domain(&self) -> &str {
        self.address.split('@').nth(1).unwrap_or("")
    }
    
    pub fn local_part(&self) -> &str {
        self.address.split('@').next().unwrap_or("")
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmailError {
    #[error("Email cannot be empty")]
    Empty,
    #[error("Invalid email format")]
    InvalidFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub number: String,
    pub country_code: String,
    pub extension: Option<String>,
}

impl PhoneNumber {
    pub fn new(number: String, country_code: String, extension: Option<String>) -> Result<Self, PhoneNumberError> {
        let cleaned_number: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        let cleaned_country = country_code.trim().to_string();
        
        if cleaned_number.len() < 7 || cleaned_number.len() > 15 {
            return Err(PhoneNumberError::InvalidLength);
        }
        if cleaned_country.is_empty() || !cleaned_country.starts_with('+') {
            return Err(PhoneNumberError::InvalidCountryCode);
        }
        
        Ok(Self {
            number: cleaned_number,
            country_code: cleaned_country,
            extension: extension.map(|e| e.trim().to_string()).filter(|e| !e.is_empty()),
        })
    }
    
    pub fn e164(&self) -> String {
        format!("{}{}", self.country_code, self.number)
    }
    
    pub fn national_format(&self) -> String {
        if self.number.len() == 10 {
            format!("({}) {}-{}", &self.number[0..3], &self.number[3..6], &self.number[6..])
        } else {
            self.number.clone()
        }
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ext) = &self.extension {
            write!(f, "{} x{}", self.e164(), ext)
        } else {
            write!(f, "{}", self.e164())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PhoneNumberError {
    #[error("Phone number must be between 7 and 15 digits")]
    InvalidLength,
    #[error("Country code must start with +")]
    InvalidCountryCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullName {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
}

impl FullName {
    pub fn new(
        first_name: String,
        last_name: String,
        middle_name: Option<String>,
    ) -> Result<Self, FullNameError> {
        let first = first_name.trim();
        let last = last_name.trim();
        let middle = middle_name.as_ref().map(|m| m.trim()).filter(|m| !m.is_empty());
        
        if first.is_empty() {
            return Err(FullNameError::EmptyFirstName);
        }
        if last.is_empty() {
            return Err(FullNameError::EmptyLastName);
        }
        if first.len() > 50 || last.len() > 50 {
            return Err(FullNameError::TooLong);
        }
        if let Some(m) = &middle {
            if m.len() > 50 {
                return Err(FullNameError::TooLong);
            }
        }
        
        Ok(Self {
            first_name: first.to_string(),
            last_name: last.to_string(),
            middle_name: middle.map(|m| m.to_string()),
        })
    }
    
    pub fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle) => format!("{} {} {}", self.first_name, middle, self.last_name),
            None => format!("{} {}", self.first_name, self.last_name),
        }
    }
    
    pub fn sorted_name(&self) -> String {
        format!("{}, {}", self.last_name.to_uppercase(), self.first_name)
    }
    
    pub fn initials(&self) -> String {
        let mut init = String::new();
        init.push(self.first_name.chars().next().unwrap_or('?'));
        if let Some(m) = &self.middle_name {
            init.push(m.chars().next().unwrap_or('?'));
        }
        init.push(self.last_name.chars().next().unwrap_or('?'));
        init.to_uppercase()
    }
}

impl fmt::Display for FullName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FullNameError {
    #[error("First name cannot be empty")]
    EmptyFirstName,
    #[error("Last name cannot be empty")]
    EmptyLastName,
    #[error("Name too long (max 50 characters)")]
    TooLong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub additional_info: Option<String>,
}

impl Address {
    pub fn new(
        street: String,
        city: String,
        state: String,
        postal_code: String,
        country: String,
        additional_info: Option<String>,
    ) -> Result<Self, AddressError> {
        if street.trim().is_empty() {
            return Err(AddressError::EmptyStreet);
        }
        if city.trim().is_empty() {
            return Err(AddressError::EmptyCity);
        }
        if state.trim().is_empty() {
            return Err(AddressError::EmptyState);
        }
        if postal_code.trim().is_empty() {
            return Err(AddressError::EmptyPostalCode);
        }
        if country.trim().is_empty() {
            return Err(AddressError::EmptyCountry);
        }
        
        Ok(Self {
            street: street.trim().to_string(),
            city: city.trim().to_string(),
            state: state.trim().to_string(),
            postal_code: postal_code.trim().to_string(),
            country: country.trim().to_string(),
            additional_info: additional_info.map(|a| a.trim().to_string()).filter(|a| !a.is_empty()),
        })
    }
    
    pub fn full_address(&self) -> String {
        let mut parts = vec![
            self.street.clone(),
            format!("{}, {} {}", self.city, self.state, self.postal_code),
            self.country.clone(),
        ];
        if let Some(add) = &self.additional_info {
            parts.push(add.clone());
        }
        parts.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AddressError {
    #[error("Street cannot be empty")]
    EmptyStreet,
    #[error("City cannot be empty")]
    EmptyCity,
    #[error("State cannot be empty")]
    EmptyState,
    #[error("Postal code cannot be empty")]
    EmptyPostalCode,
    #[error("Country cannot be empty")]
    EmptyCountry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub name: FullName,
    pub relationship: String,
    pub phone: PhoneNumber,
    pub email: Option<Email>,
}

impl EmergencyContact {
    pub fn new(
        name: FullName,
        relationship: String,
        phone: PhoneNumber,
        email: Option<Email>,
    ) -> Result<Self, EmergencyContactError> {
        if relationship.trim().is_empty() {
            return Err(EmergencyContactError::EmptyRelationship);
        }
        
        Ok(Self {
            name,
            relationship: relationship.trim().to_string(),
            phone,
            email,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmergencyContactError {
    #[error("Relationship cannot be empty")]
    EmptyRelationship,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_number() {
        let doc = DocumentNumber::new("12345678".to_string(), DocumentType::NationalId, "AR".to_string()).unwrap();
        assert_eq!(doc.number, "12345678");
        assert_eq!(doc.document_type, DocumentType::NationalId);
        assert_eq!(doc.country_code, "AR");
        assert_eq!(doc.formatted(), "DNI:12345678");
    }
    
    #[test]
    fn test_document_number_validation() {
        assert!(DocumentNumber::new("".to_string(), DocumentType::NationalId, "AR".to_string()).is_err());
        assert!(DocumentNumber::new("123".to_string(), DocumentType::NationalId, "AR".to_string()).is_ok());
        assert!(DocumentNumber::new("123".to_string(), DocumentType::NationalId, "ARG".to_string()).is_err());
        assert!(DocumentNumber::new("123".to_string(), DocumentType::NationalId, "".to_string()).is_err());
    }
    
    #[test]
    fn test_email() {
        let email = Email::new("Test@Example.COM".to_string()).unwrap();
        assert_eq!(email.address, "test@example.com");
        assert_eq!(email.domain(), "example.com");
        assert_eq!(email.local_part(), "test");
    }
    
    #[test]
    fn test_email_validation() {
        assert!(Email::new("invalid".to_string()).is_err());
        assert!(Email::new("@example.com".to_string()).is_err());
        assert!(Email::new("test@".to_string()).is_err());
        assert!(Email::new("test@.com".to_string()).is_err());
        assert!(Email::new("test@example".to_string()).is_err());
    }
    
    #[test]
    fn test_phone_number() {
        let phone = PhoneNumber::new("123-456-7890".to_string(), "+54".to_string(), Some("123".to_string())).unwrap();
        assert_eq!(phone.number, "1234567890");
        assert_eq!(phone.country_code, "+54");
        assert_eq!(phone.extension, Some("123".to_string()));
        assert_eq!(phone.e164(), "+541234567890");
        assert_eq!(phone.national_format(), "(123) 456-7890");
    }
    
    #[test]
    fn test_phone_number_validation() {
        assert!(PhoneNumber::new("123".to_string(), "+54".to_string(), None).is_err());
        assert!(PhoneNumber::new("1234567890".to_string(), "54".to_string(), None).is_err());
    }
    
    #[test]
    fn test_full_name() {
        let name = FullName::new("Juan".to_string(), "Perez".to_string(), Some("Carlos".to_string())).unwrap();
        assert_eq!(name.full_name(), "Juan Carlos Perez");
        assert_eq!(name.sorted_name(), "PEREZ, Juan");
        assert_eq!(name.initials(), "JCP");
        
        let name = FullName::new("Maria".to_string(), "Gonzalez".to_string(), None).unwrap();
        assert_eq!(name.full_name(), "Maria Gonzalez");
        assert_eq!(name.initials(), "MG");
    }
    
    #[test]
    fn test_full_name_validation() {
        assert!(FullName::new("".to_string(), "Perez".to_string(), None).is_err());
        assert!(FullName::new("Juan".to_string(), "".to_string(), None).is_err());
        assert!(FullName::new("A".repeat(51), "Perez".to_string(), None).is_err());
    }
    
    #[test]
    fn test_address() {
        let addr = Address::new(
            "123 Main St".to_string(),
            "Buenos Aires".to_string(),
            "CABA".to_string(),
            "1000".to_string(),
            "Argentina".to_string(),
            Some("Apt 4B".to_string()),
        ).unwrap();
        
        assert_eq!(addr.full_address(), "123 Main St, Buenos Aires, CABA 1000, Argentina, Apt 4B");
    }
    
    #[test]
    fn test_address_validation() {
        assert!(Address::new("".to_string(), "City".to_string(), "State".to_string(), "1000".to_string(), "Country".to_string(), None).is_err());
        assert!(Address::new("Street".to_string(), "".to_string(), "State".to_string(), "1000".to_string(), "Country".to_string(), None).is_err());
    }
    
    #[test]
    fn test_emergency_contact() {
        let name = FullName::new("Ana".to_string(), "Perez".to_string(), None).unwrap();
        let phone = PhoneNumber::new("9876543210".to_string(), "+54".to_string(), None).unwrap();
        
        let contact = EmergencyContact::new(name, "Spouse".to_string(), phone, None).unwrap();
        assert_eq!(contact.relationship, "Spouse");
        assert_eq!(contact.name.full_name(), "Ana Perez");
    }
}