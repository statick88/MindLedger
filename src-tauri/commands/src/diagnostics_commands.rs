use crate::error::{AppError, AppResult};
use soft_gloria_domain::{
    diagnostics::{DiagnosticoCIE10, DiagnosticoDSM5, MapeoDiagnostico},
    repositories::DiagnosticsRepository,
};
use soft_gloria_infrastructure::SqliteDiagnosticsRepository;
use tauri::command;
use uuid::Uuid;
use chrono::NaiveDate;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct CreateMapeoRequest {
    pub paciente_id: String,
    pub diagnostico_id: String,
    pub fuente: String,
    pub notas: Option<String>,
    pub fecha: Option<String>,
}

#[derive(serde::Serialize, Debug)]
pub struct DiagnosticoCIE10Response {
    pub codigo: String,
    pub descripcion: String,
    pub categoria: String,
    pub subcategoria: Option<String>,
}

impl From<DiagnosticoCIE10> for DiagnosticoCIE10Response {
    fn from(d: DiagnosticoCIE10) -> Self {
        Self {
            codigo: d.codigo,
            descripcion: d.descripcion,
            categoria: d.categoria.to_string(),
            subcategoria: d.subcategoria,
        }
    }
}

#[derive(serde::Serialize, Debug)]
pub struct DiagnosticoDSM5Response {
    pub codigo: String,
    pub descripcion: String,
    pub categoria: String,
    pub criterios_diagnosticos: Option<Vec<String>>,
    pub especificadores: Option<Vec<String>>,
}

impl From<DiagnosticoDSM5> for DiagnosticoDSM5Response {
    fn from(d: DiagnosticoDSM5) -> Self {
        Self {
            codigo: d.codigo,
            descripcion: d.descripcion,
            categoria: d.categoria.to_string(),
            criterios_diagnosticos: d.criterios_diagnosticos,
            especificadores: d.especificadores,
        }
    }
}

#[derive(serde::Serialize, Debug)]
pub struct MapeoDiagnosticoResponse {
    pub id: String,
    pub paciente_id: String,
    pub diagnostico_id: String,
    pub fuente: String,
    pub notas: Option<String>,
    pub fecha: String,
}

impl From<MapeoDiagnostico> for MapeoDiagnosticoResponse {
    fn from(m: MapeoDiagnostico) -> Self {
        Self {
            id: m.id.to_string(),
            paciente_id: m.paciente_id.to_string(),
            diagnostico_id: m.diagnostico_id,
            fuente: m.fuente,
            notas: m.notas,
            fecha: m.fecha.format("%Y-%m-%d").to_string(),
        }
    }
}

// ── Inner functions (testable without Tauri State) ──

pub async fn search_cie10_impl(pool: &soft_gloria_infrastructure::DbPool, query: &str, limit: usize) -> AppResult<Vec<DiagnosticoCIE10Response>> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());
    let limit = limit.min(100);

    let results = repo.search_cie10(query, limit).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(results.into_iter().map(Into::into).collect())
}

pub async fn search_dsm5_impl(pool: &soft_gloria_infrastructure::DbPool, query: &str, limit: usize) -> AppResult<Vec<DiagnosticoDSM5Response>> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());
    let limit = limit.min(100);

    let results = repo.search_dsm5(query, limit).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(results.into_iter().map(Into::into).collect())
}

pub async fn get_cie10_by_codigo_impl(pool: &soft_gloria_infrastructure::DbPool, codigo: &str) -> AppResult<Option<DiagnosticoCIE10Response>> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());

    let result = repo.get_cie10_by_codigo(codigo).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(result.map(Into::into))
}

pub async fn get_dsm5_by_codigo_impl(pool: &soft_gloria_infrastructure::DbPool, codigo: &str) -> AppResult<Option<DiagnosticoDSM5Response>> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());

    let result = repo.get_dsm5_by_codigo(codigo).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(result.map(Into::into))
}

pub async fn create_mapeo_impl(pool: &soft_gloria_infrastructure::DbPool, request: CreateMapeoRequest) -> AppResult<MapeoDiagnosticoResponse> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());

    let paciente_id = Uuid::parse_str(&request.paciente_id)
        .map_err(|e| AppError::Validation(format!("Invalid patient UUID: {}", e)))?;

    let _fecha = match request.fecha {
        Some(f) => NaiveDate::parse_from_str(&f, "%Y-%m-%d")
            .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?,
        None => chrono::Utc::now().date_naive(),
    };

    let mapeo = MapeoDiagnostico::new(
        paciente_id,
        request.diagnostico_id,
        request.fuente,
        request.notas,
    );

    repo.create_mapeo(&mapeo).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(mapeo.into())
}

