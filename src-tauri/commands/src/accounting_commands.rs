use crate::error::{AppError, AppResult};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use soft_gloria_domain::{
    accounting::{AsientoContable, BalanceGeneral, EstadoResultados, LineaAsiento},
    repositories::{AccountingRepository, Pagination},
};
use soft_gloria_infrastructure::{DbPool, SqliteAccountingRepository};
use std::sync::Arc;
use tauri::command;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateAsientoRequest {
    pub fecha: String,
    pub descripcion: String,
    pub lineas: Vec<CreateLineaAsientoRequest>,
}

#[derive(Deserialize)]
pub struct CreateLineaAsientoRequest {
    pub cuenta: String,
    pub debito: Option<String>,
    pub credito: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAsientoRequest {
    pub fecha: Option<String>,
    pub descripcion: Option<String>,
    pub lineas: Option<Vec<CreateLineaAsientoRequest>>,
}

#[derive(Serialize, Debug)]
pub struct AsientoResponse {
    pub id: String,
    pub fecha: String,
    pub descripcion: String,
    pub lineas: Vec<LineaAsientoResponse>,
    pub total_debitos: String,
    pub total_creditos: String,
    pub is_balanced: bool,
}

#[derive(Serialize, Debug)]
pub struct LineaAsientoResponse {
    pub cuenta: String,
    pub debito: String,
    pub credito: String,
}

impl From<AsientoContable> for AsientoResponse {
    fn from(a: AsientoContable) -> Self {
        let total_debitos = a.total_debitos().to_string();
        let total_creditos = a.total_creditos().to_string();
        let is_balanced = a.is_balanced();
        Self {
            id: a.id.to_string(),
            fecha: a.fecha.format("%Y-%m-%d").to_string(),
            descripcion: a.descripcion,
            lineas: a.lineas.clone().into_iter().map(Into::into).collect(),
            total_debitos,
            total_creditos,
            is_balanced,
        }
    }
}

impl From<LineaAsiento> for LineaAsientoResponse {
    fn from(l: LineaAsiento) -> Self {
        Self {
            cuenta: l.cuenta,
            debito: l.debito.to_string(),
            credito: l.credito.to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BalanceGeneralResponse {
    pub fecha: String,
    pub activos: Vec<(String, String)>,
    pub pasivos: Vec<(String, String)>,
    pub patrimonio: Vec<(String, String)>,
    pub total_activos: String,
    pub total_pasivos: String,
    pub total_patrimonio: String,
    pub is_balanced: bool,
}

impl From<BalanceGeneral> for BalanceGeneralResponse {
    fn from(b: BalanceGeneral) -> Self {
        Self {
            fecha: b.fecha.format("%Y-%m-%d").to_string(),
            activos: b.activos.clone().into_iter().map(|(c, v)| (c, v.to_string())).collect(),
            pasivos: b.pasivos.clone().into_iter().map(|(c, v)| (c, v.to_string())).collect(),
            patrimonio: b.patrimonio.clone().into_iter().map(|(c, v)| (c, v.to_string())).collect(),
            total_activos: b.total_activos().to_string(),
            total_pasivos: b.total_pasivos().to_string(),
            total_patrimonio: b.total_patrimonio().to_string(),
            is_balanced: b.is_balanced(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct EstadoResultadosResponse {
    pub fecha: String,
    pub ingresos: Vec<(String, String)>,
    pub gastos: Vec<(String, String)>,
    pub total_ingresos: String,
    pub total_gastos: String,
    pub utilidad_neta: String,
}

impl From<EstadoResultados> for EstadoResultadosResponse {
    fn from(e: EstadoResultados) -> Self {
        Self {
            fecha: e.fecha.format("%Y-%m-%d").to_string(),
            ingresos: e.ingresos.clone().into_iter().map(|(c, v)| (c, v.to_string())).collect(),
            gastos: e.gastos.clone().into_iter().map(|(c, v)| (c, v.to_string())).collect(),
            total_ingresos: e.total_ingresos().to_string(),
            total_gastos: e.total_gastos().to_string(),
            utilidad_neta: e.utilidad_neta.to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct ListAsientosQuery {
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct PaginatedAsientosResponse {
    pub items: Vec<AsientoResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

fn parse_date(date_str: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid date format (expected YYYY-MM-DD): {}", e)))
}

fn parse_decimal(value: &str) -> AppResult<Decimal> {
    value.parse::<Decimal>()
        .map_err(|e| AppError::Validation(format!("Invalid decimal value '{}': {}", value, e)))
}

fn validate_linea_request(linea: &CreateLineaAsientoRequest) -> AppResult<()> {
    let has_debito = linea.debito.is_some();
    let has_credito = linea.credito.is_some();

    if !has_debito && !has_credito {
        return Err(AppError::Validation("Line must have either debito or credito".to_string()));
    }
    if has_debito && has_credito {
        return Err(AppError::Validation("Line cannot have both debito and credito".to_string()));
    }
    if linea.cuenta.trim().is_empty() {
        return Err(AppError::Validation("Account name cannot be empty".to_string()));
    }
    Ok(())
}

// ── Inner functions (testable without Tauri State) ──

pub async fn add_asiento_impl(pool: &DbPool, request: CreateAsientoRequest) -> AppResult<AsientoResponse> {
    let repo = SqliteAccountingRepository::new(pool.clone());

    let fecha = parse_date(&request.fecha)?;

    if request.descripcion.trim().is_empty() {
        return Err(AppError::Validation("Description cannot be empty".to_string()));
    }

    if request.lineas.is_empty() {
        return Err(AppError::Validation("Asiento must have at least one line".to_string()));
    }

    for linea in &request.lineas {
        validate_linea_request(linea)?;
    }

    let mut lineas = Vec::new();
    for linea in request.lineas {
        if let Some(debito) = linea.debito {
            let monto = parse_decimal(&debito)?;
            if monto <= Decimal::ZERO {
                return Err(AppError::Validation("Debit amount must be positive".to_string()));
            }
            lineas.push(LineaAsiento::new_debito(linea.cuenta, monto)
                .map_err(|e| AppError::Accounting(e.to_string()))?);
        } else if let Some(credito) = linea.credito {
            let monto = parse_decimal(&credito)?;
            if monto <= Decimal::ZERO {
                return Err(AppError::Validation("Credit amount must be positive".to_string()));
            }
            lineas.push(LineaAsiento::new_credito(linea.cuenta, monto)
                .map_err(|e| AppError::Accounting(e.to_string()))?);
        }
    }

    let asiento = AsientoContable::new(fecha, request.descripcion, lineas)
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    repo.create_asiento(&asiento).await
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    Ok(asiento.into())
}

pub async fn remove_asiento_impl(pool: &DbPool, id: String) -> AppResult<bool> {
    let repo = SqliteAccountingRepository::new(pool.clone());
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| AppError::Validation(format!("Invalid UUID: {}", e)))?;

    let deleted = repo.delete_asiento(uuid).await
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    Ok(deleted)
}

pub async fn list_asientos_impl(pool: &DbPool, query: ListAsientosQuery) -> AppResult<PaginatedAsientosResponse> {
    let repo = SqliteAccountingRepository::new(pool.clone());

    let fecha_desde = query.fecha_desde.as_deref().map(parse_date).transpose()?;
    let fecha_hasta = query.fecha_hasta.as_deref().map(parse_date).transpose()?;

    let page = query.page.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let pagination = Pagination::new(page, page_size);

    let asientos = repo.list_asientos(fecha_desde, fecha_hasta, pagination).await
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    let total = repo.count_asientos(fecha_desde, fecha_hasta).await
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    Ok(PaginatedAsientosResponse {
        items: asientos.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
        total_pages: (total + page_size - 1) / page_size,
    })
}

pub async fn generate_balance_general_impl(pool: &DbPool, fecha: String) -> AppResult<BalanceGeneralResponse> {
    let repo = SqliteAccountingRepository::new(pool.clone());
    let fecha = parse_date(&fecha)?;

    let balance = repo.get_balance_general(fecha).await
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    Ok(balance.into())
}

pub async fn generate_estado_resultados_impl(pool: &DbPool, fecha_desde: String, fecha_hasta: String) -> AppResult<EstadoResultadosResponse> {
    let repo = SqliteAccountingRepository::new(pool.clone());
    let desde = parse_date(&fecha_desde)?;
    let hasta = parse_date(&fecha_hasta)?;

    if hasta < desde {
        return Err(AppError::Validation("fecha_hasta must be >= fecha_desde".to_string()));
    }

    let estado = repo.get_estado_resultados(desde, hasta).await
        .map_err(|e| AppError::Accounting(e.to_string()))?;

    Ok(estado.into())
}

// ── Tauri command wrappers ──

#[command]
pub async fn add_asiento(
    db: tauri::State<'_, Arc<DbPool>>,
    request: CreateAsientoRequest,
) -> AppResult<AsientoResponse> {
    add_asiento_impl(&db, request).await
}

#[command]
pub async fn remove_asiento(
    db: tauri::State<'_, Arc<DbPool>>,
    id: String,
) -> AppResult<bool> {
    remove_asiento_impl(&db, id).await
}

#[command]
pub async fn list_asientos(
    db: tauri::State<'_, Arc<DbPool>>,
    query: ListAsientosQuery,
) -> AppResult<PaginatedAsientosResponse> {
    list_asientos_impl(&db, query).await
}

#[command]
pub async fn generate_balance_general(
    db: tauri::State<'_, Arc<DbPool>>,
    fecha: String,
) -> AppResult<BalanceGeneralResponse> {
    generate_balance_general_impl(&db, fecha).await
}

#[command]
pub async fn generate_estado_resultados(
    db: tauri::State<'_, Arc<DbPool>>,
    fecha_desde: String,
    fecha_hasta: String,
) -> AppResult<EstadoResultadosResponse> {
    generate_estado_resultados_impl(&db, fecha_desde, fecha_hasta).await
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use soft_gloria_infrastructure::create_memory_pool;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use soft_gloria_domain::accounting::{AsientoContable, LineaAsiento};
    use soft_gloria_domain::repositories::AccountingRepository;

    fn create_test_pool() -> DbPool {
        let pool = create_memory_pool().unwrap();
        let conn = pool.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS asientos_contables (
                id TEXT PRIMARY KEY NOT NULL,
                fecha TEXT NOT NULL,
                descripcion TEXT NOT NULL,
                lineas TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_asientos_fecha ON asientos_contables(fecha);
            "#,
        ).unwrap();
        drop(conn);
        pool
    }

    fn create_test_asiento() -> AsientoContable {
        let lineas = vec![
            LineaAsiento::new_debito("1110 Caja".to_string(), dec!(1000)).unwrap(),
            LineaAsiento::new_credito("4110 Capital".to_string(), dec!(1000)).unwrap(),
        ];
        AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Aporte de capital".to_string(),
            lineas,
        ).unwrap()
    }

    #[tokio::test]
    async fn test_add_asiento_valid() {
        let pool = create_test_pool();
        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Test asiento".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest { cuenta: "1110 Caja".to_string(), debito: Some("1000".to_string()), credito: None },
                CreateLineaAsientoRequest { cuenta: "4110 Capital".to_string(), debito: None, credito: Some("1000".to_string()) },
            ],
        };

        let result = add_asiento_impl(&pool, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.descripcion, "Test asiento");
        assert!(response.is_balanced);
    }

    #[tokio::test]
    async fn test_add_asiento_invalid_empty_lines() {
        let pool = create_test_pool();
        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Test".to_string(),
            lineas: vec![],
        };

        let result = add_asiento_impl(&pool, request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_add_asiento_invalid_both_debito_credito() {
        let pool = create_test_pool();
        let request = CreateAsientoRequest {
            fecha: "2024-01-15".to_string(),
            descripcion: "Test".to_string(),
            lineas: vec![
                CreateLineaAsientoRequest { cuenta: "1110 Caja".to_string(), debito: Some("100".to_string()), credito: Some("100".to_string()) },
            ],
        };

        let result = add_asiento_impl(&pool, request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_remove_asiento() {
        let pool = create_test_pool();
        let asiento = create_test_asiento();
        let repo = SqliteAccountingRepository::new(pool.clone());
        repo.create_asiento(&asiento).await.unwrap();

        let result = remove_asiento_impl(&pool, asiento.id.to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        let retrieved = repo.get_asiento_by_id(asiento.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_asientos_by_date_range() {
        let pool = create_test_pool();
        let repo = SqliteAccountingRepository::new(pool.clone());

        let asiento1 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            "Asiento 1".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(500)).unwrap(),
                LineaAsiento::new_credito("4110 Capital".to_string(), dec!(500)).unwrap(),
            ],
        ).unwrap();

        let asiento2 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "Asiento 2".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(300)).unwrap(),
                LineaAsiento::new_credito("4110 Capital".to_string(), dec!(300)).unwrap(),
            ],
        ).unwrap();

        repo.create_asiento(&asiento1).await.unwrap();
        repo.create_asiento(&asiento2).await.unwrap();

        let query = ListAsientosQuery {
            fecha_desde: Some("2024-01-01".to_string()),
            fecha_hasta: Some("2024-01-31".to_string()),
            page: Some(0),
            page_size: Some(10),
        };

        let result = list_asientos_impl(&pool, query).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.items.len(), 2);
    }

    #[tokio::test]
    async fn test_generate_balance_general() {
        let pool = create_test_pool();
        let repo = SqliteAccountingRepository::new(pool.clone());

        let asiento1 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Aporte capital".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(10000)).unwrap(),
                LineaAsiento::new_credito("3110 Capital Social".to_string(), dec!(10000)).unwrap(),
            ],
        ).unwrap();

        repo.create_asiento(&asiento1).await.unwrap();

        let result = generate_balance_general_impl(&pool, "2024-01-31".to_string()).await;
        assert!(result.is_ok());
        let balance = result.unwrap();
        assert!(balance.is_balanced);
    }

    #[tokio::test]
    async fn test_generate_estado_resultados() {
        let pool = create_test_pool();
        let repo = SqliteAccountingRepository::new(pool.clone());

        let asiento1 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Venta".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(10000)).unwrap(),
                LineaAsiento::new_credito("4110 Ventas".to_string(), dec!(10000)).unwrap(),
            ],
        ).unwrap();

        let asiento2 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "Alquiler".to_string(),
            vec![
                LineaAsiento::new_debito("5110 Alquiler".to_string(), dec!(3000)).unwrap(),
                LineaAsiento::new_credito("1110 Caja".to_string(), dec!(3000)).unwrap(),
            ],
        ).unwrap();

        repo.create_asiento(&asiento1).await.unwrap();
        repo.create_asiento(&asiento2).await.unwrap();

        let result = generate_estado_resultados_impl(
            &pool,
            "2024-01-01".to_string(),
            "2024-01-31".to_string(),
        ).await;
        assert!(result.is_ok());
        let estado = result.unwrap();
        assert_eq!(estado.total_ingresos, "10000");
        assert_eq!(estado.total_gastos, "3000");
        assert_eq!(estado.utilidad_neta, "7000");
    }
}
