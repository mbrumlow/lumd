use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LumdError {
    Io(io::Error),
    Parse(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    DeviceNotFound(String),
    InvalidData(String),
    Communication(String),
}

impl fmt::Display for LumdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LumdError::Io(e) => write!(f, "IO error: {}", e),
            LumdError::Parse(e) => write!(f, "Parse error: {}", e),
            LumdError::ParseFloat(e) => write!(f, "Parse float error: {}", e),
            LumdError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            LumdError::InvalidData(s) => write!(f, "Invalid data: {}", s),
            LumdError::Communication(s) => write!(f, "Communication error: {}", s),
        }
    }
}

impl Error for LumdError {}

impl From<io::Error> for LumdError {
    fn from(err: io::Error) -> Self {
        LumdError::Io(err)
    }
}

impl From<std::num::ParseIntError> for LumdError {
    fn from(err: std::num::ParseIntError) -> Self {
        LumdError::Parse(err)
    }
}

impl From<std::num::ParseFloatError> for LumdError {
    fn from(err: std::num::ParseFloatError) -> Self {
        LumdError::ParseFloat(err)
    }
}

pub type Result<T> = std::result::Result<T, LumdError>;
