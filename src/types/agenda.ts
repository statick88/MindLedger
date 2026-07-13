/**
 * Agenda TypeScript contracts — mirrors Rust DTOs from src-tauri/commands/src/agenda_commands.rs
 *
 * Serde rename_all conventions:
 *   - Modality / AppointmentStatus → PascalCase
 *   - ReminderChannel → lowercase
 *   - ReminderTemplate → snake_case
 */

// ─── Enums ──────────────────────────────────────────────────────────────────

export type Modality = 'Presencial' | 'Virtual' | 'Hibrida';

export type AppointmentStatus =
  | 'Programada'
  | 'Realizada'
  | 'Reagendada'
  | 'Cancelada';

export type ReminderChannel = 'push' | 'email' | 'sms' | 'in_app';

export type ReminderTemplate =
  | 'session_30_min'
  | 'session_1_hour'
  | 'session_1_day'
  | 'custom';

// ─── Value objects ──────────────────────────────────────────────────────────

export interface DateTimeRange {
  start: string; // ISO 8601 DateTime
  end: string;   // ISO 8601 DateTime
}

// ─── Appointment ────────────────────────────────────────────────────────────

export interface AppointmentResponse {
  id: string;                          // UUID
  patient_id: string;                  // UUID
  therapist_id: string;                // UUID
  modality: Modality;
  start_at: string;                    // ISO 8601 DateTime
  end_at: string;                      // ISO 8601 DateTime
  fee_cents: number;                   // i64
  status: AppointmentStatus;
  reagendada_from_id: string | null;   // UUID | null
  notes: string | null;
  reminder_minutes_before: number | null;
  created_at: string;                  // ISO 8601 DateTime
  updated_at: string;                  // ISO 8601 DateTime
}

// ─── Reminder ───────────────────────────────────────────────────────────────

export interface ReminderResponse {
  id: string;                          // UUID
  appointment_id: string;              // UUID
  patient_id: string;                  // UUID
  remind_at: string;                   // ISO 8601 DateTime
  channel: ReminderChannel;
  template_id: ReminderTemplate;
  sent_at: string | null;              // ISO 8601 DateTime | null
  external_id: string | null;
  created_at: string;                  // ISO 8601 DateTime
  updated_at: string;                  // ISO 8601 DateTime
}

// ─── Agenda metrics ─────────────────────────────────────────────────────────

export interface AgendaMetrics {
  total_citas: number;
  programadas: number;
  realizadas: number;
  canceladas: number;
  reagendadas: number;
  utilization_rate: number;            // f64 0.0–1.0
  revenue_cents: number;
  average_session_minutes: number;
}

// ─── Request payloads ──────────────────────────────────────────────────────

export interface CreateAppointmentRequest {
  patient_id: string;
  therapist_id: string;
  modality: Modality;
  start_at: string;
  end_at: string;
  fee_cents: number;
  notes?: string;
  reminder_minutes_before?: number;
}

export interface UpdateAppointmentRequest {
  modality?: Modality;
  start_at?: string;
  end_at?: string;
  fee_cents?: number;
  notes?: string;
  reminder_minutes_before?: number;
}

// ─── Calendar helpers ──────────────────────────────────────────────────────

/** Status → badge color mapping for UI */
export const STATUS_COLORS: Record<AppointmentStatus, string> = {
  Programada: 'bg-blue-100 text-blue-800 border-blue-200',
  Realizada: 'bg-green-100 text-green-800 border-green-200',
  Reagendada: 'bg-yellow-100 text-yellow-800 border-yellow-200',
  Cancelada: 'bg-red-100 text-red-800 border-red-200',
};

/** Modality → display label */
export const MODALITY_LABELS: Record<Modality, string> = {
  Presencial: 'Presencial',
  Virtual: 'Virtual',
  Hibrida: 'Híbrida',
};

/** Status → display label */
export const STATUS_LABELS: Record<AppointmentStatus, string> = {
  Programada: 'Programada',
  Realizada: 'Realizada',
  Reagendada: 'Reagendada',
  Cancelada: 'Cancelada',
};

/** Fee cents → formatted currency string */
export function formatFeeCents(cents: number): string {
  return `$${(cents / 100).toLocaleString('es-EC', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}
