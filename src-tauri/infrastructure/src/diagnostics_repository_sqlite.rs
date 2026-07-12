use crate::database::DbPool;
use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::params;
use serde_json;
use soft_gloria_domain::{
    diagnostics::{DiagnosticoCIE10, DiagnosticoDSM5, CategoriaCIE10, CategoriaDSM5, MapeoDiagnostico},
    RepositoryError,
};
use uuid::Uuid;

pub struct SqliteDiagnosticsRepository {
    pool: DbPool,
}

impl SqliteDiagnosticsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_cie10(row: &rusqlite::Row) -> rusqlite::Result<DiagnosticoCIE10> {
        let codigo: String = row.get("codigo")?;
        let descripcion: String = row.get("descripcion")?;
        let categoria: String = row.get("categoria")?;
        let subcategoria: Option<String> = row.get("subcategoria")?;

        let categoria_enum = match categoria.as_str() {
            "EnfermedadesInfecciosas" => CategoriaCIE10::EnfermedadesInfecciosas,
            "Neoplasias" => CategoriaCIE10::Neoplasias,
            "EnfermedadesSangre" => CategoriaCIE10::EnfermedadesSangre,
            "EndocrinasNutricionalesMetabolicas" => CategoriaCIE10::EndocrinasNutricionalesMetabolicas,
            "TrastornosMentales" => CategoriaCIE10::TrastornosMentales,
            "SistemaNervioso" => CategoriaCIE10::SistemaNervioso,
            "OjoAnexos" => CategoriaCIE10::OjoAnexos,
            "OidoMastoides" => CategoriaCIE10::OidoMastoides,
            "SistemaCirculatorio" => CategoriaCIE10::SistemaCirculatorio,
            "SistemaRespiratorio" => CategoriaCIE10::SistemaRespiratorio,
            "SistemaDigestivo" => CategoriaCIE10::SistemaDigestivo,
            "PielTejiidoSubcutaneo" => CategoriaCIE10::PielTejiidoSubcutaneo,
            "OsteomuscularConectivo" => CategoriaCIE10::OsteomuscularConectivo,
            "Genitourinario" => CategoriaCIE10::Genitourinario,
            "EmbarazoPartoPuerperio" => CategoriaCIE10::EmbarazoPartoPuerperio,
            "Perinatal" => CategoriaCIE10::Perinatal,
            "MalformacionesCongenitas" => CategoriaCIE10::MalformacionesCongenitas,
            "SintomasSignosHallazgos" => CategoriaCIE10::SintomasSignosHallazgos,
            "LesionesEnvenenamiento" => CategoriaCIE10::LesionesEnvenenamiento,
            "CausasExternas" => CategoriaCIE10::CausasExternas,
            "FactoresInfluyenSalud" => CategoriaCIE10::FactoresInfluyenSalud,
            "CodigosEspeciales" => CategoriaCIE10::CodigosEspeciales,
            _ => CategoriaCIE10::CodigosEspeciales,
        };

