# AGENTS.md — MindLedger Coding Standards

## Architecture

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **Database**: SQLCipher (encrypted SQLite)
- **State**: Zustand (global) + React Query (server)
- **Styling**: Tailwind CSS

## Frontend Standards (TypeScript/React)

### Components
- Functional components only, no class components
- Props must be typed with explicit interfaces
- No inline styles — use Tailwind classes
- Component files: PascalCase (`UserCard.tsx`)
- Hooks files: camelCase with `use` prefix (`useAuth.ts`)

### State Management
- Zustand for global client state
- React Query for server/database state
- No prop drilling beyond 2 levels
- Prefer derived state over stored state

### Security
- No hardcoded secrets or encryption keys
- Environment variables for sensitive config
- Input validation on all user-facing forms
- Sanitize any HTML/user-generated content
- Never log sensitive data (keys, passwords, PII)

### Tauri IPC
- Use `invoke()` with typed commands
- Validate all inputs at Rust boundary
- Return structured `Result<T, String>` from Rust

## Backend Standards (Rust)

### Code Style
- Follow `rustfmt` defaults
- Use `clippy` with no warnings
- Prefer `thiserror` for custom errors
- Use `anyhow` for application-level errors
- Document public functions with `///` doc comments

### Security
- Use parameterized SQL queries (never string interpolation)
- Validate encryption key format before use
- Handle errors gracefully — no `unwrap()` in production paths
- Use `secrecy` crate for sensitive values when appropriate
- Ensure key file permissions are 0o600 on Unix

### Database
- All migrations must be reversible
- Use transactions for multi-step operations
- Encrypt sensitive fields at rest
- Index foreign keys and frequently queried columns

## Code Review Checklist

- [ ] TypeScript compiles without errors (`tsc --noEmit`)
- [ ] Rust compiles without warnings (`cargo clippy`)
- [ ] No `console.log` or `println!` in production code
- [ ] Error handling covers all failure paths
- [ ] No hardcoded secrets, keys, or credentials
- [ ] SQL queries use parameterized statements
- [ ] New functions have doc comments
- [ ] No unused imports or variables
- [ ] Tailwind classes used (no inline styles)
- [ ] Tauri IPC commands have typed request/response
