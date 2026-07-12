use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// CIE-10 Diagnostic classification for Ecuador medical practice
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticoCIE10 {
    pub codigo: String,
    pub descripcion: String,
    pub categoria: CategoriaCIE10,
    pub subcategoria: Option<String>,
}

impl DiagnosticoCIE10 {
    pub fn new(codigo: String, descripcion: String, categoria: CategoriaCIE10, subcategoria: Option<String>) -> Self {
        Self {
            codigo: codigo.to_uppercase(),
            descripcion,
            categoria,
            subcategoria,
        }
    }

    pub fn codigo_normalizado(&self) -> String {
        self.codigo.replace(".", "").to_uppercase()
    }
}

impl fmt::Display for DiagnosticoCIE10 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.codigo, self.descripcion)
    }
}

/// CIE-10 Categories (Capítulos principales)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CategoriaCIE10 {
    /// I - Ciertas enfermedades infecciosas y parasitarias
    EnfermedadesInfecciosas,
    /// II - Neoplasias
    Neoplasias,
    /// III - Enfermedades de la sangre y de los órganos hematopoyéticos
    EnfermedadesSangre,
    /// IV - Enfermedades endocrinas, nutricionales y metabólicas
    EndocrinasNutricionalesMetabolicas,
    /// V - Trastornos mentales y del comportamiento
    TrastornosMentales,
    /// VI - Enfermedades del sistema nervioso
    SistemaNervioso,
    /// VII - Enfermedades del ojo y sus anexos
    OjoAnexos,
    /// VIII - Enfermedades del oído y de la apófisis mastoides
    OidoMastoides,
    /// IX - Enfermedades del sistema circulatorio
    SistemaCirculatorio,
    /// X - Enfermedades del sistema respiratorio
    SistemaRespiratorio,
    /// XI - Enfermedades del sistema digestivo
    SistemaDigestivo,
    /// XII - Enfermedades de la piel y del tejido subcutáneo
    PielTejiidoSubcutaneo,
    /// XIII - Enfermedades del sistema osteomuscular y del tejido conectivo
    OsteomuscularConectivo,
    /// XIV - Enfermedades del sistema genitourinario
    Genitourinario,
    /// XV - Embarazo, parto y puerperio
    EmbarazoPartoPuerperio,
    /// XVI - Ciertas afecciones originadas en el período perinatal
    Perinatal,
    /// XVII - Malformaciones congénitas, deformidades y anomalías cromosómicas
    MalformacionesCongenitas,
    /// XVIII - Síntomas, signos y hallazgos anormales clínicos y de laboratorio
    SintomasSignosHallazgos,
    /// XIX - Lesiones, envenenamiento y ciertas otras consecuencias de causas externas
    LesionesEnvenenamiento,
    /// XX - Causas externas de morbilidad y mortalidad
    CausasExternas,
    /// XXI - Factores que influyen en el estado de salud y contactan con los servicios de salud
    FactoresInfluyenSalud,
    /// XXII - Códigos para propósitos especiales
    CodigosEspeciales,
}

impl fmt::Display for CategoriaCIE10 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CategoriaCIE10::EnfermedadesInfecciosas => "I - Enfermedades infecciosas y parasitarias",
            CategoriaCIE10::Neoplasias => "II - Neoplasias",
            CategoriaCIE10::EnfermedadesSangre => "III - Enfermedades de la sangre",
            CategoriaCIE10::EndocrinasNutricionalesMetabolicas => "IV - Endocrinas, nutricionales y metabólicas",
            CategoriaCIE10::TrastornosMentales => "V - Trastornos mentales y del comportamiento",
            CategoriaCIE10::SistemaNervioso => "VI - Sistema nervioso",
            CategoriaCIE10::OjoAnexos => "VII - Ojo y anexos",
            CategoriaCIE10::OidoMastoides => "VIII - Oído y apófisis mastoides",
            CategoriaCIE10::SistemaCirculatorio => "IX - Sistema circulatorio",
            CategoriaCIE10::SistemaRespiratorio => "X - Sistema respiratorio",
            CategoriaCIE10::SistemaDigestivo => "XI - Sistema digestivo",
            CategoriaCIE10::PielTejiidoSubcutaneo => "XII - Piel y tejido subcutáneo",
            CategoriaCIE10::OsteomuscularConectivo => "XIII - Sistema osteomuscular y conectivo",
            CategoriaCIE10::Genitourinario => "XIV - Sistema genitourinario",
            CategoriaCIE10::EmbarazoPartoPuerperio => "XV - Embarazo, parto y puerperio",
            CategoriaCIE10::Perinatal => "XVI - Perinatal",
            CategoriaCIE10::MalformacionesCongenitas => "XVII - Malformaciones congénitas",
            CategoriaCIE10::SintomasSignosHallazgos => "XVIII - Síntomas, signos y hallazgos",
            CategoriaCIE10::LesionesEnvenenamiento => "XIX - Lesiones y envenenamiento",
            CategoriaCIE10::CausasExternas => "XX - Causas externas",
            CategoriaCIE10::FactoresInfluyenSalud => "XXI - Factores que influyen en la salud",
            CategoriaCIE10::CodigosEspeciales => "XXII - Códigos especiales",
        };
        write!(f, "{}", s)
    }
}

