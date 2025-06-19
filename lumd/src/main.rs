use std::{
    fs, io,
    io::Read,
    os::unix::fs::PermissionsExt,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use nix::unistd;

#[derive(Debug)]
enum LumdCommand {
    Resample,
    BrightnessUp,
    BrightnessDown,
}

fn get_socket_path() -> PathBuf {
    let uid = unistd::getuid().as_raw();
    let dir = PathBuf::from(format!("/var/run/user/{}", uid));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    dir.join("lumd.sock")
}

fn read_and_adjust_ambient_light(
    iio_path: PathBuf,
    backlight_path: PathBuf,
    max_brightness: i32,
    min_brightness: i32,
    offset: i32,
    instant: bool,
    force: bool,
) -> bool {
    println!("[sample] Reading ambient light and adjusting brightness");

    let mut current_brightness = read_brightness(&backlight_path).unwrap_or(max_brightness / 2);
    let mut target_brightness = current_brightness;
    let mut force = force;

    loop {
        match read_lux(&iio_path) {
            Ok(lux) => {
                let mut new_target = lux_to_brightness(lux, max_brightness);
                new_target += offset;
                if new_target < 0 {
                    new_target = 1;
                } else if new_target > max_brightness {
                    new_target = max_brightness
                }
                if new_target != target_brightness
                    && (((new_target - target_brightness).abs() > 8 || instant) || force)
                {
                    target_brightness = new_target;
                }

                println!(
                    "[lux] lux: {:.1}, brightness: {}, min: {}, max: {}, offset: {}, target: {}, instate: {}, force: {}",
                    lux,
                    current_brightness,
                    min_brightness,
                    max_brightness,
                    offset,
                    target_brightness,
                    instant,
                    force,
                );
            }
            Err(e) => {
                eprintln!("[lux] failed to read lux: {e}");
                return false;
            }
        }

        if !force && (current_brightness == target_brightness) {
            return false;
        }
        force = false;

        if instant {
            if let Err(e) = set_brightness(&backlight_path, target_brightness) {
                eprintln!("[set] brightness failed: {e}");
            }
            return true;
        }

        let steps = 10;
        let delay = std::time::Duration::from_millis(10);
        let start_brightness = current_brightness;
        for i in 0..steps {
            let t = (i as f32) / (steps as f32);
            let interp = lerp(start_brightness as f32, target_brightness as f32, t).round() as i32;

            println!(
                "[lux]: step: {}, current: {} interp: {}",
                i, current_brightness, interp
            );

            if let Err(e) = set_brightness(&backlight_path, interp) {
                eprintln!("[set] brightness failed: {e}");
            }

            current_brightness = interp;

            if let Ok(lux) = read_lux(&iio_path) {
                let mut new_target = lux_to_brightness(lux, max_brightness);
                new_target += offset;
                if new_target < 0 {
                    new_target = 1;
                } else if new_target > max_brightness {
                    new_target = max_brightness
                }
                if new_target != target_brightness && (new_target - target_brightness).abs() > 8 {
                    target_brightness = new_target;
                    break; // restart transition with new target
                }
            }

            std::thread::sleep(delay)
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn lux_to_brightness(lux: f32, max_brightness: i32) -> i32 {
    let scaled = (lux / 1000.0) * (max_brightness as f32);
    scaled.clamp(1.0, max_brightness as f32) as i32
}

fn socket_server(socket_path: PathBuf, trigger_tx: Sender<LumdCommand>) -> std::io::Result<()> {
    if socket_path.exists() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("lumd listening on {}", socket_path.display());

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0u8; 1024];
                if let Ok(n) = stream.read(&mut buf) {
                    let cmd = String::from_utf8_lossy(&buf[..n]);
                    match cmd.trim() {
                        "up" => {
                            println!("[cmd] increase backlight");
                            let _ = trigger_tx.send(LumdCommand::BrightnessUp);
                        }
                        "down" => {
                            println!("[cmd] decrease backlight");
                            let _ = trigger_tx.send(LumdCommand::BrightnessDown);
                        }
                        "resample" => {
                            println!("[cmd] resample now");
                            let _ = trigger_tx.send(LumdCommand::Resample);
                        }
                        _ => println!("[cmd] unknown command: {}", cmd.trim()),
                    }
                }
            }
            Err(e) => eprintln!("Connection failed: {e}"),
        }
    }
    Ok(())
}

