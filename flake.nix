{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };

        rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = [ "rust-analyzer" "rust-src" "clippy" ];
        };

        nightly-rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain-nightly.toml;

        defaultAttrs = {
          nativeBuildInputs = with pkgs; [
            rust-toolchain
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            clang
          ];

          packages = with pkgs; [
            just
            parallel
            rust-analyzer
            (rustfmt.override { asNightly = true; })
            cargo-nextest
            cargo-machete
            cargo-watch
            cargo-rdme
            cargo-expand
            cargo-workspaces
          ];

          RUST_BACKTRACE = "1";
          RUST_LOG = "debug";
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages.libclang.lib ];
        };
        default = pkgs.mkShell defaultAttrs;

        # for bevy / egui / iced / nannou
        gui = pkgs.mkShell rec {
          inherit (defaultAttrs) packages;

          nativeBuildInputs = with pkgs; [
            nightly-rust-toolchain
            pkg-config
            cmake
          ];

          buildInputs = with pkgs; [
            openssl
            clang

            alsa-lib
            expat
            fontconfig
            freetype
            libGL
            libxkbcommon
            udev
            vulkan-loader
            wayland # To use the wayland feature
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr # To use the x11 feature
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };

      in
      {
        devShells = { inherit default gui; };
      }
    );
}
