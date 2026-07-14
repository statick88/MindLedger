# Technical Design: Tenant Branding Injection for Gloria Once

## Overview

Transform MindLedger from a single-tenant hardcoded app to a tenant-aware architecture where branding tokens, database isolation, and app identity are injected at build time via `TENANT_CONFIG` environment variable. Single binary = single tenant per build.

---

## 1. Data Flow Diagrams

### 1.1 Build-Time Flow (Rust)

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           BUILD TIME (cargo tauri build)                         │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  TENANT_CONFIG=tenant-configs/gloria-once.json                                  │
│           │                                                                      │
│           ▼                                                                      │
│  ┌──────────────────┐                                                           │
│  │   build.rs       │  1. Reads TENANT_CONFIG env var                           │
│  │                  │  2. Copies JSON to OUT_DIR/tenant-config.json             │
│  │                  │  3. Validates JSON via serde_json                         │
│  │                  │  4. Emits: cargo:rustc-env=TENANT_CONFIG_PATH=...         │
│  └────────┬─────────┘                                                           │
│           │                                                                      │
│           ▼                                                                      │
│  ┌──────────────────┐                                                           │
│  │   tauri.conf.json│  5. Template substitution via build.rs:                   │
│  │   (templated)    │     - identifier: com.mindledger.gloriaonce.desktop       │
│  │                  │     - productName: MindLedger - Psic. Gloria Once         │
│  │                  │     - window.title: MindLedger - Psic. Gloria Once        │
│  └────────┬─────────┘                                                           │
│           │                                                                      │
│           ▼                                                                      │
│  ┌──────────────────┐                                                           │
│  │  Compiled Binary │  6. tenant-config.json embedded via include_str!()        │
│  │                  │     in commands/src/tenant.rs                             │
│  └──────────────────┘                                                           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Runtime Flow (Rust → Frontend)

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              RUNTIME (App Launch)                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌──────────────┐     ┌──────────────────┐     ┌─────────────────────────┐    │
│  │  main.rs     │────▶│  database.rs     │     │  SqlCipherKeyManager    │    │
│  │              │     │  create_pool()   │────▶│  .new_with_fallback()   │    │
│  │ - Gets       │     │                  │     │  - service: mind-ledger │    │
│  │  data_dir    │     │ - Uses tenant    │     │  - account: sqlcipher-  │    │
│  │ - Derives    │     │   specific paths │     │    key-gloria-once      │    │
│  │  tenant DB   │     │ - Uses tenant    │     │  - db: mind_ledger_     │    │
│  │  path        │     │   db filename    │     │    gloria_once.db       │    │
│  └──────────────┘     └──────────────────┘     └─────────────────────────┘    │
│         │                                                                    │
│         ▼                                                                    │
│  ┌──────────────────┐                                                        │
│  │  Tauri Commands  │                                                        │
│  │  get_tenant_config() ──────────────────────────────────────┐             │
│  └────────┬─────────┘                                        │             │
│           │                                                  │             │
│           ▼                                                  ▼             │
│  ┌──────────────────┐     ┌──────────────────────────────────────────┐   │
│  │  Frontend Mount  │     │  useTenantConfig() hook (TanStack Query) │   │
│  │  (App.tsx)       │────▶│  - staleTime: Infinity                   │   │
│  └──────────────────┘     │  - Returns TenantConfig                  │   │
│                            └──────────────┬───────────────────────────┘   │
│                                           │                                 │
│                    ┌──────────────────────┼──────────────────────┐         │
│                    ▼                      ▼                      ▼         │
│         ┌─────────────────┐   ┌─────────────────┐    ┌─────────────────┐   │
│         │ Layout.tsx      │   │ index.css       │    │ Feature Flags   │   │
│         │ - sidebar brand │   │ - CSS vars      │    │ - clinicalNotes │   │
│         │ - window title  │   │   injected at   │    │ - accounting    │   │
│         └─────────────────┘   │   runtime       │    │ - agenda        │   │
│                               │ - Tailwind uses │    │ - diagnostics   │   │
│                               │   CSS vars      │    └─────────────────┘   │
│                               └─────────────────┘                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Database Isolation Flow

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          DATABASE ISOLATION PER TENANT                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  Tenant Config (gloria-once.json)                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │ "crypto": {                                                             │   │
│  │   "keyringService": "mind-ledger",        ◄── Shared service name       │   │
│  │   "keyringAccount": "sqlcipher-key-gloria-once",  ◄── Unique per tenant │   │
│  │   "dbFileName": "mind_ledger_gloria_once.db"       ◄── Unique per tenant│   │
│  │ }                                                                       │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│           │                                                                      │
│           ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │ main.rs → create_pool()                                                 │   │
│  │   data_dir = $APPDATA/mind-ledger-gloria-once/   ◄── Derived from       │   │
│  │   db_path = data_dir/mind_ledger_gloria_once.db   ◄── tenant id         │   │
│  │   keyring_account = "sqlcipher-key-gloria-once"   ◄── From config       │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│           │                                                                      │
│           ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │ Keyring Lookup                                                          │   │
│  │   Service: "mind-ledger"                                                │   │
│  │   Account: "sqlcipher-key-gloria-once"                                  │   │
│  │   → Returns 64-char hex key OR generates new one                        │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│           │                                                                      │
│           ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │ SQLCipher Database                                                      │   │
│  │   PRAGMA key = '<key-from-keyring>';                                    │   │
│  │   File: $APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db     │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Module Interaction Design

