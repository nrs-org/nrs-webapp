# Copilot Coding Agent Instructions for nrs-webapp

## Project Overview

This is **nrs-webapp**, a monolithic web application for NRS (Natural Resource Service) version 3.0, written in Rust. It uses a server-rendered architecture with Axum (HTTP framework), HTMX + Alpine.js for interactivity, and Tailwind CSS + DaisyUI for styling.

## Repository Structure

This is a **Cargo workspace** with the following crates:

| Crate | Type | Description |
|---|---|---|
| `nrs-webapp/` | Binary | Main HTTP server (Axum-based) |
| `nrs-webapp-core/` | Library | Shared data types, WASM-compatible (feature-gated `sql` for DB types) |
| `nrs-webapp-frontend/` | Library | Frontend views, templates (hypertext `rsx!` macro), and static assets |
| `nrs-webapp-keygen/` | Binary | CLI utility to generate secrets (password pepper, cookie keys) |
| `sqlbindable/` | Library | Custom SQL field-binding abstraction for type-safe queries |
| `sqlbindable-macros/` | Proc-macro | Derives `FieldNames` and `Fields` traits for sqlbindable |

### Key directories within `nrs-webapp/src/`

- `auth/` — Authentication logic (OAuth2, OpenID Connect, session management)
- `config.rs` — Configuration management (environment variables)
- `crypt/` — Cryptography (AES-GCM encryption, Argon2 password hashing, HMAC)
- `error.rs` — Top-level error enum with HTTP status mapping
- `extract/` — Axum request extractors
- `mail/` — Email sending (via Resend)
- `middleware/` — HTTP middleware (response mapping, auth checks)
- `model/` — Database models using BMC (Business Model Controller) pattern
- `routes/` — HTTP route handlers
- `toasts/` — Flash message/toast system
- `validate/` — Input validation
- `_dev_utils/` — Development-only utilities (DB seeding, compiled out in release)

## Build, Test, Lint, and Format Commands

### Prerequisites

- **Rust toolchain**: Edition 2024 (requires Rust 1.85+)
- **Package manager**: Cargo (Rust), pnpm (frontend assets)
- **Database**: PostgreSQL 17 (via Docker on port 3622 for dev)

### Commands

```sh
# Check compilation (fastest feedback)
cargo check --all

# Run all tests (unit tests + doc-tests across all crates)
cargo test --all

# Check formatting
cargo fmt -- --check

# Apply formatting fixes
cargo fmt

# Run linter (strict: warnings are errors)
cargo clippy --all-targets --all-features -- -D warnings

# Run a specific crate's tests
cargo test -p nrs-webapp
cargo test -p nrs-webapp-core
cargo test -p sqlbindable
```

### Database setup for tests

Tests that interact with the database require a PostgreSQL 17 instance. The dev database is **wiped and re-created on each test run**. Start it with:

```sh
docker run --rm --name pg -p 3622:5432 -e POSTGRES_PASSWORD=password -d postgres:17
```

> Do NOT change the password — `_dev_utils` expects `password` for the `postgres` superuser.

### Development environment variables

Dev-only environment variables are configured in `.cargo/config.toml` and are automatically loaded by Cargo during development. Key variables include `SERVICE_DB_URL`, `SERVICE_PASSWORD_PEPPER`, `SERVICE_COOKIE_KEY`, and others. See `.cargo/config.toml` for the full list.

## CI Pipeline

GitHub Actions runs on push/PR to `master` branch (`.github/workflows/ci.yml`):

1. **cargo check --all** — Compilation check
2. **cargo test --all** — All tests
3. **cargo fmt -- --check** — Formatting validation
4. **cargo clippy --all-targets --all-features -- -D warnings** — Linting (warnings = errors)
5. **cargo llvm-cov** — Code coverage (uploaded to Codecov)

There is also a pre-commit lint workflow (`.github/workflows/lint.yml`) that uses Nix to run pre-commit hooks (nixfmt, statix, YAML check, TOML check, rustfmt, trailing whitespace, end-of-file fixer).

