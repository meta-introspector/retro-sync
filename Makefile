NIX = nix develop --command
CARGO = $(NIX) cargo
BUN = $(NIX) bun

.PHONY: all rust frontend clean shell

all: rust frontend

rust:
	$(CARGO) build --workspace

frontend:
	$(BUN) install --frozen-lockfile
	$(BUN) run build

clean:
	$(CARGO) clean
	rm -rf dist node_modules

shell:
	nix develop