### 2.1 Rust Module Dependencies

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              RUST MODULE GRAPH                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  build.rs                                                                       │
│       │                                                                         │
│       ├─▶ Copies tenant-configs/gloria-once.json → OUT_DIR/tenant-config.json  │
│       │                                                                         │
│       ├─▶ Emits cargo:rustc-env=TENANT_CONFIG_PATH                              │
│       │                                                                         │
│       └─▶ Templates tauri.conf.json (identifier, productName, window.title)    │
│                                                                                 │
│  soft_gloria_commands                                                           │
│  ├── tenant.rs ◀────────────────── NEW MODULE                                   │
│  │   ├── TenantConfig, BrandTokens, TypographyConfig, CryptoConfig,            │
│  │   │   FeatureFlags (serializable structs)                                    │
│  │   ├── get_tenant_config() → Tauri command                                   │
│  │   ├── get_tenant_keyring_account() → helper                                 │
│  │   ├── get_tenant_db_filename() → helper                                     │
│  │   └── get_tenant_id() → helper                                              │
│  │                                                                              │
│  └── lib.rs (exports pub use tenant::*)                                        │
│                                                                                 │
│  soft_gloria_infrastructure                                                     │
│  ├── database.rs                                                                │
│  │   ├── create_pool(database_path, data_dir) → MODIFIED                       │
│  │   │   Now accepts tenant-specific keyring_account, db_filename              │
│  │   └── create_pool_with_key() → UNCHANGED                                    │
│  │                                                                              │
│  └── keyring.rs → SqlCipherKeyManager                                           │
│      └── new_with_fallback(service, account, data_dir) → UNCHANGED             │
│                                                                                 │
│  soft_gloria_app                                                                │
│  └── main.rs → MODIFIED                                                         │
│      ├── Derives data_dir from tenant_id: $APPDATA/mind-ledger-{tenant_id}/    │
│      ├── Derives db_path from tenant_config.crypto.dbFileName                  │
│      ├── Passes tenant keyring_account to create_pool()                        │
│      └── Registers get_tenant_config command                                   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Frontend Module Interactions

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            FRONTEND MODULE GRAPH                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  src/                                                                           │
│  ├── hooks/useTenantConfig.ts ◀────────────────── NEW                          │
│  │   └── Uses @tanstack/react-query                                            │
│  │       queryKey: ['tenant-config']                                           │
│  │       queryFn: invoke('get_tenant_config')                                  │
│  │       staleTime: Infinity                                                   │
│  │       Returns: TenantConfig | null                                          │
│  │                                                                              │
│  ├── App.tsx ◀────────────────── MODIFIED                                      │
│  │   ├── Calls useTenantConfig() at root                                       │
│  │   ├── On success: injects CSS variables via useEffect                      │
│  │   │   document.documentElement.style.setProperty('--primary', ...)          │
│  │   ├── Sets document.title = tenant.commercialName                          │
│  │   └── Renders <Layout> with tenant config context                           │
│  │                                                                              │
│  ├── components/layout/Layout.tsx ◀────────────────── MODIFIED                 │
│  │   ├── Receives tenant config via props or context                          │
│  │   ├── Sidebar brand: tenant.commercialName + clinicalRole subtitle        │
│  │   └── Window title sync (handled by App.tsx)                              │
│  │                                                                              │
│  ├── index.css ◀────────────────── MODIFIED                                    │
│  │   ├── :root { CSS variable defaults (current MindLedger brand) }           │
│  │   └── .dark { Dark mode defaults }                                         │
│  │   // Runtime injection overrides these                                      │
│  │                                                                              │
│  └── tailwind.config.js ◀────────────────── MODIFIED                           │
│      └── Colors reference CSS variables:                                       │
│          primary: 'hsl(var(--primary))',                                       │
│          secondary: 'hsl(var(--secondary))',                                   │
│          // ... all shadcn/ui color scales                                     │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. API Contracts (Rust ↔ TypeScript)

### 3.1 Tauri Command: `get_tenant_config`

**Rust Signature:**
```rust
#[tauri::command]
pub fn get_tenant_config() -> Result<TenantConfig, String>
```