/// DSM-5 Diagnostic classification for mental health
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticoDSM5 {
    pub codigo: String,
    pub descripcion: String,
    pub categoria: CategoriaDSM5,
    pub criterios_diagnosticos: Option<Vec<String>>,
    pub especificaores: Option<Vec<String>>,
}

impl DiagnosticoDSM5 {
    pub fn new(
        codigo: String,
        descripcion: String,
        categoria: CategoriaDSM5,
        criterios_diagnosticos: Option<Vec<String>>,
        especificaores: Option<Vec<String>>,
    ) -> Self {
        Self {
            codigo: codigo.to_uppercase(),
            descripcion,
            categoria,
            criterios_diagnosticos,
            especificaores,
        }
    }
}

impl fmt::Display for DiagnosticoDSM5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.codigo, self.descripcion)
    }
}

/// DSM-5 Categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CategoriaDSM5 {
    TrastornosNeurodelDesarrollo,
    EspectroEsquizofreniaYTrastornosPsicoticos,
    TrastornosBipolaresYRelacionados,
    TrastornosDepresivos,
    TrastornosDeAnsiedad,
    TrastornosObsesivoCompulsivosYRelacionados,
    TrastornosRelacionadosConTraumaYFactoresDeEstres,
    TrastornosDisociativos,
    TrastornosSomaticosYRelacionados,
    TrastornosDeLaIngestaDeAlimentos,
    TrastornosDeEliminacion,
    TrastornosDelSueñoYVigilia,
    DisfuncionesSexuales,
    DisforiaDeGenero,
    TrastornosDisruptivosDelControlDeImpulsosYDeLaConducta,
    TrastornosRelacionadosConSustanciasYAdictivos,
    TrastornosNeurocognitivos,
    TrastornosDeLaPersonalidad,
    TrastornosParafiliicos,
    OtrosTrastornosMentales,
    TrastornosRelacionadosConProblemasDeSalud,
}

impl fmt::Display for CategoriaDSM5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CategoriaDSM5::TrastornosNeurodelDesarrollo => "Trastornos del neurodesarrollo",
            CategoriaDSM5::EspectroEsquizofreniaYTrastornosPsicoticos => "Espectro esquizofrenia y trastornos psicóticos",
            CategoriaDSM5::TrastornosBipolaresYRelacionados => "Trastornos bipolares y relacionados",
            CategoriaDSM5::TrastornosDepresivos => "Trastornos depresivos",
            CategoriaDSM5::TrastornosDeAnsiedad => "Trastornos de ansiedad",
            CategoriaDSM5::TrastornosObsesivoCompulsivosYRelacionados => "Trastornos obsesivo-compulsivos y relacionados",
            CategoriaDSM5::TrastornosRelacionadosConTraumaYFactoresDeEstres => "Trastornos relacionados con trauma y estrés",
            CategoriaDSM5::TrastornosDisociativos => "Trastornos disociativos",
            CategoriaDSM5::TrastornosSomaticosYRelacionados => "Trastornos somáticos y relacionados",
            CategoriaDSM5::TrastornosDeLaIngestaDeAlimentos => "Trastornos de la ingesta de alimentos",
            CategoriaDSM5::TrastornosDeEliminacion => "Trastornos de eliminación",
            CategoriaDSM5::TrastornosDelSueñoYVigilia => "Trastornos del sueño-vigilia",
            CategoriaDSM5::DisfuncionesSexuales => "Disfunciones sexuales",
            CategoriaDSM5::DisforiaDeGenero => "Disforia de género",
            CategoriaDSM5::TrastornosDisruptivosDelControlDeImpulsosYDeLaConducta => "Trastornos disruptivos, control de impulsos y conducta",
            CategoriaDSM5::TrastornosRelacionadosConSustanciasYAdictivos => "Trastornos relacionados con sustancias y adictivos",
            CategoriaDSM5::TrastornosNeurocognitivos => "Trastornos neurocognitivos",
            CategoriaDSM5::TrastornosDeLaPersonalidad => "Trastornos de la personalidad",
            CategoriaDSM5::TrastornosParafiliicos => "Trastornos parafílicos",
            CategoriaDSM5::OtrosTrastornosMentales => "Otros trastornos mentales",
            CategoriaDSM5::TrastornosRelacionadosConProblemasDeSalud => "Trastornos relacionados con problemas de salud",
        };
        write!(f, "{}", s)
    }
}

