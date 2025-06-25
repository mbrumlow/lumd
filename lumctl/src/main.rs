use nix::unistd;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::{env, error::Error, fmt, fs, process};

#[derive(Debug)]
enum LumctlError {
    Io(std::io::Error),
    Connection(String),
    Usage(String),
}

impl fmt::Display for LumctlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LumctlError::Io(e) => write!(f, "IO error: {}", e),
            LumctlError::Connection(s) => write!(f, "Failed to connect to lumd: {}", s),
            LumctlError::Usage(s) => write!(f, "Usage error: {}", s),
        }
    }
}

impl Error for LumctlError {}

impl From<std::io::Error> for LumctlError {
    fn from(err: std::io::Error) -> Self {
        LumctlError::Io(err)
    }
}

type Result<T> = std::result::Result<T, LumctlError>;

// Simple command enum for our supported commands
enum Command {
    Up,
    Down,
    Resample,
    Shutdown,
}

impl Command {
    // Parse a command from a string
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "up" => Ok(Command::Up),
            "down" => Ok(Command::Down),
            "resample" => Ok(Command::Resample),
            "shutdown" => Ok(Command::Shutdown),
            _ => Err(LumctlError::Usage(format!("Unknown command: {}", s))),
        }
    }

    // Convert to a string for sending to the daemon
    fn as_str(&self) -> &'static str {
        match self {
            Command::Up => "up",
            Command::Down => "down",
            Command::Resample => "resample",
            Command::Shutdown => "shutdown",
        }
    }
}

fn get_socket_path() -> Result<PathBuf> {
    // Use XDG runtime dir if available
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(runtime_dir).join("lumd").join("lumd.sock"));
    }

    // Fall back to /var/run/user/$UID/
    let uid = unistd::getuid().as_raw();
    let dir = PathBuf::from(format!("/var/run/user/{}", uid));

    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| LumctlError::Io(e))?;
    }

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).map_err(|e| LumctlError::Io(e))?;

    Ok(dir.join("lumd.sock"))
}

fn send_command(command: &Command) -> Result<()> {
    let socket_path = get_socket_path()?;

    // Get the string representation of the command
    let cmd_str = command.as_str();

    // Try to connect to the socket
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            stream.write_all(cmd_str.as_bytes())?;
            Ok(())
        }
        Err(e) => Err(LumctlError::Connection(format!(
            "Is lumd running? Error: {}",
            e
        ))),
    }
}

fn print_usage() {
    eprintln!("lumctl - Control the lumd ambient light daemon");
    eprintln!("Usage: lumctl <command>");
    eprintln!("Commands:");
    eprintln!("  up        - Increase brightness");
    eprintln!("  down      - Decrease brightness");
    eprintln!("  resample  - Force a resampling of ambient light");
    eprintln!("  shutdown  - Shutdown the daemon");
    eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
}

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check that we have a command
    if args.len() != 2 {
        print_usage();
        process::exit(1);
    }

    // Parse the command
    let command = match Command::from_str(&args[1]) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {}", e);
            print_usage();
            process::exit(1);
        }
    };

    // Send the command to the daemon
    match send_command(&command) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
