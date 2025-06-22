use clap::{Parser, ValueEnum};
use nix::unistd;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::{env, fs, process};
use thiserror::Error;

#[derive(Error, Debug)]
enum LumctlError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to connect to lumd: {0}")]
    Connection(String),
}

type Result<T> = std::result::Result<T, LumctlError>;

#[derive(Debug, Clone, ValueEnum)]
enum Command {
    Up,
    Down,
    Resample,
    Shutdown,
}

#[derive(Parser, Debug)]
#[command(version, about = "Control the lumd ambient light daemon")]
struct Cli {
    /// Command to send to lumd
    #[arg(value_enum)]
    command: Command,
}

fn get_socket_path() -> Result<PathBuf> {
    // Use XDG runtime dir if available
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(runtime_dir).join("lumd.sock"));
    }
    
    // Fall back to /var/run/user/$UID/
    let uid = unistd::getuid().as_raw();
    let dir = PathBuf::from(format!("/var/run/user/{}", uid));
    
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| LumctlError::Io(e))?;
    }
    
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
        .map_err(|e| LumctlError::Io(e))?;
        
    Ok(dir.join("lumd.sock"))
}

fn send_command(command: Command) -> Result<()> {
    let socket_path = get_socket_path()?;
    
    // Convert the enum to a string
    let cmd_str = match command {
        Command::Up => "up",
        Command::Down => "down",
        Command::Resample => "resample",
        Command::Shutdown => "shutdown",
    };
    
    // Try to connect to the socket
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            stream.write_all(cmd_str.as_bytes())?;
            Ok(())
        }
        Err(e) => {
            Err(LumctlError::Connection(format!("Is lumd running? Error: {}", e)))
        }
    }
}

fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Send the command to the daemon
    match send_command(cli.command) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}