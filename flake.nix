{
  description = "Automatic screen bigness";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    {
      # Home Manager module
      homeManagerModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          system = pkgs.stdenv.hostPlatform.system;
          lumdPackage = self.packages.${system}.default;
        in
        {
          imports = [ ./home-manager-module.nix ];

          # Pre-configure the package option to point to our flake's package
          config = lib.mkIf config.services.lumd.enable {
            services.lumd.package = lib.mkDefault lumdPackage;
          };
        };

      # NixOS module
      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          system = pkgs.stdenv.hostPlatform.system;
          lumdPackage = self.packages.${system}.default;
        in
        {
          imports = [ ./nixos-module.nix ];

          # Pre-configure the package option to point to our flake's package
          config = lib.mkIf config.services.lumd.enable {
            services.lumd.package = lib.mkDefault lumdPackage;
          };
        };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };
        rust = pkgs.rust-bin.stable.latest.default;
      in
      {
        packages = {
          lumd = pkgs.rustPlatform.buildRustPackage {
            pname = "lumd";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            cargoInstallFlags = [
              "--bin"
              "lumd"
            ];
            buildAndTestSubdir = "lumd";
            # Add necessary build inputs and native build inputs
            buildInputs = [
              pkgs.glibc
              pkgs.openssl
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
            # Disable static linking for binary
            RUSTFLAGS = "-C target-feature=-crt-static";
          };

          lumctl = pkgs.rustPlatform.buildRustPackage {
            pname = "lumctl";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            cargoInstallFlags = [
              "--bin"
              "lumctl"
            ];
            buildAndTestSubdir = "lumctl";
            # Add necessary build inputs and native build inputs
            buildInputs = [
              pkgs.glibc
              pkgs.openssl
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
            # Disable static linking for binary
            RUSTFLAGS = "-C target-feature=-crt-static";
          };

          default = pkgs.symlinkJoin {
            name = "lum-tools";
            paths = [
              self.packages.${system}.lumd
              self.packages.${system}.lumctl
            ];
            postBuild = ''
              mkdir -p $out/lib/systemd/user
              cp ${./lumd.service} $out/lib/systemd/user/lumd.service
            '';
          };

        };
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.pkg-config
            pkgs.glibc
            pkgs.glibc.dev
            pkgs.gcc
            pkgs.binutils

            # Linux-specific libraries
            pkgs.linuxHeaders

            # System libraries needed for nix crate
            pkgs.openssl
            pkgs.openssl.dev

            # Development tools
            pkgs.rustup
            pkgs.cargo-edit
            pkgs.cargo-watch
            pkgs.cargo-audit
            pkgs.rust-analyzer
          ];

          # Set environment variables
          shellHook = ''
            # Set linker flags to avoid static linking
            export RUSTFLAGS="-C target-feature=-crt-static"

            # Set path to system libraries
            export LD_LIBRARY_PATH=${
              pkgs.lib.makeLibraryPath [
                pkgs.glibc
                pkgs.gcc.cc.lib
                pkgs.openssl
              ]
            }

            # Set C compiler path
            export CC=${pkgs.gcc}/bin/gcc

            # Set linker path
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=${pkgs.gcc}/bin/gcc

            # Add pkg-config path for system libraries
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"

            # Make rust-src available for rust-analyzer
            export RUST_SRC_PATH="${rust}/lib/rustlib/src/rust/library"

            echo "Welcome to lumd development environment!"
          '';
        };
      }
    );
}
