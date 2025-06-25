use crate::error::{LumdError, Result};
use std::fs;
use std::path::Path;

// Simple configuration struct without serde derive macros
#[derive(Debug, Clone)]
pub struct Config {
    // Backlight settings
    pub min_brightness: i32,
    pub brightness_offset: i32,

    // Sampling settings
    pub sample_interval_secs: u64,
    pub transition_steps: u32,
    pub step_delay_ms: u64,

    // Interpolation threshold
    pub brightness_threshold: i32,

    // Adjustment amount for manual controls
    pub manual_adjustment_amount: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_brightness: 1,
            brightness_offset: 40,
            sample_interval_secs: 3,
            transition_steps: 10,
            step_delay_ms: 10,
            brightness_threshold: 8,
            manual_adjustment_amount: 8,
        }
    }
}

impl Config {
    // Manual parsing of TOML file without using serde derive
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            // If config file doesn't exist, return default config
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(LumdError::from)?;

        // Create config with default values that will be overridden by any values in the file
        let mut config = Self::default();

        // Parse the TOML content using toml crate directly
        let parsed: toml::Value = toml::from_str(&content)
            .map_err(|e| LumdError::InvalidData(format!("Config parse error: {}", e)))?;

        // Extract values from the parsed TOML if they exist
        if let Some(table) = parsed.as_table() {
            // Backlight settings
            if let Some(value) = table.get("min_brightness").and_then(|v| v.as_integer()) {
                config.min_brightness = value as i32;
            }

            if let Some(value) = table.get("brightness_offset").and_then(|v| v.as_integer()) {
                config.brightness_offset = value as i32;
            }

            // Sampling settings
            if let Some(value) = table
                .get("sample_interval_secs")
                .and_then(|v| v.as_integer())
            {
                config.sample_interval_secs = value as u64;
            }

            if let Some(value) = table.get("transition_steps").and_then(|v| v.as_integer()) {
                config.transition_steps = value as u32;
            }

            if let Some(value) = table.get("step_delay_ms").and_then(|v| v.as_integer()) {
                config.step_delay_ms = value as u64;
            }

            // Interpolation threshold
            if let Some(value) = table
                .get("brightness_threshold")
                .and_then(|v| v.as_integer())
            {
                config.brightness_threshold = value as i32;
            }

            // Adjustment amount
            if let Some(value) = table
                .get("manual_adjustment_amount")
                .and_then(|v| v.as_integer())
            {
                config.manual_adjustment_amount = value as i32;
            }
        }

        Ok(config)
    }
}
