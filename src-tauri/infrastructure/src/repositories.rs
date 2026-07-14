use crate::database::DbPool;
use soft_mindledger_domain::{
    Patient, PatientId, DocumentNumber, Email, PhoneNumber, FullName, Address,
    EmergencyContact, PatientFilter, Pagination, RepositoryError,
    PatientRepository,
};
use rusqlite::params;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use async_trait::async_trait;

pub struct SqlitePatientRepository {
    pool: DbPool,
}

impl SqlitePatientRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_patient(row: &rusqlite::Row) -> rusqlite::Result<Patient> {
        let id: String = row.get("id")?;
        let document_number = DocumentNumber::new(
            row.get::<_, String>("document_number")?,
            row.get::<_, String>("document_type")?.parse().map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?,
            row.get::<_, String>("country_code")?,
        ).map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;

        let full_name = FullName::new(
            row.get::<_, String>("first_name")?,
            row.get::<_, String>("last_name")?,
            row.get::<_, Option<String>>("middle_name")?,
        ).map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;

        let date_of_birth: String = row.get("date_of_birth")?;
        let gender: String = row.get("gender")?;

        let email: Option<String> = row.get("email")?;
        let email = email
            .map(|e| Email::new(e))
            .transpose()
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;

        let phone_number: Option<String> = row.get("phone_number")?;
        let phone_country_code: Option<String> = row.get("phone_country_code")?;
        let phone_extension: Option<String> = row.get("phone_extension")?;

        let phone = match (phone_number, phone_country_code) {
            (Some(num), Some(cc)) => Some(
                PhoneNumber::new(num, cc, phone_extension)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?,
            ),
            _ => None,
        };

        let address_street: Option<String> = row.get("address_street")?;
        let address_city: Option<String> = row.get("address_city")?;
        let address_state: Option<String> = row.get("address_state")?;
        let address_postal_code: Option<String> = row.get("address_postal_code")?;
        let address_country: Option<String> = row.get("address_country")?;
        let address_additional: Option<String> = row.get("address_additional_info")?;

        let address = match (address_street, address_city, address_state, address_postal_code, address_country) {
            (Some(street), Some(city), Some(state), Some(postal), Some(country)) => Some(
                Address::new(street, city, state, postal, country, address_additional)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?,
            ),
            _ => None,
        };

        let ec_first: Option<String> = row.get("emergency_contact_name_first")?;
        let ec_last: Option<String> = row.get("emergency_contact_name_last")?;
        let ec_middle: Option<String> = row.get("emergency_contact_name_middle")?;
        let ec_rel: Option<String> = row.get("emergency_contact_relationship")?;
        let ec_phone: Option<String> = row.get("emergency_contact_phone_number")?;
        let ec_cc: Option<String> = row.get("emergency_contact_phone_country_code")?;
        let ec_email: Option<String> = row.get("emergency_contact_email")?;

        let emergency_contact = match (ec_first, ec_last, ec_rel, ec_phone, ec_cc) {
            (Some(first), Some(last), Some(rel), Some(phone), Some(cc)) => {
                let ec_name = FullName::new(first, last, ec_middle)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;
                let ec_phone = PhoneNumber::new(phone, cc, None)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;
                let ec_email = ec_email
                    .map(|e| Email::new(e))
                    .transpose()
                    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;
                Some(EmergencyContact::new(ec_name, rel, ec_phone, ec_email)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?)
            }
            _ => None,
        };

        let allergies_json: String = row.get("allergies").unwrap_or_default();
        let chronic_json: String = row.get("chronic_conditions").unwrap_or_default();
        let meds_json: String = row.get("medications").unwrap_or_default();

        let allergies: Vec<String> = serde_json::from_str(&allergies_json)
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;
        let chronic_conditions: Vec<String> = serde_json::from_str(&chronic_json)
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;
        let medications: Vec<String> = serde_json::from_str(&meds_json)
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?;

        let is_active: i64 = row.get("is_active")?;
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;

        Ok(Patient {
            id: PatientId(Uuid::parse_str(&id).map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?),
            document_number,
            full_name,
            date_of_birth: NaiveDate::parse_from_str(&date_of_birth, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?,
            gender: gender.parse().map_err(|e: String| rusqlite::Error::InvalidParameterName(e))?,
            email,
            phone,
            address,
            emergency_contact,
            blood_type: row.get("blood_type")?,
            allergies,
            chronic_conditions,
            medications,
            notes: row.get("notes")?,
            is_active: is_active == 1,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("{}", e)))?
                .with_timezone(&Utc),
        })
    }
}