        Ok(DiagnosticoCIE10 {
            codigo,
            descripcion,
            categoria: categoria_enum,
            subcategoria,
        })
    }

    fn row_to_dsm5(row: &rusqlite::Row) -> rusqlite::Result<DiagnosticoDSM5> {
        let codigo: String = row.get("codigo")?;
        let descripcion: String = row.get("descripcion")?;
        let categoria: String = row.get("categoria")?;
        let criterios_json: Option<String> = row.get("criterios_diagnosticos")?;
        let especificadores_json: Option<String> = row.get("especificadores")?;

        let categoria_enum = match categoria.as_str() {
            "TrastornosNeurodelDesarrollo" => CategoriaDSM5::TrastornosNeurodelDesarrollo,
            "EspectroEsquizofreniaYTrastornosPsicoticos" => CategoriaDSM5::EspectroEsquizofreniaYTrastornosPsicoticos,
            "TrastornosBipolaresYRelacionados" => CategoriaDSM5::TrastornosBipolaresYRelacionados,
            "TrastornosDepresivos" => CategoriaDSM5::TrastornosDepresivos,
            "TrastornosDeAnsiedad" => CategoriaDSM5::TrastornosDeAnsiedad,
            "TrastornosObsesivoCompulsivosYRelacionados" => CategoriaDSM5::TrastornosObsesivoCompulsivosYRelacionados,
            "TrastornosRelacionadosConTraumaYFactoresDeEstres" => CategoriaDSM5::TrastornosRelacionadosConTraumaYFactoresDeEstres,
            "TrastornosDisociativos" => CategoriaDSM5::TrastornosDisociativos,
            "TrastornosSomaticosYRelacionados" => CategoriaDSM5::TrastornosSomaticosYRelacionados,
            "TrastornosDeLaIngestaDeAlimentos" => CategoriaDSM5::TrastornosDeLaIngestaDeAlimentos,
            "TrastornosDeEliminacion" => CategoriaDSM5::TrastornosDeEliminacion,
            "TrastornosDelSueñoYVigilia" => CategoriaDSM5::TrastornosDelSueñoYVigilia,
            "DisfuncionesSexuales" => CategoriaDSM5::DisfuncionesSexuales,
            "DisforiaDeGenero" => CategoriaDSM5::DisforiaDeGenero,
            "TrastornosDisruptivosDelControlDeImpulsosYDeLaConducta" => CategoriaDSM5::TrastornosDisruptivosDelControlDeImpulsosYDeLaConducta,
            "TrastornosRelacionadosConSustanciasYAdictivos" => CategoriaDSM5::TrastornosRelacionadosConSustanciasYAdictivos,
            "TrastornosNeurocognitivos" => CategoriaDSM5::TrastornosNeurocognitivos,
            "TrastornosDeLaPersonalidad" => CategoriaDSM5::TrastornosDeLaPersonalidad,
            "TrastornosParafiliicos" => CategoriaDSM5::TrastornosParafiliicos,
            "OtrosTrastornosMentales" => CategoriaDSM5::OtrosTrastornosMentales,
            "TrastornosRelacionadosConProblemasDeSalud" => CategoriaDSM5::TrastornosRelacionadosConProblemasDeSalud,
            _ => CategoriaDSM5::OtrosTrastornosMentales,
        };

        let criterios_diagnosticos = criterios_json
            .and_then(|s| serde_json::from_str(&s).ok());
        let especificadores = especificadores_json
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(DiagnosticoDSM5 {
            codigo,
            descripcion,
            categoria: categoria_enum,
            criterios_diagnosticos,
            especificadores,
        })
    }

    fn row_to_mapeo(row: &rusqlite::Row) -> rusqlite::Result<MapeoDiagnostico> {
        let id: String = row.get("id")?;
        let paciente_id: String = row.get("paciente_id")?;
        let diagnostico_id: String = row.get("diagnostico_id")?;
        let fuente: String = row.get("fuente")?;
        let notas: Option<String> = row.get("notas")?;
        let fecha: String = row.get("fecha")?;

        Ok(MapeoDiagnostico {
            id: Uuid::parse_str(&id).map_err(|e| rusqlite::Error::InvalidParameterName(format!("UUID parse error: {}", e)))?,
            paciente_id: Uuid::parse_str(&paciente_id).map_err(|e| rusqlite::Error::InvalidParameterName(format!("UUID parse error: {}", e)))?,
            diagnostico_id,
            fuente,
            notas,
            fecha: NaiveDate::parse_from_str(&fecha, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Date parse error: {}", e)))?,
        })
    }
}

