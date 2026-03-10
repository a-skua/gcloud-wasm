# gcloud.wasm

Google Cloud APIs as WebAssembly Components (WASI Preview 2).

## Components

### auth

Google Cloud authentication component. Exports `gcloud:auth/token-source` interface.

Supports Application Default Credentials (ADC) with `authorized_user` refresh token flow.

```bash
cd auth
make build
```

## Examples

### examples/auth

Demonstrates using the `gcloud:auth/token-source` interface to obtain an access token.

#### Prerequisites

- Rust (`wasm32-wasip2` target)
- [wkg](https://github.com/bytecodealliance/wasm-pkg-tools) (WIT dependency management)
- [wac](https://github.com/bytecodealliance/wac) (component composition)
- [wasmtime](https://wasmtime.dev/)

#### Build & Run

```bash
cd examples/auth
wkg wit fetch        # fetch WIT dependencies
make build           # build & compose
make run             # run with wasmtime
```

#### How it works

1. `wkg wit fetch` resolves WIT dependencies defined in `wkg.toml`
2. `cargo build --target wasm32-wasip2` builds the example as a WASM component that imports `gcloud:auth/token-source`
3. `wac plug` composes the example with `auth.wasm` to satisfy the import
4. `wasmtime run` executes the composed component
