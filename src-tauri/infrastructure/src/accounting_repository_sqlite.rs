use crate::database::DbPool;
use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rusqlite::params;
use serde_json;
use soft_mindledger_domain::{
    accounting::{AsientoContable, LineaAsiento},
    repositories::{AccountingRepository, Pagination, RepositoryError},
};
use uuid::Uuid;

pub struct SqliteAccountingRepository {
    pub pool: DbPool,
}

impl SqliteAccountingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn row_to_asiento(row: &rusqlite::Row) -> rusqlite::Result<AsientoContable> {
        let id: String = row.get("id")?;
        let fecha: String = row.get("fecha")?;
        let descripcion: String = row.get("descripcion")?;
        let lineas_json: String = row.get("lineas")?;

        let lineas: Vec<LineaAsiento> = serde_json::from_str(&lineas_json)
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("JSON parse error: {}", e)))?;

        Ok(AsientoContable {
            id: Uuid::parse_str(&id).map_err(|e| rusqlite::Error::InvalidParameterName(format!("UUID parse error: {}", e)))?,
            fecha: NaiveDate::parse_from_str(&fecha, "%Y-%m-%d")
                .map_err(|e| rusqlite::Error::InvalidParameterName(format!("Date parse error: {}", e)))?,
            descripcion,
            lineas,
        })
    }

    fn asiento_to_params(asiento: &AsientoContable) -> Result<(String, String, String, String)> {
        let id = asiento.id.to_string();
        let fecha = asiento.fecha.format("%Y-%m-%d").to_string();
        let descripcion = asiento.descripcion.clone();
        let lineas = serde_json::to_string(&asiento.lineas)?;
        Ok((id, fecha, descripcion, lineas))
    }
}