**TypeScript Signature:**
```typescript
interface TenantConfig {
  tenant: TenantInfo;
  brand: BrandTokens;
  brandDark: BrandTokens;
  typography: TypographyConfig;
  crypto: CryptoConfig;
  features: FeatureFlags;
}

interface TenantInfo {
  id: string;
  commercialName: string;
  clinicalRole: string;
  ownerName: string;
  ownerTitle: string;
}

interface BrandTokens {
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
}

interface TypographyConfig {
  fontFamily: string;
  headingWeight: string;
  bodyWeight: string;
}

interface CryptoConfig {
  keyringService: string;
  keyringAccount: string;
  dbFileName: string;
}

interface FeatureFlags {
  clinicalNotes: boolean;
  accounting: boolean;
  agenda: boolean;
  diagnostics: boolean;
}
```

**Invocation (Frontend):**
```typescript
import { invoke } from '@tauri-apps/api/core';

const config = await invoke<TenantConfig>('get_tenant_config');
```

**Error Handling:**
- Returns `String` error on JSON parse failure (malformed embedded config)
- Frontend: TanStack Query `onError` → show error boundary, log to console

---

### 3.2 CSS Variable Mapping Contract

**Tenant Config (HEX) → CSS Variables (HSL)**

| Brand Token | HEX (config) | CSS Variable | HSL Format |
|-------------|--------------|--------------|------------|
| primary | `#1A5F60` | `--primary` | `192 72% 21%` |
| primaryForeground | `#FFFFFF` | `--primary-foreground` | `0 0% 100%` |
| secondary | `#E5F1EE` | `--secondary` | `165 30% 92%` |
| secondaryForeground | `#212529` | `--secondary-foreground` | `213 11% 15%` |
| accent | `#E3645F` | `--accent` | `2 72% 63%` |
| accentForeground | `#FFFFFF` | `--accent-foreground` | `0 0% 100%` |
| background | `#F8F9FA` | `--background` | `210 20% 98%` |
| foreground | `#212529` | `--foreground` | `213 11% 15%` |
| muted | `#F0F2F5` | `--muted` | `210 14% 94%` |
| mutedForeground | `#6B7280` | `--muted-foreground` | `215 16% 47%` |
| card | `#FFFFFF` | `--card` | `0 0% 100%` |
| cardForeground | `#212529` | `--card-foreground` | `213 11% 15%` |
| border | `#DEE2E6` | `--border` | `214 20% 90%` |
| input | `#DEE2E6` | `--input` | `214 20% 90%` |
| ring | `#1A5F60` | `--ring` | `192 72% 21%` |
| destructive | `#DC3545` | `--destructive` | `0 72% 51%` |
| destructiveForeground | `#FFFFFF` | `--destructive-foreground` | `0 0% 100%` |

**Dark Mode Mapping (brandDark section):**

| Brand Token (dark) | HEX | CSS Variable (in `.dark`) | HSL Format |
|--------------------|-----|---------------------------|------------|
| primary | `#2A8F90` | `--primary` | `192 60% 35%` |
| secondary | `#1E3A3A` | `--secondary` | `165 25% 18%` |
| accent | `#D45A55` | `--accent` | `2 68% 59%` |
| background | `#1A1D21` | `--background` | `213 11% 11%` |
| ... | ... | ... | ... |

**Conversion Utility (Frontend):**
```typescript
// src/lib/color-utils.ts
export function hexToHsl(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0, s = 0, l = (max + min) / 2;
  
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
    }
    h *= 60;
  }
  
  return `${Math.round(h)} ${Math.round(s * 100)}% ${Math.round(l * 100)}%`;
}
```

---

## 4. CSS Variable Injection Strategy

### 4.1 Injection Point: `src/App.tsx`

```typescript
// src/App.tsx
import { useEffect } from 'react';
import { useTenantConfig } from '@/hooks/useTenantConfig';
import { hexToHsl } from '@/lib/color-utils';

function App() {
  const { data: tenantConfig, isLoading } = useTenantConfig();
  
  useEffect(() => {
    if (!tenantConfig) return;
    
    const root = document.documentElement;
    const { brand, brandDark, typography } = tenantConfig;
    
    // Light mode variables
    Object.entries(brand).forEach(([key, value]) => {
      root.style.setProperty(`--${kebabCase(key)}`, hexToHsl(value));
    });
    
    // Dark mode variables (applied when .dark class is present)
    Object.entries(brandDark).forEach(([key, value]) => {
      // We set these on :root but they only take effect under .dark
      // via CSS specificity. Alternative: inject into a: inject into a <style> tag
      root.style.setProperty(`--${kebabCase(key)}-dark`, hexToHsl(value));
    });
    
    // Typography
    root.style.setProperty('--font-family', typography.fontFamily);
    root.style.setProperty('--heading-weight', typography.headingWeight);
    root.style.setProperty('--body-weight', typography.bodyWeight);
    
    // Window title
    document.title = tenantConfig.tenant.commercialName;
  }, [tenantConfig]);
  
  if (isLoading) return <AppSkeleton />;
  
  return <Layout tenantConfig={tenantConfig} />;
}
```

