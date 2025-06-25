use crate::error::{LumdError, Result};
use std::path::PathBuf;
use xdg::BaseDirectories;

pub struct Paths {
    // Base directories
    pub config_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_file_path: PathBuf,
    pub socket_path: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        // Create XDG base directories
        let xdg = BaseDirectories::with_prefix("lumd")
            .map_err(|e| LumdError::InvalidData(format!("XDG error: {}", e)))?;

        // Get the base directories
        let config_dir = xdg.get_config_home();

        // Runtime directory handling based on environment
        let runtime_dir = if let Ok(runtime_dir) = std::env::var("RUNTIME_DIRECTORY") {
            // When running under systemd with RuntimeDirectory=lumd
            match std::env::var("XDG_RUNTIME_DIR") {
                Ok(xdg_dir) => PathBuf::from(xdg_dir).join(runtime_dir),
                Err(_) => {
                    let uid = nix::unistd::getuid().as_raw();
                    PathBuf::from(format!("/run/user/{}/{}", uid, runtime_dir))
                }
            }
        } else {
            // Fallback to standard XDG runtime dir
            match std::env::var("XDG_RUNTIME_DIR") {
                Ok(dir) => PathBuf::from(dir),
                Err(_) => {
                    let uid = nix::unistd::getuid().as_raw();
                    PathBuf::from(format!("/run/user/{}", uid))
                }
            }
        };

        let cache_dir = xdg.get_cache_home();

        // Create config file path
        let config_file_path = xdg
            .place_config_file("config.toml")
            .map_err(|e| LumdError::InvalidData(format!("Could not create config path: {}", e)))?;

        // Create socket path
        let socket_path = runtime_dir.join("lumd.sock");

        Ok(Self {
            config_dir,
            runtime_dir,
            cache_dir,
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
