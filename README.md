# lumd

A daemon for automatic ambient light-based brightness adjustment.

## Overview

`lumd` is a system daemon that:
- Reads ambient light levels from supported illuminance sensors
- Automatically adjusts screen backlight based on ambient light
- Supports smooth brightness transitions
- Provides a client utility (`lumctl`) for manual controls

## Features

- Automatic brightness adjustment based on ambient light sensors
- Smooth transitions between brightness levels
- Manual brightness adjustment via the `lumctl` command
- User-configurable settings
- Proper error handling with systemd journal logging
- Graceful shutdown on signals

## Development Environment

This project uses Nix for reproducible builds and development environments. The Nix setup is designed to ensure consistent builds across different Linux environments.

### Using Nix Flakes (recommended)

```bash
# Enter development shell
nix develop

# Build both programs
nix build

# Or build individual components
nix build .#lumd
nix build .#lumctl
```

### Using Traditional Nix

```bash
# Enter development shell
nix-shell

# Build using cargo
cargo build
```

### Using direnv for automatic environment loading

If you have direnv installed, the project will automatically load the Nix environment when you navigate to the project directory:

```bash
# Install direnv if you don't already have it
nix-env -i direnv

# Allow the direnv configuration
cd /path/to/lumd
direnv allow
```

### Troubleshooting Nix Development

If you encounter build issues in the Nix environment:

1. Make sure the environment is properly loaded:
   ```bash
   echo $IN_NIX_SHELL
   ```
   This should output "1" if you're in a Nix shell.

2. Verify system libraries are available:
   ```bash
   pkg-config --list-all | grep openssl
   ```

3. Check the linker configuration:
   ```bash
   cat .cargo/config.toml
   ```
   This should show that static linking is disabled.

4. For manual cargo builds, ensure dynamic linking:
   ```bash
   RUSTFLAGS="-C target-feature=-crt-static" cargo build
   ```

### Required System Packages

If not using Nix, you'll need:
- Rust toolchain (1.70+)
- pkg-config
- gcc
- Linux headers (for nix crate)
- OpenSSL development headers

## Usage

### Configuration

`lumd` follows the XDG Base Directory Specification and looks for a configuration file at `$XDG_CONFIG_HOME/lumd/config.toml` (typically `~/.config/lumd/config.toml`). If the config file doesn't exist, default settings will be used.

Example configuration:

```toml
# Backlight settings
min_brightness = 40
brightness_offset = 40

# Sampling settings
sample_interval_secs = 3
transition_steps = 10
step_delay_ms = 10

# Interpolation threshold
brightness_threshold = 8

# Adjustment amount for manual controls
manual_adjustment_amount = 8
```

### Running the Daemon

```
cargo run --bin lumd
```

### Client Usage

```
# Increase brightness
lumctl up

# Decrease brightness
lumctl down

# Force a resample
lumctl resample

# Shutdown the daemon
lumctl shutdown
```

## Building

```
cargo build --release
```

## Installation

### Using Cargo

```
cargo install --path .
```

### Manual Systemd Setup

After installing with Cargo, you can set up the systemd user service:

```bash
# Create required directories
mkdir -p ~/.config/lumd/
mkdir -p ~/.local/share/lumd
mkdir -p ~/.local/run/lumd
mkdir -p ~/.cache/lumd

# Copy example config
cp /path/to/repo/examples/config.toml ~/.config/lumd/

# Install the systemd user service
mkdir -p ~/.config/systemd/user/
cp /path/to/repo/lumd.service ~/.config/systemd/user/

# Enable and start the service
systemctl --user enable lumd
systemctl --user start lumd

# Check status
systemctl --user status lumd

# View logs
journalctl --user -u lumd
```

#### Troubleshooting Systemd Service

If you encounter permission errors:

1. Check that the service has access to required directories:
   ```bash
   # The service needs write access to:
   ls -la ~/.config/lumd
   ls -la ~/.local/share/lumd
   ls -la ~/.local/run/lumd
   ls -la ~/.cache/lumd
   ```

2. Check systemd journal logs for specific errors:
   ```bash
   journalctl --user -u lumd -n 50 --no-pager
   ```

3. Make sure the backlight device is accessible:
   ```bash
   ls -la /sys/class/backlight/*/brightness
   ```
   You may need to add yourself to a group or create a udev rule to allow access.

### Using Nix

#### As a Package

```bash
# Install using flake
nix profile install github:mbrumlow/lumd

# Or build it
nix build github:mbrumlow/lumd
```

### With Home Manager

Add the following to your Home Manager configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager.url = "github:nix-community/home-manager";
    lumd.url = "github:mbrumlow/lumd";
  };
  
  outputs = { self, nixpkgs, home-manager, lumd, ... }: {
    homeConfigurations."yourusername" = home-manager.lib.homeManagerConfiguration {
      # ...
      modules = [
        lumd.homeManagerModules.default
        {
          services.lumd = {
            enable = true;
            # The package is automatically set by the module
            # Optional settings
            minBrightness = 30;
            brightnessOffset = 50;
            sampleIntervalSecs = 5;
          };
        }
      ];
    };
  };
}
```

### With NixOS

Add the following to your NixOS configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    lumd.url = "github:mbrumlow/lumd";
  };
  
  outputs = { self, nixpkgs, lumd, ... }: {
    nixosConfigurations."yourhostname" = nixpkgs.lib.nixosSystem {
      # ...
      modules = [
        lumd.nixosModules.default
        {
          services.lumd = {
            enable = true;
            # The package is automatically set by the module
            users = [ "yourusername" ];
            # Optional global settings
            globalConfig = {
              minBrightness = 30;
              brightnessOffset = 50;
            };
          };
        }
      ];
    };
  };
}
```
