use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;
use std::str::FromStr;

macro_rules! newtype_uuid {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
        
        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
            
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
            
            pub fn nil() -> Self {
                Self(Uuid::nil())
            }
            
            pub fn is_nil(&self) -> bool {
                self.0.is_nil()
            }
        }
        
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        
        impl FromStr for $name {
            type Err = UuidError;
            
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|_| UuidError::InvalidFormat)
            }
        }
        
        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
        
        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UuidError {
    #[error("Invalid UUID format")]
    InvalidFormat,
}

newtype_uuid!(PatientId);
newtype_uuid!(AppointmentId);
newtype_uuid!(ClinicalNoteId);
newtype_uuid!(DiagnosisId);
newtype_uuid!(PrescriptionId);
newtype_uuid!(LabOrderId);
newtype_uuid!(DocumentId);
newtype_uuid!(UserId);
newtype_uuid!(OrganizationId);
newtype_uuid!(LocationId);

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_patient_id() {
        let id = PatientId::new();
        assert!(!id.is_nil());
        
        let uuid_str = id.to_string();
        let parsed = PatientId::from_str(&uuid_str).unwrap();
        assert_eq!(id, parsed);
    }
    
    #[test]
    fn test_appointment_id() {
        let id = AppointmentId::new();
        assert!(!id.is_nil());
    }
    
    #[test]
    fn test_uuid_from_str_invalid() {
        assert!(PatientId::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_uuid_conversion() {
        let uuid = Uuid::new_v4();
        let patient_id = PatientId::from(uuid);
        let back: Uuid = patient_id.into();
        assert_eq!(uuid, back);
    }
}