/**
 * TypeScript types mirroring Rust TenantConfig from design.md §3.1
 * These must stay in sync with src-tauri/commands/src/tenant.rs
 */

export interface TenantConfig {
  tenant: TenantInfo;
  brand: BrandTokens;
  brand_dark: BrandTokens;
  typography: TypographyConfig;
  crypto: CryptoConfig;
  features: FeatureFlags;
}

export interface TenantInfo {
  id: string;
  commercial_name: string;
  clinical_role: string;
  owner_name: string;
  owner_title: string;
}

export interface BrandTokens {
  primary: string;
  primary_foreground: string;
  secondary: string;
  secondary_foreground: string;
  accent: string;
  accent_foreground: string;
  background: string;
  foreground: string;
  muted: string;
  muted_foreground: string;
  card: string;
  card_foreground: string;
  border: string;
  input: string;
  ring: string;
  destructive: string;
  destructive_foreground: string;
  [key: string]: string;
}

export interface TypographyConfig {
  font_family: string;
  heading_weight: string;
  body_weight: string;
}

export interface CryptoConfig {
  keyring_service: string;
  keyring_account: string;
  db_file_name: string;
}

export interface FeatureFlags {
  clinical_notes: boolean;
  accounting: boolean;
  agenda: boolean;
  diagnostics: boolean;
}