#[async_trait::async_trait]
impl AccountingRepository for SqliteAccountingRepository {
    async fn create_asiento(&self, asiento: &AsientoContable) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let asiento = asiento.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let (id, fecha, descripcion, lineas) = Self::asiento_to_params(&asiento)
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            conn.execute(
                "INSERT INTO asientos_contables (id, fecha, descripcion, lineas, created_at) VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                params![id, fecha, descripcion, lineas],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_asiento_by_id(&self, id: Uuid) -> Result<Option<AsientoContable>, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let mut stmt = conn.prepare("SELECT id, fecha, descripcion, lineas FROM asientos_contables WHERE id = ?1")
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            let mut rows = stmt.query(params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| RepositoryError::Database(e.to_string()))? {
                let asiento = Self::row_to_asiento(row)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(Some(asiento))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn list_asientos(&self, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>, pagination: Pagination) -> Result<Vec<AsientoContable>, RepositoryError> {
        let pool = self.pool.clone();
        let desde_str = fecha_desde.map(|d| d.format("%Y-%m-%d").to_string());
        let hasta_str = fecha_hasta.map(|d| d.format("%Y-%m-%d").to_string());
        let limit = pagination.limit as i64;
        let offset = pagination.offset as i64;

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;

            let mut where_clauses = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(desde) = desde_str {
                where_clauses.push(format!("fecha >= ?{}", param_values.len() + 1));
                param_values.push(Box::new(desde));
            }
            if let Some(hasta) = hasta_str {
                where_clauses.push(format!("fecha <= ?{}", param_values.len() + 1));
                param_values.push(Box::new(hasta));
            }

            let where_sql = if where_clauses.is_empty() {
                "1=1".to_string()
            } else {
                where_clauses.join(" AND ")
            };

            let sql = format!(
                "SELECT id, fecha, descripcion, lineas FROM asientos_contables WHERE {} ORDER BY fecha, created_at LIMIT ?{} OFFSET ?{}",
                where_sql,
                param_values.len() + 1,
                param_values.len() + 2
            );

            param_values.push(Box::new(limit));
            param_values.push(Box::new(offset));

            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| Self::row_to_asiento(row))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            let mut asientos = Vec::new();
            for row in rows {
                asientos.push(row.map_err(|e| RepositoryError::Database(e.to_string()))?);
            }
            Ok(asientos)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn update_asiento(&self, asiento: &AsientoContable) -> Result<(), RepositoryError> {
        let pool = self.pool.clone();
        let asiento = asiento.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let (id, fecha, descripcion, lineas) = Self::asiento_to_params(&asiento)
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            let affected = conn.execute(
                "UPDATE asientos_contables SET fecha = ?1, descripcion = ?2, lineas = ?3, updated_at = datetime('now') WHERE id = ?4",
                params![fecha, descripcion, lineas, id],
            ).map_err(|e| RepositoryError::Database(e.to_string()))?;

            if affected == 0 {
                return Err(RepositoryError::NotFound(format!("Asiento not found: {}", id)));
            }
            Ok(())
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn delete_asiento(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let pool = self.pool.clone();
        let id_str = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;
            let affected = conn.execute("DELETE FROM asientos_contables WHERE id = ?1", params![id_str])
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            Ok(affected > 0)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn count_asientos(&self, fecha_desde: Option<NaiveDate>, fecha_hasta: Option<NaiveDate>) -> Result<u64, RepositoryError> {
        let pool = self.pool.clone();
        let desde_str = fecha_desde.map(|d| d.format("%Y-%m-%d").to_string());
        let hasta_str = fecha_hasta.map(|d| d.format("%Y-%m-%d").to_string());

        tokio::task::spawn_blocking(move || {
            let conn = pool.lock().map_err(|e| RepositoryError::Database(format!("Lock poisoned: {}", e)))?;

            let mut where_clauses = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(desde) = desde_str {
                where_clauses.push(format!("fecha >= ?{}", param_values.len() + 1));
                param_values.push(Box::new(desde));
            }
            if let Some(hasta) = hasta_str {
                where_clauses.push(format!("fecha <= ?{}", param_values.len() + 1));
                param_values.push(Box::new(hasta));
            }

            let where_sql = if where_clauses.is_empty() {
                "1=1".to_string()
            } else {
                where_clauses.join(" AND ")
            };

            let sql = format!("SELECT COUNT(*) FROM asientos_contables WHERE {}", where_sql);

            let mut stmt = conn.prepare(&sql).map_err(|e| RepositoryError::Database(e.to_string()))?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            Ok(count as u64)
        }).await.map_err(|e| RepositoryError::Database(format!("Task join error: {}", e)))?
    }

    async fn get_balance_general(&self, fecha: NaiveDate) -> Result<soft_mindledger_domain::BalanceGeneral, RepositoryError> {
        let asientos = self.list_asientos(
            Some(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()),
            Some(fecha),
            Pagination::new(0, 10000),
        ).await?;

        let mut activos: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();
        let mut pasivos: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();
        let mut patrimonio: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();

        for asiento in asientos {
            for linea in asiento.lineas {
                let cuenta = linea.cuenta.trim();
                let primer_char = cuenta.chars().next().unwrap_or('0');

                let target_map = match primer_char {
                    '1' => (&mut activos, 1),      // Activos (1xxx) - normal balance: debit (+)
                    '2' => (&mut pasivos, -1),     // Pasivos (2xxx) - normal balance: credit (-)
                    '3' => (&mut patrimonio, -1),  // Patrimonio (3xxx) - normal balance: credit (-)
                    '4' => (&mut patrimonio, -1),  // Ingresos (4xxx) - normal balance: credit (-), increases equity
                    '5' => (&mut patrimonio, -1),  // Gastos (5xxx) - normal balance: debit (+), but DECREASES equity
                    '6' => (&mut patrimonio, -1),  // Costos (6xxx) - normal balance: debit (+), but DECREASES equity
                    '7' => (&mut patrimonio, -1),  // Otros ingresos (7xxx) - normal balance: credit (-)
                    _ => (&mut patrimonio, -1),    // Default a patrimonio
                };

                let entry = target_map.0.entry(cuenta.to_string()).or_insert(Decimal::ZERO);
                let multiplier = target_map.1;
                if linea.is_debito() {
                    *entry += linea.monto() * Decimal::from(multiplier);
                } else {
                    *entry -= linea.monto() * Decimal::from(multiplier);
                }
            }
        }

        let activos_vec: Vec<(String, Decimal)> = activos.into_iter().filter(|(_, v)| *v != Decimal::ZERO).collect();
        let pasivos_vec: Vec<(String, Decimal)> = pasivos.into_iter().filter(|(_, v)| *v != Decimal::ZERO).collect();
        let patrimonio_vec: Vec<(String, Decimal)> = patrimonio.into_iter().filter(|(_, v)| *v != Decimal::ZERO).collect();

        Ok(soft_mindledger_domain::BalanceGeneral {
            fecha,
            activos: activos_vec,
            pasivos: pasivos_vec,
            patrimonio: patrimonio_vec,
        })
    }

    async fn get_estado_resultados(&self, desde: NaiveDate, hasta: NaiveDate) -> Result<soft_mindledger_domain::EstadoResultados, RepositoryError> {
        let asientos = self.list_asientos(Some(desde), Some(hasta), Pagination::new(0, 10000)).await?;

        let mut ingresos: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();
        let mut gastos: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();

        for asiento in asientos {
            for linea in asiento.lineas {
                let cuenta = linea.cuenta.trim();
                let primer_char = cuenta.chars().next().unwrap_or('0');

                match primer_char {
                    '4' | '7' => { // Ingresos
                        let entry = ingresos.entry(cuenta.to_string()).or_insert(Decimal::ZERO);
                        if linea.is_credito() {
                            *entry += linea.monto();
                        } else {
                            *entry -= linea.monto();
                        }
                    }
                    '5' | '6' => { // Gastos y Costos
                        let entry = gastos.entry(cuenta.to_string()).or_insert(Decimal::ZERO);
                        if linea.is_debito() {
                            *entry += linea.monto();
                        } else {
                            *entry -= linea.monto();
                        }
                    }
                    _ => {}
                }
            }
        }

        let total_ingresos: Decimal = ingresos.values().sum();
        let total_gastos: Decimal = gastos.values().sum();
        let utilidad_neta = total_ingresos - total_gastos;

        let ingresos_vec: Vec<(String, Decimal)> = ingresos.into_iter().filter(|(_, v)| *v != Decimal::ZERO).collect();
        let gastos_vec: Vec<(String, Decimal)> = gastos.into_iter().filter(|(_, v)| *v != Decimal::ZERO).collect();

        Ok(soft_mindledger_domain::EstadoResultados {
            fecha: hasta,
            ingresos: ingresos_vec,
            gastos: gastos_vec,
            utilidad_neta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_memory_pool;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use soft_mindledger_domain::accounting::{AsientoContable, LineaAsiento};
    use soft_mindledger_domain::repositories::AccountingRepository;

    fn create_test_repo() -> SqliteAccountingRepository {
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
        SqliteAccountingRepository::new(pool)
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
    async fn test_create_and_get_asiento() {
        let repo = create_test_repo();
        let asiento = create_test_asiento();
        let id = asiento.id;

        repo.create_asiento(&asiento).await.unwrap();

        let retrieved = repo.get_asiento_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.fecha, asiento.fecha);
        assert_eq!(retrieved.descripcion, asiento.descripcion);
        assert_eq!(retrieved.lineas.len(), 2);
        assert!(retrieved.is_balanced());
    }

    #[tokio::test]
    async fn test_list_asientos_by_date_range() {
        let repo = create_test_repo();

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

        let asiento3 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            "Asiento 3".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(200)).unwrap(),
                LineaAsiento::new_credito("4110 Capital".to_string(), dec!(200)).unwrap(),
            ],
        ).unwrap();

        repo.create_asiento(&asiento1).await.unwrap();
        repo.create_asiento(&asiento2).await.unwrap();
        repo.create_asiento(&asiento3).await.unwrap();

        let resultados = repo.list_asientos(
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            soft_mindledger_domain::repositories::Pagination::new(0, 10),
        ).await.unwrap();

        assert_eq!(resultados.len(), 2);
        assert_eq!(resultados[0].descripcion, "Asiento 1");
        assert_eq!(resultados[1].descripcion, "Asiento 2");
    }

    #[tokio::test]
    async fn test_update_asiento() {
        let repo = create_test_repo();
        let mut asiento = create_test_asiento();
        repo.create_asiento(&asiento).await.unwrap();

        asiento.descripcion = "Descripción actualizada".to_string();
        repo.update_asiento(&asiento).await.unwrap();

        let retrieved = repo.get_asiento_by_id(asiento.id).await.unwrap().unwrap();
        assert_eq!(retrieved.descripcion, "Descripción actualizada");
    }

    #[tokio::test]
    async fn test_delete_asiento() {
        let repo = create_test_repo();
        let asiento = create_test_asiento();
        repo.create_asiento(&asiento).await.unwrap();

        let deleted = repo.delete_asiento(asiento.id).await.unwrap();
        assert!(deleted);

        let retrieved = repo.get_asiento_by_id(asiento.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_get_balance_general() {
        let repo = create_test_repo();

        // Activo: Caja (1110)
        let asiento1 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Aporte capital".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(10000)).unwrap(),
                LineaAsiento::new_credito("3110 Capital Social".to_string(), dec!(10000)).unwrap(),
            ],
        ).unwrap();

        // Activo: Bancos (1120)
        let asiento2 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "Depósito bancario".to_string(),
            vec![
                LineaAsiento::new_debito("1120 Bancos".to_string(), dec!(5000)).unwrap(),
                LineaAsiento::new_credito("1110 Caja".to_string(), dec!(5000)).unwrap(),
            ],
        ).unwrap();

        // Pasivo: Proveedores (2110)
        let asiento3 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            "Compra a crédito".to_string(),
            vec![
                LineaAsiento::new_debito("6110 Mercaderías".to_string(), dec!(3000)).unwrap(),
                LineaAsiento::new_credito("2110 Proveedores".to_string(), dec!(3000)).unwrap(),
            ],
        ).unwrap();

        repo.create_asiento(&asiento1).await.unwrap();
        repo.create_asiento(&asiento2).await.unwrap();
        repo.create_asiento(&asiento3).await.unwrap();

        let balance = repo.get_balance_general(NaiveDate::from_ymd_opt(2024, 2, 28).unwrap()).await.unwrap();

        // Activos: Caja 5000 + Bancos 5000 = 10000
        let caja = balance.activos.iter().find(|(c, _)| c == "1110 Caja").map(|(_, v)| *v).unwrap_or(dec!(0));
        let bancos = balance.activos.iter().find(|(c, _)| c == "1120 Bancos").map(|(_, v)| *v).unwrap_or(dec!(0));
        assert_eq!(caja, dec!(5000));
        assert_eq!(bancos, dec!(5000));

        // Pasivos: Proveedores 3000
        let proveedores = balance.pasivos.iter().find(|(c, _)| c == "2110 Proveedores").map(|(_, v)| *v).unwrap_or(dec!(0));
        assert_eq!(proveedores, dec!(3000));

        // Patrimonio: Capital Social 10000 - Mercaderías 3000 (gasto) = 7000
        let capital = balance.patrimonio.iter().find(|(c, _)| c == "3110 Capital Social").map(|(_, v)| *v).unwrap_or(dec!(0));
        let mercaderias = balance.patrimonio.iter().find(|(c, _)| c == "6110 Mercaderías").map(|(_, v)| *v).unwrap_or(dec!(0));
        assert_eq!(capital, dec!(10000));
        assert_eq!(mercaderias, dec!(-3000)); // Gasto reduce patrimonio
    }

    #[tokio::test]
    async fn test_get_estado_resultados() {
        let repo = create_test_repo();

        // Ingreso: Ventas (4110)
        let asiento1 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Venta de mercadería".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(10000)).unwrap(),
                LineaAsiento::new_credito("4110 Ventas".to_string(), dec!(10000)).unwrap(),
            ],
        ).unwrap();

        // Gasto: Alquiler (5110)
        let asiento2 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "Pago alquiler".to_string(),
            vec![
                LineaAsiento::new_debito("5110 Alquiler".to_string(), dec!(3000)).unwrap(),
                LineaAsiento::new_credito("1110 Caja".to_string(), dec!(3000)).unwrap(),
            ],
        ).unwrap();

