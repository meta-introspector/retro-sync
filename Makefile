# Retrosync Media Group — Developer Makefile
#
# All commands run inside `nix develop` for a hermetic, reproducible
# environment (pinned Rust toolchain, Foundry, Bun, Node).
# To drop into the dev shell manually:  make shell
#
# Prerequisites: Nix with flakes enabled.
#   curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh
#
# Quick start:
#   make          # build everything
#   make check    # fmt + clippy + forge fmt
#   make test     # all test suites
#   make dev      # start the Vite dev server

NIX   := nix develop --command
CARGO := $(NIX) cargo
BUN   := $(NIX) bun
NPM   := $(NIX) npm
FORGE := $(NIX) forge

# ── Top-level targets ───────────────────────────────────────────────────────

.PHONY: all
all: rust frontend contracts

.PHONY: dev
dev:
        $(NPM) --prefix apps/web-client run dev

.PHONY: check
check: rust-fmt rust-clippy forge-fmt treefmt-check

.PHONY: test
test: rust-test contracts-test

.PHONY: clean
clean: rust-clean frontend-clean

.PHONY: shell
shell:
        nix develop

# ── Rust / Backend ──────────────────────────────────────────────────────────

.PHONY: rust
rust:
        $(CARGO) build --workspace

.PHONY: rust-release
rust-release:
        $(CARGO) build --workspace --release

.PHONY: rust-test
rust-test:
        $(CARGO) test --all

.PHONY: rust-fmt
rust-fmt:
        $(CARGO) fmt --all -- --check

.PHONY: rust-fmt-fix
rust-fmt-fix:
        $(CARGO) fmt --all

.PHONY: rust-clippy
rust-clippy:
        $(CARGO) clippy --all -- -D warnings

.PHONY: rust-clean
rust-clean:
        $(CARGO) clean

# Key source directories:
#   apps/api-server/src/   — Axum HTTP server (main binary)
#   apps/wasm-frontend/    — Rust/Yew WASM frontend
#   libs/shared/src/       — shared parsers, types, master-pattern
#   libs/zk-circuits/src/  — arkworks Groth16 ZK circuit

# ── WASM Frontend ───────────────────────────────────────────────────────────

.PHONY: wasm
wasm:
        $(NIX) trunk build --release apps/wasm-frontend/index.html

.PHONY: wasm-dev
wasm-dev:
        $(NIX) trunk serve apps/wasm-frontend/index.html

# ── React / Vite Frontend ───────────────────────────────────────────────────

.PHONY: frontend
frontend: frontend-install frontend-build

.PHONY: frontend-install
frontend-install:
        $(NPM) --prefix apps/web-client install --legacy-peer-deps

.PHONY: frontend-build
frontend-build:
        $(NPM) --prefix apps/web-client run build

.PHONY: frontend-typecheck
frontend-typecheck:
        $(NPM) --prefix apps/web-client exec tsc -- --noEmit

.PHONY: frontend-clean
frontend-clean:
        rm -rf apps/web-client/dist apps/web-client/node_modules

# ── Contracts (Foundry) ─────────────────────────────────────────────────────

.PHONY: contracts
contracts:
        cd contracts && $(FORGE) build

.PHONY: contracts-test
contracts-test:
        cd contracts && $(FORGE) test --fuzz-runs 2000 -v

.PHONY: contracts-coverage
contracts-coverage:
        cd contracts && $(FORGE) coverage

.PHONY: forge-fmt
forge-fmt:
        cd contracts && $(FORGE) fmt --check

.PHONY: forge-fmt-fix
forge-fmt-fix:
        cd contracts && $(FORGE) fmt

# Deploy to BTTC testnet (requires Ledger hardware wallet):
#   make deploy RPC=<bttc-rpc-url>
.PHONY: deploy
deploy:
        cd contracts && $(FORGE) script script/Deploy.s.sol:DeployScript \
                --rpc-url $(RPC) \
                --ledger \
                --hd-paths "m/44'/60'/0'/0/0" \
                --legacy \
                --broadcast

# ── treefmt ─────────────────────────────────────────────────────────────────
# treefmt is the unified formatter wired in flake.nix (rustfmt + nixpkgs-fmt +
# prettier + forge fmt).  `nix fmt` runs it via the flake formatter output.

.PHONY: treefmt-check
treefmt-check:
        $(NIX) treefmt --fail-on-change

.PHONY: treefmt-fix
treefmt-fix:
        $(NIX) treefmt

.PHONY: fmt
fmt: rust-fmt-fix treefmt-fix forge-fmt-fix

# ── Nix ─────────────────────────────────────────────────────────────────────

.PHONY: nix-check
nix-check:
        nix flake check --all-systems

.PHONY: nix-build-backend
nix-build-backend:
        nix build .#backend

.PHONY: nix-build-frontend
nix-build-frontend:
        nix build .#frontend

.PHONY: nix-build
nix-build: nix-build-backend nix-build-frontend

.PHONY: nix-update
nix-update:
        nix flake update

# ── Help ────────────────────────────────────────────────────────────────────

.PHONY: help
help:
        @echo "Retrosync Media Group — Makefile targets"
        @echo ""
        @echo "  make              Build Rust workspace + frontend + contracts"
        @echo "  make dev          Start Vite dev server (apps/web-client)"
        @echo "  make check        rustfmt + clippy + forge fmt (no changes)"
        @echo "  make test         cargo test --all + forge test --fuzz-runs 2000"
        @echo "  make clean        Remove build artifacts"
        @echo "  make shell        Drop into nix develop shell"
        @echo ""
        @echo "  make rust         cargo build --workspace"
        @echo "  make rust-release cargo build --workspace --release"
        @echo "  make rust-test    cargo test --all"
        @echo "  make rust-clippy  cargo clippy --all -- -D warnings"
        @echo "  make rust-fmt-fix cargo fmt --all (auto-fix)"
        @echo ""
        @echo "  make wasm         trunk build --release (apps/wasm-frontend)"
        @echo "  make wasm-dev     trunk serve (apps/wasm-frontend)"
        @echo ""
        @echo "  make frontend     npm install + npm run build (apps/web-client)"
        @echo "  make frontend-typecheck  tsc --noEmit"
        @echo ""
        @echo "  make contracts    forge build (contracts/)"
        @echo "  make contracts-test      forge test --fuzz-runs 2000"
        @echo "  make contracts-coverage  forge coverage"
        @echo "  make deploy RPC=<url>    Deploy to BTTC testnet via Ledger"
        @echo ""
        @echo "  make nix-check    nix flake check --all-systems"
        @echo "  make nix-build    nix build .#backend .#frontend"
        @echo "  make nix-update   nix flake update"
        @echo ""
        @echo "  make fmt          rustfmt + treefmt + forge fmt (auto-fix all)"
        @echo "  make treefmt-check  treefmt --fail-on-change (CI mode)"
        @echo "  make treefmt-fix    treefmt (auto-fix all formatters)"
