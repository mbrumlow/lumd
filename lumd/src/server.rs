use std::{
    fs, io,
    io::Read,
    os::unix::net::UnixListener,
    path::PathBuf,
    sync::{mpsc::Sender, atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::Duration,
};
use slog::{Logger, info, warn, error};
use crate::error::{Result, LumdError};

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
    running: Arc<AtomicBool>
) -> Result<()> {
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
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
                            trigger_tx.send(LumdCommand::BrightnessUp)
                                .map_err(|_| LumdError::Communication("Channel send error".into()))?;
                        }
                        "down" => {
                            info!(log, "Command received: decrease backlight");
                            trigger_tx.send(LumdCommand::BrightnessDown)
                                .map_err(|_| LumdError::Communication("Channel send error".into()))?;
                        }
                        "resample" => {
                            info!(log, "Command received: resample now");
                            trigger_tx.send(LumdCommand::Resample)
                                .map_err(|_| LumdError::Communication("Channel send error".into()))?;
                        }
                        "shutdown" => {
                            info!(log, "Command received: shutdown");
                            trigger_tx.send(LumdCommand::Shutdown)
                                .map_err(|_| LumdError::Communication("Channel send error".into()))?;
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