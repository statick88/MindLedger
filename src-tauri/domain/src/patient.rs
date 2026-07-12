use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use crate::identifiers::PatientId;
use crate::value_objects::{DocumentNumber, Email, PhoneNumber, FullName, Address, EmergencyContact, Gender};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Patient {
    pub id: PatientId,
    pub document_number: DocumentNumber,
    pub full_name: FullName,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub email: Option<Email>,
    pub phone: Option<PhoneNumber>,
    pub address: Option<Address>,
    pub emergency_contact: Option<EmergencyContact>,
    pub blood_type: Option<String>,
    pub allergies: Vec<String>,
    pub chronic_conditions: Vec<String>,
    pub medications: Vec<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Patient {
    pub fn new(
        document_number: DocumentNumber,
        full_name: FullName,
        date_of_birth: NaiveDate,
        gender: Gender,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PatientId::new(),
            document_number,
            full_name,
            date_of_birth,
            gender,
            email: None,
            phone: None,
            address: None,
            emergency_contact: None,
            blood_type: None,
            allergies: Vec::new(),
            chronic_conditions: Vec::new(),
            medications: Vec::new(),
            notes: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn age(&self) -> crate::age::Age {
        crate::age::Age::from_birth_date(self.date_of_birth, Utc::now().date_naive())
    }
    
    pub fn age_at(&self, date: NaiveDate) -> crate::age::Age {
        crate::age::Age::from_birth_date(self.date_of_birth, date)
    }
    
    pub fn update_contact_info(&mut self, email: Option<Email>, phone: Option<PhoneNumber>, address: Option<Address>) {
        self.email = email;
        self.phone = phone;
        self.address = address;
        self.updated_at = Utc::now();
    }
    
    pub fn update_medical_info(&mut self, blood_type: Option<String>, allergies: Vec<String>, chronic_conditions: Vec<String>, medications: Vec<String>) {
        self.blood_type = blood_type;
        self.allergies = allergies;
        self.chronic_conditions = chronic_conditions;
        self.medications = medications;
        self.updated_at = Utc::now();
    }
    
    pub fn update_emergency_contact(&mut self, contact: Option<EmergencyContact>) {
        self.emergency_contact = contact;
        self.updated_at = Utc::now();
    }
    
    pub fn update_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }
    
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
    
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
    
    pub fn is_minor(&self, age_of_majority: u32) -> bool {
        self.age().years < age_of_majority
    }
    
    pub fn has_allergy(&self, allergy: &str) -> bool {
        self.allergies.iter().any(|a| a.eq_ignore_ascii_case(allergy))
    }
    
    pub fn has_condition(&self, condition: &str) -> bool {
        self.chronic_conditions.iter().any(|c| c.eq_ignore_ascii_case(condition))
    }
    
    pub fn add_allergy(&mut self, allergy: String) {
        let cleaned = allergy.trim().to_string();
        if !cleaned.is_empty() && !self.has_allergy(&cleaned) {
            self.allergies.push(cleaned);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn remove_allergy(&mut self, allergy: &str) {
        self.allergies.retain(|a| !a.eq_ignore_ascii_case(allergy));
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{DocumentNumber, DocumentType, Email, PhoneNumber, FullName, Address, Gender};
    use uuid::Uuid;
    use chrono::NaiveDate;
    
    fn create_test_patient() -> Patient {
        Patient::new(
            DocumentNumber::new("12345678".to_string(), DocumentType::NationalId, "AR".to_string()).unwrap(),
            FullName::new("Juan".to_string(), "Perez".to_string(), Some("Carlos".to_string())).unwrap(),
            NaiveDate::from_ymd_opt(1990, 5, 15).unwrap(),
            Gender::Male,
        )
    }
    
    #[test]
    fn test_patient_creation() {
        let patient = create_test_patient();
        assert!(patient.id.0 != Uuid::nil());
        assert_eq!(patient.document_number.number, "12345678");
        assert_eq!(patient.full_name.first_name, "Juan");
        assert_eq!(patient.full_name.last_name, "Perez");
        assert_eq!(patient.full_name.middle_name, Some("Carlos".to_string()));
        assert_eq!(patient.date_of_birth, NaiveDate::from_ymd_opt(1990, 5, 15).unwrap());
        assert_eq!(patient.gender, Gender::Male);
        assert!(patient.is_active);
    }
    
    #[test]
    fn test_patient_age_calculation() {
        let birth = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
        let patient = Patient::new(
            DocumentNumber::new("12345678".to_string(), DocumentType::NationalId, "AR".to_string()).unwrap(),
            FullName::new("Test".to_string(), "Patient".to_string(), None).unwrap(),
            birth,
            Gender::Female,
        );
        
        let age = patient.age();
        assert!(age.years >= 14);
    }
    
    #[test]
    fn test_patient_contact_update() {
        let mut patient = create_test_patient();
        let email = Email::new("test@example.com".to_string()).unwrap();
        let phone = PhoneNumber::new("1234567890".to_string(), "+54".to_string(), None).unwrap();
        let address = Address::new(
            "123 Main St".to_string(),
            "Buenos Aires".to_string(),
            "CABA".to_string(),
            "1000".to_string(),
            "Argentina".to_string(),
            None,
        ).unwrap();
        
        patient.update_contact_info(Some(email.clone()), Some(phone.clone()), Some(address.clone()));
        
        assert_eq!(patient.email, Some(email));
        assert_eq!(patient.phone, Some(phone));
        assert_eq!(patient.address, Some(address));
    }
    
    #[test]
    fn test_patient_medical_info_update() {
        let mut patient = create_test_patient();
        
        patient.update_medical_info(
            Some("A+".to_string()),
            vec!["Penicillin".to_string(), "Latex".to_string()],
            vec!["Hypertension".to_string()],
            vec!["Lisinopril 10mg".to_string()],
        );
        
        assert_eq!(patient.blood_type, Some("A+".to_string()));
        assert_eq!(patient.allergies.len(), 2);
        assert_eq!(patient.chronic_conditions.len(), 1);
        assert_eq!(patient.medications.len(), 1);
    }
    
    #[test]
    fn test_patient_allergy_management() {
        let mut patient = create_test_patient();
        
        patient.add_allergy("Aspirin".to_string());
        assert!(patient.has_allergy("aspirin"));
        assert!(patient.has_allergy("ASPIRIN"));
        
        patient.add_allergy("Aspirin".to_string());
        assert_eq!(patient.allergies.len(), 1);
        
        patient.remove_allergy("aspirin");
        assert!(!patient.has_allergy("aspirin"));
        assert_eq!(patient.allergies.len(), 0);
    }
    
    #[test]
    fn test_patient_activation() {
        let mut patient = create_test_patient();
        assert!(patient.is_active);
        
        patient.deactivate();
        assert!(!patient.is_active);
        
        patient.activate();
        assert!(patient.is_active);
    }
    
    #[test]
    fn test_patient_minor_check() {
        let birth = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
        let patient = Patient::new(
            DocumentNumber::new("12345678".to_string(), DocumentType::NationalId, "AR".to_string()).unwrap(),
            FullName::new("Test".to_string(), "Patient".to_string(), None).unwrap(),
            birth,
            Gender::Female,
        );
        
        assert!(patient.is_minor(18));
        
        let birth = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let patient = Patient::new(
            DocumentNumber::new("12345678".to_string(), DocumentType::NationalId, "AR".to_string()).unwrap(),
            FullName::new("Test".to_string(), "Patient".to_string(), None).unwrap(),
            birth,
            Gender::Female,
        );
        
        assert!(!patient.is_minor(18));
    }
}