# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build

```bash
# Build the WASM component
cargo build --target wasm32-wasip2

# Release build
cargo build --target wasm32-wasip2 --release
```

Target: `wasm32-wasip2`. Output: `target/wasm32-wasip2/{debug,release}/auth.wasm`

No test suite yet. No linter configured.

## Architecture

Google Cloud authentication library as a WASM component (WASI Preview 2).

### WIT Interface (`wit/auth.wit`)

Package `gcloud:auth@0.1.0` defines:
- **`types`** interface — `token` record (access-token, token-type, expires-in) and `error` variant
- **`token-source`** interface — `get-token(scopes) -> result<token, error>`
- **`token-provider`** world — exports `token-source`

### Implementation (`src/lib.rs`)

- `wit-bindgen::generate!` generates export bindings from the WIT world
- `wstd::runtime::block_on` bridges async HTTP (via `wstd::http::Client`) into the synchronous WIT export
- ADC (Application Default Credentials) flow: reads JSON from `GOOGLE_APPLICATION_CREDENTIALS` env var, falls back to `~/.config/gcloud/application_default_credentials.json`
- `Adc` enum uses `#[serde(tag = "type")]` for tagged dispatch — currently supports `authorized_user` (refresh token flow), extensible to `service_account` etc.
- Token endpoint: `POST https://oauth2.googleapis.com/token` with form-encoded refresh token grant
