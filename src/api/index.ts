import { invoke } from '@tauri-apps/api/core';
import type {
  Patient,
  PaginatedResponse,
  ListPatientsQuery,
  CreatePatientRequest,
  Appointment,
  ClinicalNote,
  Diagnosis,
  VitalSigns,
  Settings,
} from '@/types';
import type {
  AppointmentResponse,
  ReminderResponse,
  AgendaMetrics,
  CreateAppointmentRequest,
  UpdateAppointmentRequest,
} from '@/types/agenda';

// ─── Patient API ────────────────────────────────────────────────────────────

export const patientApi = {
  create: (request: CreatePatientRequest) =>
    invoke<Patient>('create_patient', { request }),

  get: (id: string) =>
    invoke<Patient>('get_patient', { id }),

  list: (query: ListPatientsQuery) =>
    invoke<PaginatedResponse<Patient>>('list_patients', { query }),

  update: (id: string, request: Partial<CreatePatientRequest>) =>
    invoke<Patient>('update_patient', { id, request }),

  delete: (id: string) =>
    invoke<boolean>('delete_patient', { id }),

  search: (query: string, page: number, pageSize: number) =>
    invoke<PaginatedResponse<Patient>>('search_patients', { query, page, pageSize }),

  count: (activeOnly?: boolean) =>
    invoke<number>('get_patient_count', { activeOnly }),
};

// ─── Appointment API ────────────────────────────────────────────────────────

export const appointmentApi = {
  create: (request: {
    patient_id: string;
    appointment_type: Appointment['appointment_type'];
    scheduled_date: string;
    scheduled_time: string;
    duration_minutes: number;
    reason: string;
    doctor_name: string;
    room?: string;
  }) => invoke<Appointment>('create_appointment', { request }),

  get: (id: string) =>
    invoke<Appointment>('get_appointment', { id }),

  list: (query: {
    page?: number;
    page_size?: number;
    patient_id?: string;
    status?: Appointment['status'];
    appointment_type?: Appointment['appointment_type'];
    date_from?: string;
    date_to?: string;
    doctor_name?: string;
  }) => invoke<PaginatedResponse<Appointment>>('list_appointments', { query }),

  update: (id: string, request: Partial<Appointment>) =>
    invoke<Appointment>('update_appointment', { id, request }),

  delete: (id: string) =>
    invoke<boolean>('delete_appointment', { id }),

  forPatient: (patientId: string, page?: number, pageSize?: number) =>
    invoke<PaginatedResponse<Appointment>>('get_appointments_for_patient', { patientId, page, pageSize }),

  byDate: (date: string, page?: number, pageSize?: number) =>
    invoke<PaginatedResponse<Appointment>>('get_appointments_by_date', { date, page, pageSize }),

  conflicts: (request: {
    doctor_name: string;
    date: string;
    start_time: string;
    end_time: string;
    exclude_id?: string;
  }) => invoke<Appointment[]>('get_appointment_conflicts', { request }),
};

// ─── Clinical Notes API ─────────────────────────────────────────────────────