### Known CI considerations

- The coverage job requires a `CODECOV_TOKEN` secret configured in the repository.
- The pre-commit lint workflow requires Nix. If Nix is not available, focus on `cargo fmt` and `cargo clippy` for validation.
- Tests that need PostgreSQL will fail if no database is running. Unit tests in `nrs-webapp-core`, `sqlbindable`, and `nrs-webapp-keygen` do not require a database.

## Architecture and Coding Patterns

### Error handling

- Each module defines its own `Error` enum using `thiserror`.
- Errors compose upward via `#[from]` attributes: `model::Error` → `crate::Error`, `auth::Error` → `crate::Error`, etc.
- The top-level `Error` in `nrs-webapp/src/error.rs` implements `IntoResponse` to map errors to HTTP status codes.
- A `Result<T>` type alias is defined in each module for convenience.
- Sensitive error details are logged server-side; clients receive generic messages.

### Model layer (BMC pattern)

- Each entity has a `*Bmc` struct (e.g., `UserBmc`, `EntryBmc`) implementing the `DbBmc` and `DbBmcWithPkey` traits.
- Multiple struct variants per entity for different operations: `*ForCreate`, `*ForUpdate`, `*ForAuth`, etc.
- SQL queries are built with **sea-query** and executed with **sqlx**.
- The `sqlbindable` crate provides `#[derive(FieldNames, Fields)]` for type-safe field binding.
- `ModelManager` is the central service object holding the DB pool, auth providers, and HTTP client.
- `PrimaryStore` trait abstracts over connection pool vs. transaction for flexible query execution.

### Route/handler pattern

- Each route module exports a `router()` function returning `Router<ModelManager>`.
- Handlers use Axum extractors: `State(mm)`, `Path(id)`, `HxRequest`, `DocProps(props)`.
- `HxRequest` from `axum-htmx` distinguishes HTMX requests (return HTML fragments) from full page loads (return complete documents).
- The `maybe_document()` helper wraps content in a full HTML document or returns a fragment based on the request type.

### Frontend view pattern

- Views use the `hypertext` crate with `rsx!` macro (Rust JSX-like syntax).
- View functions return `impl Renderable` and are decorated with `#[component]`.
- Views are organized under `nrs-webapp-frontend/src/views/`: `pages/`, `components/`, `email/`, `document.rs`, `error.rs`.
- Static assets are in `nrs-webapp-frontend/static/serve/` with generated CSS output.
- Frontend libraries: HTMX (4.0.0-alpha4), Alpine.js (3.15.2), Tailwind CSS (4.x) + DaisyUI (5.x).

### Testing conventions

- Tests use `#[cfg(test)]` modules within source files (no separate test directory).
- Doc-tests are used extensively in `nrs-webapp-frontend` for view rendering validation.
- Database tests wipe and re-create the dev database on each run.
- The project has 11 unit tests and 18 doc-tests across the workspace.

## Things to Watch Out For

- **Rust edition 2024**: This project uses `edition = "2024"` which requires Rust 1.85+. Some syntax and behavior may differ from earlier editions.
- **sea-query release candidates**: The project uses `sea-query v1.0.0-rc.x` and `sea-query-sqlx v0.8.0-rc.14` — these are pre-release versions. Check compatibility when updating.
- **HTMX alpha**: The frontend uses `htmx.org@4.0.0-alpha4` — an alpha release. Be cautious with HTMX API assumptions.
- **Dev-only secrets in `.cargo/config.toml`**: These are intentionally committed for development convenience. Never use them in production.
- **The `_dev_utils` module** is conditionally compiled with `#[cfg(debug_assertions)]` — it only exists in debug builds.
- **WASM compilation**: `nrs-webapp-core` can be compiled to WebAssembly via `wasm-bindgen`. The build pipeline uses `bacon build-core-wasm` which chains `cargo rustc --target wasm32-unknown-unknown` with `wasm-bindgen`.
