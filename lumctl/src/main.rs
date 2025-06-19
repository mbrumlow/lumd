use nix::unistd;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::{env, fs};

fn get_socket_path() -> PathBuf {
    let uid = unistd::getuid().as_raw();
    let dir = PathBuf::from(format!("/var/run/user/{}", uid));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    dir.join("lumd.sock")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: lumctl <up|down|resample>");
        std::process::exit(1);
    }

    let socket_path = get_socket_path();
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            let _ = stream.write_all(args[1].as_bytes());
        }
        Err(e) => {
            eprintln!("Failed to connect to lumd: {e}");
            std::process::exit(1);
        }
    }
}
