# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Google Cloud APIs as WASM components targeting WASI Preview 2 (`wasm32-wasip2`). Each component is an independent Rust crate (no workspace) using Rust edition 2024.

## Build Commands

Each crate has its own Makefile. Run commands from the respective directory.

```bash
# Build a component (from auth/ or storage/)
make build                    # cargo build --release --target wasm32-wasip2

# Format
make fmt                      # cargo fmt

# Clean
make clean                    # cargo clean

# Examples (from examples/auth/ or examples/storage/)
make build                    # fetches deps from OCI registry, builds, composes with wac plug
make run                      # runs composed WASM with wasmtime
```

No test suite or linter is configured.

## Architecture

### Component Model

Components communicate through WIT (WebAssembly Interface Type) definitions. Each component is a `cdylib` crate using `wit-bindgen::generate!` for bindings.

- **auth/** — Exports `gcloud:auth/token-source` (get-token with ADC flow)
- **storage/** — Imports `gcloud:auth/token-source`, exports `gcloud:storage/buckets` (list-buckets)
- **examples/** — Import-only apps (no WIT exports, `main()` entry point)

Dependency chain: `example → storage → auth`

### Component Composition

Components are composed at build time using `wac plug` (not the deprecated `wasm-tools compose`):

```bash
wac plug <consumer>.wasm --plug <provider>.wasm -o composed.wasm
```

### WIT Dependency Management

- `wkg` manages WIT dependencies (fetch, lock, OCI push/pull)
- `wkg.toml` in storage/ overrides `gcloud:auth` to use local path during development
- WIT definitions live in each crate's `wit/` directory

### Runtime

- `wstd::runtime::block_on` bridges async HTTP into synchronous WIT exports
- `wasmtime` executes composed WASM modules with filesystem/env/HTTP capabilities

### Distribution

Components are published to GHCR as OCI artifacts via `wkg oci push` (see `.github/workflows/release.yaml`).