### 4.2 CSS Architecture: `src/index.css`

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    /* DEFAULT THEME (fallback when no tenant config or before load) */
    --background: 210 20% 98%;
    --foreground: 213 11% 15%;
    --card: 0 0% 100%;
    --card-foreground: 213 11% 15%;
    --popover: 0 0% 100%;
    --popover-foreground: 213 11% 15%;
    --primary: 192 72% 21%;
    --primary-foreground: 0 0% 100%;
    --secondary: 165 30% 92%;
    --secondary-foreground: 213 11% 15%;
    --muted: 210 14% 94%;
    --muted-foreground: 215 16% 47%;
    --accent: 2 72% 63%;
    --accent-foreground: 0 0% 100%;
    --destructive: 0 72% 51%;
    --destructive-foreground: 0 0% 100%;
    --border: 214 20% 90%;
    --input: 214 20% 90%;
    --ring: 192 72% 21%;
    --radius: 0.5rem;
    
    /* Typography defaults */
    --font-family: 'Inter', system-ui, -apple-system, sans-serif;
    --heading-weight: 700;
    --body-weight: 400;
  }

  .dark {
    --background: 213 11% 11%;
    --foreground: 210 20% 96%;
    --card: 213 11% 13%;
    --card-foreground: 210 20% 96%;
    --popover: 213 11% 13%;
    --popover-foreground: 210 20% 96%;
    --primary: 192 60% 35%;
    --primary-foreground: 0 0% 100%;
    --secondary: 165 25% 18%;
    --secondary-foreground: 165 30% 92%;
    --muted: 213 14% 18%;
    --muted-foreground: 215 16% 65%;
    --accent: 2 68% 59%;
    --accent-foreground: 0 0% 100%;
    --destructive: 0 65% 45%;
    --destructive-foreground: 0 0% 100%;
    --border: 213 14% 18%;
    --input: 213 14% 18%;
    --ring: 192 60% 35%;
  }
}

/* Runtime-injected dark mode variables use this selector */
.dark {
  --primary: var(--primary-dark);
  --primary-foreground: var(--primary-foreground-dark);
  --secondary: var(--secondary-dark);
  --secondary-foreground: var(--secondary-foreground-dark);
  --accent: var(--accent-dark);
  --accent-foreground: var(--accent-foreground-dark);
  --background: var(--background-dark);
  --foreground: var(--foreground-dark);
  --muted: var(--muted-dark);
  --muted-foreground: var(--muted-foreground-dark);
  --card: var(--card-dark);
  --card-foreground: var(--card-foreground-dark);
  --border: var(--border-dark);
  --input: var(--input-dark);
  --ring: var(--ring-dark);
  --destructive: var(--destructive-dark);
  --destructive-foreground: var(--destructive-foreground-dark);
}
```

### 4.3 Tailwind Configuration: `tailwind.config.js`

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // All shadcn/ui semantic colors reference CSS variables
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        secondary: {
          DEFAULT: 'hsl(var(--secondary))',
          foreground: 'hsl(var(--secondary-foreground))',
        },
        destructive: {
          DEFAULT: 'hsl(var(--destructive))',
          foreground: 'hsl(var(--destructive-foreground))',
        },
        muted: {
          DEFAULT: 'hsl(var(--muted))',
          foreground: 'hsl(var(--muted-foreground))',
        },
        accent: {
          DEFAULT: 'hsl(var(--accent))',
          foreground: 'hsl(var(--accent-foreground))',
        },
        popover: {
          DEFAULT: 'hsl(var(--popover))',
          foreground: 'hsl(var(--popover-foreground))',
        },
        card: {
          DEFAULT: 'hsl(var(--card))',
          foreground: 'hsl(var(--card-foreground))',
        },
      },
      fontFamily: {
        sans: ['var(--font-family)', 'system-ui', '-apple-system', 'sans-serif'],
      },
      fontWeight: {
        heading: 'var(--heading-weight)',
        body: 'var(--body-weight)',
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
    },
  },
  plugins: [],
}
```

---

## 5. Build Pipeline Design

### 5.1 Build.rs Enhancement

