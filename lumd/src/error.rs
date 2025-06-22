use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LumdError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error("Parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Socket error: {0}")]
    Socket(String),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Communication error: {0}")]
    Communication(String),
}

pub type Result<T> = std::result::Result<T, LumdError>;