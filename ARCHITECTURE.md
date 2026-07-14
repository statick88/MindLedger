# MindLdger — Manual de Arquitectura de Software

> **Versión:** 1.0.0  
> **Fecha:** 2026-07-13  
> **Autor:** Diego Medardo Saavedra García (Statick)  
> **Metodología:** Spec-Driven Development (SDD) — Fase 7: Arquitectura Cognitiva de Onboarding  
> **Estado:** 124/124 tests aprobados · 0 warnings · Binarios estables

---

## 1. Visión General del Sistema

MindLdger es un **Sistema de Gestión Clínica y Contable Multi-Tenant** diseñado para psicólogos clínicos en Ecuador (cumplimiento LOPD). Arquitectura **Clean Architecture** con separación estricta en 4 capas, ejecutándose como aplicación **Tauri v2 (Rust + React/TypeScript)** con base de datos **SQLCipher** cifrada y llaves en **Keychain/Keyring nativo**.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            CLEAN ARCHITECTURE LAYERS                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────┐  │
│  │   PRESENTATION      │    │   APPLICATION       │    │    DOMAIN       │  │
│  │   (React/TS)        │───▶│   (Tauri Commands)  │───▶│    (Rust)       │  │
│  │                     │    │                     │    │                 │  │
│  │ • Pages/Components  │    │ • Command Handlers  │    │ • Entities      │  │
│  │ • Hooks/State       │    │ • DTOs/Validation   │    │ • Value Objects │  │
│  │ • API Client        │    │ • Use Cases         │    │ • Repositories  │  │
│  │   (Tauri Invoke)    │    │   (Pure Functions)  │    │   (Traits)      │  │
│  └─────────────────────┘    └─────────────────────┘    └────────┬────────┘  │
│                                                                   │         │
│  ┌───────────────────────────────────────────────────────────────┘         │
│  │                                                                          │
│  ▼                                                                          │
│  ┌─────────────────────┐                                                   │
│  │   INFRASTRUCTURE    │                                                   │
│  │   (Rust + SQLite)   │                                                   │
│  │                     │                                                   │
│  │ • SQLCipher Pool    │                                                   │
│  │ • Keyring Manager   │                                                   │
│  │ • Migrations        │                                                   │
│  │ • Repository Impls  │                                                   │
│  └─────────────────────┘                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Mapeo del Flujo de Datos (Data Flow Blueprint)

### 2.1 Ciclo de Vida Completo de una Petición

```mermaid
sequenceDiagram
    participant U as Usuario
    participant F as Frontend (React/TS)
    participant I as IPC Channel (Tauri invoke)
    participant C as Command Handler (Rust)
    participant UC as Use Case / Domain Logic
    participant R as Repository Trait
    participant IR as SQLite Repo Impl
    participant DB as SQLCipher DB
    participant K as Keyring/Native Keystore

    U->>F: Interacción UI (ej. Crear Paciente)
    F->>F: Validación Zod + Optimistic Update
    F->>I: invoke('create_patient', {request})
    I->>C: Tauri Router → patient_commands::create_patient
    C->>C: Deserialización + Validación DTO
    C->>UC: Llama Use Case (puro, sin estado)
    UC->>R: patient_repo.create(&patient)
    R->>IR: SqlitePatientRepository::create()
    IR->>K-->IR: get_or_create_key() → Clave 256-bit hex
IR->>DB: PRAGMA key = '...'; INSERT ...
DB-->>IR: OK
IR-->>UC: Result<(), RepoError>
UC-->>C: AppResult<PatientResponse>
C-->>I: Serialización JSON → Tauri Response
I-->>F: Promise resolve → Patient object
F->>F: Invalidate React Query cache → Re-render
F-->>U: UI actualizada
```

### 2.2 Detalle por Capa

#### **Capa 1: Presentation (Frontend — `src/`)**

| Componente | Responsabilidad | Tecnología |
|------------|----------------|------------|
| `src/pages/*.tsx` | Vistas de ruta (Dashboard, Pacientes, Contabilidad, Citas, Diagnósticos, Config) | React 18 + TypeScript + Vite |
| `src/components/ui/*` | Design System (shadcn/ui + Radix + Tailwind) | 50+ componentes accesibles |
| `src/hooks/*` | Estado reactivo, tema, toasts | TanStack Query v5, Zustand |
| `src/api/index.ts` | **Cliente IPC tipado** — `invoke<ReturnType>('command', args)` | `@tauri-apps/api/core` |
| `src/types/index.ts` | **Contrato compartido** Frontend↔Backend (Patient, Asiento, Settings, etc.) | Zod-ready interfaces |