```rust
// src-tauri/build.rs
use std::path::Path;
use std::fs;

fn main() {
    // 1. Read tenant config path from env (or default)
    let tenant_config_path = std::env::var("TENANT_CONFIG")
        .unwrap_or_else(|_| "tenant-configs/gloria-once.json".to_string());
    
    let config_path = Path::new(&tenant_config_path);
    
    if config_path.exists() {
        println!("cargo:rerun-if-changed={}", config_path.display());
        
        // 2. Copy to OUT_DIR for embed via include_str!
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("tenant-config.json");
        fs::copy(config_path, &dest_path)
            .expect("Failed to copy tenant config to OUT_DIR");
        
        // 3. Validate JSON
        let config_str = fs::read_to_string(config_path)
            .expect("Failed to read tenant config");
        let _: serde_json::Value = serde_json::from_str(&config_str)
            .expect("Tenant config is not valid JSON");
        
        // 4. Emit env var for runtime access
        println!("cargo:rustc-env=TENANT_CONFIG_PATH={}", dest_path.display());
        
        // 5. Parse config for tauri.conf.json templating
        let config: serde_json::Value = serde_json::from_str(&config_str).unwrap();
        let identifier = config["tenant"]["id"].as_str().unwrap_or("default");
        let commercial_name = config["tenant"]["commercialName"].as_str().unwrap_or("MindLedger");
        
        // 6. Template tauri.conf.json
        template_tauri_conf(identifier, commercial_name);
    }
    
    tauri_build::build()
}

fn template_tauri_conf(tenant_id: &str, commercial_name: &str) {
    let tauri_conf_path = Path::new("src-tauri/tauri.conf.json");
    let content = fs::read_to_string(tauri_conf_path).expect("Failed to read tauri.conf.json");
    
    // Parse and modify
    let mut conf: serde_json::Value = serde_json::from_str(&content).expect("Invalid tauri.conf.json");
    
    conf["identifier"] = serde_json::Value::String(format!("com.mindledger.{}.desktop", tenant_id));
    conf["productName"] = serde_json::Value::String(commercial_name.to_string());
    conf["app"]["windows"][0]["title"] = serde_json::Value::String(commercial_name.to_string());
    
    // Write back (tauri-build reads this)
    let new_content = serde_json::to_string_pretty(&conf).expect("Failed to serialize tauri.conf.json");
    fs::write(tauri_conf_path, new_content).expect("Failed to write templated tauri.conf.json");
}
```

### 5.2 Build Commands

```bash
# Default build (falls back to gloria-once if no TENANT_CONFIG)
cargo tauri build

# Explicit tenant build
TENANT_CONFIG=../tenant-configs/gloria-once.json cargo tauri build

# Future: other tenants
TENANT_CONFIG=../tenant-configs/other-tenant.json cargo tauri build
```

### 5.3 Fallback Strategy (Default Tenant)

```rust
// commands/src/tenant.rs - get_tenant_config() fallback
#[tauri::command]
pub fn get_tenant_config() -> Result<TenantConfig, String> {
    // Try embedded config first
    if let Ok(config_str) = std::str::from_utf8(include_bytes!(concat!(env!("OUT_DIR"), "/tenant-config.json"))) {
        return serde_json::from_str(config_str).map_err(|e| format!("Parse error: {}", e));
    }
    
    // Fallback: compile-time default (for backward compat)
    const DEFAULT_CONFIG: &str = include_str!("../../../tenant-configs/default.json");
    serde_json::from_str(DEFAULT_CONFIG).map_err(|e| format!("Default config parse error: {}", e))
}
```

---

## 6. Database Isolation Design

### 6.1 Modified `database.rs`

```rust
// src-tauri/infrastructure/src/database.rs

use crate::keyring::SqlCipherKeyManager;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

/// Create pool with tenant-specific configuration
pub fn create_pool_for_tenant(
    data_dir: &Path,
    keyring_account: &str,
    db_filename: &str,
) -> Result<DbPool> {
    let db_path = data_dir.join(db_filename);
    let service_name = "mind-ledger"; // Shared service, unique account per tenant
    
    let key_manager = SqlCipherKeyManager::new_with_fallback(
        service_name,
        keyring_account,
        data_dir,
    );
    
    let key = key_manager.get_or_create_key()?;
    create_pool_with_key(&db_path, &key)
}

/// Backward-compatible create_pool (for tests, default tenant)
pub fn create_pool(database_path: &Path, data_dir: &Path) -> Result<DbPool> {
    create_pool_for_tenant(
        data_dir,
        "sqlcipher-key",        // default account
        "mind_ledger.db",       // default filename
    )
}

pub fn create_pool_with_key(database_path: &Path, key: &str) -> Result<DbPool> {
    // ... unchanged implementation ...
}
```

### 6.2 Modified `main.rs`

