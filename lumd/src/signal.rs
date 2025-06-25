use signal_hook::{consts::signal::*, iterator::Signals};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use slog::{Logger, info, warn};
use crate::error::Result;

pub fn setup_signal_handler(logger: Logger, running: Arc<AtomicBool>) -> Result<()> {
    let mut signals = Signals::new(&[SIGINT, SIGTERM])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
    let logger = logger.clone();
    
    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT | SIGTERM => {
                    info!(logger, "Received signal {}, initiating shutdown", sig);
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                _ => warn!(logger, "Unexpected signal: {}", sig),
            }
        }
    });
    
    Ok(())
}