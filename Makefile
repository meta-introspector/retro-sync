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

.PHONY: all dev test test-stego render render-all svg stego wasm build \
        witness notes erdfa pipeline deploy deploy-hf-space deploy-hf-data deploy-gh clean

all: pipeline

dev:
	$(NIX) develop

# ── Tests ──────────────────────────────────────────────────────────
test:
	$(RUN) cargo test -p stego

test-stego:
	$(RUN) cargo test -p stego -- --nocapture

# ── Project-based targets (ISO 9001 SOP-ONBOARD) ─────────────
PROJECT ?= bach-invention
P_DIR   := projects/$(PROJECT)
P_SVG   := $(P_DIR)/output/svg
P_STEGO := $(P_DIR)/output/stego
P_QA    := $(P_DIR)/output/qa_report.txt

.PHONY: onboard midi2svg stego-project verify-project qa deploy-project

onboard:
	bash scripts/onboard.sh "$(TERM)" -n $(N)

midi2svg:
	bash scripts/midi2svg.sh $(P_DIR)/midi $(P_SVG) 71

stego-project: midi2svg
	@mkdir -p $(P_STEGO)
	cp $(P_SVG)/*.svg fixtures/output/nft71_svg/
	$(RUN) cargo run --release --example nft71_stego_svg -p fixtures
	cp fixtures/output/nft71_stego_png/*.png $(P_STEGO)/
	@echo "→ 71 stego PNGs in $(P_STEGO)/"

verify-project: stego-project
	$(RUN) cargo run --release --example verify_stego -p fixtures

qa: verify-project
	@echo "=== QA REPORT ===" > $(P_QA)
	@echo "Project: $(PROJECT)" >> $(P_QA)
	@echo "Date: $$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> $(P_QA)
	@echo "Tiles: $$(ls $(P_STEGO)/*.png | wc -l)" >> $(P_QA)
	@convert $(P_SVG)/01.svg -resize 512x512! /tmp/qa_ref.png 2>/dev/null && \
		magick compare -metric PSNR /tmp/qa_ref.png $(P_STEGO)/01.png /dev/null 2>&1 | \
		tee -a $(P_QA) || echo "PSNR: skipped" >> $(P_QA)
	@tesseract $(P_STEGO)/01.png stdout 2>/dev/null | head -3 >> $(P_QA) || echo "OCR: skipped" >> $(P_QA)
	@cat $(P_QA)

deploy-project: qa
	python3 tools/upload_hf.py dataset --project $(PROJECT)

# ── Server build + deploy (ISO 9001) ─────────────────────────
.PHONY: build-server install-service start stop status

build-server:
	$(NIX) build .#backend

build-server-dev:
	$(RUN) cargo build --release -p backend

install-service: build-server-dev
	sudo cp ops/retro-sync.service /etc/systemd/system/
	sudo systemctl daemon-reload
	sudo systemctl enable retro-sync
	@echo "✅ service installed"

start: install-service
	sudo systemctl start retro-sync
	@sleep 1
	@curl -sf http://localhost:8443/health && echo " ✅ healthy" || echo " ⚠ not responding"

stop:
	sudo systemctl stop retro-sync

status:
	@systemctl is-active retro-sync 2>/dev/null || echo "stopped"
	@curl -sf http://localhost:8443/health 2>/dev/null && echo "healthy" || echo "unreachable"

# ── Catalog + publish ────────────────────────────────────────
.PHONY: catalog publish

catalog:
	python3 scripts/catalog-gen.py
	python3 scripts/artist-ids.py
	python3 scripts/export-cwr.py

publish: catalog
	bash scripts/publish-catalog.sh

# ── Legacy targets (Hurrian — now in separate repo) ──────────────
SOURCES_DIR := $(OUT)/sources

witness:
	$(RUN) bash fixtures/scripts/witness_sources.sh $(SOURCES_DIR)

# ── Step 0b: Extract notes from witnessed audio ──────────────────
notes: witness
	$(RUN) bash fixtures/scripts/extract_notes.sh $(SOURCES_DIR)/audio $(SOURCES_DIR)/analysis

# ── Step 0c: Convert all note extractions to lilypond → MIDI → WAV
ANALYSIS_DIR := $(SOURCES_DIR)/analysis
render-all: notes
	@for f in $(ANALYSIS_DIR)/yt_*.notes; do \
	  base=$$(basename "$$f" .notes); \
	  python3 fixtures/scripts/notes_to_ly.py "$$f" "fixtures/lilypond/$${base}.ly"; \
	done
	@for ly in fixtures/lilypond/yt_*.ly; do \
	  base=$$(basename "$$ly" .ly); \
	  $(RUN) bash fixtures/scripts/render.sh "$$ly" $(OUT); \
	done

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

# ── Step 0c: Encode all artifacts as DA51 CBOR shards ────────────
erdfa: stego notes
	$(RUN) cargo run --example nft71_erdfa -p fixtures

# ── Verify stego round-trip ──────────────────────────────────────
verify: stego
	$(RUN) cargo run --example verify_stego -p fixtures

# ── Pipeline (test + build all artifacts) ─────────────────────────
pipeline: test stego wasm erdfa
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

## ── NotebookLM ──────────────────────────────────────────────────────
notebooklm: dist/notebooklm-dump.txt
dist/notebooklm-dump.txt: scripts/notebooklm-dump.sh
	@mkdir -p dist
	bash scripts/notebooklm-dump.sh > $@
	@ls -lh $@