pub async fn list_mapeos_impl(pool: &soft_gloria_infrastructure::DbPool, paciente_id: &str) -> AppResult<Vec<MapeoDiagnosticoResponse>> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());

    let paciente_id = Uuid::parse_str(paciente_id)
        .map_err(|e| AppError::Validation(format!("Invalid patient UUID: {}", e)))?;

    let mapeos = repo.get_mapeos_by_paciente(paciente_id).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(mapeos.into_iter().map(Into::into).collect())
}

pub async fn update_mapeo_impl(
    pool: &soft_gloria_infrastructure::DbPool,
    id: &str,
    paciente_id: &str,
    diagnostico_id: Option<String>,
    fuente: Option<String>,
    notas: Option<String>,
    fecha: Option<String>,
) -> AppResult<MapeoDiagnosticoResponse> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());

    let mapeo_id = Uuid::parse_str(id)
        .map_err(|e| AppError::Validation(format!("Invalid UUID: {}", e)))?;

    let paciente_uuid = Uuid::parse_str(paciente_id)
        .map_err(|e| AppError::Validation(format!("Invalid patient UUID: {}", e)))?;

    let existing = repo.get_mapeos_by_paciente(paciente_uuid).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?
        .into_iter()
        .find(|m| m.id == mapeo_id)
        .ok_or_else(|| AppError::NotFound(format!("Mapeo with id {} not found", id)))?;

    let mut mapeo = existing;
    if let Some(d) = diagnostico_id {
        mapeo.diagnostico_id = d;
    }
    if let Some(f) = fuente {
        mapeo.fuente = f;
    }
    if notas.is_some() {
        mapeo.notas = notas;
    }
    if let Some(f) = fecha {
        mapeo.fecha = NaiveDate::parse_from_str(&f, "%Y-%m-%d")
            .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;
    }

    repo.update_mapeo(&mapeo).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(mapeo.into())
}

pub async fn delete_mapeo_impl(pool: &soft_gloria_infrastructure::DbPool, id: &str) -> AppResult<bool> {
    let repo = SqliteDiagnosticsRepository::new(pool.clone());

    let mapeo_id = Uuid::parse_str(id)
        .map_err(|e| AppError::Validation(format!("Invalid UUID: {}", e)))?;

    let deleted = repo.delete_mapeo(mapeo_id).await
        .map_err(|e| AppError::Diagnostics(e.to_string()))?;

    Ok(deleted)
}

// ── Tauri command wrappers ──

#[command]
pub async fn search_cie10(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    query: String,
    limit: Option<usize>,
) -> AppResult<Vec<DiagnosticoCIE10Response>> {
    search_cie10_impl(&db, &query, limit.unwrap_or(50)).await
}

#[command]
pub async fn search_dsm5(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    query: String,
    limit: Option<usize>,
) -> AppResult<Vec<DiagnosticoDSM5Response>> {
    search_dsm5_impl(&db, &query, limit.unwrap_or(50)).await
}

#[command]
pub async fn get_cie10_by_codigo(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    codigo: String,
) -> AppResult<Option<DiagnosticoCIE10Response>> {
    get_cie10_by_codigo_impl(&db, &codigo).await
}

#[command]
pub async fn get_dsm5_by_codigo(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    codigo: String,
) -> AppResult<Option<DiagnosticoDSM5Response>> {
    get_dsm5_by_codigo_impl(&db, &codigo).await
}

#[command]
pub async fn create_mapeo(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    request: CreateMapeoRequest,
) -> AppResult<MapeoDiagnosticoResponse> {
    create_mapeo_impl(&db, request).await
}

#[command]
pub async fn list_mapeos(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    paciente_id: String,
) -> AppResult<Vec<MapeoDiagnosticoResponse>> {
    list_mapeos_impl(&db, &paciente_id).await
}

#[command]
pub async fn update_mapeo(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    id: String,
    paciente_id: String,
    diagnostico_id: Option<String>,
    fuente: Option<String>,
    notas: Option<String>,
    fecha: Option<String>,
) -> AppResult<MapeoDiagnosticoResponse> {
    update_mapeo_impl(&db, &id, &paciente_id, diagnostico_id, fuente, notas, fecha).await
}

