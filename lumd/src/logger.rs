use slog::{Drain, Logger, o};
use std::fs::OpenOptions;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ptr;
use std::io;
use std::sync::Mutex;

/// Get a simple timestamp string using the Linux libc directly
fn get_timestamp() -> String {
    // First try using libc's localtime
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => return String::from("unknown_time"),
    };
    
    let mut buffer = [0u8; 64];
    
    // Safe libc call to avoid Windows dependencies
    unsafe {
        let mut tm: libc::tm = MaybeUninit::zeroed().assume_init();
        let tm_ptr = &mut tm as *mut libc::tm;
        let time_ptr = &now as *const u64 as *const libc::time_t;
        
        if libc::localtime_r(time_ptr, tm_ptr) != ptr::null_mut() {
            // Format time using strftime
            let format = b"%Y-%m-%dT%H:%M:%S\0";
            let format_ptr = format.as_ptr() as *const libc::c_char;
            let buffer_ptr = buffer.as_mut_ptr() as *mut libc::c_char;
            
            libc::strftime(
                buffer_ptr,
                buffer.len(),
                format_ptr,
                tm_ptr,
            );
            
            let c_str = CStr::from_ptr(buffer_ptr);
            return c_str.to_string_lossy().into_owned();
        }
    }
    
    // Fallback to simple seconds since epoch if localtime_r fails
    format!("{}", now)
}

/// Get the system hostname using libc directly to avoid Windows dependencies
fn get_hostname() -> String {
    let mut buffer = [0u8; 256];
    
    // Use libc gethostname directly
    unsafe {
        if libc::gethostname(buffer.as_mut_ptr() as *mut libc::c_char, buffer.len()) == 0 {
            // Find the null terminator
            let mut len = 0;
            while len < buffer.len() && buffer[len] != 0 {
                len += 1;
            }
            
            // Convert to string
            if let Ok(hostname) = std::str::from_utf8(&buffer[0..len]) {
                return hostname.to_string();
            }
        }
    }
    
    // Fallback to "unknown" if gethostname fails
    "unknown".to_string()
}

pub fn setup_logger<P: AsRef<Path>>(log_file_path: Option<P>) -> Logger {
    // Set up timestamp for logging using direct libc calls to avoid Windows dependencies
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