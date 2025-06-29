use crate::config::Config;
use crate::device::{lerp, lux_to_brightness, read_brightness, read_lux, set_brightness};
use crate::error::Result;
use slog::{Logger, debug, error, info};
use std::{path::PathBuf, thread, time::Duration};

pub fn read_and_adjust_ambient_light(
    log: &Logger,
    iio_path: &PathBuf,
    backlight_path: &PathBuf,
    max_brightness: i32,
    config: &Config,
    offset: i32,
    instant: bool,
    force: bool,
) -> Result<bool> {
    info!(log, "Reading ambient light and adjusting brightness");

    let mut current_brightness = read_brightness(backlight_path)?;
    let mut target_brightness = current_brightness;
    let should_force = force; // Rename to avoid the unused assignment
    let threshold = config.brightness_threshold;

    loop {
        match read_lux(iio_path) {
            Ok(lux) => {
                let mut new_target = lux_to_brightness(lux, max_brightness);
                new_target += offset;
                if new_target < config.min_brightness {
                    new_target = config.min_brightness;
                } else if new_target > max_brightness {
                    new_target = max_brightness
                }

                if new_target != target_brightness
                    && (((new_target - target_brightness).abs() > threshold || instant)
                        || should_force)
                {
                    target_brightness = new_target;
                }

                debug!(log, "Light and brightness data";
                    "lux" => format!("{:.1}", lux),
                    "current_brightness" => current_brightness,
                    "min" => config.min_brightness,
                    "max" => max_brightness,
                    "offset" => offset,
                    "target" => target_brightness,
                    "instant" => instant,
                    "force" => should_force
                );
            }
            Err(e) => {
                error!(log, "Failed to read lux: {}", e);
                return Ok(false);
            }
        }

        if !should_force && (current_brightness == target_brightness) {
            return Ok(false);
        }

        if instant {
            set_brightness(backlight_path, target_brightness)?;
            return Ok(true);
        }

        // Gradual brightness adjustment with steps
        let steps = config.transition_steps;
        let delay = Duration::from_millis(config.step_delay_ms);
        let start_brightness = current_brightness;

        for i in 0..steps {
            let t = (i as f32) / (steps as f32);
            let interp = lerp(start_brightness as f32, target_brightness as f32, t).round() as i32;

            debug!(log, "Brightness adjustment step";
                "step" => i,
                "current" => current_brightness,
                "interpolated" => interp
            );

            set_brightness(backlight_path, interp)?;
            current_brightness = interp;

            // Check if ambient light has changed significantly during transition
            if let Ok(lux) = read_lux(iio_path) {
                let mut new_target = lux_to_brightness(lux, max_brightness);
                new_target += offset;
                if new_target < config.min_brightness {
                    new_target = config.min_brightness;
                } else if new_target > max_brightness {
                    new_target = max_brightness
                }

                if new_target != target_brightness
                    && (new_target - target_brightness).abs() > threshold
                {
                    target_brightness = new_target;
                    break; // restart transition with new target
                }
            }

            thread::sleep(delay)
        }

        // Make sure we reach the final target
        if current_brightness != target_brightness {
            set_brightness(backlight_path, target_brightness)?;
        }

        return Ok(true);
    }
}