```rust
// src-tauri/app/src/main.rs

use soft_gloria_commands::tenant::{
    get_tenant_config, get_tenant_keyring_account, get_tenant_db_filename, get_tenant_id
};
use soft_gloria_infrastructure::create_pool_for_tenant;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Get tenant config at startup (sync, embedded)
            let tenant_config = get_tenant_config().map_err(|e| {
                eprintln!("[MindLedger] Failed to load tenant config: {}", e);
                e
            })?;
            
            let tenant_id = tenant_config.tenant.id.clone();
            let keyring_account = tenant_config.crypto.keyringAccount.clone();
            let db_filename = tenant_config.crypto.dbFileName.clone();
            
            // Derive tenant-specific data directory
            let base_data_dir = app.path().app_data_dir().map_err(|e| {
                eprintln!("[MindLedger] Failed to get app data dir: {}", e);
                e
            })?;
            
            let data_dir = base_data_dir.join(format!("mind-ledger-{}", tenant_id));
            std::fs::create_dir_all(&data_dir).map_err(|e| {
                eprintln!("[MindLedger] Failed to create tenant data dir: {}", e);
                e
            })?;
            
            tauri::async_runtime::block_on(async move {
                let db = create_pool_for_tenant(&data_dir, &keyring_account, &db_filename)
                    .map_err(|e| {
                        eprintln!("[MindLedger] Failed to initialize database: {}", e);
                        e
                    })?;
                
                // Run migrations...
                run_migrations(&db).map_err(|e| {
                    eprintln!("[MindLedger] Failed to run migrations: {}", e);
                    e
                })?;
                
                app_handle.manage(Arc::new(db));
                Ok::<(), Box<dyn std::error::Error>>(())
            }).unwrap_or_else(|e| {
                eprintln!("[MindLedger] Critical error during setup: {}", e);
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            get_tenant_config,  // NEW: register tenant command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 6.3 Keyring Isolation Verification

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        KEYRING ISOLATION VERIFICATION                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  Tenant: gloria-once                                                           │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │ Keychain (macOS) / Credential Manager (Windows) / Secret Service (Linux)│   │
│  │                                                                         │   │
│  │  Service: "mind-ledger"                                                │   │
│  │  Account: "sqlcipher-key-gloria-once"  ◄── UNIQUE PER TENANT            │   │
│  │  Secret:  "a1b2c3d4e5f6... (64 hex chars)"                             │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  Tenant: default (fallback)                                                    │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │  Service: "mind-ledger"                                                │   │
│  │  Account: "sqlcipher-key"              ◄── DEFAULT ACCOUNT              │   │
│  │  Secret:  "f6e5d4c3b2a1... (64 hex chars)"                             │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│  ✓ Zero crossover: Different account = different encryption key              │
│  ✓ Different DB file: mind_ledger_gloria_once.db vs mind_ledger.db           │
│  ✓ Different data dir: mind-ledger-gloria-once/ vs mind-ledger/              │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Frontend Integration Details

### 7.1 `useTenantConfig` Hook

```typescript
// src/hooks/useTenantConfig.ts
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { TenantConfig } from '@/types/tenant';

export function useTenantConfig() {
  return useQuery({
    queryKey: ['tenant-config'],
    queryFn: () => invoke<TenantConfig>('get_tenant_config'),
    staleTime: Infinity, // Config never changes at runtime
    gcTime: Infinity,    // Keep in cache forever
    retry: false,        // Fail fast if command unavailable
  });
}
```

### 7.2 Layout Component Integration

```typescript
// src/components/layout/Layout.tsx
import { Sidebar } from '@/components/ui/sidebar';
import { Header } from '@/components/ui/header';

interface LayoutProps {
  children: React.ReactNode;
  tenantConfig?: TenantConfig | null;
}

