use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use crate::error::{LumdError, Result};

pub struct Paths {
    // Base directories
    pub config_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        // Get user's XDG directories
        let home = dirs::home_dir()
            .ok_or_else(|| LumdError::InvalidData("Could not determine home directory".into()))?;
            
        // XDG Base Directory specification paths
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| home.join(".config"))
            .join("lumd");
            
        let runtime_dir = match dirs::runtime_dir() {
            Some(dir) => dir,
            None => {
                let uid = nix::unistd::getuid().as_raw();
                PathBuf::from(format!("/var/run/user/{}", uid))
            }
        };
        
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| home.join(".cache"))
            .join("lumd");
            
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| home.join(".local/share"))
            .join("lumd");
        
        // Ensure directories exist with proper permissions
        Self::ensure_dir_exists(&config_dir, 0o755)?;
        Self::ensure_dir_exists(&runtime_dir, 0o700)?;
        Self::ensure_dir_exists(&cache_dir, 0o755)?;
        Self::ensure_dir_exists(&data_dir, 0o755)?;
        
        Ok(Self {
            config_dir,
            runtime_dir,
            cache_dir,
            data_dir,
        })
    }
    
    // Helper method to ensure a directory exists with proper permissions
    fn ensure_dir_exists(path: &Path, mode: u32) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(LumdError::from)?;
        }
        
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(LumdError::from)?;
            
        Ok(())
    }
    
    // File paths
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }
    
    pub fn socket_path(&self) -> PathBuf {
        self.runtime_dir.join("lumd.sock")
    }
    
    pub fn state_file(&self) -> PathBuf {
        self.data_dir.join("state.json")
    }
    
    pub fn log_file(&self) -> PathBuf {
        self.cache_dir.join("lumd.log")
    }
}