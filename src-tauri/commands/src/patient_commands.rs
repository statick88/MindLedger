use crate::error::{AppError, AppResult};
use soft_mindledger_domain::{
    Patient, PatientId, PatientFilter, Pagination,
    DocumentNumber, Email, PhoneNumber, FullName, Address, EmergencyContact,
    Gender, PatientRepository,
};
use soft_mindledger_infrastructure::{DbPool, SqlitePatientRepository};
use tauri::command;
use uuid::Uuid;
use chrono::NaiveDate;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct CreatePatientRequest {
    pub document_number: String,
    pub document_type: soft_mindledger_domain::DocumentType,
    pub country_code: String,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub date_of_birth: String,
    pub gender: Gender,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub phone_country_code: Option<String>,
    pub phone_extension: Option<String>,
    pub address_street: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_country: Option<String>,
    pub address_additional_info: Option<String>,
    pub emergency_contact_first_name: Option<String>,
    pub emergency_contact_last_name: Option<String>,
    pub emergency_contact_middle_name: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub emergency_contact_phone_number: Option<String>,
    pub emergency_contact_phone_country_code: Option<String>,
    pub emergency_contact_email: Option<String>,
    pub blood_type: Option<String>,
    pub allergies: Option<Vec<String>>,
    pub chronic_conditions: Option<Vec<String>>,
    pub medications: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PatientResponse {
    pub id: String,
    pub document_number: String,
    pub document_type: soft_mindledger_domain::DocumentType,
    pub country_code: String,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub date_of_birth: String,
    pub gender: Gender,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub emergency_contact: Option<String>,
    pub blood_type: Option<String>,
    pub allergies: Vec<String>,
    pub chronic_conditions: Vec<String>,
    pub medications: Vec<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Patient> for PatientResponse {
    fn from(p: Patient) -> Self {
        Self {
            id: p.id.to_string(),
            document_number: p.document_number.number,
            document_type: p.document_number.document_type,
            country_code: p.document_number.country_code,
            first_name: p.full_name.first_name,
            last_name: p.full_name.last_name,
            middle_name: p.full_name.middle_name,
            date_of_birth: p.date_of_birth.format("%Y-%m-%d").to_string(),
            gender: p.gender,
            email: p.email.map(|e| e.address),
            phone: p.phone.map(|p| p.to_string()),
            address: p.address.map(|a| a.full_address()),
            emergency_contact: p.emergency_contact.map(|ec| format!("{} - {} - {}", ec.name.full_name(), ec.relationship, ec.phone)),
            blood_type: p.blood_type,
            allergies: p.allergies,
            chronic_conditions: p.chronic_conditions,
            medications: p.medications,
            notes: p.notes,
            is_active: p.is_active,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[command]
pub async fn create_patient(
    db: tauri::State<'_, Arc<DbPool>>,
    request: CreatePatientRequest,
) -> AppResult<PatientResponse> {
    let repo = SqlitePatientRepository::new((**db).clone());
    
    let doc_number = DocumentNumber::new(
        request.document_number,
        request.document_type,
        request.country_code,
    )?;
    
    let full_name = FullName::new(
        request.first_name,
        request.last_name,
        request.middle_name,
    )?;
    
    let date_of_birth = NaiveDate::parse_from_str(&request.date_of_birth, "%Y-%m-%d")?;
    
    let email = request.email.map(Email::new).transpose()?;
    let phone = match (request.phone_number, request.phone_country_code) {
        (Some(num), Some(cc)) => Some(PhoneNumber::new(num, cc, request.phone_extension)?),
        _ => None,
    };
    
    let address = match (
        request.address_street,
        request.address_city,
        request.address_state,
        request.address_postal_code,
        request.address_country,
    ) {
        (Some(street), Some(city), Some(state), Some(postal), Some(country)) => {
            Some(Address::new(street, city, state, postal, country, request.address_additional_info)?)
        }
        _ => None,
    };
    
    let emergency_contact = match (
        request.emergency_contact_first_name,
        request.emergency_contact_last_name,
        request.emergency_contact_relationship,
        request.emergency_contact_phone_number,
        request.emergency_contact_phone_country_code,
    ) {
        (Some(first), Some(last), Some(rel), Some(phone), Some(cc)) => {
            let ec_name = FullName::new(first, last, request.emergency_contact_middle_name)?;
            let ec_phone = PhoneNumber::new(phone, cc, None)?;
            let ec_email = request.emergency_contact_email.map(Email::new).transpose()?;
            Some(EmergencyContact::new(ec_name, rel, ec_phone, ec_email)?)
        }
        _ => None,
    };
    
    let mut patient = Patient::new(doc_number, full_name, date_of_birth, request.gender);
    patient.update_contact_info(email, phone, address);
    patient.update_emergency_contact(emergency_contact);
    
    if let Some(bt) = request.blood_type {
        patient.update_medical_info(
            Some(bt),
            request.allergies.unwrap_or_default(),
            request.chronic_conditions.unwrap_or_default(),
            request.medications.unwrap_or_default(),
        );
    }
    
    if let Some(notes) = request.notes {
        patient.update_notes(Some(notes));
    }
    
    repo.create(&patient).await?;
    Ok(patient.into())
}

#[command]
pub async fn get_patient(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
) -> AppResult<PatientResponse> {
    let repo = SqlitePatientRepository::new((**db).clone());
    let patient_id = PatientId(Uuid::parse_str(&id)?);
    
    let patient = repo.get_by_id(patient_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Patient with id {} not found", id)))?;
    
    Ok(patient.into())
}

#[derive(serde::Deserialize)]
pub struct ListPatientsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub active_only: Option<bool>,
    pub gender: Option<Gender>,
    pub name_contains: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

#[command]
pub async fn list_patients(
    db: tauri::State<'_, Arc<DbPool>>,
    query: ListPatientsQuery,
) -> AppResult<PaginatedResponse<PatientResponse>> {
    let repo = SqlitePatientRepository::new((**db).clone());
    
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    let filter = PatientFilter {
        active_only: query.active_only,
        gender: query.gender,
        name_contains: query.name_contains,
        ..Default::default()
    };
    
    let pagination = Pagination::new(page, page_size);
    let total = repo.count(filter.clone()).await?;
    let patients = repo.list(filter, pagination).await?;
    
    Ok(PaginatedResponse {
        items: patients.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
        total_pages: (total + page_size - 1) / page_size,
    })
}

#[derive(serde::Deserialize)]
pub struct UpdatePatientRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub phone_country_code: Option<String>,
    pub phone_extension: Option<String>,
    pub address_street: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_country: Option<String>,
    pub address_additional_info: Option<String>,
    pub emergency_contact_first_name: Option<String>,
    pub emergency_contact_last_name: Option<String>,
    pub emergency_contact_middle_name: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub emergency_contact_phone_number: Option<String>,
    pub emergency_contact_phone_country_code: Option<String>,
    pub emergency_contact_email: Option<String>,
    pub blood_type: Option<String>,
    pub allergies: Option<Vec<String>>,
    pub chronic_conditions: Option<Vec<String>>,
    pub medications: Option<Vec<String>>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
}

#[command]
pub async fn update_patient(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
    request: UpdatePatientRequest,
) -> AppResult<PatientResponse> {
    let repo = SqlitePatientRepository::new((**db).clone());
    let patient_id = PatientId(Uuid::parse_str(&id)?);
    
    let mut patient = repo.get_by_id(patient_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Patient with id {} not found", id)))?;
    
    if let (Some(first), Some(last)) = (request.first_name, request.last_name) {
        patient.full_name = FullName::new(first, last, request.middle_name)?;
    }
    
    let email = request.email.map(Email::new).transpose()?;
    let phone = match (request.phone_number, request.phone_country_code) {
        (Some(num), Some(cc)) => Some(PhoneNumber::new(num, cc, request.phone_extension)?),
        _ => None,
    };
    
    let address = match (
        request.address_street,
        request.address_city,
        request.address_state,
        request.address_postal_code,
        request.address_country,
    ) {
        (Some(street), Some(city), Some(state), Some(postal), Some(country)) => {
            Some(Address::new(street, city, state, postal, country, request.address_additional_info)?)
        }
        _ => None,
    };
    
    patient.update_contact_info(email, phone, address);
    
    if let (Some(first), Some(last), Some(rel), Some(phone), Some(cc)) = (
        request.emergency_contact_first_name,
        request.emergency_contact_last_name,
        request.emergency_contact_relationship,
        request.emergency_contact_phone_number,
        request.emergency_contact_phone_country_code,
    ) {
        let ec_name = FullName::new(first, last, request.emergency_contact_middle_name)?;
        let ec_phone = PhoneNumber::new(phone, cc, None)?;
        let ec_email = request.emergency_contact_email.map(Email::new).transpose()?;
        let ec = EmergencyContact::new(ec_name, rel, ec_phone, ec_email)?;
        patient.update_emergency_contact(Some(ec));
    }
    
    if let Some(bt) = request.blood_type {
        patient.update_medical_info(
            Some(bt),
            request.allergies.unwrap_or_default(),
            request.chronic_conditions.unwrap_or_default(),
            request.medications.unwrap_or_default(),
        );
    }
    
    if let Some(notes) = request.notes {
        patient.update_notes(Some(notes));
    }
    
    if let Some(active) = request.is_active {
        if active {
            patient.activate();
        } else {
            patient.deactivate();
        }
    }
    
    repo.update(&patient).await?;
    Ok(patient.into())
}

#[command]
pub async fn delete_patient(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
) -> AppResult<bool> {
    let repo = SqlitePatientRepository::new((**db).clone());
    let patient_id = PatientId(Uuid::parse_str(&id)?);
    let deleted = repo.delete(patient_id).await?;
    Ok(deleted)
}

#[derive(serde::Deserialize)]
pub struct SearchPatientsQuery {
    pub query: String,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[command]
pub async fn search_patients(
    db: tauri::State<'_, Arc<DbPool>>,
    query: SearchPatientsQuery,
) -> AppResult<PaginatedResponse<PatientResponse>> {
    let repo = SqlitePatientRepository::new((**db).clone());
    
    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let pagination = Pagination::new(page, page_size);
    
    let patients = repo.search(&query.query, pagination).await?;
    let total = patients.len() as u64;
    
    Ok(PaginatedResponse {
        items: patients.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
        total_pages: 0,
    })
}

#[command]
pub async fn get_patient_count(
    db: tauri::State<'_, Arc<DbPool>>,
    active_only: Option<bool>,
) -> AppResult<u64> {
    let repo = SqlitePatientRepository::new((**db).clone());
    let filter = PatientFilter {
        active_only,
        gender: None,
        name_contains: None,
        ..Default::default()
    };
    
    let count = repo.count(filter).await?;
    Ok(count)
}
