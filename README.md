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

### storage

Google Cloud Storage component. Exports `gcloud:storage/buckets` interface.

```bash
cd storage
make build
```

### secretmanager

Google Cloud Secret Manager component. Exports `gcloud:secretmanager/secrets` interface.

```bash
cd secretmanager
make build
```

## Examples

### Prerequisites

- Rust (`wasm32-wasip2` target)
- [wkg](https://github.com/bytecodealliance/wasm-pkg-tools) (WIT dependency management)
- [wac](https://github.com/bytecodealliance/wac) (component composition)
- [wasmtime](https://wasmtime.dev/)

### examples/auth

Demonstrates using the `gcloud:auth/token-source` interface to obtain an access token.

```bash
cd examples/auth
make build
make run
```

### examples/storage

Demonstrates using the `gcloud:storage/buckets` interface to list buckets.

```bash
cd examples/storage
make build
GOOGLE_CLOUD_PROJECT=your-project make run
```

### examples/secretmanager

Demonstrates using the `gcloud:secretmanager/secrets` interface to access a secret.

```bash
cd examples/secretmanager
make build
SECRET_NAME=projects/your-project/secrets/your-secret/versions/latest make run
```

### How it works

1. `cargo build --target wasm32-wasip2` builds each component/example as a WASM component
2. `wac compose` composes components using a declarative `compose.wac` file to satisfy imports
3. `wasmtime run` executes the composed component
