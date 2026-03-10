SRC := $(shell find src -name "*.rs")
CONFIG := Cargo.toml Cargo.lock
WIT := $(shell find wit -name "*.wit")

PACKAGE_NAME := $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')

AUTH_WASM := ../../auth/target/wasm32-wasip2/release/gcloud_auth.wasm
EXAMPLE_WASM := target/wasm32-wasip2/release/$(PACKAGE_NAME).wasm
COMPOSED_WASM := target/wasm32-wasip2/release/composed.wasm

.PHONY: build
build: $(COMPOSED_WASM)

.PHONY: run
run: $(COMPOSED_WASM)
	wasmtime run --env HOME="$$HOME" --dir "$$HOME/.config/gcloud::$$HOME/.config/gcloud" -S http $(COMPOSED_WASM)

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clean
clean:
	cargo clean

$(AUTH_WASM):
	$(MAKE) -C ../../auth build

wit/deps: wkg.toml wit/world.wit
	wkg wit fetch

$(EXAMPLE_WASM): $(CONFIG) $(SRC) $(WIT) wit/deps
	cargo build --release --target wasm32-wasip2

$(COMPOSED_WASM): $(EXAMPLE_WASM) $(AUTH_WASM)
	wac plug $(EXAMPLE_WASM) --plug $(AUTH_WASM) -o $(COMPOSED_WASM)
