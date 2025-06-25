use slog::{Drain, Logger, o};
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;
use slog_term::{TermDecorator, CompactFormat, FullFormat};
use std::process::Command;

/// Get the system hostname using the hostname command
fn get_hostname() -> String {
    // Try to get hostname using the hostname command
    match Command::new("hostname").output() {
        Ok(output) if output.status.success() => {
            match String::from_utf8(output.stdout) {
                Ok(hostname) => return hostname.trim().to_string(),
                Err(_) => {}
            }
        },
        _ => {}
    }
    
    // Fallback to "unknown" if the command fails
    "unknown".to_string()
}

pub fn setup_logger<P: AsRef<Path>>(log_file_path: Option<P>) -> Logger {
    // Get hostname for logging
    let hostname = get_hostname();
    
    // Common logging parameters
    let log_params = o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "host" => hostname,
        "name" => "lumd"
    );
    
    // Determine logging configuration
    match log_file_path {
        Some(log_file) => {
            // Terminal logger with pretty format
            let term_decorator = TermDecorator::new().build();
            let term_drain = CompactFormat::new(term_decorator).build().fuse();
            let term_drain = Mutex::new(term_drain).fuse();
            
            // File logger with more detailed format
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .expect("Failed to open log file");
                
            // Create plain decorator for file (no colors)
            let file_decorator = slog_term::PlainDecorator::new(file);
            
            // Use full format for file logs
            let file_drain = FullFormat::new(file_decorator)
                .use_file_location()
                .build()
                .fuse();
            
            let file_drain = Mutex::new(file_drain).fuse();
            
            // Combine both drains
            let combined_drain = slog::Duplicate::new(term_drain, file_drain).fuse();
            
            // Wrap in async drain
            let async_drain = slog_async::Async::new(combined_drain)
                .chan_size(1000)
                .overflow_strategy(slog_async::OverflowStrategy::Drop)
                .build()
                .fuse();
            
            // Create logger
            Logger::root(async_drain, log_params)
        },
        None => {
            // Only terminal logger with pretty format
            let term_decorator = TermDecorator::new().build();
            let term_drain = CompactFormat::new(term_decorator).build().fuse();
            let term_drain = Mutex::new(term_drain).fuse();
                
            let async_drain = slog_async::Async::new(term_drain)
                .chan_size(1000)
                .overflow_strategy(slog_async::OverflowStrategy::Drop)
                .build()
                .fuse();
            
            // Create logger
            Logger::root(async_drain, log_params)
        }
    }
}