#[async_trait]
impl PatientRepository for SqlitePatientRepository {
    async fn create(&self, patient: &Patient) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let patient = patient.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let doc_type: String = patient.document_number.document_type.to_string();
            let allergies = serde_json::to_string(&patient.allergies).map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            let chronic = serde_json::to_string(&patient.chronic_conditions).map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            let meds = serde_json::to_string(&patient.medications).map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            let (ec_first, ec_last, ec_middle, ec_rel, ec_phone, ec_cc, ec_email) = match &patient.emergency_contact {
                Some(ec) => (
                    Some(ec.name.first_name.clone()),
                    Some(ec.name.last_name.clone()),
                    ec.name.middle_name.clone(),
                    Some(ec.relationship.clone()),
                    Some(ec.phone.number.clone()),
                    Some(ec.phone.country_code.clone()),
                    ec.email.as_ref().map(|e| e.address.clone()),
                ),
                None => (None, None, None, None, None, None, None),
            };

            conn.execute(
                "INSERT INTO patients (
                    id, document_number, document_type, country_code,
                    first_name, last_name, middle_name, date_of_birth, gender,
                    email, phone_number, phone_country_code, phone_extension,
                    address_street, address_city, address_state, address_postal_code, address_country, address_additional_info,
                    emergency_contact_name_first, emergency_contact_name_last, emergency_contact_name_middle,
                    emergency_contact_relationship, emergency_contact_phone_number, emergency_contact_phone_country_code,
                    emergency_contact_email,
                    blood_type, allergies, chronic_conditions, medications, notes,
                    is_active, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34)",
                params![
                    patient.id.to_string(),
                    patient.document_number.number,
                    doc_type,
                    patient.document_number.country_code,
                    patient.full_name.first_name,
                    patient.full_name.last_name,
                    patient.full_name.middle_name,
                    patient.date_of_birth.format("%Y-%m-%d").to_string(),
                    patient.gender.to_string(),
                    patient.email.as_ref().map(|e| e.address.clone()),
                    patient.phone.as_ref().map(|p| p.number.clone()),
                    patient.phone.as_ref().map(|p| p.country_code.clone()),
                    patient.phone.as_ref().and_then(|p| p.extension.clone()),
                    patient.address.as_ref().map(|a| a.street.clone()),
                    patient.address.as_ref().map(|a| a.city.clone()),
                    patient.address.as_ref().map(|a| a.state.clone()),
                    patient.address.as_ref().map(|a| a.postal_code.clone()),
                    patient.address.as_ref().map(|a| a.country.clone()),
                    patient.address.as_ref().and_then(|a| a.additional_info.clone()),
                    ec_first, ec_last, ec_middle, ec_rel, ec_phone, ec_cc, ec_email,
                    patient.blood_type.as_ref(),
                    allergies, chronic, meds,
                    patient.notes.as_ref(),
                    if patient.is_active { 1i64 } else { 0i64 },
                    patient.created_at.to_rfc3339(),
                    patient.updated_at.to_rfc3339(),
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_by_id(&self, id: PatientId) -> Result<Option<Patient>, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT * FROM patients WHERE id = ?1")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut rows = stmt.query(params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let patient = Self::row_to_patient(row).map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(patient))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_by_document(&self, document_number: &str) -> Result<Option<Patient>, RepositoryError> {
        let pool = self.pool.clone();
        let doc = document_number.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT * FROM patients WHERE document_number = ?1")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut rows = stmt.query(params![doc])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let patient = Self::row_to_patient(row).map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(patient))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn list(&self, filter: PatientFilter, pagination: Pagination) -> Result<Vec<Patient>, RepositoryError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut where_clauses = vec!["1=1".to_string()];
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if filter.active_only.unwrap_or(true) {
                where_clauses.push("is_active = 1".to_string());
            }
            if let Some(gender) = filter.gender {
                where_clauses.push(format!("gender = ?{}", param_values.len() + 1));
                param_values.push(Box::new(gender.to_string()));
            }
            if let Some(name) = filter.name_contains {
                let idx1 = param_values.len() + 1;
                let idx2 = param_values.len() + 2;
                where_clauses.push(format!("(first_name LIKE ?{} OR last_name LIKE ?{})", idx1, idx2));
                let pattern = format!("%{}%", name);
                param_values.push(Box::new(pattern.clone()));
                param_values.push(Box::new(pattern));
            }

            let limit_idx = param_values.len() + 1;
            let offset_idx = param_values.len() + 2;
            param_values.push(Box::new(pagination.limit as i64));
            param_values.push(Box::new(pagination.offset as i64));

            let sql = format!(
                "SELECT * FROM patients WHERE {} ORDER BY last_name, first_name LIMIT ?{} OFFSET ?{}",
                where_clauses.join(" AND "), limit_idx, offset_idx
            );

            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| Self::row_to_patient(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut patients = Vec::new();
            for row in rows {
                patients.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(patients)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn count(&self, filter: PatientFilter) -> Result<u64, RepositoryError> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut where_clauses = vec!["1=1".to_string()];
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if filter.active_only.unwrap_or(true) {
                where_clauses.push("is_active = 1".to_string());
            }
            if let Some(gender) = filter.gender {
                where_clauses.push(format!("gender = ?{}", param_values.len() + 1));
                param_values.push(Box::new(gender.to_string()));
            }
            if let Some(name) = filter.name_contains {
                let idx1 = param_values.len() + 1;
                let idx2 = param_values.len() + 2;
                where_clauses.push(format!("(first_name LIKE ?{} OR last_name LIKE ?{})", idx1, idx2));
                let pattern = format!("%{}%", name);
                param_values.push(Box::new(pattern.clone()));
                param_values.push(Box::new(pattern));
            }

            let sql = format!(
                "SELECT COUNT(*) FROM patients WHERE {}",
                where_clauses.join(" AND ")
            );

            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(count as u64)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn update(&self, patient: &Patient) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let patient = patient.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let doc_type: String = patient.document_number.document_type.to_string();
            let allergies = serde_json::to_string(&patient.allergies).map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            let chronic = serde_json::to_string(&patient.chronic_conditions).map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            let meds = serde_json::to_string(&patient.medications).map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            let (ec_first, ec_last, ec_middle, ec_rel, ec_phone, ec_cc, ec_email) = match &patient.emergency_contact {
                Some(ec) => (
                    Some(ec.name.first_name.clone()),
                    Some(ec.name.last_name.clone()),
                    ec.name.middle_name.clone(),
                    Some(ec.relationship.clone()),
                    Some(ec.phone.number.clone()),
                    Some(ec.phone.country_code.clone()),
                    ec.email.as_ref().map(|e| e.address.clone()),
                ),
                None => (None, None, None, None, None, None, None),
            };

            conn.execute(
                "UPDATE patients SET
                    document_number = ?1, document_type = ?2, country_code = ?3,
                    first_name = ?4, last_name = ?5, middle_name = ?6, date_of_birth = ?7, gender = ?8,
                    email = ?9, phone_number = ?10, phone_country_code = ?11, phone_extension = ?12,
                    address_street = ?13, address_city = ?14, address_state = ?15, address_postal_code = ?16, address_country = ?17, address_additional_info = ?18,
                    emergency_contact_name_first = ?19, emergency_contact_name_last = ?20, emergency_contact_name_middle = ?21,
                    emergency_contact_relationship = ?22, emergency_contact_phone_number = ?23, emergency_contact_phone_country_code = ?24,
                    emergency_contact_email = ?25,
                    blood_type = ?26, allergies = ?27, chronic_conditions = ?28, medications = ?29, notes = ?30,
                    is_active = ?31, updated_at = ?32
                WHERE id = ?33",
                params![
                    patient.document_number.number,
                    doc_type,
                    patient.document_number.country_code,
                    patient.full_name.first_name,
                    patient.full_name.last_name,
                    patient.full_name.middle_name,
                    patient.date_of_birth.format("%Y-%m-%d").to_string(),
                    patient.gender.to_string(),
                    patient.email.as_ref().map(|e| e.address.clone()),
                    patient.phone.as_ref().map(|p| p.number.clone()),
                    patient.phone.as_ref().map(|p| p.country_code.clone()),
                    patient.phone.as_ref().and_then(|p| p.extension.clone()),
                    patient.address.as_ref().map(|a| a.street.clone()),
                    patient.address.as_ref().map(|a| a.city.clone()),
                    patient.address.as_ref().map(|a| a.state.clone()),
                    patient.address.as_ref().map(|a| a.postal_code.clone()),
                    patient.address.as_ref().map(|a| a.country.clone()),
                    patient.address.as_ref().and_then(|a| a.additional_info.clone()),
                    ec_first, ec_last, ec_middle, ec_rel, ec_phone, ec_cc, ec_email,
                    patient.blood_type.as_ref(),
                    allergies, chronic, meds,
                    patient.notes.as_ref(),
                    if patient.is_active { 1i64 } else { 0i64 },
                    patient.updated_at.to_rfc3339(),
                    patient.id.to_string(),
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn delete(&self, id: PatientId) -> Result<bool, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let affected = conn.execute("DELETE FROM patients WHERE id = ?1", params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(affected > 0)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn search(&self, query: &str, pagination: Pagination) -> Result<Vec<Patient>, RepositoryError> {
        let pool = self.pool.clone();
        let pattern = format!("%{}%", query);
        let pat = pattern.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT * FROM patients WHERE (first_name LIKE ?1 OR last_name LIKE ?1 OR document_number LIKE ?1 OR email LIKE ?1) AND is_active = 1 ORDER BY last_name, first_name LIMIT ?2 OFFSET ?3"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let rows = stmt.query_map(params![pat, pagination.limit as i64, pagination.offset as i64], |row| Self::row_to_patient(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut patients = Vec::new();
            for row in rows {
                patients.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(patients)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }
}