**Flujo de datos en Frontend:**
```
User Event → React Handler → api.patientApi.create(request)
    → Tauri invoke() → Promise<Patient>
    → queryClient.invalidateQueries(['patients'])
    → TanStack Query refetch → UI Update
```

#### **Capa 2: Application / Commands (Rust — `src-tauri/commands/src/`)**

```rust
// Estructura típica de un command handler
#[command]
pub async fn create_patient(
    db: tauri::State<'_, Arc<DbPool>>,  // Inyectado por Tauri
    request: CreatePatientRequest,       // DTO validado por serde
) -> AppResult<PatientResponse> {
    // 1. Validación de entrada (DTO → Value Objects)
    let doc_number = DocumentNumber::new(...)?;
    let full_name = FullName::new(...)?;
    let dob = NaiveDate::parse_from_str(&request.date_of_birth, "%Y-%m-%d")?;
    
    // 2. Construcción de Entidad de Dominio (pura)
    let mut patient = Patient::new(doc_number, full_name, dob, request.gender);
    patient.update_contact_info(email, phone, address);
    
    // 3. Delegación a Repository Trait (inversión de dependencia)
    let repo = SqlitePatientRepository::new((**db).clone());
    repo.create(&patient).await?;
    
    // 4. Serialización de respuesta
    Ok(patient.into())
}
```

**Principios clave:**
- **Thin handlers**: Solo serialización/deserialización + validación DTO
- **Inner functions testables**: `*_impl` functions puras sin `tauri::State`
- **Error mapping**: `AppError` → `serde_json` automático via `#[command]`

#### **Capa 3: Domain (Rust — `src-tauri/domain/src/`)**

```
domain/
├── identifiers.rs      // PatientId(Uuid), AsientoId(Uuid) — Newtypes
├── value_objects.rs    // DocumentNumber, Email, PhoneNumber, FullName, Address
├── patient.rs          // Aggregate Root: Patient + métodos de negocio
├── accounting.rs       // Libro Diario, AsientoContable, LineaAsiento, BalanceGeneral
├── diagnostics.rs      // CIE-10, DSM-5, Mapeos
├── age.rs              // Age, AgeBreakdown — lógica de edad precisa
└── repositories.rs     // TRAITS: PatientRepository, AccountingRepository...
```

**Reglas de invariante de dominio (enforced at construction):**

```rust
// Patient: Value Objects validan al construir
impl DocumentNumber {
    pub fn new(number: String, dtype: DocumentType, country: String) -> Result<Self> {
        // Validación formato Ecuador: 10 dígitos cédula, 13 RUC, etc.
        Self::validate_ecuador_format(&number, dtype)?;
        Ok(Self { number, document_type: dtype, country_code: country })
    }
}

// AsientoContable: Double-entry enforcement EN CONSTRUCTOR
impl AsientoContable {
    pub fn new(fecha, descripcion, lineas) -> Result<Self, ContabilidadError> {
        // 1. No vacío
        // 2. Cada línea: XOR(debito > 0, credito > 0) — nunca ambos, nunca ninguno
        // 3. Σ débitos == Σ créditos (±0.01 epsilon)
        // 4. Cuenta no vacía
    }
}
```

#### **Capa 4: Infrastructure (Rust — `src-tauri/infrastructure/src/`)**

```
infrastructure/
├── database.rs         // DbPool = Arc<Mutex<Connection>> + PRAGMA key
├── keyring.rs          // SqlCipherKeyManager — Keychain/Keyring + fallback file (0o600)
├── migrations.rs       // execute_batch() — NO split(';') (rompe triggers)
├── repositories.rs     // SqlitePatientRepository impl PatientRepository
├── accounting_repository_sqlite.rs  // Balance General, Estado Resultados
└── diagnostics_repository_sqlite.rs // CIE-10/DSM-5 full-text search
```

**Pool de conexión SQLCipher:**
```rust
pub fn create_pool(database_path: &Path, data_dir: &Path) -> Result<DbPool> {
    let key_manager = SqlCipherKeyManager::new_with_fallback(
        "mind-ledger",           // service name
        "sqlcipher-key",         // account
        data_dir,                // fallback dir
    );
    let key = key_manager.get_or_create_key()?;  // 64-char hex (32 bytes)
    create_pool_with_key(database_path, &key)
}

fn create_pool_with_key(path: &Path, key: &str) -> Result<DbPool> {
    let conn = Connection::open(path)?;
    conn.execute_batch(&format!("PRAGMA key = '{}';", key))?;  // Passphrase format
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;  // fallback a DELETE
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    Ok(Arc::new(Mutex::new(conn)))
}
```

