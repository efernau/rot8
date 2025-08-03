extern crate clap;
extern crate regex;

use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use wayland_client::protocol::wl_output::Transform;

mod backends;
use backends::{sway::SwayBackend, wlroots::WaylandBackend, xorg::XorgBackend, DisplayManager};

const ROT8_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Orientation {
    vector: (f32, f32),
    wayland_state: Transform,
    x_state: &'static str,
    matrix: [&'static str; 9],
}

fn main() -> Result<(), String> {
    let args = vec![
        Arg::with_name("oneshot")
            .long("oneshot")
            .short('O')
            .help("Instead of running continuously, just check the accelerometer and perform screen rotation if necessary once")
            .takes_value(false),
        Arg::with_name("sleep")
            .default_value("500")
            .long("sleep")
            .short('s')
            .value_name("SLEEP")
            .help("Set sleep millis")
            .takes_value(true),
        Arg::with_name("display")
            .default_value("eDP-1")
            .long("display")
            .short('d')
            .value_name("DISPLAY")
            .help("Set Display Device")
            .takes_value(true),
        Arg::with_name("touchscreen")
            .default_value("ELAN0732:00 04F3:22E1")
            .long("touchscreen")
            .short('i')
            .value_name("TOUCHSCREEN")
            .help("Set Touchscreen input Device (X11 only)")
            .min_values(1)
            .takes_value(true),
        Arg::with_name("threshold")
            .default_value("0.5")
            .long("threshold")
            .short('t')
            .value_name("THRESHOLD")
            .help("Set a rotation threshold between 0 and 1")
            .takes_value(true),
        Arg::with_name("invert-x")
            .long("invert-x")
            .short('X')
            .help("Invert readings from the HW x axis")
            .takes_value(false),
        Arg::with_name("invert-y")
            .long("invert-y")
            .short('Y')
            .help("Invert readings from the HW y axis")
            .takes_value(false),
        Arg::with_name("invert-z")
            .long("invert-z")
            .short('Z')
            .help("Invert readings from the HW z axis")
            .takes_value(false),
        Arg::with_name("invert-xy")
            .default_value("xy")
            .long("invert-xy")
            .value_name("XY")
            .help("Map hardware accelerometer axes to internal x and y respectively")
            .possible_values(["xy", "yx", "zy", "yz", "xz", "zx"])
            .takes_value(true),
        Arg::with_name("normalization-factor")
            .long("normalization-factor")
            .short('n')
            .value_name("NORMALIZATION_FACTOR")
            .help("Set factor for sensor value normalization manually. By default this factor is calculated dynamically using the sensor's data.")
            .takes_value(true),
        Arg::with_name("keyboard")
            .long("disable-keyboard")
            .short('k')
            .help("Disable keyboard for tablet modes (Sway only)")
            .takes_value(false),
        Arg::with_name("version")
            .long("version")
            .short('V')
            .value_name("VERSION")
            .help("Displays rot8 version")
            .takes_value(false),
        Arg::with_name("beforehooks")
            .long("beforehooks")
            .short('b')
            .value_name("BEFOREHOOKS")
            .help("Run hook(s) before screen rotation. Passes $ORIENTATION and $PREV_ORIENTATION to hooks. Comma-seperated.")
            .takes_value(true)
            .use_value_delimiter(true)
            .require_value_delimiter(true),
        Arg::with_name("hooks")
            .long("hooks")
            .short('h')
            .value_name("HOOKS")
            .help("Run hook(s) after screen rotation. Passes $ORIENTATION and $PREV_ORIENTATION to hooks. Comma-seperated.")
            .takes_value(true)
            .use_value_delimiter(true)
            .require_value_delimiter(true),
        Arg::with_name("accelerometer")
            .long("accelerometer")
            .short('a')
            .value_name("ACCELEROMETER")
            .help("Set accelerometer device path")
            .takes_value(true)
    ];

    let cmd_lines = App::new("rot8").version(ROT8_VERSION).args(&args);

    let matches = cmd_lines.get_matches();

    if matches.is_present("version") {
        println!("{}", ROT8_VERSION);
        return Ok(());
    }

    let oneshot = matches.is_present("oneshot");
    let sleep = matches.value_of("sleep").unwrap_or("default.conf");
    let display = matches.value_of("display").unwrap_or("default.conf");
    let touchscreens: Vec<String> = matches.get_many("touchscreen").unwrap().cloned().collect();
    let hooks: Vec<&str> = matches.values_of("hooks").unwrap_or_default().collect();
    let beforehooks: Vec<&str> = matches
        .values_of("beforehooks")
        .unwrap_or_default()
        .collect();
    let disable_keyboard = matches.is_present("keyboard");
    let threshold = matches.value_of("threshold").unwrap_or("default.conf");
    let accelerometer = 'acc: {
        if let Some(p) = matches.value_of("accelerometer") {
            p.into()
        } else {
            let entries = fs::read_dir("/sys/bus/iio/devices")
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            for e in entries {
                if !e
                    .file_name()
                    .as_os_str()
                    .as_encoded_bytes()
                    .starts_with(b"iio:device")
                {
                    continue;
                }

                if ["in_accel_x_raw", "in_accel_y_raw", "in_accel_z_raw"]
                    .iter()
                    .all(|c| e.path().join(c).exists())
                {
                    break 'acc e.path();
                }
            }

            return Err("Unknown Accelerometer Device".to_string());
        }
    };

    let flip_x = matches.is_present("invert-x");
    let flip_y = matches.is_present("invert-y");
    let flip_z = matches.is_present("invert-z");
    let mut xy = matches.value_of("invert-xy").unwrap_or("xy").chars();
    let x_source = xy.next().unwrap();
    let y_source = xy.next().unwrap();

    let mut normalization_factor: Option<f32> = None;
    if let Some(v) = matches.value_of("normalization-factor") {
        match v.parse::<f32>() {
            Ok(p) => normalization_factor = Some(p),
            Err(_) => {
                return Err(
                    "The argument 'normalization-factor' is no valid float literal".to_string(),
                );
            }
        }
    }

    let mut backend: Box<dyn DisplayManager> = match WaylandBackend::new(display) {
        Ok(wayland_backend) => {
            if process_exists("sway") {
                Box::new(SwayBackend::new(wayland_backend, disable_keyboard))
            } else {
                Box::new(wayland_backend)
            }
        }
        Err(e) => {
            if process_exists("Xorg") || process_exists("X") {
                Box::new(XorgBackend::new(display, touchscreens))
            } else {
                return Err(format!(
                    "Unable to find supported Xorg process or wayland compositor: {}.",
                    e
                ));
            }
        }
    };

    let path_x = accelerometer.join("in_accel_x_raw");
    let path_y = accelerometer.join("in_accel_y_raw");
    let path_z = accelerometer.join("in_accel_z_raw");

    let orientations = [
        Orientation {
            vector: (0.0, -1.0),
            wayland_state: Transform::Normal,
            x_state: "normal",
            matrix: ["1", "0", "0", "0", "1", "0", "0", "0", "1"],
        },
        Orientation {
            vector: (0.0, 1.0),
            wayland_state: Transform::_180,
            x_state: "inverted",
            matrix: ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"],
        },
        Orientation {
            vector: (-1.0, 0.0),
            wayland_state: Transform::_270,
            x_state: "right",
            matrix: ["0", "1", "0", "-1", "0", "1", "0", "0", "1"],
        },
        Orientation {
            vector: (1.0, 0.0),
            wayland_state: Transform::_90,
            x_state: "left",
            matrix: ["0", "-1", "1", "1", "0", "0", "0", "0", "1"],
        },
    ];

    let mut old_state = backend.get_rotation_state()?;
    let mut current_orient: &Orientation = &orientations[0];

    loop {
        let x_raw = fs::read_to_string(&path_x).unwrap();
        let y_raw = fs::read_to_string(&path_y).unwrap();
        let z_raw = fs::read_to_string(&path_z).unwrap();
        let x_clean = x_raw.trim_end_matches('\n').parse::<f32>().unwrap_or(0.);
        let y_clean = y_raw.trim_end_matches('\n').parse::<f32>().unwrap_or(0.);
        let z_clean = z_raw.trim_end_matches('\n').parse::<f32>().unwrap_or(0.);

        // Normalize vectors
        let norm_factor = normalization_factor.unwrap_or_else(|| {
            f32::sqrt(x_clean * x_clean + y_clean * y_clean + z_clean * z_clean)
        });

        let mut mut_x: f32 = x_clean / norm_factor;
        let mut mut_y: f32 = y_clean / norm_factor;
        let mut mut_z: f32 = z_clean / norm_factor;

        // Apply inversions
        if flip_x {
            mut_x = -mut_x;
        }
        if flip_y {
            mut_y = -mut_y;
        }
        if flip_z {
            mut_z = -mut_z;
        }
        // Switch axes as requested
        let x = match x_source {
            'y' => mut_y,
            'z' => mut_z,
            _ => mut_x,
        };
        let y = match y_source {
            'x' => mut_x,
            'z' => mut_z,
            _ => mut_y,
        };

        for orient in orientations.iter() {
            let d = (x - orient.vector.0).powf(2.0) + (y - orient.vector.1).powf(2.0);
            if d < threshold.parse::<f32>().unwrap_or(0.5) {
                current_orient = orient;
                break;
            }
        }

        if current_orient.wayland_state != old_state {
            let old_env = transform_to_env(&old_state);
            let new_env = transform_to_env(&current_orient.wayland_state);
            for bhook in beforehooks.iter() {
                Command::new("bash")
                    .arg("-c")
                    .arg(bhook)
                    .env("ORIENTATION", new_env)
                    .env("PREV_ORIENTATION", old_env)
                    .spawn()
                    .expect("A hook failed to start.")
                    .wait()
                    .expect("Waiting for a hook failed.");
            }

            backend.change_rotation_state(current_orient);

            for hook in hooks.iter() {
                Command::new("bash")
                    .arg("-c")
                    .arg(hook)
                    .env("ORIENTATION", new_env)
                    .env("PREV_ORIENTATION", old_env)
                    .spawn()
                    .expect("A hook failed to start.")
                    .wait()
                    .expect("Waiting for a hook failed.");
            }

            old_state = current_orient.wayland_state;
        }

        if oneshot {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(sleep.parse::<u64>().unwrap_or(0)));
    }
}

fn process_exists(proc_name: &str) -> bool {
    !String::from_utf8(
        Command::new("pidof")
            .arg(proc_name)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .is_empty()
}

fn transform_to_env(transform: &Transform) -> &str {
    match transform {
        Transform::Normal => "normal",
        Transform::_90 => "270",
        Transform::_180 => "inverted",
        Transform::_270 => "90",
        _ => "normal",
    }
}
