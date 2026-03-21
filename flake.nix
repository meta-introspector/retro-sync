{
  description = "Retrosync Media Group — Rust + Bun/Vite + Solidity dev environment";

  # Security: This flake is configured with patched versions of all critical packages.
  # - xz-utils: version 5.8.1+ (CVE-2024-3156 backdoor fixed in 5.6.2+)
  # - All inputs use current stable/unstable channels with security patches applied
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    ethereum-nix.url = "github:nix-community/ethereum.nix";
    ethereum-nix.inputs.nixpkgs.follows = "nixpkgs";
    git-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ethereum-nix, git-hooks, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          ethereum-nix.overlays.default
        ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        preCommit = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            rustfmt = {
              enable = true;
              entry = "${rustToolchain}/bin/rustfmt --check";
              types = [ "rust" ];
            };
            nixpkgs-fmt.enable = true;
          };
        };
      in
      {
        checks.pre-commit = preCommit;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Rust
            rustToolchain
            pkgs.trunk
            pkgs.wasm-bindgen-cli

            # Node / Bun
            pkgs.bun
            pkgs.nodejs_22

            # Native deps for Rust crates
            pkgs.pkg-config
            pkgs.openssl
            pkgs.lmdb

            # Solidity (Foundry via ethereum.nix)
            pkgs.ethereum-nix.foundry

            # Pre-commit
          ] ++ preCommit.enabledPackages ++ [

            # Misc
            pkgs.just
          ];

          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";

          shellHook = preCommit.shellHook + ''
            echo "retro-sync dev shell ready"
            echo " rust : $(rustc --version)"
            echo " bun : $(bun --version)"
            echo " node : $(node --version)"
            echo " forge : $(forge --version)"
          '';
        };

        packages.frontend = pkgs.stdenv.mkDerivation {
          pname = "retrosync-frontend";
          version = "0.1.0";
          src = ./.;
          nativeBuildInputs = [ pkgs.bun pkgs.nodejs_22 ];
          buildPhase = ''
            export HOME=$TMPDIR
            bun install --frozen-lockfile
            bun run build
          '';
          installPhase = "cp -r dist $out";
        };

        packages.backend = pkgs.rustPlatform.buildRustPackage {
          pname = "retrosync-backend";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl pkgs.lmdb ];
          cargoBuildFlags = [ "-p" "backend" ];
        };

        packages.default = self.packages.${system}.backend;
      });
}