---

## 3. Invariantes del Negocio (Business Invariants)

### 3.1 Perfil de Release: `lto = "thin"` — **NO NEGOCIABLE**

```toml
# src-tauri/Cargo.toml — [profile.release]
[profile.release]
opt-level = 3
lto = "thin"        # ⚠️ FAT LTO CORROMPE SQLCIPHER C FFI
codegen-units = 1
strip = true
panic = "abort"
```

**Evidencia empírica (FASE 5 — Release Archive):**
- **Fat LTO (`lto = "fat"` o `lto = true`)**: Corrompe la tabla de símbolos del C FFI de SQLCipher bundled → `PRAGMA key` falla silenciosamente → BD inaccesible en producción
- **Thin LTO**: Preserva boundaries de crate, permite optimización cross-crate sin corromper `sqlite3_*` symbols
- **Descubrimiento**: Binario universal (x86_64 + arm64) de 13 MB funcionando en macOS 10.15+ solo con `thin`

> **Regla de oro:** Cualquier cambio a `Cargo.toml` que toque `[profile.release]` **requiere** test de smoke completo (abrir BD, insertar, leer, cerrar, reabrir) antes de merge.

### 3.2 Ecuación Contable Fundamental — Partida Doble Blindada

La invariante **Activos = Pasivos + Patrimonio** se garantiza en **3 capas simultáneas**:

| Capa | Mecanismo | Código |
|------|-----------|--------|
| **Dominio** | `AsientoContable::new()` valida Σ débitos = Σ créditos (±0.01) | `accounting.rs:78-88` |
| **Repositorio** | `get_balance_general()` clasifica por primer dígito: 1=Activo, 2=Pasivo, 3/4/5/6/7=Patrimonio | `accounting_repository_sqlite.rs:223-232` |
| **Validación** | `validar_balance_general(activos, pasivos, patrimonio)` → `Result<(), ContabilidadError>` | `accounting.rs:185-203` |

**Clasificación automática de cuentas (Plan Contable Ecuatoriano):**

```rust
// Primer dígito determina naturaleza y signo en Balance General
let target_map = match primer_char {
    '1' => (&mut activos,  1),   // Activos — balance normal: DÉBITO (+)
    '2' => (&mut pasivos, -1),   // Pasivos — balance normal: CRÉDITO (-)
    '3' | '4' | '7' => (&mut patrimonio, -1),  // Capital, Ingresos, Otros Ingresos
    '5' | '6' => (&mut patrimonio, -1),        // Gastos, Costos — REDUCEN patrimonio
    _   => (&mut patrimonio, -1),              // Default conservador
};
```

**Prueba de invariante (test real):**
```rust
#[test]
fn test_balance_general_invariant() {
    // Activos: Caja 5000 + Bancos 5000 = 10000
    // Pasivos: Proveedores 3000
    // Patrimonio: Capital 10000 - Mercaderías 3000 (gasto) = 7000
    // 10000 = 3000 + 7000 ✓
    assert!(balance.is_balanced());
}
```

### 3.3 Aislamiento Criptográfico por Tenant

Cada instancia de MindLdger (cada clínica) tiene:
- **Base de datos independiente**: `mind_ledger.db` en `$APPDATA/mind-ledger/`
- **Clave de cifrado única**: 256-bit generada via `rand::rngs::OsRng` (CSPRNG del OS)
- **Almacenamiento de clave**: 
  - **Preferido**: Keychain (macOS), Credential Manager (Windows), Secret Service (Linux)
  - **Fallback**: `$APPDATA/mind-ledger/mind-ledger.key` con permisos `0o600`
- **Zeroize**: `Zeroizing<Vec<u8>>` para clave en memoria

```rust
// keyring.rs — Generación y persistencia segura
fn generate_hex_key() -> String {
    let mut rng = rand::thread_rng();
    let key_bytes: Zeroizing<Vec<u8>> = Zeroizing::new(
        (0..KEY_LENGTH).map(|_| rng.gen()).collect()
    );
    Zeroizing::new(key_bytes.iter().map(|b| format!("{:02x}", b)).collect())
}
```

---

## 4. Mapa de Módulos y Dependencias

### 4.1 Workspace Cargo

```toml
[workspace]
resolver = "2"
members = ["domain", "application", "infrastructure", "commands"]
exclude = ["app"]

[workspace.dependencies]
# Core
tauri = { version = "2.0" }
serde = { version = "1.0", features = ["derive"] }
# Database
rusqlite = { version = "0.31", features = ["bundled-sqlcipher", "load_extension", "chrono", "serde_json", "uuid"] }
# Security
keyring = "3"
zeroize = "1.0"
argon2 = "0.5"
```

