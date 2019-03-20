CARGO = cargo

all: build

prebuild:
	@mkdir -p build

build: prebuild
	@cd rust/lib/c64 && \
	$(CARGO) build --release --target wasm32-unknown-unknown --verbose && \
	cp target/wasm32-unknown-unknown/release/rustc64lib.wasm ../../../build/wasm_c64.wasm

test:
	@cd rust && $(CARGO) test

clean:
	@cd rust && $(CARGO) clean

.PHONY: all prebuild build test clean