export function Layout({ children, tenantConfig }: LayoutProps) {
  const brandName = tenantConfig?.tenant.commercialName ?? 'MindLedger';
  const subtitle = tenantConfig?.tenant.clinicalRole ?? 'Clinical Psychology';
  
  return (
    <div className="flex h-screen bg-background font-sans antialiased">
      <Sidebar 
        brandName={brandName}
        subtitle={subtitle}
      />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <main className="flex-1 overflow-x-hidden overflow-y-auto p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
```

```typescript
// src/components/ui/sidebar.tsx
interface SidebarProps {
  brandName: string;
  subtitle: string;
}

export function Sidebar({ brandName, subtitle }: SidebarProps) {
  return (
    <aside className="fixed inset-y-0 left-0 z-50 w-64 border-r bg-card transition-all duration-200">
      <div className="flex h-16 items-center px-6 border-b">
        <div className="flex items-center gap-3">
          <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
            <Brain className="h-5 w-5 text-primary-foreground" />
          </div>
          <div>
            <h1 className="font-heading font-bold text-lg text-foreground">
              {brandName}
            </h1>
            <p className="text-xs text-muted-foreground">{subtitle}</p>
          </div>
        </div>
      </div>
      <nav className="flex-1 p-4 space-y-1 overflow-y-auto">
        {/* Navigation items */}
      </nav>
    </aside>
  );
}
```

### 7.3 Login Page Integration (Deferred)

```typescript
// src/pages/LoginPage.tsx (if exists, otherwise create)
import { useTenantConfig } from '@/hooks/useTenantConfig';

export function LoginPage() {
  const { data: tenantConfig } = useTenantConfig();
  
  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="w-full max-w-md p-8 space-y-6 bg-card border rounded-lg">
        <div className="text-center">
          <h1 className="font-heading text-3xl font-bold text-foreground">
            {tenantConfig?.tenant.commercialName ?? 'MindLedger'}
          </h1>
          <p className="text-muted-foreground mt-2">
            {tenantConfig?.tenant.clinicalRole ?? 'Clinical Psychology Practice'}
          </p>
        </div>
        {/* Login form */}
      </div>
    </div>
  );
}
```

---

## 8. Feature Flag Integration

### 8.1 Runtime Feature Gates

```typescript
// src/hooks/useFeatureFlags.ts
import { useTenantConfig } from '@/hooks/useTenantConfig';

export function useFeatureFlags() {
  const { data: config } = useTenantConfig();
  
  return {
    clinicalNotes: config?.features.clinicalNotes ?? true,
    accounting: config?.features.accounting ?? true,
    agenda: config?.features.agenda ?? true,
    diagnostics: config?.features.diagnostics ?? true,
  };
}
```

### 8.2 Conditional Route Rendering

```typescript
// src/App.tsx
import { useFeatureFlags } from '@/hooks/useFeatureFlags';

function AppRoutes() {
  const { clinicalNotes, accounting, agenda, diagnostics } = useFeatureFlags();
  
  return (
    <Routes>
      <Route path="/" element={<DashboardPage />} />
      {clinicalNotes && <Route path="/notes" element={<ClinicalNotesPage />} />}
      {accounting && <Route path="/accounting" element={<AccountingPage />} />}
      {agenda && <Route path="/agenda" element={<AppointmentsPage />} />}
      {diagnostics && <Route path="/diagnostics" element={<DiagnosticsPage />} />}
      <Route path="/settings" element={<SettingsPage />} />
    </Routes>
  );
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests (Rust)

```rust
// commands/src/tenant.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_config_deserialization() {
        let config_str = include_str!(concat!(env!("OUT_DIR"), "/tenant-config.json"));
        let config: TenantConfig = serde_json::from_str(config_str).unwrap();
        
        assert_eq!(config.tenant.id, "gloria-once");
        assert_eq!(config.crypto.keyringAccount, "sqlcipher-key-gloria-once");
        assert_eq!(config.crypto.dbFileName, "mind_ledger_gloria_once.db");
    }
    
    #[test]
    fn test_get_tenant_keyring_account() {
        let account = get_tenant_keyring_account();
        assert_eq!(account, "sqlcipher-key-gloria-once");
    }
}

// infrastructure/src/database.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_pool_for_tenant_isolation() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("mind-ledger-gloria-once");
        
        let pool = create_pool_for_tenant(
            &data_dir,
            "sqlcipher-key-gloria-once",
            "mind_ledger_gloria_once.db"
        );
        
        assert!(pool.is_ok());
        
        // Verify DB file created in tenant-specific directory
        let db_path = data_dir.join("mind_ledger_gloria_once.db");
        assert!(db_path.exists());
    }
}
```

### 9.2 Integration Tests (Frontend)

```typescript
// src/hooks/__tests__/useTenantConfig.test.tsx
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTenantConfig } from '@/hooks/useTenantConfig';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue({
    tenant: {
      id: 'gloria-once',
      commercialName: 'MindLedger - Psic. Gloria Once',
      clinicalRole: 'Neuropsicóloga Clínica',
      ownerName: 'Psic. Gloria Once',
      ownerTitle: 'Neuropsicóloga Clínica',
    },
    brand: { /* ... */ },
    brandDark: { /* ... */ },
    typography: { fontFamily: 'Inter', headingWeight: '700', bodyWeight: '400' },
    crypto: { keyringService: 'mind-ledger', keyringAccount: 'sqlcipher-key-gloria-once', dbFileName: 'mind_ledger_gloria_once.db' },
    features: { clinicalNotes: true, accounting: true, agenda: true, diagnostics: true },
  }),
}));

test('useTenantConfig fetches and returns tenant config', async () => {
  const queryClient = new QueryClient();
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  
  const { result } = renderHook(() => useTenantConfig(), { wrapper });
  
  await waitFor(() => expect(result.current.isSuccess).toBe(true));
  
  expect(result.current.data?.tenant.id).toBe('gloria-once');
  expect(result.current.data?.tenant.commercialName).toBe('MindLedger - Psic. Gloria Once');
});
```

---

## 10. Rollback Procedures

### 10.1 Code Rollback

```bash
# 1. Revert build.rs to original (no tenant config copy)
git checkout HEAD~1 -- src-tauri/build.rs

# 2. Restore original tauri.conf.json
git checkout HEAD~1 -- src-tauri/tauri.conf.json

# 3. Remove tenant.rs module
rm src-tauri/commands/src/tenant.rs
# Edit src-tauri/commands/src/lib.rs to remove tenant export

# 4. Revert database.rs to use DEFAULT_SERVICE_NAME/ACCOUNT_NAME
git checkout HEAD~1 -- src-tauri/infrastructure/src/database.rs

# 5. Revert main.rs to original database initialization
git checkout HEAD~1 -- src-tauri/app/src/main.rs

# 6. Remove frontend hooks and CSS changes
rm src/hooks/useTenantConfig.ts
rm src/hooks/useFeatureFlags.ts
git checkout HEAD~1 -- src/index.css tailwind.config.js src/App.tsx src/components/layout/Layout.tsx

# 7. Remove tenant config (or keep as reference)
rm tenant-configs/gloria-once.json
```

### 10.2 Database Migration Rollback

```bash
# If tenant DB was created, user data is isolated in:
# $APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db
# 
# To migrate back to default:
# 1. Export data from tenant DB (if needed)
# 2. Delete tenant data directory
# 3. Default build will use $APPDATA/mind-ledger/mind_ledger.db
```

---

## 11. Acceptance Criteria Checklist

| # | Criterion | Verification Method |
|---|-----------|---------------------|
| 1 | `TENANT_CONFIG=../tenant-configs/gloria-once.json cargo tauri build` produces app | Manual build + inspect bundle |
| 2 | Bundle ID: `com.mindledger.gloriaonce.desktop` | Check `tauri.conf.json` output / macOS `codesign -d` |
| 3 | App name: `MindLedger - Psic. Gloria Once` | Window title, installer name |
| 4 | Sidebar shows "MindLedger - Psic. Gloria Once" + "Neuropsicóloga Clínica" | Visual inspection |
| 5 | Primary `#1A5F60` (Teal) on buttons, sidebar, focus rings | Visual + devtools CSS var check |
| 6 | Secondary `#E5F1EE` (Sage) on metric cards, hovers | Visual + devtools CSS var check |
| 7 | Accent `#E3645F` (Coral) on destructive actions only | Visual + devtools CSS var check |
| 8 | Dark mode colors from `brandDark` work | Toggle dark mode, verify |
| 9 | DB at `$APPDATA/mind-ledger-gloria-once/mind_ledger_gloria_once.db` | File system check |
| 10 | SQLCipher key in keyring: `mind-ledger` / `sqlcipher-key-gloria-once` | Keychain Access / Credential Manager |
| 11 | Zero data crossover: run Gloria Once + default builds side by side | Open both, verify separate data |
| 12 | All 74+ existing tests pass | `cargo test` + `pnpm test` |
| 13 | No TypeScript errors | `pnpm tsc --noEmit` |
| 14 | No Rust clippy warnings | `cargo clippy -- -D warnings` |

---

## 12. Future Extensibility

### 12.1 Adding New Tenants

```bash
# 1. Create tenant config
cp tenant-configs/gloria-once.json tenant-configs/new-tenant.json
# Edit with new tenant's brand tokens, crypto config, features

# 2. Build for new tenant
TENANT_CONFIG=../tenant-configs/new-tenant.json cargo tauri build
```

### 12.2 CI/CD Pipeline (Deferred)

```yaml
# .github/workflows/release.yml (future)
matrix:
  tenant: [gloria-once, future-tenant-2, future-tenant-3]
steps:
  - name: Build tenant
    run: TENANT_CONFIG=../tenant-configs/${{ matrix.tenant }}.json cargo tauri build
```

### 12.3 Asset Pipeline (Deferred)

- Logo/favicon injection via `tauri.conf.json` `bundle.icon` templating
- Splash screen customization
- Installer branding (NSIS, DMG background)

---

## 13. Security Considerations

| Aspect | Implementation |
|--------|----------------|
| Config validation | `build.rs` validates JSON at compile time via `serde_json::from_str` |
| Key isolation | Unique keyring account per tenant (`sqlcipher-key-{tenant_id}`) |
| DB isolation | Unique filename + data directory per tenant |
| No runtime tenant switching | Single binary = single tenant (build-time binding) |
| CSP unchanged | Existing `tauri.conf.json` CSP maintained |
| No secrets in config | `tenant-configs/*.json` contains only public branding + keyring identifiers |

---

*Design complete. Ready for implementation phase.*