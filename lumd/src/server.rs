use crate::error::{LumdError, Result};
use slog::{Logger, error, info, warn};
use std::{
    fs, io,
    io::Read,
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::Duration,
};

#[derive(Debug)]
pub enum LumdCommand {
    Resample,
    BrightnessUp,
    BrightnessDown,
    Shutdown,
}

pub fn socket_server(
    log: Logger,
    socket_path: PathBuf,
    trigger_tx: Sender<LumdCommand>,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Ensure socket directory exists
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                LumdError::InvalidData(format!("Failed to create socket directory: {}", e))
            })?;
        }
    }

    // Remove existing socket if it exists
    if socket_path.exists() {
        match fs::remove_file(&socket_path) {
            Ok(_) => info!(log, "Removed existing socket file"; "path" => %socket_path.display()),
            Err(e) => {
                warn!(log, "Failed to remove existing socket file"; "path" => %socket_path.display(), "error" => %e)
            }
        }
    }

    // Create the socket
    let listener = match UnixListener::bind(&socket_path) {
        Ok(listener) => {
            // Set socket permissions to user-only for security
            if let Err(e) = fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600)) {
                warn!(log, "Failed to set socket permissions"; "error" => %e);
            }
            listener
        }
        Err(e) => {
            error!(log, "Failed to bind to socket"; "path" => %socket_path.display(), "error" => %e);
            return Err(e.into());
        }
    };
    info!(log, "Listening on socket"; "path" => %socket_path.display());

    listener.set_nonblocking(true)?;

    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _addr)) => {
                let mut buf = [0u8; 1024];
                if let Ok(n) = stream.read(&mut buf) {
                    let cmd = String::from_utf8_lossy(&buf[..n]);
                    match cmd.trim() {
                        "up" => {
                            info!(log, "Command received: increase backlight");
                            trigger_tx.send(LumdCommand::BrightnessUp).map_err(|_| {
                                LumdError::Communication("Channel send error".into())
                            })?;
                        }
                        "down" => {
                            info!(log, "Command received: decrease backlight");
                            trigger_tx.send(LumdCommand::BrightnessDown).map_err(|_| {
                                LumdError::Communication("Channel send error".into())
                            })?;
                        }
                        "resample" => {
                            info!(log, "Command received: resample now");
                            trigger_tx.send(LumdCommand::Resample).map_err(|_| {
                                LumdError::Communication("Channel send error".into())
                            })?;
                        }
                        "shutdown" => {
                            info!(log, "Command received: shutdown");
                            trigger_tx.send(LumdCommand::Shutdown).map_err(|_| {
                                LumdError::Communication("Channel send error".into())
                            })?;
                        }
                        _ => warn!(log, "Unknown command received"; "command" => cmd.trim()),
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No connection available, sleep a bit and try again
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                error!(log, "Socket connection failed: {}", e);
            }
        }
    }

    // Clean up socket file on exit
    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    Ok(())
}
