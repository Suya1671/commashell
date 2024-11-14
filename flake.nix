{
  description = "Build a cargo project with a custom toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };

      toolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src" "rust-analyzer"];
      };

      crate = pkgs.stdenv.mkDerivation {
        pname = "commashell";
        version = "0.1.0";

        src = ./.;
        cargoDeps = pkgs.rustPlatform.importCargoLock {
          lockFile = ./Cargo.lock;
          outputHashes = {};
        };

        nativeBuildInputs = [
          toolchain
          pkgs.meson
          pkgs.ninja
          pkgs.cmake
          pkgs.desktop-file-utils
          pkgs.pkg-config
          pkgs.rustPlatform.cargoSetupHook
          pkgs.wrapGAppsHook4
          pkgs.appstream
          pkgs.blueprint-compiler
        ];

        buildInputs = [
          pkgs.gtk4
          pkgs.libadwaita
        ];
      };
    in {
      packages.default = crate;

      devShells.default = pkgs.mkShell {
        inputsFrom = [crate];

        # Extra inputs can be added here; cargo and rustc are provided by default
        # from the toolchain that was specified earlier.
        packages = [
          toolchain
          pkgs.nixd
          pkgs.alejandra
        ];
      };
    });
}
