use slog::{Drain, Logger, o};
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::path::Path;

pub fn setup_logger<P: AsRef<Path>>(log_file_path: Option<P>) -> Logger {
    // Terminal logger
    let decorator = slog_term::TermDecorator::new().build();
    let term_drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    
    match log_file_path {
        Some(log_file) => {
            // File logger
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .expect("Failed to open log file");
                
            let file_drain = Mutex::new(slog_term::FullFormat::new(slog_term::PlainDecorator::new(file)).build()).fuse();
            
            // Combine both drains
            let drain = slog::Duplicate::new(term_drain, file_drain).fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            
            Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")))
        },
        None => {
            // Only terminal logger
            let drain = slog_async::Async::new(term_drain).build().fuse();
            Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")))
        }
    }
}