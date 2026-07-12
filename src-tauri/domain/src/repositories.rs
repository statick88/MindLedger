use crate::{
    Patient, PatientId, AsientoContable, BalanceGeneral, EstadoResultados,
    DiagnosticoCIE10, DiagnosticoDSM5, MapeoDiagnostico,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[async_trait]
pub trait PatientRepository: Send + Sync {
    async fn create(&self, patient: &Patient) -> Result<(), RepositoryError>;
    async fn get_by_id(&self, id: PatientId) -> Result<Option<Patient>, RepositoryError>;
    async fn get_by_document(&self, document_number: &str) -> Result<Option<Patient>, RepositoryError>;
    async fn list(&self, filter: PatientFilter, pagination: Pagination) -> Result<Vec<Patient>, RepositoryError>;
    async fn count(&self, filter: PatientFilter) -> Result<u64, RepositoryError>;
    async fn update(&self, patient: &Patient) -> Result<(), RepositoryError>;
    async fn delete(&self, id: PatientId) -> Result<bool, RepositoryError>;
    async fn search(&self, query: &str, pagination: Pagination) -> Result<Vec<Patient>, RepositoryError>;
}

#[async_trait]
pub trait AccountingRepository: Send + Sync {
    async fn create_asiento(&self, asiento: &AsientoContable) -> Result<(), RepositoryError>;
    async fn get_asiento_by_id(&self, id: Uuid) -> Result<Option<AsientoContable>, RepositoryError>;
    async fn list_asientos(&self, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>, pagination: Pagination) -> Result<Vec<AsientoContable>, RepositoryError>;
    async fn count_asientos(&self, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>) -> Result<u64, RepositoryError>;
    async fn update_asiento(&self, asiento: &AsientoContable) -> Result<(), RepositoryError>;
    async fn delete_asiento(&self, id: Uuid) -> Result<bool, RepositoryError>;
    async fn get_balance_general(&self, fecha: NaiveDate) -> Result<BalanceGeneral, RepositoryError>;
    async fn get_estado_resultados(&self, fecha_desde: NaiveDate, fecha_hasta: NaiveDate) -> Result<EstadoResultados, RepositoryError>;
}

#[async_trait]
pub trait DiagnosticsRepository: Send + Sync {
    async fn search_cie10(&self, query: &str, limit: usize) -> Result<Vec<DiagnosticoCIE10>, RepositoryError>;
    async fn search_dsm5(&self, query: &str, limit: usize) -> Result<Vec<DiagnosticoDSM5>, RepositoryError>;
    async fn get_cie10_by_codigo(&self, codigo: &str) -> Result<Option<DiagnosticoCIE10>, RepositoryError>;
    async fn get_dsm5_by_codigo(&self, codigo: &str) -> Result<Option<DiagnosticoDSM5>, RepositoryError>;
    async fn create_mapeo(&self, mapeo: &MapeoDiagnostico) -> Result<(), RepositoryError>;
    async fn get_mapeos_by_paciente(&self, paciente_id: Uuid) -> Result<Vec<MapeoDiagnostico>, RepositoryError>;
    async fn update_mapeo(&self, mapeo: &MapeoDiagnostico) -> Result<(), RepositoryError>;
    async fn delete_mapeo(&self, id: Uuid) -> Result<bool, RepositoryError>;
}

#[derive(Debug, Clone, Default)]
pub struct PatientFilter {
    pub active_only: Option<bool>,
    pub gender: Option<crate::Gender>,
    pub name_contains: Option<String>,
    pub min_age: Option<u32>,
    pub max_age: Option<u32>,
    pub has_allergy: Option<String>,
    pub has_condition: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pagination {
    pub offset: u64,
    pub limit: u64,
}

impl Pagination {
    pub fn new(page: u64, page_size: u64) -> Self {
        Self {
            offset: page * page_size,
            limit: page_size,
        }
    }

    pub fn first(page_size: u64) -> Self {
        Self {
            offset: 0,
            limit: page_size,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Constraint violation: {0}")]
    Constraint(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Migration error: {0}")]
    Migration(String),
}
