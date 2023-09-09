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
use wayland_client::Connection;

mod backends;
use backends::{sway::SwayLoop, wlroots::WaylandLoop, xorg::XLoop, AppLoop};

const ROT8_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Orientation {
    vector: (f32, f32),
    new_state: &'static str,
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
            .takes_value(false)
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
    let touchscreens: Vec<String> = matches
        .values_of("touchscreen")
        .unwrap()
        .map(|s| s.to_string())
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

    let mut loop_runner: Box<dyn AppLoop> = match Connection::connect_to_env() {
        Ok(conn) => {
            // We are wayland
            if !String::from_utf8(Command::new("pidof").arg("sway").output().unwrap().stdout)
                .unwrap()
                .is_empty()
            {
                Box::new(SwayLoop::new(conn, display, disable_keyboard))
            } else {
                Box::new(WaylandLoop::new(conn, display))
            }
        }
        Err(_) => {
            if !String::from_utf8(Command::new("pidof").arg("Xorg").output().unwrap().stdout)
                .unwrap()
                .is_empty()
                || !String::from_utf8(Command::new("pidof").arg("X").output().unwrap().stdout)
                    .unwrap()
                    .is_empty()
            {
                Box::new(XLoop { touchscreens })
            } else {
                return Err("Unable to find Sway or Xorg procceses".to_owned());
            }
        }
    };

    let old_state_owned = loop_runner.get_rotation_state(display)?;
    let mut old_state = old_state_owned.as_str();

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
                new_state: "normal",
                x_state: "normal",
                matrix: ["1", "0", "0", "0", "1", "0", "0", "0", "1"],
            },
            Orientation {
                vector: (0.0, 1.0),
                new_state: "180",
                x_state: "inverted",
                matrix: ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"],
            },
            Orientation {
                vector: (-1.0, 0.0),
                new_state: "90",
                x_state: "right",
                matrix: ["0", "1", "0", "-1", "0", "1", "0", "0", "1"],
            },
            Orientation {
                vector: (1.0, 0.0),
                new_state: "270",
                x_state: "left",
                matrix: ["0", "-1", "1", "1", "0", "0", "0", "0", "1"],
            },
        ];

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

            loop_runner.tick_always();
            if current_orient.new_state != old_state {
                loop_runner.tick(current_orient);
                old_state = current_orient.new_state;
            }

            if oneshot {
                return Ok(());
            }

            thread::sleep(Duration::from_millis(sleep.parse::<u64>().unwrap_or(0)));
        }
    }
}
