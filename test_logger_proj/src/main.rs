use slog::{debug, error, info, o, warn, Drain, Logger};
use slog_term::{CompactFormat, TermDecorator};
use std::sync::Mutex;

fn main() {
    // Create a simple logger
    let decorator = TermDecorator::new().build();
    let drain = CompactFormat::new(decorator).build().fuse();
    let drain = Mutex::new(drain).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = Logger::root(
        drain,
        o!(
            "version" => "0.1.0",
            "host" => "test-host",
            "name" => "lumd-test"
        ),
    );

    // Log various messages
    info!(log, "Starting application"; "pid" => std::process::id());
    debug!(log, "Initializing components"; "components" => 3);

    // Log with structured data
    info!(log, "User logged in";
        "user_id" => 12345,
        "username" => "testuser",
        "login_time" => "2025-06-24T14:30:00"
    );

    // Warning and error messages
    warn!(log, "Resource usage high";
        "cpu" => "78%",
        "memory" => "1.2GB"
    );

    error!(log, "Failed to connect to device";
        "device" => "ambient_light_sensor",
        "error_code" => 404,
        "retry" => true
    );

    info!(log, "Application shutting down"; "uptime" => "00:05:30");
}
