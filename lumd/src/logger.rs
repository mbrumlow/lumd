use slog::{Drain, Logger, o};
use std::fs::OpenOptions;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io;
use std::sync::Mutex;
use std::process::Command;

/// Get a timestamp string using Rust std library
fn get_timestamp() -> String {
    // Get current time since epoch
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(_) => return String::from("unknown_time"),
    };
    
    // Format the timestamp manually
    let secs = now.as_secs();
    let seconds = secs % 60;
    let minutes = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    
    // Calculate days since epoch and approximate date
    // This is a simple implementation and doesn't account for leap years properly
    let days_since_epoch = secs / 86400;
    let years_since_epoch = days_since_epoch / 365;
    let year = 1970 + years_since_epoch;
    
    let days_this_year = days_since_epoch - (years_since_epoch * 365);
    
    // Very simple month calculation
    let (month, day) = calculate_month_day(year as i32, days_this_year as i32);
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", 
            year, month, day, hours, minutes, seconds)
}

/// Very simple month/day calculation
/// Not accurate for all edge cases but sufficient for logging timestamps
fn calculate_month_day(year: i32, day_of_year: i32) -> (u32, u32) {
    let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    
    let days_in_month = [
        31, 
        if is_leap_year { 29 } else { 28 }, 
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31
    ];
    
    let mut remaining_days = day_of_year;
    for (month_index, &days) in days_in_month.iter().enumerate() {
        if remaining_days <= days {
            return ((month_index + 1) as u32, remaining_days as u32);
        }
        remaining_days -= days;
    }
    
    // Fallback
    (12, 31)
}

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
    // Set up timestamp for logging
    let timestamp = get_timestamp();
    let hostname = get_hostname();
    
    // Common logging parameters
    let log_params = o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "timestamp" => timestamp,
        "host" => hostname,
        "name" => "lumd"
    );
    
    // Determine logging configuration
    match log_file_path {
        Some(log_file) => {
            // File logger with JSON format
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .expect("Failed to open log file");
                
            // Create a JSON logger for file
            let file_drain = Mutex::new(slog_json::Json::new(file)
                .add_default_keys()
                .build()).fuse();
            
            // Use JSON format for terminal as well to avoid Windows dependencies
            let term_drain = Mutex::new(slog_json::Json::new(io::stdout())
                .add_default_keys()
                .build()).fuse();
            
            // Combine both drains
            let combined_drain = slog::Duplicate::new(term_drain, file_drain).fuse();
            
            // Wrap in async drain
            let async_drain = slog_async::Async::new(combined_drain).build().fuse();
            
            // Create logger
            Logger::root(async_drain, log_params)
        },
        None => {
            // Only terminal logger with JSON format
            let term_drain = Mutex::new(slog_json::Json::new(io::stdout())
                .add_default_keys()
                .build()).fuse();
                
            let async_drain = slog_async::Async::new(term_drain).build().fuse();
            
            // Create logger
            Logger::root(async_drain, log_params)
        }
    }
}