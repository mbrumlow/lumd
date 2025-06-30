use crate::error::{LumdError, Result};
use std::path::PathBuf;
use xdg::BaseDirectories;

pub struct Paths {
    pub config_file_path: PathBuf,
    pub socket_path: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        // Create XDG base directories
        let xdg = BaseDirectories::with_prefix("lumd");

        // Runtime directory handling
        // First try XDG_RUNTIME_DIR (the standard environment variable)
        // Then fall back to a subdirectory of /tmp which should always exist
        let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
            Ok(dir) => PathBuf::from(dir).join("lumd"),
            Err(_) => {
                let uid = nix::unistd::getuid().as_raw();
                PathBuf::from(format!("/tmp/lumd-{}", uid))
            }
        };

        // Ensure the runtime directory exists
        if !runtime_dir.exists() {
            std::fs::create_dir_all(&runtime_dir).map_err(|e| {
                LumdError::InvalidData(format!("Failed to create runtime directory: {}", e))
            })?;
        }

        // Create config file path
        let config_file_path = xdg
            .place_config_file("config.toml")
            .map_err(|e| LumdError::InvalidData(format!("Could not create config path: {}", e)))?;

        // Create socket path
        let socket_path = runtime_dir.join("lumd.sock");

        Ok(Self {
            config_file_path,
            socket_path,
        })
    }

    // File paths accessors
    pub fn config_file(&self) -> &PathBuf {
        &self.config_file_path
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}
