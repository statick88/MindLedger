import { invoke } from '@tauri-apps/api/core';

export interface Patient {
  id: string;
  document_number: string;
  document_type: string;
  country_code: string;
  first_name: string;
  last_name: string;
  middle_name?: string;
  date_of_birth: string;
  gender: string;
  email?: string;
  phone?: string;
  address?: string;
  emergency_contact?: string;
  blood_type?: string;
  allergies: string[];
  chronic_conditions: string[];
  medications: string[];
  notes?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface Appointment {
  id: string;
  patient_id: string;
  appointment_type: string;
  status: string;
  scheduled_date: string;
  scheduled_time: string;
  duration_minutes: number;
  reason: string;
  notes?: string;
  doctor_name: string;
  room?: string;
  created_at: string;
  updated_at: string;
  completed_at?: string;
  cancelled_at?: string;
  cancellation_reason?: string;
}

export interface ClinicalNote {
  id: string;
  patient_id: string;
  appointment_id?: string;
  note_type: string;
  chief_complaint: string;
  history_of_present_illness: string;
  physical_examination: string;
  assessment: string;
  plan: string;
  diagnoses: Diagnosis[];
  vital_signs?: VitalSigns;
  attachments: Attachment[];
  is_signed: boolean;
  signed_at?: string;
  signed_by?: string;
  created_at: string;
  updated_at: string;
}

export interface Diagnosis {
  id: string;
  icd10_code?: string;
  description: string;
  diagnosis_type: string;
  is_chronic: boolean;
  onset_date?: string;
  resolved_date?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface VitalSigns {
  temperature_celsius?: number;
  blood_pressure_systolic?: number;
  blood_pressure_diastolic?: number;
  heart_rate_bpm?: number;
  respiratory_rate_bpm?: number;
  oxygen_saturation?: number;
  weight_kg?: number;
  height_cm?: number;
  bmi?: number;
  pain_scale?: number;
  recorded_at: string;
}

export interface Attachment {
  id: string;
  file_name: string;
  file_type: string;
  file_size: number;
  storage_path: string;
  description?: string;
  uploaded_at: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface CreatePatientRequest {
  document_number: string;
  document_type: 'NationalId' | 'Passport' | 'DriversLicense' | 'ForeignId' | 'BirthCertificate' | 'Other';
  country_code: string;
  first_name: string;
  last_name: string;
  middle_name?: string;
  date_of_birth: string;
  gender: 'Male' | 'Female' | 'Other' | 'PreferNotToSay';
  email?: string;
  phone_number?: string;
  phone_country_code?: string;
  phone_extension?: string;
  address_street?: string;
  address_city?: string;
  address_state?: string;
  address_postal_code?: string;
  address_country?: string;
  address_additional_info?: string;
  emergency_contact_first_name?: string;
  emergency_contact_last_name?: string;
  emergency_contact_middle_name?: string;
  emergency_contact_relationship?: string;
  emergency_contact_phone_number?: string;
  emergency_contact_phone_country_code?: string;
  emergency_contact_email?: string;
  blood_type?: string;
  allergies?: string[];
  chronic_conditions?: string[];
  medications?: string[];
  notes?: string;
}

export interface ListPatientsQuery {
  page?: number;
  page_size?: number;
  active_only?: boolean;
  gender?: string;
  name_contains?: string;
  min_age?: number;
  max_age?: number;
  has_allergy?: string;
  has_condition?: string;
}

export interface CreateAppointmentRequest {
  patient_id: string;
  appointment_type: 'Consultation' | 'FollowUp' | 'Procedure' | 'Emergency' | 'Telemedicine' | 'CheckUp';
  scheduled_date: string;
  scheduled_time: string;
  duration_minutes: number;
  reason: string;
  doctor_name: string;
  room?: string;
}

export interface ListAppointmentsQuery {
  page?: number;
  page_size?: number;
  patient_id?: string;
  status?: string;
  appointment_type?: string;
  date_from?: string;
  date_to?: string;
  doctor_name?: string;
}

export interface CreateClinicalNoteRequest {
  patient_id: string;
  note_type: 'Consultation' | 'Progress' | 'Procedure' | 'Emergency' | 'Discharge' | 'FollowUp' | 'Telemedicine' | 'Preoperative' | 'Postoperative';
  chief_complaint: string;
  history_of_present_illness: string;
  physical_examination: string;
  assessment: string;
  plan: string;
  appointment_id?: string;
}

export interface Settings {
  clinic_name: string;
  clinic_address: string;
  clinic_phone: string;
  clinic_email: string;
  timezone: string;
  appointment_duration_default: number;
  age_of_majority: number;
  currency: string;
  language: string;
}

export interface UpdateSettingsRequest {
  clinic_name?: string;
  clinic_address?: string;
  clinic_phone?: string;
  clinic_email?: string;
  timezone?: string;
  appointment_duration_default?: number;
  age_of_majority?: number;
  currency?: string;
  language?: string;
}

// Patient API
export const patientApi = {
  create: (data: CreatePatientRequest) => invoke<Patient>('create_patient', { request: data }),
  get: (id: string) => invoke<Patient>('get_patient', { id }),
  list: (query: ListPatientsQuery) => invoke<PaginatedResponse<Patient>>('list_patients', { query }),
  update: (id: string, data: Partial<CreatePatientRequest>) => invoke<Patient>('update_patient', { id, request: data }),
  delete: (id: string) => invoke<boolean>('delete_patient', { id }),
  search: (query: string, page?: number, pageSize?: number) => 
    invoke<PaginatedResponse<Patient>>('search_patients', { query, page, pageSize }),
  count: (activeOnly?: boolean) => invoke<number>('get_patient_count', { activeOnly }),
};

// Appointment API
export const appointmentApi = {
  create: (data: CreateAppointmentRequest) => invoke<Appointment>('create_appointment', { request: data }),
  get: (id: string) => invoke<Appointment>('get_appointment', { id }),
  list: (query: ListAppointmentsQuery) => invoke<PaginatedResponse<Appointment>>('list_appointments', { query }),
  update: (id: string, data: Partial<CreateAppointmentRequest>) => invoke<Appointment>('update_appointment', { id, request: data }),
  delete: (id: string) => invoke<boolean>('delete_appointment', { id }),
  getByPatient: (patientId: string, page?: number, pageSize?: number) => 
    invoke<PaginatedResponse<Appointment>>('get_appointments_for_patient', { patientId, page, pageSize }),
  getByDate: (date: string, page?: number, pageSize?: number) => 
    invoke<PaginatedResponse<Appointment>>('get_appointments_by_date', { date, page, pageSize }),
  count: (filter?: Partial<ListAppointmentsQuery>) => invoke<number>('get_appointment_count', { filter }),
  getConflicts: (doctor: string, date: string, start: string, end: string, excludeId?: string) => 
    invoke<Appointment[]>('get_appointment_conflicts', { doctor, date, start, end, excludeId }),
};

// Clinical Notes API
export const clinicalNoteApi = {
  create: (data: CreateClinicalNoteRequest) => invoke<ClinicalNote>('create_clinical_note', { request: data }),
  get: (id: string) => invoke<ClinicalNote>('get_clinical_note', { id }),
  list: (filter: { patient_id?: string; note_type?: string; is_signed?: boolean; date_from?: string; date_to?: string }, page?: number, pageSize?: number) => 
    invoke<PaginatedResponse<ClinicalNote>>('list_clinical_notes', { filter, page, pageSize }),
  update: (id: string, data: Partial<CreateClinicalNoteRequest>) => invoke<ClinicalNote>('update_clinical_note', { id, request: data }),
  delete: (id: string) => invoke<boolean>('delete_clinical_note', { id }),
  getByPatient: (patientId: string, page?: number, pageSize?: number) => 
    invoke<PaginatedResponse<ClinicalNote>>('get_clinical_notes_for_patient', { patientId, page, pageSize }),
  getByAppointment: (appointmentId: string) => invoke<ClinicalNote | null>('get_clinical_note_by_appointment', { appointmentId }),
  getUnsigned: (page?: number, pageSize?: number) => 
    invoke<PaginatedResponse<ClinicalNote>>('get_unsigned_clinical_notes', { page, pageSize }),
  count: (filter?: { patient_id?: string; note_type?: string; is_signed?: boolean; date_from?: string; date_to?: string }) => 
    invoke<number>('get_clinical_note_count', { filter }),
  sign: (id: string, signedBy: string) => invoke<ClinicalNote>('sign_clinical_note', { id, signedBy }),
  addDiagnosis: (noteId: string, diagnosis: { description: string; diagnosis_type: string; icd10_code?: string; is_chronic?: boolean; onset_date?: string }) => 
    invoke<ClinicalNote>('add_diagnosis_to_note', { noteId, diagnosis }),
  removeDiagnosis: (noteId: string, diagnosisId: string) => invoke<ClinicalNote>('remove_diagnosis_from_note', { noteId, diagnosisId }),
};

// Settings API
export const settingsApi = {
  get: () => invoke<Settings>('get_settings'),
  update: (data: UpdateSettingsRequest) => invoke<Settings>('update_settings', { request: data }),
};

// Health check
export const healthCheck = () => invoke<string>('health_check');