### 4.2 Grafo de Dependencias (Dirección = depende de)

```
app (bin)
  └── commands
        ├── domain          ◀───  NO dependencies (pure)
        ├── infrastructure  ◀───  domain
        └── application     ◀───  domain + infrastructure
```

**Regla arquitectónica:** `domain` **nunca** depende de `infrastructure`, `commands`, ni `tauri`. Es Rust puro, testeable sin runtime.

---

## 5. Esquema de Base de Datos (SQLCipher)

### 5.1 Tablas Principales

```sql
-- Pacientes (core clínico)
CREATE TABLE patients (
    id TEXT PRIMARY KEY NOT NULL,
    document_number TEXT NOT NULL UNIQUE,
    document_type TEXT NOT NULL DEFAULT 'cedula',
    country_code TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    middle_name TEXT,
    date_of_birth TEXT NOT NULL,
    gender TEXT NOT NULL,
    email TEXT,
    phone_number TEXT,
    phone_country_code TEXT,
    phone_extension TEXT,
    address_street TEXT,
    address_city TEXT,
    address_state TEXT,
    address_postal_code TEXT,
    address_country TEXT,
    address_additional_info TEXT,
    emergency_contact_name_first TEXT,
    emergency_contact_name_last TEXT,
    emergency_contact_name_middle TEXT,
    emergency_contact_relationship TEXT,
    emergency_contact_phone_number TEXT,
    emergency_contact_phone_country_code TEXT,
    emergency_contact_email TEXT,
    blood_type TEXT,
    allergies TEXT DEFAULT '[]',           -- JSON array
    chronic_conditions TEXT DEFAULT '[]',  -- JSON array
    medications TEXT DEFAULT '[]',         -- JSON array
    notes TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Asientos Contables (partida doble)
CREATE TABLE asientos_contables (
    id TEXT PRIMARY KEY NOT NULL,
    fecha TEXT NOT NULL,
    descripcion TEXT NOT NULL,
    lineas TEXT NOT NULL,                  -- JSON: Vec<LineaAsiento>
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_asientos_fecha ON asientos_contables(fecha);

-- Diagnósticos CIE-10 / DSM-5
CREATE TABLE diagnostics (
    id TEXT PRIMARY KEY NOT NULL,
    patient_id TEXT NOT NULL REFERENCES patients(id),
    session_id TEXT REFERENCES sessions(id),
    code TEXT NOT NULL,
    system TEXT NOT NULL,                  -- 'CIE10' | 'DSM5'
    description TEXT NOT NULL,
    severity TEXT,
    status TEXT,
    date TEXT NOT NULL,
    notes TEXT
);

-- CIE-10 / DSM-5 Catálogos (pre-cargados via migraciones)
CREATE TABLE cie10_catalog (codigo TEXT PRIMARY KEY, descripcion TEXT, categoria TEXT);
CREATE TABLE dsm5_catalog (codigo TEXT PRIMARY KEY, descripcion TEXT, categoria TEXT);
CREATE TABLE cie10_dsm5_mapping (cie10_codigo TEXT, dsm5_codigo TEXT, PRIMARY KEY (cie10_codigo, dsm5_codigo));
```

### 5.2 Migraciones — Patrón Crítico

```rust
// infrastructure/src/migrations.rs
pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    // ⚠️ USAR execute_batch DIRECTO — NO split(';')
    // El split rompe CREATE TRIGGER (BEGIN...END; bloques)
    conn.execute_batch(MIGRATIONS)?;
    Ok(())
}
```

> **Lección aprendida:** `split(';')` naïf rompe triggers y vistas. SQLite ejecuta multi-statement nativamente.

---

## 6. IPC / Tauri v2 Command Surface

### 6.1 Registro de Comandos (main.rs)

```rust
.invoke_handler(tauri::generate_handler![
    // Pacientes
    create_patient, get_patient, list_patients, update_patient, delete_patient,
    search_patients, get_patient_count,
    // Contabilidad
    add_asiento, remove_asiento, list_asientos,
    generate_balance_general, generate_estado_resultados,
    // Diagnósticos
    search_cie10, search_dsm5, get_cie10_by_codigo, get_dsm5_by_codigo,
    create_mapeo, list_mapeos, update_mapeo, delete_mapeo,
    // Edad
    calculate_age, calculate_age_at, calculate_age_breakdown,
])
```

### 6.2 Convención de Nombres

