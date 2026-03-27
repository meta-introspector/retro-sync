{
  description = "Bach Inventions — retro-sync NFT tile series";

  inputs.retro-sync.url = "path:../..";

  outputs = { self, retro-sync, ... }: {
    packages.x86_64-linux.default = retro-sync.inputs.nixpkgs.legacyPackages.x86_64-linux.stdenv.mkDerivation {
      pname = "bach-invention-tiles";
      version = "0.1.0";
      src = ./.;
      nativeBuildInputs = with retro-sync.inputs.nixpkgs.legacyPackages.x86_64-linux; [
        python3
        python3Packages.pypng
        lilypond
        resvg
      ];
      buildPhase = ''
        export HOME=$TMPDIR
        bash ${retro-sync}/scripts/midi2svg.sh midi output/svg 71
        bash ${retro-sync}/scripts/stego-build.sh .
      '';
      installPhase = ''
        mkdir -p $out/tiles $out/svg
        cp output/stego/*.png $out/tiles/ 2>/dev/null || true
        cp output/svg/*.svg $out/svg/ 2>/dev/null || true
        cp project.toml $out/
      '';
    };
  };
}
