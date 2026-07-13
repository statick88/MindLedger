/**
 * Translate Rust/Tauri error messages to friendly Spanish UI text.
 * Maps common error patterns from the backend to human-readable messages.
 */

const ERROR_MAP: Record<string, string> = {
  // Database errors
  'database is locked': 'La base de datos está ocupada. Intenta de nuevo en unos segundos.',
  'database is already open': 'La base de datos ya está en uso.',
  'SQLITE_CONSTRAINT': 'Ya existe un registro con esos datos.',
  'SQLITE_CONSTRAINT_UNIQUE': 'Ya existe un registro con esos datos.',
  'SQLITE_CONSTRAINT_NOTNULL': 'Faltan datos obligatorios.',
  'SQLITE_CONSTRAINT_FOREIGN_KEY': 'El registro está referenciado por otro dato.',
  'SQLITE_BUSY': 'La base de datos está ocupada. Intenta de nuevo.',

  // Patient errors
  'patient not found': 'No se encontró el paciente.',
  'patient already exists': 'Ya existe un paciente con esos datos.',
  'invalid document number': 'El número de documento no es válido.',
  'invalid date of birth': 'La fecha de nacimiento no es válida.',
  'invalid email': 'El correo electrónico no es válido.',
  'minor requires guardian': 'El paciente menor de edad requiere un responsable.',

  // Appointment errors
  'appointment not found': 'No se encontró la cita.',
  'appointment conflict': 'Ya existe una cita en ese horario.',
  'invalid appointment time': 'La hora de la cita no es válida.',
  'appointment in the past': 'No se puede agendar una cita en el pasado.',
  'room already occupied': 'La sala ya está ocupada en ese horario.',
  'patient not active': 'El paciente no está activo.',

  // Accounting errors
  'accounting entry not found': 'No se encontró el asiento contable.',
  'balance mismatch': 'El balance no cuadra. Verifica los montos.',
  'invalid amount': 'El monto no es válido.',
  'account not found': 'No se encontró la cuenta contable.',

  // Clinical notes errors
  'clinical note not found': 'No se encontró la nota clínica.',
  'note already signed': 'La nota clínica ya está firmada.',
  'note requires signature': 'La nota clínica requiere firma.',
  'invalid session type': 'El tipo de sesión no es válido.',

  // Settings errors
  'settings not found': 'No se encontró la configuración.',
  'invalid timezone': 'La zona horaria no es válida.',

  // Generic errors
  'not found': 'El recurso solicitado no se encontró.',
  'permission denied': 'No tienes permiso para realizar esta acción.',
  'invalid input': 'Los datos ingresados no son válidos.',
  'unknown': 'Ocurrió un error inesperado.',
};

const PATTERN_MAP: Array<{ pattern: RegExp; message: string }> = [
  { pattern: /Unique constraint.*violation/i, message: 'Ya existe un registro con esos datos.' },
  { pattern: /NOT NULL constraint/i, message: 'Faltan campos obligatorios.' },
  { pattern: /FOREIGN KEY constraint/i, message: 'El registro está referenciado por otro dato.' },
  { pattern: /CHECK constraint/i, message: 'Los datos no cumplen con las validaciones.' },
  { pattern: /no such table/i, message: 'Error interno de base de datos.' },
  { pattern: /database is locked/i, message: 'La base de datos está ocupada. Intenta de nuevo.' },
  { pattern: /invalid.*uuid/i, message: 'Identificador no válido.' },
  { pattern: /invalid.*date/i, message: 'Fecha no válida.' },
  { pattern: /invalid.*email/i, message: 'Correo electrónico no válido.' },
];

export function translateError(error: unknown): string {
  const raw = extractErrorMessage(error);

  // Check exact matches
  if (ERROR_MAP[raw]) {
    return ERROR_MAP[raw];
  }

  // Check pattern matches
  for (const { pattern, message } of PATTERN_MAP) {
    if (pattern.test(raw)) {
      return message;
    }
  }

  // Fallback: return raw message if short enough, otherwise generic
  if (raw.length > 0 && raw.length < 120) {
    return raw;
  }

  return 'Ocurrió un error inesperado. Por favor, intenta de nuevo.';
}

function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as { message: unknown }).message);
  }
  return 'unknown';
}
