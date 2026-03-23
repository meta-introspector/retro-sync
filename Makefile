SHELL := bash
NIX   := nix --extra-experimental-features 'nix-command flakes'

.PHONY: dev test build wasm clean

# Enter dev shell (interactive)
dev:
	$(NIX) develop

# Run all tests inside the flake dev shell
test:
	$(NIX) develop --command cargo test -p stego

# Build the backend
build:
	$(NIX) develop --command cargo build --release -p backend

# Build stego WASM for the browser viewer
wasm:
	$(NIX) develop --command bash -c '\
		cargo build -p stego --target wasm32-unknown-unknown --release --features wasm && \
		wasm-bindgen target/wasm32-unknown-unknown/release/stego.wasm \
			--out-dir docs/pkg --target web --no-typescript'

# Test just the stego crate
test-stego:
	$(NIX) develop --command cargo test -p stego -- --nocapture

# Build the nix package
nix-build:
	$(NIX) build

clean:
	$(NIX) develop --command cargo clean