#[command]
pub async fn delete_mapeo(
    db: tauri::State<'_, Arc<soft_gloria_infrastructure::DbPool>>,
    id: String,
) -> AppResult<bool> {
    delete_mapeo_impl(&db, &id).await
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use soft_gloria_infrastructure::create_memory_pool;
    use soft_gloria_domain::diagnostics::MapeoDiagnostico;
    use soft_gloria_domain::repositories::DiagnosticsRepository;
    use uuid::Uuid;

    fn create_test_pool() -> soft_gloria_infrastructure::DbPool {
        let pool = create_memory_pool().unwrap();
        let conn = pool.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS cie10 (
                codigo TEXT PRIMARY KEY NOT NULL,
                descripcion TEXT NOT NULL,
                categoria TEXT NOT NULL,
                subcategoria TEXT
            );
            CREATE TABLE IF NOT EXISTS dsm5 (
                codigo TEXT PRIMARY KEY NOT NULL,
                descripcion TEXT NOT NULL,
                categoria TEXT NOT NULL,
                criterios_diagnosticos TEXT,
                especificadores TEXT
            );
            CREATE TABLE IF NOT EXISTS mapeos_diagnosticos (
                id TEXT PRIMARY KEY NOT NULL,
                paciente_id TEXT NOT NULL,
                diagnostico_id TEXT NOT NULL,
                fuente TEXT NOT NULL,
                notas TEXT,
                fecha TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_mapeos_paciente ON mapeos_diagnosticos(paciente_id);
            "#,
        ).unwrap();
        drop(conn);
        pool
    }

    fn seed_test_data(pool: &soft_gloria_infrastructure::DbPool) {
        let conn = pool.lock().unwrap();

        // Seed CIE-10
        conn.execute(
            "INSERT OR IGNORE INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            ["F32.0", "Episodio depresivo leve", "TrastornosMentales", "Episodio depresivo leve"],
        ).unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            ["F41.1", "Trastorno de ansiedad generalizada", "TrastornosMentales", "Trastorno de ansiedad generalizada"],
        ).unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            ["I10", "Hipertensión esencial (primaria)", "SistemaCirculatorio", "Enfermedades hipertensivas"],
        ).unwrap();

        // Seed DSM-5
        conn.execute(
            "INSERT OR IGNORE INTO dsm5 (codigo, descripcion, categoria, criterios_diagnosticos, especificadores) VALUES (?1, ?2, ?3, ?4, ?5)",
            ["296.21", "Major depressive disorder, single episode, mild", "TrastornosDepresivos", "[\"Depressed mood\", \"Anhedonia\"]", "[\"Mild\"]"],
        ).unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO dsm5 (codigo, descripcion, categoria, criterios_diagnosticos, especificadores) VALUES (?1, ?2, ?3, ?4, ?5)",
            ["300.02", "Generalized anxiety disorder", "TrastornosDeAnsiedad", "[\"Excessive anxiety\", \"Difficulty controlling worry\"]", "[]"],
        ).unwrap();

        drop(conn);
    }

    // ── Repository layer tests ──

    #[tokio::test]
    async fn test_search_cie10_repo() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let repo = SqliteDiagnosticsRepository::new(pool);

        let results = repo.search_cie10("depres", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].codigo, "F32.0");

        let results = repo.search_cie10("ansied", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].codigo, "F41.1");
    }

    #[tokio::test]
    async fn test_search_dsm5_repo() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let repo = SqliteDiagnosticsRepository::new(pool);

        let results = repo.search_dsm5("depress", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].codigo, "296.21");
    }

    #[tokio::test]
    async fn test_get_cie10_by_codigo_repo() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let repo = SqliteDiagnosticsRepository::new(pool);

        let result = repo.get_cie10_by_codigo("F32.0").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().codigo, "F32.0");

        let result = repo.get_cie10_by_codigo("F99.9").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_mapeo_repo() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let repo = SqliteDiagnosticsRepository::new(pool);
        let paciente_id = Uuid::new_v4();

        let mapeo = MapeoDiagnostico::new(
            paciente_id,
            "F32.0".to_string(),
            "CIE-10".to_string(),
            Some("Diagnóstico principal".to_string()),
        );

        repo.create_mapeo(&mapeo).await.unwrap();

        let mapeos = repo.get_mapeos_by_paciente(paciente_id).await.unwrap();
        assert_eq!(mapeos.len(), 1);
        assert_eq!(mapeos[0].diagnostico_id, "F32.0");
        assert_eq!(mapeos[0].fuente, "CIE-10");
        assert_eq!(mapeos[0].notas, Some("Diagnóstico principal".to_string()));
    }

    #[tokio::test]
    async fn test_delete_mapeo_repo() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let repo = SqliteDiagnosticsRepository::new(pool);
        let paciente_id = Uuid::new_v4();

        let mapeo = MapeoDiagnostico::new(
            paciente_id,
            "F32.0".to_string(),
            "CIE-10".to_string(),
            None,
        );
        repo.create_mapeo(&mapeo).await.unwrap();

        let deleted = repo.delete_mapeo(mapeo.id).await.unwrap();
        assert!(deleted);

        let mapeos = repo.get_mapeos_by_paciente(paciente_id).await.unwrap();
        assert_eq!(mapeos.len(), 0);

        let deleted = repo.delete_mapeo(mapeo.id).await.unwrap();
        assert!(!deleted);
    }

    // ── _impl function tests ──

    #[tokio::test]
    async fn test_search_cie10_impl() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let results = search_cie10_impl(&pool, "depres", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].codigo, "F32.0");
        assert!(!results[0].categoria.is_empty());
    }

    #[tokio::test]
    async fn test_search_dsm5_impl() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let results = search_dsm5_impl(&pool, "anxiety", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].codigo, "300.02");
    }

    #[tokio::test]
    async fn test_get_cie10_by_codigo_impl_found() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let result = get_cie10_by_codigo_impl(&pool, "F32.0").await.unwrap();
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.codigo, "F32.0");
        assert_eq!(d.subcategoria, Some("Episodio depresivo leve".to_string()));
    }

    #[tokio::test]
    async fn test_get_cie10_by_codigo_impl_not_found() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let result = get_cie10_by_codigo_impl(&pool, "Z99.9").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_dsm5_by_codigo_impl_found() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let result = get_dsm5_by_codigo_impl(&pool, "296.21").await.unwrap();
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.codigo, "296.21");
        assert!(d.criterios_diagnosticos.is_some());
    }

    #[tokio::test]
    async fn test_create_mapeo_impl() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let paciente_id = Uuid::new_v4();

        let request = CreateMapeoRequest {
            paciente_id: paciente_id.to_string(),
            diagnostico_id: "F32.0".to_string(),
            fuente: "CIE-10".to_string(),
            notas: Some("Test mapeo".to_string()),
            fecha: None,
        };

        let response = create_mapeo_impl(&pool, request).await.unwrap();
        assert_eq!(response.paciente_id, paciente_id.to_string());
        assert_eq!(response.diagnostico_id, "F32.0");
        assert_eq!(response.fuente, "CIE-10");
        assert_eq!(response.notas, Some("Test mapeo".to_string()));
    }

    #[tokio::test]
    async fn test_create_mapeo_impl_invalid_uuid() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let request = CreateMapeoRequest {
            paciente_id: "not-a-uuid".to_string(),
            diagnostico_id: "F32.0".to_string(),
            fuente: "CIE-10".to_string(),
            notas: None,
            fecha: None,
        };

        let result = create_mapeo_impl(&pool, request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_list_mapeos_impl() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let paciente_id = Uuid::new_v4();

        for i in 0..3 {
            let request = CreateMapeoRequest {
                paciente_id: paciente_id.to_string(),
                diagnostico_id: format!("F32.{}", i),
                fuente: "CIE-10".to_string(),
                notas: Some(format!("Nota {}", i)),
                fecha: None,
            };
            create_mapeo_impl(&pool, request).await.unwrap();
        }

        let mapeos = list_mapeos_impl(&pool, &paciente_id.to_string()).await.unwrap();
        assert_eq!(mapeos.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_mapeo_impl() {
        let pool = create_test_pool();
        seed_test_data(&pool);
        let paciente_id = Uuid::new_v4();

        let request = CreateMapeoRequest {
            paciente_id: paciente_id.to_string(),
            diagnostico_id: "F32.0".to_string(),
            fuente: "CIE-10".to_string(),
            notas: None,
            fecha: None,
        };
        let created = create_mapeo_impl(&pool, request).await.unwrap();

        let deleted = delete_mapeo_impl(&pool, &created.id).await.unwrap();
        assert!(deleted);

        let mapeos = list_mapeos_impl(&pool, &paciente_id.to_string()).await.unwrap();
        assert_eq!(mapeos.len(), 0);

        let deleted = delete_mapeo_impl(&pool, &created.id).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_delete_mapeo_impl_invalid_uuid() {
        let pool = create_test_pool();
        seed_test_data(&pool);

        let result = delete_mapeo_impl(&pool, "not-a-uuid").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }
}