export const clinicalNoteApi = {
  create: (request: {
    patient_id: string;
    appointment_id?: string;
    note_type: ClinicalNote['note_type'];
    chief_complaint: string;
    history_of_present_illness: string;
    physical_examination: string;
    assessment: string;
    plan: string;
    diagnoses?: {
      description: string;
      diagnosis_type: Diagnosis['diagnosis_type'];
      icd10_code?: string;
      is_chronic?: boolean;
      onset_date?: string;
    }[];
    vital_signs?: Partial<VitalSigns>;
  }) => invoke<ClinicalNote>('create_clinical_note', { request }),

  get: (id: string) =>
    invoke<ClinicalNote>('get_clinical_note', { id }),

  list: (query: {
    page?: number;
    page_size?: number;
    patient_id?: string;
    note_type?: ClinicalNote['note_type'];
    is_signed?: boolean;
    date_from?: string;
    date_to?: string;
  }) => invoke<PaginatedResponse<ClinicalNote>>('list_clinical_notes', { query }),

  update: (id: string, request: {
    chief_complaint?: string;
    history_of_present_illness?: string;
    physical_examination?: string;
    assessment?: string;
    plan?: string;
  }) => invoke<ClinicalNote>('update_clinical_note', { id, request }),

  delete: (id: string) =>
    invoke<boolean>('delete_clinical_note', { id }),

  forPatient: (patientId: string, page?: number, pageSize?: number) =>
    invoke<PaginatedResponse<ClinicalNote>>('get_clinical_notes_for_patient', { patientId, page, pageSize }),

  byAppointment: (appointmentId: string) =>
    invoke<ClinicalNote | null>('get_clinical_note_by_appointment', { appointmentId }),

  unsigned: (page?: number, pageSize?: number) =>
    invoke<PaginatedResponse<ClinicalNote>>('get_unsigned_notes', { page, pageSize }),

  sign: (id: string, signedBy: string) =>
    invoke<ClinicalNote>('sign_clinical_note', { id, signedBy }),
};

// ─── Settings API ───────────────────────────────────────────────────────────

export const settingsApi = {
  get: () => invoke<Settings>('get_settings'),
  update: (request: Partial<Settings>) => invoke<Settings>('update_settings', { request }),
};

// ─── Age API ────────────────────────────────────────────────────────────────

export interface AgeBreakdown {
  years: number;
  months: number;
  days: number;
  total_days: number;
  total_months: number;
  is_minor: boolean;
  age_of_majority: number;
}

export const ageApi = {
  /** Calculate age in years from date_of_birth to today */
  calculate: (dateOfBirth: string) =>
    invoke<number>('calculate_age', { dateOfBirth }),

  /** Calculate age at a specific reference date */
  calculateAt: (dateOfBirth: string, referenceDate: string) =>
    invoke<number>('calculate_age_at', { dateOfBirth, referenceDate }),

  /** Full breakdown: years, months, days, is_minor, etc. */
  breakdown: (dateOfBirth: string, ageOfMajority?: number) =>
    invoke<AgeBreakdown>('calculate_age_breakdown', { dateOfBirth, ageOfMajority }),
};

// ─── Accounting API ─────────────────────────────────────────────────────────

export interface Asiento {
  id: string;
  fecha: string;
  descripcion: string;
  debe: number;
  haber: number;
  categoria: string;
  created_at: string;
}

export interface CreateAsientoRequest {
  fecha: string;
  descripcion: string;
  debe: number;
  haber: number;
  categoria: string;
}

export interface BalanceGeneral {
  activos: { cuenta: string; monto: number }[];
  pasivos: { cuenta: string; monto: number }[];
  patrimonio: { cuenta: string; monto: number }[];
  total_activos: number;
  total_pasivos: number;
  total_patrimonio: number;
}

export interface EstadoResultados {
  ingresos: { cuenta: string; monto: number }[];
  gastos: { cuenta: string; monto: number }[];
  total_ingresos: number;
  total_gastos: number;
  utilidad_neta: number;
}

export const accountingApi = {
  addAsiento: (request: CreateAsientoRequest) =>
    invoke<Asiento>('add_asiento', { request }),

  removeAsiento: (id: string) =>
    invoke<boolean>('remove_asiento', { id }),

  listAsientos: (query?: { page?: number; page_size?: number; categoria?: string; fecha_desde?: string; fecha_hasta?: string }) =>
    invoke<PaginatedResponse<Asiento>>('list_asientos', { query }),

  balanceGeneral: () =>
    invoke<BalanceGeneral>('generate_balance_general'),

  estadoResultados: () =>
    invoke<EstadoResultados>('generate_estado_resultados'),
};

// ─── Diagnostics API (CIE-10 / DSM-5) ──────────────────────────────────────

export interface Cie10Entry {
  id: string;
  codigo: string;
  descripcion: string;
  categoria: string;
}

