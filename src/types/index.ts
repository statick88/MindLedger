export interface Patient {
  id: string;
  document_number: string;
  document_type: DocumentType;
  country_code: string;
  first_name: string;
  last_name: string;
  middle_name?: string;
  date_of_birth: string;
  gender: Gender;
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

export type DocumentType = 'NationalId' | 'Passport' | 'DriversLicense' | 'ForeignId' | 'BirthCertificate' | 'Other';
export type Gender = 'Male' | 'Female' | 'Other' | 'PreferNotToSay';

export interface CreatePatientRequest {
  document_number: string;
  document_type: DocumentType;
  country_code: string;
  first_name: string;
  last_name: string;
  middle_name?: string;
  date_of_birth: string;
  gender: Gender;
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

export interface UpdatePatientRequest {
  first_name?: string;
  last_name?: string;
  middle_name?: string;
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
  is_active?: boolean;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface PatientFilter {
  active_only?: boolean;
  gender?: Gender;
  name_contains?: string;
  min_age?: number;
  max_age?: number;
  has_allergy?: string;
  has_condition?: string;
}

export interface ListPatientsQuery {
  page?: number;
  page_size?: number;
  active_only?: boolean;
  gender?: Gender;
  name_contains?: string;
  min_age?: number;
  max_age?: number;
  has_allergy?: string;
  has_condition?: string;
}

export interface Appointment {
  id: string;
  patient_id: string;
  appointment_type: AppointmentType;
  status: AppointmentStatus;
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

export type AppointmentType = 'Consultation' | 'FollowUp' | 'Procedure' | 'Emergency' | 'Telemedicine' | 'CheckUp';
export type AppointmentStatus = 'Scheduled' | 'Confirmed' | 'InProgress' | 'Completed' | 'Cancelled' | 'NoShow';

export interface CreateAppointmentRequest {
  patient_id: string;
  appointment_type: AppointmentType;
  scheduled_date: string;
  scheduled_time: string;
  duration_minutes: number;
  reason: string;
  doctor_name: string;
  room?: string;
}

export interface AppointmentFilter {
  patient_id?: string;
  status?: AppointmentStatus;
  appointment_type?: AppointmentType;
  date_from?: string;
  date_to?: string;
  doctor_name?: string;
}

export interface ClinicalNote {
  id: string;
  patient_id: string;
  appointment_id?: string;
  note_type: NoteType;
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

export type NoteType = 'Consultation' | 'Progress' | 'Procedure' | 'Emergency' | 'Discharge' | 'FollowUp' | 'Telemedicine' | 'Preoperative' | 'Postoperative';

export interface Diagnosis {
  id: string;
  icd10_code?: string;
  description: string;
  diagnosis_type: DiagnosisType;
  is_chronic: boolean;
  onset_date?: string;
  resolved_date?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export type DiagnosisType = 'Primary' | 'Secondary' | 'Differential' | 'RuledOut' | 'Historical';

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

export interface CreateClinicalNoteRequest {
  patient_id: string;
  appointment_id?: string;
  note_type: NoteType;
  chief_complaint: string;
  history_of_present_illness: string;
  physical_examination: string;
  assessment: string;
  plan: string;
  diagnoses?: { description: string; diagnosis_type: DiagnosisType; icd10_code?: string; is_chronic?: boolean; onset_date?: string }[];
  vital_signs?: Omit<VitalSigns, 'recorded_at'>;
}

export interface ClinicalNoteFilter {
  patient_id?: string;
  note_type?: NoteType;
  is_signed?: boolean;
  date_from?: string;
  date_to?: string;
}

export interface AddDiagnosisRequest {
  description: string;
  diagnosis_type: DiagnosisType;
  icd10_code?: string;
  is_chronic?: boolean;
  onset_date?: string;
}

export interface SignNoteRequest {
  signed_by: string;
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

export interface PatientSearchResult {
  id: string;
  document_number: string;
  full_name: string;
  date_of_birth: string;
  age: number;
}