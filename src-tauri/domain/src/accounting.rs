use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Represents a General Ledger (Libro Diario) containing multiple accounting entries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibroDiario {
    pub asientos: Vec<AsientoContable>,
}

impl LibroDiario {
    pub fn new() -> Self {
        Self { asientos: Vec::new() }
    }

    pub fn add_asiento(&mut self, asiento: AsientoContable) {
        self.asientos.push(asiento);
    }

    pub fn total_debitos(&self) -> Decimal {
        self.asientos
            .iter()
            .flat_map(|a| &a.lineas)
            .map(|l| l.debito)
            .sum()
    }

    pub fn total_creditos(&self) -> Decimal {
        self.asientos
            .iter()
            .flat_map(|a| &a.lineas)
            .map(|l| l.credito)
            .sum()
    }

    pub fn is_balanced(&self) -> bool {
        let diff = (self.total_debitos() - self.total_creditos()).abs();
        diff < Decimal::new(1, 2) // 0.01 epsilon for decimal precision
    }
}

impl Default for LibroDiario {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a single accounting entry (Asiento Contable)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AsientoContable {
    pub id: Uuid,
    pub fecha: chrono::NaiveDate,
    pub descripcion: String,
    pub lineas: Vec<LineaAsiento>,
}

impl AsientoContable {
    pub fn new(fecha: chrono::NaiveDate, descripcion: String, lineas: Vec<LineaAsiento>) -> Result<Self, ContabilidadError> {
        if lineas.is_empty() {
            return Err(ContabilidadError::AsientoVacio);
        }

        // Validate each line has either debit or credit (not both, not neither)
        for linea in &lineas {
            if linea.debito > Decimal::ZERO && linea.credito > Decimal::ZERO {
                return Err(ContabilidadError::LineaDualDebitoCredito);
            }
            if linea.debito == Decimal::ZERO && linea.credito == Decimal::ZERO {
                return Err(ContabilidadError::LineaSinMonto);
            }
            if linea.cuenta.trim().is_empty() {
                return Err(ContabilidadError::CuentaVacia);
            }
        }

        // Validate the entry is balanced
        let total_debitos: Decimal = lineas.iter().map(|l| l.debito).sum();
        let total_creditos: Decimal = lineas.iter().map(|l| l.credito).sum();
        
        if (total_debitos - total_creditos).abs() >= Decimal::new(1, 2) {
            return Err(ContabilidadError::BalanceDesbalanceado {
                activos: total_debitos,
                pasivos: total_creditos,
                patrimonio: Decimal::ZERO,
            });
        }

        Ok(Self {
            id: Uuid::new_v4(),
            fecha,
            descripcion,
            lineas,
        })
    }

    pub fn total_debitos(&self) -> Decimal {
        self.lineas.iter().map(|l| l.debito).sum()
    }

    pub fn total_creditos(&self) -> Decimal {
        self.lineas.iter().map(|l| l.credito).sum()
    }

    pub fn is_balanced(&self) -> bool {
        let diff = (self.total_debitos() - self.total_creditos()).abs();
        diff < Decimal::new(1, 2)
    }
}

/// Represents a single line in an accounting entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineaAsiento {
    pub cuenta: String,
    pub debito: Decimal,
    pub credito: Decimal,
}

impl LineaAsiento {
    pub fn new_debito(cuenta: String, monto: Decimal) -> Result<Self, ContabilidadError> {
        if monto <= Decimal::ZERO {
            return Err(ContabilidadError::MontoInvalido(monto));
        }
        if cuenta.trim().is_empty() {
            return Err(ContabilidadError::CuentaVacia);
        }
        Ok(Self {
            cuenta: cuenta.trim().to_string(),
            debito: monto,
            credito: Decimal::ZERO,
        })
    }

    pub fn new_credito(cuenta: String, monto: Decimal) -> Result<Self, ContabilidadError> {
        if monto <= Decimal::ZERO {
            return Err(ContabilidadError::MontoInvalido(monto));
        }
        if cuenta.trim().is_empty() {
            return Err(ContabilidadError::CuentaVacia);
        }
        Ok(Self {
            cuenta: cuenta.trim().to_string(),
            debito: Decimal::ZERO,
            credito: monto,
        })
    }

