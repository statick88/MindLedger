use anyhow::{Context, Result};
use docx_rs::read_docx;
use std::fs;

#[derive(Debug, Clone)]
pub struct ClinicalNote {
    pub patient_id: Option<String>,
    pub session_date: Option<String>,
    pub diagnosis_code: Option<String>,
    pub session_type: Option<String>,
    pub notes: Option<String>,
    pub treatment_plan: Option<String>,
}

pub struct ClinicalNoteParser;

impl ClinicalNoteParser {
    pub fn parse_docx(file_path: &str) -> Result<ClinicalNote> {
        let bytes = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;
        
        let doc = read_docx(&bytes)
            .context("Failed to parse DOCX file")?;
        
        let paragraphs = Self::extract_paragraphs(&doc);
        let full_text = paragraphs.join("\n");
        
        let note = ClinicalNote {
            patient_id: Self::extract_field(&full_text, &["Paciente", "Patient ID", "ID"]),
            session_date: Self::extract_field(&full_text, &["Fecha", "Date", "Fecha de sesión"]),
            diagnosis_code: Self::extract_field(&full_text, &["Diagnóstico", "Diagnosis", "Código", "CIE"]),
            session_type: Self::extract_field(&full_text, &["Tipo de sesión", "Session Type", "Tipo"]),
            notes: Self::extract_field(&full_text, &["Notas", "Notes", "Observaciones", "Hallazgos"]),
            treatment_plan: Self::extract_field(&full_text, &["Plan de tratamiento", "Treatment Plan", "Plan"]),
        };
        
        Ok(note)
    }

    fn extract_paragraphs(doc: &docx_rs::Docx) -> Vec<String> {
        let mut paragraphs = Vec::new();
        for child in &doc.document.children {
            if let docx_rs::DocumentChild::Paragraph(para) = child {
                let text: String = para
                    .children
                    .iter()
                    .filter_map(|run| {
                        if let docx_rs::ParagraphChild::Run(run) = run {
                            Some(
                                run.children
                                    .iter()
                                    .filter_map(|child| {
                                        if let docx_rs::RunChild::Text(text) = child {
                                            Some(text.text.as_str())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<String>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect();
                if !text.trim().is_empty() {
                    paragraphs.push(text);
                }
            }
        }
        paragraphs
    }

    pub fn extract_field(text: &str, keys: &[&str]) -> Option<String> {
        for line in text.lines() {
            let line_lower = line.to_lowercase();
            for key in keys {
                let key_lower = key.to_lowercase();
                if line_lower.starts_with(&key_lower) {
                    if let Some(value) = line.split(':').nth(1) {
                        let value = value.trim().to_string();
                        if !value.is_empty() {
                            return Some(value);
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docx_rs::*;
    use tempfile::tempdir;

    fn create_test_docx(path: &std::path::Path, content: &str) {
        let mut doc = Docx::new();
        for line in content.lines() {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .add_run(Run::new().add_text(line.to_string())),
            );
        }
        let mut file = fs::File::create(path).unwrap();
        doc.build().pack(&mut file).unwrap();
    }

    #[test]
    fn test_parse_docx_basic() {
        let dir = tempdir().unwrap();
        let docx_path = dir.path().join("test_note.docx");
        
        let content = "Paciente: PT-001\nFecha: 2024-01-15\nDiagnóstico: F32.1\nTipo de sesión: Terapia cognitivo-conductual\nNotas: Paciente presenta mejoría en síntomas de ansiedad.\nPlan de tratamiento: Continuar con terapia semanal.";
        create_test_docx(&docx_path, content);
        
        let note = ClinicalNoteParser::parse_docx(docx_path.to_str().unwrap()).unwrap();
        
        assert_eq!(note.patient_id, Some("PT-001".to_string()));
        assert_eq!(note.session_date, Some("2024-01-15".to_string()));
        assert_eq!(note.diagnosis_code, Some("F32.1".to_string()));
        assert_eq!(note.session_type, Some("Terapia cognitivo-conductual".to_string()));
        assert!(note.notes.is_some());
        assert!(note.treatment_plan.is_some());
    }

    #[test]
    fn test_parse_docx_missing_fields() {
        let dir = tempdir().unwrap();
        let docx_path = dir.path().join("test_note_missing.docx");
        
        let content = "Paciente: PT-002\nSolo tiene paciente.";
        create_test_docx(&docx_path, content);
        
        let note = ClinicalNoteParser::parse_docx(docx_path.to_str().unwrap()).unwrap();
        
        assert_eq!(note.patient_id, Some("PT-002".to_string()));
        assert_eq!(note.session_date, None);
        assert_eq!(note.diagnosis_code, None);
    }

    #[test]
    fn test_parse_docx_nonexistent_file() {
        let result = ClinicalNoteParser::parse_docx("/nonexistent/path.docx");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_field_case_insensitive() {
        let text = "Paciente: TEST-123\nFecha: 2024-01-01";
        assert_eq!(
            ClinicalNoteParser::extract_field(text, &["paciente"]),
            Some("TEST-123".to_string())
        );
    }

    #[test]
    fn test_extract_field_no_match() {
        let text = "Some random text without the field";
        assert_eq!(
            ClinicalNoteParser::extract_field(text, &["Paciente", "Fecha"]),
            None
        );
    }
}