#[async_trait::async_trait]
impl soft_gloria_domain::DiagnosticsRepository for SqliteDiagnosticsRepository {
    async fn search_cie10(&self, query: &str, limit: usize) -> Result<Vec<DiagnosticoCIE10>, RepositoryError> {
        let pool = self.pool.clone();
        let pattern = format!("%{}%", query.to_uppercase());
        let limit = limit as i64;

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT codigo, descripcion, categoria, subcategoria FROM cie10 
                 WHERE codigo LIKE ?1 OR descripcion LIKE ?1 OR subcategoria LIKE ?1
                 ORDER BY codigo LIMIT ?2"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            let rows = stmt.query_map(params![pattern, limit], |row| Self::row_to_cie10(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(results)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn search_dsm5(&self, query: &str, limit: usize) -> Result<Vec<DiagnosticoDSM5>, RepositoryError> {
        let pool = self.pool.clone();
        let pattern = format!("%{}%", query.to_uppercase());
        let limit = limit as i64;

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT codigo, descripcion, categoria, criterios_diagnosticos, especificadores FROM dsm5 
                 WHERE codigo LIKE ?1 OR descripcion LIKE ?1
                 ORDER BY codigo LIMIT ?2"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            let rows = stmt.query_map(params![pattern, limit], |row| Self::row_to_dsm5(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(results)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_cie10_by_codigo(&self, codigo: &str) -> Result<Option<DiagnosticoCIE10>, RepositoryError> {
        let pool = self.pool.clone();
        let codigo = codigo.to_uppercase();

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT codigo, descripcion, categoria, subcategoria FROM cie10 WHERE codigo = ?1"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut rows = stmt.query(params![codigo])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let diag = Self::row_to_cie10(row)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(diag))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_dsm5_by_codigo(&self, codigo: &str) -> Result<Option<DiagnosticoDSM5>, RepositoryError> {
        let pool = self.pool.clone();
        let codigo = codigo.to_uppercase();

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT codigo, descripcion, categoria, criterios_diagnosticos, especificadores FROM dsm5 WHERE codigo = ?1"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut rows = stmt.query(params![codigo])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let diag = Self::row_to_dsm5(row)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(diag))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn create_mapeo(&self, mapeo: &MapeoDiagnostico) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let mapeo = mapeo.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;

            conn.execute(
                "INSERT INTO mapeos_diagnosticos (id, paciente_id, diagnostico_id, fuente, notas, fecha, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
                params![
                    mapeo.id.to_string(),
                    mapeo.paciente_id.to_string(),
                    mapeo.diagnostico_id,
                    mapeo.fuente,
                    mapeo.notas,
                    mapeo.fecha.format("%Y-%m-%d").to_string(),
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_mapeos_by_paciente(&self, paciente_id: Uuid) -> Result<Vec<MapeoDiagnostico>, RepositoryError> {
        let pool = self.pool.clone();
        let paciente_id_str = paciente_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT id, paciente_id, diagnostico_id, fuente, notas, fecha FROM mapeos_diagnosticos 
                 WHERE paciente_id = ?1 ORDER BY fecha DESC"
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            let rows = stmt.query_map(params![paciente_id_str], |row| Self::row_to_mapeo(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut results = Vec::new();
            for row in rows {
                results.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(results)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn update_mapeo(&self, mapeo: &MapeoDiagnostico) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let mapeo = mapeo.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;

            let affected = conn.execute(
                "UPDATE mapeos_diagnosticos SET diagnostico_id = ?1, fuente = ?2, notas = ?3, fecha = ?4, updated_at = datetime('now')
                 WHERE id = ?5",
                params![
                    mapeo.diagnostico_id,
                    mapeo.fuente,
                    mapeo.notas,
                    mapeo.fecha.format("%Y-%m-%d").to_string(),
                    mapeo.id.to_string(),
                ],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            if affected == 0 {
                return Err(RepositoryError::NotFound(format!("Mapeo not found: {}", mapeo.id)));
            }
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn delete_mapeo(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let affected = conn.execute("DELETE FROM mapeos_diagnosticos WHERE id = ?1", params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(affected > 0)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_memory_pool;
    use soft_gloria_domain::diagnostics::MapeoDiagnostico;
    use soft_gloria_domain::repositories::DiagnosticsRepository;

    fn create_test_repo() -> SqliteDiagnosticsRepository {
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
            CREATE INDEX IF NOT EXISTS idx_cie10_codigo ON cie10(codigo);
            CREATE INDEX IF NOT EXISTS idx_cie10_descripcion ON cie10(descripcion);

            CREATE TABLE IF NOT EXISTS dsm5 (
                codigo TEXT PRIMARY KEY NOT NULL,
                descripcion TEXT NOT NULL,
                categoria TEXT NOT NULL,
                criterios_diagnosticos TEXT,
                especificadores TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_dsm5_codigo ON dsm5(codigo);
            CREATE INDEX IF NOT EXISTS idx_dsm5_descripcion ON dsm5(descripcion);

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
            CREATE INDEX IF NOT EXISTS idx_mapeos_fecha ON mapeos_diagnosticos(fecha);
            "#,
        ).unwrap();
        drop(conn);
        SqliteDiagnosticsRepository::new(pool)
    }

    #[tokio::test]
    async fn test_search_cie10() {
        let repo = create_test_repo();
        let conn = repo.pool.lock().unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            params!["F32.0", "Episodio depresivo leve", "TrastornosMentales", "Episodio depresivo leve"],
        ).unwrap();
        conn.execute(
            "INSERT INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            params!["F32.1", "Episodio depresivo moderado", "TrastornosMentales", "Episodio depresivo moderado"],
        ).unwrap();
        conn.execute(
            "INSERT INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            params!["I10", "Hipertensión esencial", "SistemaCirculatorio", "Hipertensión esencial"],
        ).unwrap();
        drop(conn);

        // Search by code prefix
        let results = repo.search_cie10("F32", 10).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|d| d.codigo == "F32.0"));
        assert!(results.iter().any(|d| d.codigo == "F32.1"));

        // Search by description
        let results = repo.search_cie10("depresivo", 10).await.unwrap();
        assert_eq!(results.len(), 2);

        // Search with limit
        let results = repo.search_cie10("F", 1).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_search_dsm5() {
        let repo = create_test_repo();
        let conn = repo.pool.lock().unwrap();

        conn.execute(
            "INSERT INTO dsm5 (codigo, descripcion, categoria, criterios_diagnosticos, especificadores) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["296.21", "Major depressive disorder, single episode, mild", "TrastornosDepresivos", "[\"c1\", \"c2\"]", "[\"Mild\"]"],
        ).unwrap();
        conn.execute(
            "INSERT INTO dsm5 (codigo, descripcion, categoria, criterios_diagnosticos, especificadores) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["296.22", "Major depressive disorder, single episode, moderate", "TrastornosDepresivos", "[\"c1\", \"c2\"]", "[\"Moderate\"]"],
        ).unwrap();
        conn.execute(
            "INSERT INTO dsm5 (codigo, descripcion, categoria, criterios_diagnosticos, especificadores) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["300.02", "Generalized anxiety disorder", "TrastornosDeAnsiedad", "[\"c1\"]", "[]"],
        ).unwrap();
        drop(conn);

        let results = repo.search_dsm5("296", 10).await.unwrap();
        assert_eq!(results.len(), 2);

        let results = repo.search_dsm5("depressive", 10).await.unwrap();
        assert_eq!(results.len(), 2);

        let results = repo.search_dsm5("296", 1).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_get_cie10_by_codigo() {
        let repo = create_test_repo();
        let conn = repo.pool.lock().unwrap();
        conn.execute(
            "INSERT INTO cie10 (codigo, descripcion, categoria, subcategoria) VALUES (?1, ?2, ?3, ?4)",
            params!["F41.1", "Trastorno de ansiedad generalizada", "TrastornosMentales", "Trastorno de ansiedad generalizada"],
        ).unwrap();
        drop(conn);

        let result = repo.get_cie10_by_codigo("F41.1").await.unwrap();
        assert!(result.is_some());
        let diag = result.unwrap();
        assert_eq!(diag.codigo, "F41.1");
        assert_eq!(diag.descripcion, "Trastorno de ansiedad generalizada");
        assert_eq!(diag.categoria, CategoriaCIE10::TrastornosMentales);

        let result = repo.get_cie10_by_codigo("F99.9").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_dsm5_by_codigo() {
        let repo = create_test_repo();
        let conn = repo.pool.lock().unwrap();
        conn.execute(
            "INSERT INTO dsm5 (codigo, descripcion, categoria, criterios_diagnosticos, especificadores) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["300.02", "Generalized anxiety disorder", "TrastornosDeAnsiedad", "[\"c1\"]", "[]"],
        ).unwrap();
        drop(conn);

        let result = repo.get_dsm5_by_codigo("300.02").await.unwrap();
        assert!(result.is_some());
        let diag = result.unwrap();
        assert_eq!(diag.codigo, "300.02");
        assert_eq!(diag.categoria, CategoriaDSM5::TrastornosDeAnsiedad);

        let result = repo.get_dsm5_by_codigo("999.99").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mapeo_crud() {
        let repo = create_test_repo();
        let paciente_id = Uuid::new_v4();

        let mapeo = MapeoDiagnostico::new(
            paciente_id,
            "F32.0".to_string(),
            "CIE-10".to_string(),
            Some("Paciente en seguimiento".to_string()),
        );

        // Create
        repo.create_mapeo(&mapeo).await.unwrap();

        // Get by paciente
        let mapeos = repo.get_mapeos_by_paciente(paciente_id).await.unwrap();
        assert_eq!(mapeos.len(), 1);
        assert_eq!(mapeos[0].diagnostico_id, "F32.0");
        assert_eq!(mapeos[0].fuente, "CIE-10");
        assert_eq!(mapeos[0].notas, Some("Paciente en seguimiento".to_string()));

        // Update
        let mut updated = mapeos[0].clone();
        updated.notas = Some("Notas actualizadas".to_string());
        repo.update_mapeo(&updated).await.unwrap();

        let mapeos = repo.get_mapeos_by_paciente(paciente_id).await.unwrap();
        assert_eq!(mapeos[0].notas, Some("Notas actualizadas".to_string()));

        // Delete
        let deleted = repo.delete_mapeo(mapeo.id).await.unwrap();
        assert!(deleted);

        let mapeos = repo.get_mapeos_by_paciente(paciente_id).await.unwrap();
        assert_eq!(mapeos.len(), 0);
    }
}