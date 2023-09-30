extern crate clap;
extern crate glob;
extern crate regex;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use glob::glob;
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
    let mut path_x: String = "".to_string();
    let mut path_y: String = "".to_string();
    let mut path_z: String = "".to_string();

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
            .default_value("1e6")
            .long("normalization-factor")
            .short('n')
            .value_name("NORMALIZATION_FACTOR")
            .help("Set factor for sensor value normalization")
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
            .require_value_delimiter(true)
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

    let flip_x = matches.is_present("invert-x");
    let flip_y = matches.is_present("invert-y");
    let flip_z = matches.is_present("invert-z");
    let mut xy = matches.value_of("invert-xy").unwrap_or("xy").chars();
    let x_source = xy.next().unwrap();
    let y_source = xy.next().unwrap();

    let normalization_factor = matches.value_of("normalization-factor").unwrap_or("1e6");
    let normalization_factor = normalization_factor.parse::<f32>().unwrap_or(1e6);

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

    for entry in glob("/sys/bus/iio/devices/iio:device*/in_accel_*_raw").unwrap() {
        match entry {
            Ok(path) => {
                if path.to_str().unwrap().contains("x_raw") {
                    path_x = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("y_raw") {
                    path_y = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("z_raw") {
                    path_z = path.to_str().unwrap().to_owned();
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    if !Path::new(&path_x).exists() && !Path::new(&path_y).exists() && !Path::new(&path_z).exists()
    {
        Err("Unknown Accelerometer Device".to_string())
    } else {
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
            let x_raw = fs::read_to_string(path_x.as_str()).unwrap();
            let y_raw = fs::read_to_string(path_y.as_str()).unwrap();
            let z_raw = fs::read_to_string(path_z.as_str()).unwrap();
            let x_clean = x_raw.trim_end_matches('\n').parse::<i32>().unwrap_or(0);
            let y_clean = y_raw.trim_end_matches('\n').parse::<i32>().unwrap_or(0);
            let z_clean = z_raw.trim_end_matches('\n').parse::<i32>().unwrap_or(0);

            // Normalize vectors
            let mut mut_x: f32 = (x_clean as f32) / normalization_factor;
            let mut mut_y: f32 = (y_clean as f32) / normalization_factor;
            let mut mut_z: f32 = (z_clean as f32) / normalization_factor;

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
        &Transform::Normal => "normal",
        &Transform::_90 => "270",
        &Transform::_180 => "inverted",
        &Transform::_270 => "90",
        &_ => "normal",
    }
}
