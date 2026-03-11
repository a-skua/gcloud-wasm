SRC := $(shell find src -name "*.rs")
CONFIG := Cargo.toml Cargo.lock
WIT := $(shell find wit -name "*.wit")

PACKAGE_NAME := $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')

AUTH_WASM_REF :=  ghcr.io/a-skua/gcloud/auth:0.1.0
AUTH_WASM := target/wasm32-wasip2/release/gcloud_auth_0_1_0.wasm
SECRETMANAGER_WASM_REF := ghcr.io/a-skua/gcloud/secretmanager:0.1.0
SECRETMANAGER_WASM := target/wasm32-wasip2/release/gcloud_secretmanager_0_1_0.wasm
EXAMPLE_WASM := target/wasm32-wasip2/release/$(PACKAGE_NAME).wasm
COMPOSED_WASM := target/wasm32-wasip2/release/composed.wasm

.PHONY: build
build: $(COMPOSED_WASM)

.PHONY: run
run: $(COMPOSED_WASM)
	wasmtime run --env HOME="$$HOME" --env SECRET_NAME="$$SECRET_NAME" --dir "$$HOME/.config/gcloud::$$HOME/.config/gcloud" -S http $(COMPOSED_WASM)

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clean
clean:
	cargo clean

wit/deps: wkg.toml wit/world.wit
	wkg wit fetch

$(AUTH_WASM):
	wkg oci pull -o $@ $(AUTH_WASM_REF)

$(SECRETMANAGER_WASM):
	wkg oci pull -o $@ $(SECRETMANAGER_WASM_REF)

$(EXAMPLE_WASM): $(CONFIG) $(SRC) $(WIT) wit/deps
	cargo build --release --target wasm32-wasip2

$(COMPOSED_WASM): compose.wac $(EXAMPLE_WASM) $(AUTH_WASM) $(SECRETMANAGER_WASM)
	wac compose compose.wac \
		-d "gcloud:auth=$(AUTH_WASM)" \
		-d "gcloud:secretmanager=$(SECRETMANAGER_WASM)" \
		-d "example:secretmanager=$(EXAMPLE_WASM)" \
		-o $@