fn find_backlight_device() -> io::Result<PathBuf> {
    let base = PathBuf::from("/sys/class/backlight/");
    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        let path = entry.path();
        if path.join("brightness").exists() && path.join("max_brightness").exists() {
            return Ok(path);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No backlight device found",
    ))
}

fn find_illuminance_device() -> io::Result<PathBuf> {
    let base = Path::new("/sys/bus/iio/devices/");
    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let raw = path.join("in_illuminance_raw");
            let scale = path.join("in_illuminance_scale");
            if raw.exists() && scale.exists() {
                return Ok(path);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No IIO illuminance device found",
    ))
}

fn read_f32(path: &Path) -> io::Result<f32> {
    let s = fs::read_to_string(path)?;
    s.trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_i32(path: &Path) -> io::Result<i32> {
    let s = fs::read_to_string(path)?;
    s.trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_lux(iio_path: &Path) -> io::Result<f32> {
    let raw = read_i32(&iio_path.join("in_illuminance_raw"))?;
    let scale = read_f32(&iio_path.join("in_illuminance_scale"))?;
    Ok((raw as f32) * scale)
}

fn read_max_brightness(iio_path: &Path) -> io::Result<i32> {
    let path = iio_path.join("max_brightness");
    let contents = fs::read_to_string(path)?;
    contents
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("parse error: {e}")))
}

fn set_brightness(iio_path: &Path, value: i32) -> io::Result<()> {
    let path = iio_path.join("brightness");
    fs::write(path, value.to_string())?;
    Ok(())
}

fn read_brightness(backlight_path: &Path) -> io::Result<i32> {
    let path = backlight_path.join("brightness");
    let contents = fs::read_to_string(&path)?;
    contents
        .trim()
        .parse::<i32>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("parse error: {e}")))
}

fn main() -> std::io::Result<()> {
    let socket_path = get_socket_path();
    let iio_path = find_illuminance_device()?;
    let backlight_path = find_backlight_device()?;
    let max_brightness = read_max_brightness(&backlight_path)?;
    let min_brightness = 40;
    let mut sleep = 3;
    let mut offset = 40;
    let mut instant = true;
    let (tx, rx): (Sender<LumdCommand>, Receiver<LumdCommand>) = mpsc::channel();

    // Spawn socket server
    let tx_clone = tx.clone();
    thread::spawn(move || {
        if let Err(e) = socket_server(socket_path, tx_clone) {
            eprintln!("Socket server error: {e}");
        }
    });

    // Main sampling loop
    loop {
        let mut force = false;
        let mut next_offset = offset;
        if !instant {
            // Wait for either a manual trigger or 5 seconds
            match rx.recv_timeout(Duration::from_secs(sleep)) {
                Ok(cmd) => match cmd {
                    LumdCommand::Resample => {
                        println!("[trigger] received early resample signal");
                        instant = false;
                        sleep = 30;
                        force = true;
                    }
                    LumdCommand::BrightnessUp => {
                        next_offset += 8;
                        instant = true;
                        sleep = 30;
                    }
                    LumdCommand::BrightnessDown => {
                        next_offset -= 8;
                        instant = true;
                        sleep = 30;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // fall through to next iteration (normal sampling)
                    sleep = 3;
                }
                Err(e) => {
                    eprintln!("Trigger channel error: {e}");
                    break;
                }
            }
        }

        if read_and_adjust_ambient_light(
            iio_path.clone(),
            backlight_path.clone(),
            max_brightness,
            min_brightness,
            next_offset,
            instant,
            force,
        ) {
            offset = next_offset;
        }
        instant = false;
    }

    Ok(())
}
