{
  description = "KeenCLI - Keenetic Router CLI Tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        keencli = rustPlatform.buildRustPackage rec {
          pname = "keencli";
          version = "1.0.4";
          # builtins.path: git'e eklenmemiş dosyalar (Cargo.lock) da dahil edilir
          src = builtins.path {
            path = ./.;
            name = "keencli-src";
            filter = path: type:
              let
                base = pkgs.lib.baseNameOf path;
              in
              !(base == "target"
                || base == "outputs"
                || base == ".env"
                || base == ".direnv"
                || base == ".git"
                || base == "result");
          };
          cargoLock.lockFile = builtins.path {
            path = ./Cargo.lock;
            name = "Cargo.lock";
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };
      in {
        packages.default = keencli;
        packages.keencli = keencli;

        apps.default = {
          type = "app";
          program = "${keencli}/bin/keencli";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.nixd
            pkgs.nixfmt
            pkgs.nil
            pkgs.statix
            pkgs.deadnix
            pkgs.pkg-config
            pkgs.openssl
          ];

          shellHook = ''
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export PS1="\[\e[32m\][nix-shell:keencli]\[\e[0m\] $PS1"
            echo "KeenCLI geliştirme ortamı hazır"
          '';
        };
      });
}
