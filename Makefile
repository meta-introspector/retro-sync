SHELL := bash
NIX   := nix --extra-experimental-features 'nix-command flakes'
RUN   := $(NIX) develop --command

# Repos
SPACE_DIR  := ../retro-sync-space
GH_REMOTE  := meta-introspector

# Source inputs
LY     := fixtures/lilypond/h6_west.ly
SRC    := fixtures/data/hurrian_h6.txt

# Generated outputs
OUT     := fixtures/output
WAV     := $(OUT)/h6_west.wav
MIDI    := $(OUT)/h6_west.midi
PDF     := $(OUT)/h6_west.pdf
SVG_DIR := $(OUT)/nft71_svg
PNG_DIR := $(OUT)/nft71_stego_png
WASM    := docs/pkg/stego_bg.wasm

.PHONY: all dev test test-stego render svg stego wasm build \
        pipeline deploy deploy-hf-space deploy-hf-data deploy-gh clean

all: pipeline

dev:
	$(NIX) develop

# ── Tests ──────────────────────────────────────────────────────────
test:
	$(RUN) cargo test -p stego

test-stego:
	$(RUN) cargo test -p stego -- --nocapture

# ── Step 1: LilyPond → MIDI + PDF, FluidSynth → WAV ──────────────
$(MIDI) $(PDF): $(LY)
	$(RUN) bash fixtures/scripts/render.sh $(LY) $(OUT)

$(WAV): $(MIDI)

render: $(WAV)

# ── Step 2: SVG tiles ─────────────────────────────────────────────
$(SVG_DIR)/01.svg: $(SRC)
	$(RUN) cargo run -p fixtures --example nft71_svg

svg: $(SVG_DIR)/01.svg
	@echo "→ Open $(SVG_DIR)/gallery.html to inspect"

# ── Step 3: Stego embed → PNG ─────────────────────────────────────
$(PNG_DIR)/01.png: $(SVG_DIR)/01.svg $(WAV) $(MIDI) $(PDF)
	$(RUN) cargo run -p fixtures --example nft71_stego_svg

stego: $(PNG_DIR)/01.png

# ── Step 4: WASM ──────────────────────────────────────────────────
$(WASM): libs/stego/src/lib.rs libs/stego/Cargo.toml
	$(RUN) bash -c '\
		cargo build -p stego --target wasm32-unknown-unknown --release --features wasm && \
		wasm-bindgen target/wasm32-unknown-unknown/release/stego.wasm \
			--out-dir docs/pkg --target web --no-typescript'

wasm: $(WASM)

# ── Step 5: Backend ───────────────────────────────────────────────
build:
	$(RUN) cargo build --release -p backend

# ── Pipeline (test + build all artifacts) ─────────────────────────
pipeline: test stego wasm
	@echo "=== pipeline complete ==="
	@echo "  tiles: $(PNG_DIR)/"
	@echo "  wasm:  docs/pkg/"
	@echo "  svg:   $(SVG_DIR)/"
	@echo "Run 'make deploy' to publish"

# ── Deploy ────────────────────────────────────────────────────────

# HF Space: viewer HTML via git, binaries (tiles, wasm) via API
deploy-hf-space: pipeline
	cp docs/index.html $(SPACE_DIR)/index.html
	cd $(SPACE_DIR) && git add index.html && \
		git commit -m "deploy: viewer $$(date -u +%Y%m%dT%H%M%SZ)" && \
		git push origin main
	python3 tools/upload_hf.py space

# HF Dataset: tiles via API
deploy-hf-data: pipeline
	python3 tools/upload_hf.py dataset

# GitHub: commit source + docs
deploy-gh: pipeline
	git add libs/stego docs/ Makefile flake.nix fixtures/examples/ fixtures/Cargo.toml tools/
	git commit -m "deploy: pipeline $$(date -u +%Y%m%dT%H%M%SZ)" || true
	git push $(GH_REMOTE) main

# All targets
deploy: deploy-hf-space deploy-hf-data deploy-gh
	@echo "=== deployed to HF Space + Dataset + GitHub ==="

# ── Nix build ─────────────────────────────────────────────────────
nix-build:
	$(NIX) build

clean:
	$(RUN) cargo clean