    pub fn is_debito(&self) -> bool {
        self.debito > Decimal::ZERO
    }

    pub fn is_credito(&self) -> bool {
        self.credito > Decimal::ZERO
    }

    pub fn monto(&self) -> Decimal {
        if self.is_debito() { self.debito } else { self.credito }
    }
}

/// Accounting errors
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum ContabilidadError {
    #[error("Balance desbalanceado: Activos={activos}, Pasivos={pasivos}, Patrimonio={patrimonio}")]
    BalanceDesbalanceado {
        activos: Decimal,
        pasivos: Decimal,
        patrimonio: Decimal,
    },
    #[error("Asiento vacío: debe tener al menos una línea")]
    AsientoVacio,
    #[error("Monto inválido: {0}")]
    MontoInvalido(Decimal),
    #[error("Línea tiene débito y crédito simultáneamente")]
    LineaDualDebitoCredito,
    #[error("Línea sin monto: debe tener débito o crédito")]
    LineaSinMonto,
    #[error("Cuenta vacía: nombre de cuenta requerido")]
    CuentaVacia,
}

/// Validates the fundamental accounting equation: Activos = Pasivos + Patrimonio
/// Returns Ok(()) if balanced within epsilon, Err with details if not
pub fn validar_balance_general(
    activos: Decimal,
    pasivos: Decimal,
    patrimonio: Decimal,
) -> Result<(), ContabilidadError> {
    let total_pasivos_patrimonio = pasivos + patrimonio;
    let diff = (activos - total_pasivos_patrimonio).abs();
    
    // Use 0.01 epsilon for decimal precision (centavo level)
    if diff >= Decimal::new(1, 2) {
        Err(ContabilidadError::BalanceDesbalanceado {
            activos,
            pasivos,
            patrimonio,
        })
    } else {
        Ok(())
    }
}

impl fmt::Display for LineaAsiento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_debito() {
            write!(f, "{} D: {}", self.cuenta, self.debito)
        } else {
            write!(f, "{} C: {}", self.cuenta, self.credito)
        }
    }
}

