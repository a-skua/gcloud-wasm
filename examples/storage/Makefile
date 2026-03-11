SRC := $(shell find src -name "*.rs")
CONFIG := Cargo.toml Cargo.lock
WIT := $(shell find wit -name "*.wit")

PACKAGE_NAME := $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')

AUTH_WASM_SRC := ghcr.io/a-skua/gcloud/auth:0.1.0
AUTH_WASM := target/wasm32-wasip2/release/gcloud_auth_0_1_0.wasm
STORAGE_WASM_SRC := ghcr.io/a-skua/gcloud/storage:0.1.0
STORAGE_WASM := target/wasm32-wasip2/release/gcloud_storage_0_1_0.wasm
COMPOSED_STORAGE_WASM := target/wasm32-wasip2/release/composed_storage.wasm
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

$(COMPOSED_STORAGE_WASM): $(STORAGE_WASM) $(AUTH_WASM)
	wac plug $< --plug $(word 2, $^) -o $@

$(COMPOSED_WASM): $(EXAMPLE_WASM) $(COMPOSED_STORAGE_WASM)
	wac plug $< --plug $(word 2, $^) -o $@