/// Mapping between CIE-10 and DSM-5 for common mental health diagnoses
/// This is a simplified mapping - in practice would be more comprehensive
pub struct MapeoCIE10DSM5;

impl MapeoCIE10DSM5 {
    /// Get DSM-5 code(s) for a given CIE-10 code
    pub fn cie10_a_dsm5(cie10_code: &str) -> Vec<&'static str> {
        let code = cie10_code.to_uppercase().replace(".", "");
        MAP_CIE10_TO_DSM5
            .get(code.as_str())
            .cloned()
            .unwrap_or_default()
    }

    /// Get CIE-10 code(s) for a given DSM-5 code
    pub fn dsm5_a_cie10(dsm5_code: &str) -> Vec<&'static str> {
        let code = dsm5_code.to_uppercase();
        MAP_DSM5_TO_CIE10
            .get(code.as_str())
            .cloned()
            .unwrap_or_default()
    }

    /// Get all common mappings for reference
    pub fn all_mappings() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        vec![
            ("F32.0", "Episodio depresivo leve", "296.21", "Trastorno depresivo mayor, episodio único, leve"),
            ("F32.1", "Episodio depresivo moderado", "296.22", "Trastorno depresivo mayor, episodio único, moderado"),
            ("F32.2", "Episodio depresivo grave sin síntomas psicóticos", "296.23", "Trastorno depresivo mayor, episodio único, grave"),
            ("F32.3", "Episodio depresivo grave con síntomas psicóticos", "296.24", "Trastorno depresivo mayor, episodio único, grave con características psicóticas"),
            ("F33.0", "Trastorno depresivo recurrente, episodio actual leve", "296.31", "Trastorno depresivo mayor, recurrente, leve"),
            ("F33.1", "Trastorno depresivo recurrente, episodio actual moderado", "296.32", "Trastorno depresivo mayor, recurrente, moderado"),
            ("F41.0", "Trastorno de pánico [ansiedad paroxística]", "300.01", "Trastorno de pánico"),
            ("F41.1", "Trastorno de ansiedad generalizada", "300.02", "Trastorno de ansiedad generalizada"),
            ("F42.0", "Trastorno obsesivo-compulsivo predominantemente de pensamientos obsesivos", "300.3", "Trastorno obsesivo-compulsivo"),
            ("F42.1", "Trastorno obsesivo-compulsivo predominantemente de actos compulsivos", "300.3", "Trastorno obsesivo-compulsivo"),
            ("F43.0", "Reacción al estrés agudo", "308.3", "Trastorno de estrés agudo"),
            ("F43.1", "Trastorno de estrés postraumático", "309.81", "Trastorno de estrés postraumático"),
            ("F43.2", "Trastornos de adaptación", "309.28", "Trastorno de adaptación"),
            ("F60.0", "Trastorno paranoide de la personalidad", "301.0", "Trastorno paranoide de la personalidad"),
            ("F60.1", "Trastorno esquizoide de la personalidad", "301.20", "Trastorno esquizoide de la personalidad"),
            ("F60.2", "Trastorno esquizotípico", "301.22", "Trastorno esquizotípico (trastorno de la personalidad)"),
            ("F60.3", "Trastorno límite de la personalidad", "301.83", "Trastorno límite de la personalidad"),
            ("F60.5", "Trastorno histriónico de la personalidad", "301.50", "Trastorno histriónico de la personalidad"),
            ("F60.6", "Trastorno narcisista de la personalidad", "301.81", "Trastorno narcisista de la personalidad"),
            ("F84.0", "Trastorno autista", "299.00", "Trastorno del espectro autista"),
            ("F84.5", "Síndrome de Asperger", "299.80", "Trastorno del espectro autista"),
            ("F90.0", "Trastorno por déficit de atención con hiperactividad", "314.01", "Trastorno por déficit de atención/hiperactividad"),
            ("F90.1", "Trastorno hipercinético de la conducta", "314.01", "Trastorno por déficit de atención/hiperactividad"),
        ]
    }
}