        // Gasto: Servicios (5120)
        let asiento3 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            "Pago servicios".to_string(),
            vec![
                LineaAsiento::new_debito("5120 Servicios".to_string(), dec!(1000)).unwrap(),
                LineaAsiento::new_credito("1110 Caja".to_string(), dec!(1000)).unwrap(),
            ],
        ).unwrap();

        repo.create_asiento(&asiento1).await.unwrap();
        repo.create_asiento(&asiento2).await.unwrap();
        repo.create_asiento(&asiento3).await.unwrap();

        let estado = repo.get_estado_resultados(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 28).unwrap(),
        ).await.unwrap();

        let ventas = estado.ingresos.iter().find(|(c, _)| c == "4110 Ventas").map(|(_, v)| *v).unwrap_or(dec!(0));
        let alquiler = estado.gastos.iter().find(|(c, _)| c == "5110 Alquiler").map(|(_, v)| *v).unwrap_or(dec!(0));
        let servicios = estado.gastos.iter().find(|(c, _)| c == "5120 Servicios").map(|(_, v)| *v).unwrap_or(dec!(0));

        assert_eq!(ventas, dec!(10000));
        assert_eq!(alquiler, dec!(3000));
        assert_eq!(servicios, dec!(1000));
        assert_eq!(estado.utilidad_neta, dec!(6000)); // 10000 - 3000 - 1000
    }
}