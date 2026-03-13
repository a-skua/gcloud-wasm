SRC := $(shell find src -name "*.rs")
CONFIG := Cargo.toml Cargo.lock
WIT := $(shell find wit -name "*.wit")

PACKAGE_NAME := $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')

AUTH_WASM_SRC := ghcr.io/a-skua/gcloud/auth:0.2.0
AUTH_WASM := target/wasm32-wasip2/release/gcloud_auth_0_2_0.wasm
STORAGE_WASM_SRC := ghcr.io/a-skua/gcloud/storage:0.2.0
STORAGE_WASM := target/wasm32-wasip2/release/gcloud_storage_0_2_0.wasm
EXAMPLE_WASM := target/wasm32-wasip2/release/$(PACKAGE_NAME).wasm
COMPOSED_WASM := target/wasm32-wasip2/release/composed.wasm

.PHONY: build
build: $(COMPOSED_WASM)

.PHONY: run
run: $(COMPOSED_WASM)
	wasmtime run --env HOME="$$HOME" --env GOOGLE_CLOUD_PROJECT="$$GOOGLE_CLOUD_PROJECT" --dir "$$HOME/.config/gcloud::$$HOME/.config/gcloud" -S http $(COMPOSED_WASM)

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clean
clean:
	cargo clean

$(AUTH_WASM):
	wkg oci pull -o $@ $(AUTH_WASM_SRC)

$(STORAGE_WASM):
	wkg oci pull -o $@ $(STORAGE_WASM_SRC)

$(EXAMPLE_WASM): $(CONFIG) $(SRC) $(WIT)
	cargo build --release --target wasm32-wasip2

$(COMPOSED_WASM): compose.wac $(EXAMPLE_WASM) $(AUTH_WASM) $(STORAGE_WASM)
	wac compose compose.wac \
		-d "gcloud:auth=$(AUTH_WASM)" \
		-d "gcloud:storage=$(STORAGE_WASM)" \
		-d "example:storage=$(EXAMPLE_WASM)" \
		-o $@