// Static mappings - using phf would be better for production but this works for now
lazy_static::lazy_static! {
    static ref MAP_CIE10_TO_DSM5: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("F320", vec!["296.21"]);
        m.insert("F321", vec!["296.22"]);
        m.insert("F322", vec!["296.23"]);
        m.insert("F323", vec!["296.24"]);
        m.insert("F330", vec!["296.31"]);
        m.insert("F331", vec!["296.32"]);
        m.insert("F410", vec!["300.01"]);
        m.insert("F411", vec!["300.02"]);
        m.insert("F420", vec!["300.3"]);
        m.insert("F421", vec!["300.3"]);
        m.insert("F430", vec!["308.3"]);
        m.insert("F431", vec!["309.81"]);
        m.insert("F432", vec!["309.28"]);
        m.insert("F600", vec!["301.0"]);
        m.insert("F601", vec!["301.20"]);
        m.insert("F602", vec!["301.22"]);
        m.insert("F603", vec!["301.83"]);
        m.insert("F605", vec!["301.50"]);
        m.insert("F606", vec!["301.81"]);
        m.insert("F840", vec!["299.00"]);
        m.insert("F845", vec!["299.80"]);
        m.insert("F900", vec!["314.01"]);
        m.insert("F901", vec!["314.01"]);
        m
    };

    static ref MAP_DSM5_TO_CIE10: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("296.21", vec!["F32.0"]);
        m.insert("296.22", vec!["F32.1"]);
        m.insert("296.23", vec!["F32.2"]);
        m.insert("296.24", vec!["F32.3"]);
        m.insert("296.31", vec!["F33.0"]);
        m.insert("296.32", vec!["F33.1"]);
        m.insert("300.01", vec!["F41.0"]);
        m.insert("300.02", vec!["F41.1"]);
        m.insert("300.3", vec!["F42.0", "F42.1"]);
        m.insert("308.3", vec!["F43.0"]);
        m.insert("309.81", vec!["F43.1"]);
        m.insert("309.28", vec!["F43.2"]);
        m.insert("301.0", vec!["F60.0"]);
        m.insert("301.20", vec!["F60.1"]);
        m.insert("301.22", vec!["F60.2"]);
        m.insert("301.83", vec!["F60.3"]);
        m.insert("301.50", vec!["F60.5"]);
        m.insert("301.81", vec!["F60.6"]);
        m.insert("299.00", vec!["F84.0"]);
        m.insert("299.80", vec!["F84.5"]);
        m.insert("314.01", vec!["F90.0", "F90.1"]);
        m
    };
}

/// Unified diagnostic that can hold either CIE-10 or DSM-5 (or both)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticoClinico {
    pub cie10: Option<DiagnosticoCIE10>,
    pub dsm5: Option<DiagnosticoDSM5>,
    pub es_principal: bool,
    pub fecha_diagnostico: chrono::NaiveDate,
    pub observaciones: Option<String>,
}

impl DiagnosticoClinico {
    pub fn new_cie10(cie10: DiagnosticoCIE10, es_principal: bool) -> Self {
        Self {
            cie10: Some(cie10),
            dsm5: None,
            es_principal,
            fecha_diagnostico: chrono::Utc::now().date_naive(),
            observaciones: None,
        }
    }