| Dominio | Comando Tauri | Frontend API |
|---------|---------------|--------------|
| Pacientes | `create_patient` | `patientApi.create()` |
| Contabilidad | `add_asiento` | `accountingApi.addAsiento()` |
| Diagnósticos | `search_cie10` | `diagnosticsApi.searchCie10()` |
| Edad | `calculate_age` | `ageApi.calculate()` |

---

## 7. Testing Strategy — 124 Tests Verdes

| Capa | Tests | Enfoque |
|------|-------|---------|
| **Domain** | 61 | Property-based (proptest) + unitarios puros — invariantes contables, VOs |
| **Infrastructure** | 19 | Integración con `:memory:` pool — repositorios, migraciones, keyring |
| **Commands** | 22 | Inner functions (`*_impl`) + Tauri test harness — IPC contracts |
| **Docx** | 5 | Generación de informes clínicos |
| **E2E (Playwright)** | 17 | Flujos críticos: crear paciente → cita → nota → asiento → reporte |

**Ejecutar suite completa:**
```bash
# Rust (cargo nextest recomendado)
cd src-tauri && cargo nextest run --workspace

# Frontend (Vitest)
pnpm test

# E2E
pnpm test:e2e
```

---

## 8. Seguridad y Cumplimiento (LOPD Ecuador)

| Control | Implementación |
|---------|----------------|
| **Cifrado en reposo** | SQLCipher AES-256 (bundled) + PRAGMA key |
| **Gestión de claves** | Keyring nativo + fallback file (0o600) + Zeroize |
| **Integridad referencial** | `PRAGMA foreign_keys=ON` obligatorio |
| **Aislamiento tenant** | 1 BD por instalación — sin multi-tenancy lógico |
| **Auditoría** | `created_at`/`updated_at` en todas las tablas |
| **Borrado seguro** | `DELETE` + `VACUUM` (SQLCipher sobrescribe páginas) |

---

## 9. Decisiones Arquitectónicas Registradas (ADR Log)

| ADR | Decisión | Razón |
|-----|----------|-------|
| ADR-001 | Clean Architecture 4 capas | Testabilidad, separación dominio/infra, sustituibilidad BD |
| ADR-002 | SQLCipher + Keyring nativo | Cumplimiento LOPD, cero claves en disco plano |
| ADR-003 | `lto = "thin"` en release | Fat LTO corrompe SQLCipher FFI (evidencia empírica) |
| ADR-004 | Value Objects en Domain | Invalid state unrepresentable — falla rápido en constructor |
| ADR-005 | Double-entry en constructor Asiento | Invariante contable imposible de violar en runtime |
| ADR-006 | `execute_batch` para migraciones | `split(';')` rompe triggers/vistas |
| ADR-007 | TanStack Query v5 + Tauri invoke | Cache automático, invalidation, optimistic updates |
| ADR-008 | shadcn/ui + Radix + Tailwind | Accesibilidad nativa, theming via CSS variables, 0 runtime |

---

## 10. Glosario Técnico

| Término | Definición |
|---------|------------|
| **DbPool** | `Arc<Mutex<Connection>>` — Pool singleton de conexión SQLCipher |
| **Value Object** | Tipo inmutable que valida en construcción (DocumentNumber, Email, etc.) |
| **Aggregate Root** | Entidad con identidad global que garantiza consistencia (Patient, AsientoContable) |
| **Repository Trait** | Interfaz en `domain` — implementada en `infrastructure` (DIP) |
| **PRAGMA key** | Comando SQLCipher para establecer passphrase de cifrado (formato hex 64 chars) |
| **WAL Mode** | Write-Ahead Logging — fallback a DELETE si SQLCipher no soporta |
| **Zeroize** | Crate que garantiza limpieza de memoria sensible (claves) al drop |
| **LOPD Ecuador** | Ley Orgánica de Protección de Datos Personales — cifrado obligatorio datos sensibles |

---

## 11. Referencias Rápidas (Quick Reference)

```bash
# Build release (universal macOS)
cd src-tauri && cargo tauri build --target universal-apple-darwin

# Test solo dominio (rápido)
cd src-tauri && cargo test -p soft-gloria-domain

# Test integración infraestructura
cd src-tauri && cargo test -p soft-gloria-infrastructure

# Lint + format
cargo fmt --all --check && cargo clippy --workspace -- -D warnings

# Frontend dev
pnpm dev              # Vite + Tauri dev
pnpm build            # TypeScript + Vite build
pnpm test             # Vitest
pnpm test:e2e         # Playwright
```

---

> **Fin del Manual de Arquitectura**  
> Este documento es **vivo** — actualizar en cada ADR aprobado.  
> Próxima revisión: tras FASE 8 (Multi-tenant SaaS) o cambio de motor de BD.