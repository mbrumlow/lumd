use std::{fs, path::{Path, PathBuf}};
use crate::error::{Result, LumdError};

pub fn find_backlight_device() -> Result<PathBuf> {
    let base = PathBuf::from("/sys/class/backlight/");
    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        let path = entry.path();
        if path.join("brightness").exists() && path.join("max_brightness").exists() {
            return Ok(path);
        }
    }
    Err(LumdError::DeviceNotFound("No backlight device found".into()))
}

pub fn find_illuminance_device() -> Result<PathBuf> {
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
    Err(LumdError::DeviceNotFound("No IIO illuminance device found".into()))
}

pub fn read_f32(path: &Path) -> Result<f32> {
    let s = fs::read_to_string(path)?;
    s.trim()
        .parse()
        .map_err(|e| LumdError::ParseFloat(e))
}

pub fn read_i32(path: &Path) -> Result<i32> {
    let s = fs::read_to_string(path)?;
    s.trim()
        .parse()
        .map_err(|e| LumdError::Parse(e))
}

pub fn read_lux(iio_path: &Path) -> Result<f32> {
    let raw = read_i32(&iio_path.join("in_illuminance_raw"))?;
    let scale = read_f32(&iio_path.join("in_illuminance_scale"))?;
    Ok((raw as f32) * scale)
}

pub fn read_max_brightness(iio_path: &Path) -> Result<i32> {
    read_i32(&iio_path.join("max_brightness"))
}

pub fn set_brightness(iio_path: &Path, value: i32) -> Result<()> {
    let path = iio_path.join("brightness");
    fs::write(path, value.to_string())?;
    Ok(())
}

pub fn read_brightness(backlight_path: &Path) -> Result<i32> {
    read_i32(&backlight_path.join("brightness"))
}

pub fn lux_to_brightness(lux: f32, max_brightness: i32) -> i32 {
    let scaled = (lux / 1000.0) * (max_brightness as f32);
    scaled.clamp(1.0, max_brightness as f32) as i32
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}