    pub fn new_dsm5(dsm5: DiagnosticoDSM5, es_principal: bool) -> Self {
        Self {
            cie10: None,
            dsm5: Some(dsm5),
            es_principal,
            fecha_diagnostico: chrono::Utc::now().date_naive(),
            observaciones: None,
        }
    }

    pub fn new_both(cie10: DiagnosticoCIE10, dsm5: DiagnosticoDSM5, es_principal: bool) -> Self {
        Self {
            cie10: Some(cie10),
            dsm5: Some(dsm5),
            es_principal,
            fecha_diagnostico: chrono::Utc::now().date_naive(),
            observaciones: None,
        }
    }

    pub fn with_observaciones(mut self, obs: String) -> Self {
        self.observaciones = Some(obs);
        self
    }

    pub fn with_fecha(mut self, fecha: chrono::NaiveDate) -> Self {
        self.fecha_diagnostico = fecha;
        self
    }

    /// Try to auto-map CIE-10 to DSM-5 if only CIE-10 is present
    pub fn auto_map_cie10_to_dsm5(&mut self) {
        if let Some(cie10) = &self.cie10 {
            if self.dsm5.is_none() {
                let dsm5_codes = MapeoCIE10DSM5::cie10_a_dsm5(&cie10.codigo);
                if let Some(first_code) = dsm5_codes.first() {
                    // Create a basic DSM-5 entry with just the code mapping
                    self.dsm5 = Some(DiagnosticoDSM5::new(
                        first_code.to_string(),
                        format!("Mapeado desde CIE-10: {}", cie10.descripcion),
                        CategoriaDSM5::TrastornosDepresivos, // Default, would need better logic
                        None,
                        None,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cie10_creation() {
        let diag = DiagnosticoCIE10::new(
            "F32.0".to_string(),
            "Episodio depresivo leve".to_string(),
            CategoriaCIE10::TrastornosMentales,
            Some("Episodio depresivo leve".to_string()),
        );
        
        assert_eq!(diag.codigo, "F32.0");
        assert_eq!(diag.descripcion, "Episodio depresivo leve");
        assert_eq!(diag.categoria, CategoriaCIE10::TrastornosMentales);
        assert_eq!(diag.codigo_normalizado(), "F320");
    }

    #[test]
    fn test_dsm5_creation() {
        let diag = DiagnosticoDSM5::new(
            "296.21".to_string(),
            "Major depressive disorder, single episode, mild".to_string(),
            CategoriaDSM5::TrastornosDepresivos,
            Some(vec!["Depressed mood".to_string(), "Anhedonia".to_string()]),
            Some(vec!["Mild".to_string()]),
        );
        
        assert_eq!(diag.codigo, "296.21");
        assert_eq!(diag.categoria, CategoriaDSM5::TrastornosDepresivos);
        assert_eq!(diag.criterios_diagnosticos.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_cie10_to_dsm5_mapping() {
        let mappings = MapeoCIE10DSM5::cie10_a_dsm5("F32.0");
        assert_eq!(mappings, vec!["296.21"]);
        
        let mappings = MapeoCIE10DSM5::cie10_a_dsm5("F41.1");
        assert_eq!(mappings, vec!["300.02"]);
        
        let mappings = MapeoCIE10DSM5::cie10_a_dsm5("F99.9"); // Non-existent
        assert!(mappings.is_empty());
    }

    #[test]
    fn test_dsm5_to_cie10_mapping() {
        let mappings = MapeoCIE10DSM5::dsm5_a_cie10("296.21");
        assert_eq!(mappings, vec!["F32.0"]);
        
        let mappings = MapeoCIE10DSM5::dsm5_a_cie10("300.3");
        assert_eq!(mappings, vec!["F42.0", "F42.1"]);
    }

    #[test]
    fn test_all_mappings() {
        let all = MapeoCIE10DSM5::all_mappings();
        assert!(!all.is_empty());
        assert!(all.len() >= 15);
        
        // Check first mapping
        let (cie10_code, _cie10_desc, dsm5_code, _dsm5_desc) = all[0];
        assert_eq!(cie10_code, "F32.0");
        assert_eq!(dsm5_code, "296.21");
    }

    #[test]
    fn test_diagnostico_clinico_new_cie10() {
        let cie10 = DiagnosticoCIE10::new(
            "F32.1".to_string(),
            "Episodio depresivo moderado".to_string(),
            CategoriaCIE10::TrastornosMentales,
            None,
        );
        
        let clinico = DiagnosticoClinico::new_cie10(cie10, true);
        
        assert!(clinico.cie10.is_some());
        assert!(clinico.dsm5.is_none());
        assert!(clinico.es_principal);
    }

    #[test]
    fn test_diagnostico_clinico_new_both() {
        let cie10 = DiagnosticoCIE10::new(
            "F41.1".to_string(),
            "Trastorno de ansiedad generalizada".to_string(),
            CategoriaCIE10::TrastornosMentales,
            None,
        );
        
        let dsm5 = DiagnosticoDSM5::new(
            "300.02".to_string(),
            "Generalized anxiety disorder".to_string(),
            CategoriaDSM5::TrastornosDeAnsiedad,
            None,
            None,
        );
        
        let clinico = DiagnosticoClinico::new_both(cie10, dsm5, true);
        
        assert!(clinico.cie10.is_some());
        assert!(clinico.dsm5.is_some());
        assert_eq!(clinico.cie10.as_ref().unwrap().codigo, "F41.1");
        assert_eq!(clinico.dsm5.as_ref().unwrap().codigo, "300.02");
    }

    #[test]
    fn test_auto_map_cie10_to_dsm5() {
        let cie10 = DiagnosticoCIE10::new(
            "F32.0".to_string(),
            "Episodio depresivo leve".to_string(),
            CategoriaCIE10::TrastornosMentales,
            None,
        );
        
        let mut clinico = DiagnosticoClinico::new_cie10(cie10, true);
        assert!(clinico.dsm5.is_none());
        
        clinico.auto_map_cie10_to_dsm5();
        assert!(clinico.dsm5.is_some());
        assert_eq!(clinico.dsm5.as_ref().unwrap().codigo, "296.21");
    }

    #[test]
    fn test_categoria_cie10_display() {
        assert_eq!(format!("{}", CategoriaCIE10::TrastornosMentales), "V - Trastornos mentales y del comportamiento");
        assert_eq!(format!("{}", CategoriaCIE10::SistemaCirculatorio), "IX - Sistema circulatorio");
    }

    #[test]
    fn test_categoria_dsm5_display() {
        assert_eq!(format!("{}", CategoriaDSM5::TrastornosDepresivos), "Trastornos depresivos");
        assert_eq!(format!("{}", CategoriaDSM5::TrastornosDeAnsiedad), "Trastornos de ansiedad");
    }

    #[test]
    fn test_serialization_cie10() {
        let diag = DiagnosticoCIE10::new(
            "F32.0".to_string(),
            "Episodio depresivo leve".to_string(),
            CategoriaCIE10::TrastornosMentales,
            None,
        );
        
        let json = serde_json::to_string(&diag).unwrap();
        let deserialized: DiagnosticoCIE10 = serde_json::from_str(&json).unwrap();
        assert_eq!(diag, deserialized);
    }

    #[test]
    fn test_serialization_dsm5() {
        let diag = DiagnosticoDSM5::new(
            "296.21".to_string(),
            "Major depressive disorder".to_string(),
            CategoriaDSM5::TrastornosDepresivos,
            None,
            None,
        );
        
        let json = serde_json::to_string(&diag).unwrap();
        let deserialized: DiagnosticoDSM5 = serde_json::from_str(&json).unwrap();
        assert_eq!(diag, deserialized);
    }

    #[test]
    fn test_serialization_diagnostico_clinico() {
        let cie10 = DiagnosticoCIE10::new(
            "F32.0".to_string(),
            "Episodio depresivo leve".to_string(),
            CategoriaCIE10::TrastornosMentales,
            None,
        );
        
        let clinico = DiagnosticoClinico::new_cie10(cie10, true)
            .with_observaciones("Paciente en seguimiento".to_string());
        
        let json = serde_json::to_string(&clinico).unwrap();
        let deserialized: DiagnosticoClinico = serde_json::from_str(&json).unwrap();
        assert_eq!(clinico, deserialized);
    }
}
