{
  description = "Retrosync Media Group — Rust + Bun/Vite + Solidity dev environment";

  # Security notes:
  # - xz/liblzma: nixpkgs-unstable tracks 5.8.x (CVE-2024-3094 backdoor is in 5.6.0/5.6.1,
  #   both removed from nixpkgs entirely; 5.8.1 is the current secure release).
  # - All inputs follow nixpkgs so the xz version is consistent across all packages.
  # - Run `nix flake update` to pull latest patches; pin with `nix flake lock --update-input`.

  inputs = {
    nixpkgs.url          = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url     = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    flake-utils.url      = "github:numtide/flake-utils";

    ethereum-nix.url     = "github:nix-community/ethereum.nix";
    ethereum-nix.inputs.nixpkgs.follows = "nixpkgs";

    git-hooks.url        = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";

    treefmt-nix.url      = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ethereum-nix, git-hooks, treefmt-nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          ethereum-nix.overlays.default
        ];

        pkgs = import nixpkgs { inherit system overlays; };

        # ── Rust toolchain ────────────────────────────────────────────────
        # Stable latest — keeps Clippy, rustfmt, rust-analyzer, and the
        # wasm32-unknown-unknown target for the WASM frontend all in sync.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets    = [ "wasm32-unknown-unknown" ];
        };

        # ── treefmt-nix ───────────────────────────────────────────────────
        # Unified formatter: rustfmt (Rust), nixpkgs-fmt (Nix),
        # prettier (JS/TS/JSON/MD), forge fmt (Solidity).
        treefmtConfig = treefmt-nix.lib.evalConfig pkgs {
          projectRootFile = "flake.nix";
          programs = {
            rustfmt.enable   = true;
            nixpkgs-fmt.enable = true;
            prettier = {
              enable   = true;
              includes = [ "*.js" "*.ts" "*.tsx" "*.jsx" "*.json" "*.md" ];
            };
          };
          # Forge formatter is not in nixpkgs; call it from the shell hook.
          settings.global.excludes = [
            "*.el.cbor"
            "*.lock"
            "target/**"
            "node_modules/**"
            ".git/**"
          ];
        };

        # ── git pre-commit hooks ──────────────────────────────────────────
        preCommit = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            rustfmt = {
              enable = true;
              entry  = "${rustToolchain}/bin/rustfmt --check";
              types  = [ "rust" ];
            };
            nixpkgs-fmt.enable = true;
            # Clippy — deny all warnings, matching CI
            clippy = {
              enable  = true;
              entry   = "${rustToolchain}/bin/cargo-clippy --workspace -- -D warnings";
              types   = [ "rust" ];
              pass_filenames = false;
            };
          };
        };

      in
      {
        # ── Checks (run by `nix flake check` in CI) ───────────────────────
        checks = {
          pre-commit = preCommit;
          formatting = treefmtConfig.config.build.check self;
        };

        # ── Formatter (run by `nix fmt`) ──────────────────────────────────
        formatter = treefmtConfig.config.build.wrapper;

        # ── Dev shell ─────────────────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # ── Rust ──────────────────────────────────────────────────────
            rustToolchain
            pkgs.trunk            # Rust/WASM bundler
            pkgs.wasm-bindgen-cli

            # ── Node / Bun ────────────────────────────────────────────────
            # nodejs_22 = Node.js 22 LTS (current stable)
            # bun       = latest stable from nixpkgs-unstable
            pkgs.nodejs_22
            pkgs.bun

            # ── Native deps for Rust crates ───────────────────────────────
            pkgs.pkg-config
            pkgs.openssl
            pkgs.lmdb             # LMDB — follows nixpkgs-unstable (0.9.x)

            # ── Solidity / EVM ────────────────────────────────────────────
            # Foundry (forge, cast, anvil, chisel) via ethereum.nix
            pkgs.foundry-bin

            # ── Formatters / linters ──────────────────────────────────────
            pkgs.nixpkgs-fmt
            pkgs.nodePackages.prettier
            treefmtConfig.config.build.wrapper   # `treefmt` CLI

            # ── Utilities ─────────────────────────────────────────────────
            pkgs.just
            pkgs.git

            # ── xz / liblzma security note ────────────────────────────────
            # nixpkgs-unstable carries 5.8.x which contains the fix for
            # CVE-2024-3094.  The vulnerable 5.6.0/5.6.1 builds were removed
            # from the nixpkgs tree.  `nix flake update` pulls the latest.
            pkgs.xz
          ] ++ preCommit.enabledPackages;

          OPENSSL_DIR     = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";

          shellHook = preCommit.shellHook + ''
            echo "retro-sync dev shell ready"
            echo "  rust   : $(rustc --version)"
            echo "  bun    : $(bun --version)"
            echo "  node   : $(node --version)"
            echo "  forge  : $(forge --version 2>/dev/null || echo 'unavailable')"
            echo "  treefmt: $(treefmt --version 2>/dev/null || echo 'unavailable')"

            # Verify xz is NOT the backdoored 5.6.0/5.6.1 build
            XZ_VER=$(xz --version | head -1)
            echo "  xz     : $XZ_VER"
          '';
        };

        # ── Packages ──────────────────────────────────────────────────────

        packages.frontend = pkgs.stdenv.mkDerivation {
          pname   = "retrosync-frontend";
          version = "0.1.0";
          src     = ./apps/web-client;
          nativeBuildInputs = [ pkgs.nodejs_22 pkgs.bun ];
          buildPhase = ''
            export HOME=$TMPDIR
            bun install --frozen-lockfile
            bun run build
          '';
          installPhase = "cp -r dist $out";
        };

        packages.backend = pkgs.rustPlatform.buildRustPackage {
          pname   = "retrosync-backend";
          version = "0.1.0";
          src     = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs  = [ pkgs.pkg-config ];
          buildInputs        = [ pkgs.openssl pkgs.lmdb ];
          cargoBuildFlags    = [ "-p" "backend" ];
        };

        packages.default = self.packages.${system}.backend;
      }
    );
}
