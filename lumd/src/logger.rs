use slog::{Drain, Logger, o};
use slog_term::{CompactFormat, TermDecorator};
use std::process::Command;
use std::sync::Mutex;

/// Get the system hostname using the hostname command
fn get_hostname() -> String {
    // Try to get hostname using the hostname command
    match Command::new("hostname").output() {
        Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
            Ok(hostname) => return hostname.trim().to_string(),
            Err(_) => {}
        },
        _ => {}
    }

    // Fallback to "unknown" if the command fails
    "unknown".to_string()
}

pub fn setup_logger(_log_file_path: Option<&str>) -> Logger {
    // Get hostname for logging
    let hostname = get_hostname();

    // Common logging parameters
    let log_params = o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "host" => hostname,
        "name" => "lumd"
    );

    // Create stdout logger with pretty format
    let term_decorator = TermDecorator::new().build();
    let term_drain = CompactFormat::new(term_decorator).build().fuse();
    let term_drain = Mutex::new(term_drain).fuse();

    // Wrap in async drain
    let async_drain = slog_async::Async::new(term_drain)
        .chan_size(1000)
        .overflow_strategy(slog_async::OverflowStrategy::Drop)
        .build()
        .fuse();

    // Create logger
    Logger::root(async_drain, log_params)
}
