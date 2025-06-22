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
- Proper error handling and logging
- Graceful shutdown on signals

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

```
cargo install --path .
```
