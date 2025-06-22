use std::{
    sync::{mpsc::{self, Receiver, Sender}, atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::Duration,
    process,
};

use slog::{info, warn, error, debug, o};

mod backlight;
mod config;
mod device;
mod error;
mod logger;
mod paths;
mod server;
mod signal;

use config::Config;
use device::{find_backlight_device, find_illuminance_device, read_max_brightness};
use error::Result;
use paths::Paths;
use server::LumdCommand;

fn main() -> Result<()> {
    // Initialize application paths
    let paths = match Paths::new() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Failed to initialize application paths: {}", e);
            process::exit(1);
        }
    };
    
    // Set up logger
    let root_log = logger::setup_logger(Some(paths.log_file()));
    let log = root_log.clone();
    info!(log, "Lumd starting up"; "version" => env!("CARGO_PKG_VERSION"));
    
    // Set up running flag for clean shutdown
    let running = Arc::new(AtomicBool::new(true));
    
    // Set up signal handler
    signal::setup_signal_handler(log.clone(), Arc::clone(&running))?;
    
    // Load configuration
    let config = match Config::from_file(paths.config_file()) {
        Ok(config) => {
            info!(log, "Loaded configuration from file"; "path" => %paths.config_file().display());
            config
        },
        Err(e) => {
            warn!(log, "Could not load config file, using defaults"; "error" => %e);
            Config::default()
        }
    };
    
    // Find required devices
    let iio_path = match find_illuminance_device() {
        Ok(path) => {
            info!(log, "Found illuminance device"; "path" => %path.display());
            path
        },
        Err(e) => {
            error!(log, "Failed to find illuminance device"; "error" => %e);
            return Err(e);
        }
    };
    
    let backlight_path = match find_backlight_device() {
        Ok(path) => {
            info!(log, "Found backlight device"; "path" => %path.display());
            path
        },
        Err(e) => {
            error!(log, "Failed to find backlight device"; "error" => %e);
            return Err(e);
        }
    };
    
    // Read max brightness value
    let max_brightness = match read_max_brightness(&backlight_path) {
        Ok(max) => {
            info!(log, "Read max brightness"; "value" => max);
            max
        },
        Err(e) => {
            error!(log, "Failed to read max brightness"; "error" => %e);
            return Err(e);
        }
    };
    
    // Set up socket path
    let socket_path = paths.socket_path();
    
    // Initialize variables
    let mut sleep = config.sample_interval_secs;
    let mut offset = config.brightness_offset;
    let mut instant = true;
    
    // Set up channel for communication between threads
    let (tx, rx): (Sender<LumdCommand>, Receiver<LumdCommand>) = mpsc::channel();

    // Spawn socket server
    let tx_clone = tx.clone();
    let socket_log = log.new(o!("component" => "socket_server"));
    let running_clone = Arc::clone(&running);
    let server_log = log.clone();
    let socket_path_owned = socket_path.to_path_buf();
    thread::spawn(move || {
        if let Err(e) = server::socket_server(socket_log, socket_path_owned, tx_clone, running_clone) {
            error!(server_log, "Socket server error"; "error" => %e);
        }
    });

    // Main sampling loop
    let sample_log = log.new(o!("component" => "light_sampler"));
    while running.load(Ordering::SeqCst) {
        let mut force = false;
        let mut next_offset = offset;
        if !instant {
            // Wait for either a manual trigger or sleep seconds
            match rx.recv_timeout(Duration::from_secs(sleep)) {
                Ok(cmd) => match cmd {
                    LumdCommand::Resample => {
                        info!(sample_log, "Received early resample signal");
                        instant = false;
                        sleep = 30;
                        force = true;
                    }
                    LumdCommand::BrightnessUp => {
                        next_offset += config.manual_adjustment_amount;
                        info!(sample_log, "Increasing brightness offset"; 
                              "new_offset" => next_offset, 
                              "adjustment" => config.manual_adjustment_amount);
                        instant = true;
                        sleep = 30;
                    }
                    LumdCommand::BrightnessDown => {
                        next_offset -= config.manual_adjustment_amount;
                        info!(sample_log, "Decreasing brightness offset"; 
                              "new_offset" => next_offset, 
                              "adjustment" => config.manual_adjustment_amount);
                        instant = true;
                        sleep = 30;
                    }
                    LumdCommand::Shutdown => {
                        info!(sample_log, "Received shutdown command");
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // fall through to next iteration (normal sampling)
                    debug!(sample_log, "Sampling timeout reached, proceeding with normal sample");
                    sleep = config.sample_interval_secs;
                }
                Err(e) => {
                    error!(sample_log, "Trigger channel error"; "error" => %e);
                    break;
                }
            }
        }

        match backlight::read_and_adjust_ambient_light(
            &sample_log,
            &iio_path,
            &backlight_path,
            max_brightness,
            &config,
            next_offset,
            instant,
            force,
        ) {
            Ok(changed) => {
                if changed {
                    offset = next_offset;
                    debug!(sample_log, "Updated brightness offset"; "offset" => offset);
                }
            },
            Err(e) => {
                error!(sample_log, "Failed to adjust brightness"; "error" => %e);
            }
        }
        
        instant = false;
    }

    info!(log, "Shutting down lumd gracefully");
    Ok(())
}