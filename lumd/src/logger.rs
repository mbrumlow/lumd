use slog::{Drain, Logger, o};
use std::fs::OpenOptions;
use std::path::Path;
use time::OffsetDateTime;

pub fn setup_logger<P: AsRef<Path>>(log_file_path: Option<P>) -> Logger {
    // Set up timestamp for logging using the time crate
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let timestamp = now.format(&time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"))
        .unwrap_or_else(|_| String::from("unknown_time"));
    
    // Common logging parameters
    let log_params = o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "timestamp" => timestamp
    );
    
    // Determine logging configuration
    match log_file_path {
        Some(log_file) => {
            // File logger with bunyan format
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .expect("Failed to open log file");
                
            // Create a bunyan format logger for file
            let file_drain = slog_bunyan::with_name("lumd", file).build().fuse();
            
            // Use bunyan format for terminal as well to avoid Windows dependencies
            let term_drain = slog_bunyan::default(std::io::stdout()).fuse();
            
            // Combine both drains
            let combined_drain = slog::Duplicate::new(term_drain, file_drain).fuse();
            
            // Wrap in async drain
            let async_drain = slog_async::Async::new(combined_drain).build().fuse();
            
            // Create logger
            Logger::root(async_drain, log_params)
        },
        None => {
            // Only terminal logger with bunyan format
            let term_drain = slog_bunyan::default(std::io::stdout()).fuse();
            let async_drain = slog_async::Async::new(term_drain).build().fuse();
            
            // Create logger
            Logger::root(async_drain, log_params)
        }
    }
}