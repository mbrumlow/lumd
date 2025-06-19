{
  description = "Automatic screen bigness";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
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
                lockFile =  ./Cargo.lock;
              };
              cargoInstallFlags = [ "--bin" "lumd" ];  
            };

            lumctl = pkgs.rustPlatform.buildRustPackage {
              pname = "lumctl";
              version = "0.1.0";
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
              cargoInstallFlags = [ "--bin" "lumctl" ];
            };

            default = pkgs.symlinkJoin {
              name = "lum-tools";
              paths = [
                self.packages.${system}.lumd
                self.packages.${system}.lumctl
              ];
            };
            
            devShells.default = pkgs.mkShell {
              buildInputs = [
                rust
                pkgs.pkg-config
              ];
            };
          };
        });
}