export interface Dsm5Entry {
  id: string;
  codigo: string;
  descripcion: string;
  categoria: string;
}

export interface MapeoCieDsm {
  id: string;
  cie10_codigo: string;
  dsm5_codigo: string;
  created_at: string;
}

export const diagnosticsApi = {
  searchCie10: (query: string, limit?: number) =>
    invoke<Cie10Entry[]>('search_cie10', { query, limit }),

  searchDsm5: (query: string, limit?: number) =>
    invoke<Dsm5Entry[]>('search_dsm5', { query, limit }),

  getCie10ByCodigo: (codigo: string) =>
    invoke<Cie10Entry | null>('get_cie10_by_codigo', { codigo }),

  getDsm5ByCodigo: (codigo: string) =>
    invoke<Dsm5Entry | null>('get_dsm5_by_codigo', { codigo }),

  createMapeo: (cie10Codigo: string, dsm5Codigo: string) =>
    invoke<MapeoCieDsm>('create_mapeo', { cie10Codigo, dsm5Codigo }),

  listMapeos: () =>
    invoke<MapeoCieDsm[]>('list_mapeos'),

  updateMapeo: (id: string, cie10Codigo: string, dsm5Codigo: string) =>
    invoke<MapeoCieDsm>('update_mapeo', { id, cie10Codigo, dsm5Codigo }),

  deleteMapeo: (id: string) =>
    invoke<boolean>('delete_mapeo', { id }),
};

// ─── Agenda API (v1.0 — new domain model) ──────────────────────────────────

export const agendaApi = {
  /** Create a new appointment */
  crear: (request: CreateAppointmentRequest) =>
    invoke<AppointmentResponse>('crear_cita_agenda', { request }),

  /** Get a single appointment by ID */
  obtener: (id: string) =>
    invoke<AppointmentResponse>('obtener_cita', { id }),

  /** Update mutable fields of an appointment */
  actualizar: (id: string, request: UpdateAppointmentRequest) =>
    invoke<AppointmentResponse>('actualizar_cita', { id, request }),

  /** Cancel an appointment (state machine → Cancelada) */
  cancelar: (id: string) =>
    invoke<AppointmentResponse>('cancelar_cita', { id }),

  /** Finalize session — marks Realizada and auto-creates accounting entry */
  finalizarSesion: (id: string) =>
    invoke<AppointmentResponse>('finalizar_sesion_agenda', { id }),

  /** Reschedule — creates a new Appointment linked via reagendada_from_id */
  reagendar: (id: string, newStart: string, newEnd: string) =>
    invoke<AppointmentResponse>('reagendar_cita', { id, newStart, newEnd }),

  /** List all appointments with optional filters */
  listar: (query?: {
    patient_id?: string;
    therapist_id?: string;
    status?: string;
    date_from?: string;
    date_to?: string;
  }) =>
    invoke<AppointmentResponse[]>('listar_citas', { query: query ?? {} }),

  /** List appointments for a specific patient */
  citasPaciente: (patientId: string) =>
    invoke<AppointmentResponse[]>('citas_paciente', { patientId }),

  /** List reminders (optionally filtered by patient) */
  listarRecordatorios: (patientId?: string) =>
    invoke<ReminderResponse[]>('listar_recordatorios', {
      patientId: patientId ?? null,
    }),

  /** List appointments in a date range */
  citasRango: (from: string, to: string) =>
    invoke<AppointmentResponse[]>('listar_citas_rango', { from, to }),

  /** List appointments for a therapist */
  citasTerapeuta: (therapistId: string) =>
    invoke<AppointmentResponse[]>('listar_citas_terapeuta', { therapistId }),

  /** Get agenda metrics for a date range */
  metricas: (from?: string, to?: string) =>
    invoke<AgendaMetrics>('obtener_metricas', {
      from: from ?? null,
      to: to ?? null,
    }),

  /** Process pending reminders (batch job) */
  procesarRecordatorios: () =>
    invoke<number>('procesar_recordatorios'),
};
