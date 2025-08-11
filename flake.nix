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
          ];

          RUST_BACKTRACE = "1";
          RUST_LOG = "debug";
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages.libclang.lib ];
        };
        default = pkgs.mkShell defaultAttrs;

        bevy = pkgs.mkShell (defaultAttrs // {
          inputsFrom = [ default ];

          buildInputs = with pkgs; [
            # Audio dependencies
            alsa-lib

            # Graphics/Windowing dependencies
            libxkbcommon
            wayland

            # X11 dependencies
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr

            # Vulkan dependencies
            vulkan-loader

            # Other common Bevy dependencies
            udev
          ];

          packages = defaultAttrs.packages;

          # Set LD_LIBRARY_PATH for runtime linking
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            alsa-lib
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            vulkan-loader
            udev
          ]);
        });

      in
      {
        devShells = { inherit default bevy; };
      }
    );
}
