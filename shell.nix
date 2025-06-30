# shell.nix - For compatibility with older Nix setups
{ pkgs ? import <nixpkgs> {} }:

let
  # Import the flake's devShell if possible
  flake = builtins.getFlake (toString ./.);
  defaultShell = flake.devShells.${builtins.currentSystem}.default;
in
if builtins ? getFlake
then defaultShell
else pkgs.mkShell {
  packages = with pkgs; [
    
    # Basic development dependencies
    pkg-config
    glibc
    glibc.dev
    gcc
    binutils
    
    # Linux-specific libraries
    linuxHeaders
    
    # System libraries needed for nix crate
    openssl
    openssl.dev

    # Rust
    rustc
    cargo
    clippy
    rustfmt
    rust-analyzer
  ];

  # Set environment variables
  shellHook = ''
    # Set linker flags to avoid static linking
    export RUSTFLAGS="-C target-feature=-crt-static"
    
    # Set path to system libraries
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.glibc
      pkgs.gcc.cc.lib
      pkgs.openssl
    ]}
    
    # Set C compiler path
    export CC=${pkgs.gcc}/bin/gcc
    
    # Set linker path
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=${pkgs.gcc}/bin/gcc
    
    # Add pkg-config path for system libraries
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
    
    echo "Welcome to lumd development environment (via shell.nix)!"
  '';
}
