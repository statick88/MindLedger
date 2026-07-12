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

export const settingsApi = {
  get: () => invoke<Settings>('get_settings'),
  update: (request: Partial<Settings>) => invoke<Settings>('update_settings', { request }),
};