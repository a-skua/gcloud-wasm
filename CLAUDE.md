# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Google Cloud APIs as WASM components targeting WASI Preview 2 (`wasm32-wasip2`).

## Build Commands

Each crate has its own Makefile. Run commands from the respective directory.

```bash
make build       # cargo build --release --target wasm32-wasip2
make fmt         # cargo fmt
make wit/deps    # wkg wit fetch (update WIT dependencies)
```

No test suite or linter is configured.

## Design Policy

- API design and naming should follow the official Google Cloud SDK conventions (e.g. `upload-object` instead of `insert-object`)
- Refer to the Google Cloud REST API documentation and official client libraries when adding new operations
