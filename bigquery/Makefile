SRC := $(shell find src -name "*.rs")
CONFIG := Cargo.toml Cargo.lock
WIT := $(shell find wit -name "*.wit")

PACKAGE_NAME := $(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].name')

.PHONY: build
build: target/wasm32-wasip2/release/$(PACKAGE_NAME).wasm

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clean
clean:
	cargo clean

target/wasm32-wasip2/release/%.wasm: $(CONFIG) $(SRC) $(WIT)
	cargo build --release --target wasm32-wasip2

wit/deps: wit/*.wit wkg.toml
	wkg wit fetch
