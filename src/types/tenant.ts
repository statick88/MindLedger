/**
 * TypeScript types mirroring Rust TenantConfig from design.md §3.1
 * These must stay in sync with src-tauri/commands/src/tenant.rs
 */

export interface TenantConfig {
  tenant: TenantInfo;
  brand: BrandTokens;
  brandDark: BrandTokens;
  typography: TypographyConfig;
  crypto: CryptoConfig;
  features: FeatureFlags;
}

export interface TenantInfo {
  id: string;
  commercialName: string;
  clinicalRole: string;
  ownerName: string;
  ownerTitle: string;
}

export interface BrandTokens {
  primary: string;
  primaryForeground: string;
  secondary: string;
  secondaryForeground: string;
  accent: string;
  accentForeground: string;
  background: string;
  foreground: string;
  muted: string;
  mutedForeground: string;
  card: string;
  cardForeground: string;
  border: string;
  input: string;
  ring: string;
  destructive: string;
  destructiveForeground: string;
  [key: string]: string;
}

export interface TypographyConfig {
  fontFamily: string;
  headingWeight: string;
  bodyWeight: string;
}

export interface CryptoConfig {
  keyringService: string;
  keyringAccount: string;
  dbFileName: string;
}

export interface FeatureFlags {
  clinicalNotes: boolean;
  accounting: boolean;
  agenda: boolean;
  diagnostics: boolean;
}