use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{LumdError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // Paths
    #[serde(default = "default_socket_dir")]
    pub socket_dir: PathBuf,
    
    // Backlight settings
    #[serde(default = "default_min_brightness")]
    pub min_brightness: i32,
    #[serde(default = "default_brightness_offset")]
    pub brightness_offset: i32,
    
    // Sampling settings
    #[serde(default = "default_sample_interval")]
    pub sample_interval_secs: u64,
    #[serde(default = "default_transition_steps")]
    pub transition_steps: u32,
    #[serde(default = "default_step_delay_ms")]
    pub step_delay_ms: u64,
    
    // Interpolation threshold
    #[serde(default = "default_brightness_threshold")]
    pub brightness_threshold: i32,
    
    // Adjustment amount for manual controls
    #[serde(default = "default_manual_adjustment")]
    pub manual_adjustment_amount: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_dir: default_socket_dir(),
            min_brightness: default_min_brightness(),
            brightness_offset: default_brightness_offset(),
            sample_interval_secs: default_sample_interval(),
            transition_steps: default_transition_steps(),
            step_delay_ms: default_step_delay_ms(),
            brightness_threshold: default_brightness_threshold(),
            manual_adjustment_amount: default_manual_adjustment(),
        }
    }
}

// Default values (replacing hardcoded values from the original code)
fn default_socket_dir() -> PathBuf {
    let uid = nix::unistd::getuid().as_raw();
    PathBuf::from(format!("/var/run/user/{}", uid))
}

fn default_min_brightness() -> i32 { 40 }
fn default_brightness_offset() -> i32 { 40 }
fn default_sample_interval() -> u64 { 3 }
fn default_transition_steps() -> u32 { 10 }
fn default_step_delay_ms() -> u64 { 10 }
fn default_brightness_threshold() -> i32 { 8 }
fn default_manual_adjustment() -> i32 { 8 }

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(LumdError::from)?;
        
        toml::from_str(&content)
            .map_err(|e| LumdError::InvalidData(format!("Config parse error: {}", e)))
    }
    
    pub fn get_socket_path(&self) -> PathBuf {
        self.socket_dir.join("lumd.sock")
    }
}