impl fmt::Display for AsientoContable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Asiento {} - {} - {}", self.id, self.fecha, self.descripcion)?;
        for linea in &self.lineas {
            writeln!(f, "  {}", linea)?;
        }
        writeln!(f, "  Total Debitos: {}", self.total_debitos())?;
        writeln!(f, "  Total Creditos: {}", self.total_creditos())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_validar_balance_general_balanced() {
        // Activos = 1000, Pasivos = 600, Patrimonio = 400 => 1000 = 600 + 400 ✓
        let result = validar_balance_general(dec!(1000), dec!(600), dec!(400));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validar_balance_general_unbalanced() {
        // Activos = 1000, Pasivos = 600, Patrimonio = 300 => 1000 != 900 ✗
        let result = validar_balance_general(dec!(1000), dec!(600), dec!(300));
        assert!(result.is_err());
        
        match result {
            Err(ContabilidadError::BalanceDesbalanceado { activos, pasivos, patrimonio }) => {
                assert_eq!(activos, dec!(1000));
                assert_eq!(pasivos, dec!(600));
                assert_eq!(patrimonio, dec!(300));
            }
            _ => panic!("Expected BalanceDesbalanceado error"),
        }
    }

    #[test]
    fn test_validar_balance_general_with_epsilon() {
        // Small difference within epsilon (0.005) should pass
        let result = validar_balance_general(dec!(1000.005), dec!(600), dec!(400.005));
        assert!(result.is_ok());
    }

    #[test]
    fn test_asiento_contable_creation_valid() {
        let lineas = vec![
            LineaAsiento::new_debito("1110 Caja".to_string(), dec!(1000)).unwrap(),
            LineaAsiento::new_credito("4110 Capital".to_string(), dec!(1000)).unwrap(),
        ];
        
        let asiento = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Aporte de capital".to_string(),
            lineas,
        ).unwrap();
        
        assert!(asiento.is_balanced());
        assert_eq!(asiento.total_debitos(), dec!(1000));
        assert_eq!(asiento.total_creditos(), dec!(1000));
    }

    #[test]
    fn test_asiento_contable_rejects_empty() {
        let result = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Empty".to_string(),
            vec![],
        );
        assert!(matches!(result, Err(ContabilidadError::AsientoVacio)));
    }

    #[test]
    fn test_asiento_contable_rejects_unbalanced() {
        let lineas = vec![
            LineaAsiento::new_debito("1110 Caja".to_string(), dec!(1000)).unwrap(),
            LineaAsiento::new_credito("4110 Capital".to_string(), dec!(500)).unwrap(),
        ];
        
        let result = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Unbalanced".to_string(),
            lineas,
        );
        assert!(matches!(result, Err(ContabilidadError::BalanceDesbalanceado { .. })));
    }

    #[test]
    fn test_asiento_contable_rejects_both_debito_credito() {
        let lineas = vec![
            LineaAsiento {
                cuenta: "1110 Caja".to_string(),
                debito: dec!(100),
                credito: dec!(50),
            }
        ];
        
        let result = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Both".to_string(),
            lineas,
        );
        assert!(matches!(result, Err(ContabilidadError::LineaDualDebitoCredito)));
    }

    #[test]
    fn test_asiento_contable_rejects_zero_amount() {
        let lineas = vec![
            LineaAsiento {
                cuenta: "1110 Caja".to_string(),
                debito: Decimal::ZERO,
                credito: Decimal::ZERO,
            }
        ];
        
        let result = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Zero".to_string(),
            lineas,
        );
        assert!(matches!(result, Err(ContabilidadError::LineaSinMonto)));
    }

    #[test]
    fn test_linea_asiento_debito() {
        let linea = LineaAsiento::new_debito("1110 Caja".to_string(), dec!(1000)).unwrap();
        assert!(linea.is_debito());
        assert!(!linea.is_credito());
        assert_eq!(linea.monto(), dec!(1000));
    }

    #[test]
    fn test_linea_asiento_credito() {
        let linea = LineaAsiento::new_credito("4110 Capital".to_string(), dec!(1000)).unwrap();
        assert!(!linea.is_debito());
        assert!(linea.is_credito());
        assert_eq!(linea.monto(), dec!(1000));
    }

    #[test]
    fn test_linea_asiento_rejects_negative() {
        let result = LineaAsiento::new_debito("1110 Caja".to_string(), dec!(-100));
        assert!(matches!(result, Err(ContabilidadError::MontoInvalido(_))));
    }

    #[test]
    fn test_linea_asiento_rejects_zero() {
        let result = LineaAsiento::new_debito("1110 Caja".to_string(), Decimal::ZERO);
        assert!(matches!(result, Err(ContabilidadError::MontoInvalido(_))));
    }

    #[test]
    fn test_linea_asiento_rejects_empty_cuenta() {
        let result = LineaAsiento::new_debito("".to_string(), dec!(100));
        assert!(matches!(result, Err(ContabilidadError::CuentaVacia)));
    }

    #[test]
    fn test_libro_diario() {
        let mut libro = LibroDiario::new();
        
        let asiento1 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Aporte capital".to_string(),
            vec![
                LineaAsiento::new_debito("1110 Caja".to_string(), dec!(1000)).unwrap(),
                LineaAsiento::new_credito("4110 Capital".to_string(), dec!(1000)).unwrap(),
            ],
        ).unwrap();
        
        let asiento2 = AsientoContable::new(
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "Compra mercadería".to_string(),
            vec![
                LineaAsiento::new_debito("6110 Mercaderías".to_string(), dec!(500)).unwrap(),
                LineaAsiento::new_credito("1110 Caja".to_string(), dec!(500)).unwrap(),
            ],
        ).unwrap();
        
        libro.add_asiento(asiento1);
        libro.add_asiento(asiento2);
        
        assert_eq!(libro.total_debitos(), dec!(1500));
        assert_eq!(libro.total_creditos(), dec!(1500));
        assert!(libro.is_balanced());
    }
}
