extern crate clap;
extern crate glob;
extern crate regex;

use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use glob::glob;
use serde::Deserialize;
use serde_json::Value;

const ROT8_VERSION: &str = env!("CARGO_PKG_VERSION");

enum Backend {
    Sway,
    Xorg,
}

#[derive(Deserialize)]
struct SwayOutput {
    name: String,
    transform: String,
}

fn get_keyboards(backend: &Backend) -> Result<Vec<String>, String> {
    match backend {
        Backend::Sway => {
            let raw_inputs = String::from_utf8(
                Command::new("swaymsg")
                    .arg("-t")
                    .arg("get_inputs")
                    .arg("--raw")
                    .output()
                    .expect("Swaymsg get inputs command failed")
                    .stdout,
            )
            .unwrap();

            let mut keyboards = vec![];
            let deserialized: Vec<Value> = serde_json::from_str(&raw_inputs)
                .expect("Unable to deserialize swaymsg JSON output");
            for output in deserialized {
                let input_type = output["type"].as_str().unwrap();
                if input_type == "keyboard" {
                    keyboards.push(output["identifier"].to_string());
                }
            }

            Ok(keyboards)
        }
        Backend::Xorg => Ok(vec![]),
    }
}

fn get_window_server_rotation_state(display: &str, backend: &Backend) -> Result<String, String> {
    match backend {
        Backend::Sway => {
            let raw_rotation_state = String::from_utf8(
                Command::new("swaymsg")
                    .arg("-t")
                    .arg("get_outputs")
                    .arg("--raw")
                    .output()
                    .expect("Swaymsg get outputs command failed to start")
                    .stdout,
            )
            .unwrap();
            let deserialized: Vec<SwayOutput> = serde_json::from_str(&raw_rotation_state)
                .expect("Unable to deserialize swaymsg JSON output");
            for output in deserialized {
                if output.name == display {
                    return Ok(output.transform);
                }
            }

            return Err(format!(
                "Unable to determine rotation state: display {} not found in 'swaymsg -t get_outputs'",
                display
            ));
        }
        Backend::Xorg => {
            let raw_rotation_state = String::from_utf8(
                Command::new("xrandr")
                    .output()
                    .expect("Xrandr get outputs command failed to start")
                    .stdout,
            )
            .unwrap();
            let xrandr_output_pattern = regex::Regex::new(format!(
                r"^{} connected .+? .+? (normal |inverted |left |right )?\(normal left inverted right x axis y axis\) .+$",
                regex::escape(display),
            ).as_str()).unwrap();
            for xrandr_output_line in raw_rotation_state.split('\n') {
                if !xrandr_output_pattern.is_match(xrandr_output_line) {
                    continue;
                }

                let xrandr_output_captures =
                    xrandr_output_pattern.captures(xrandr_output_line).unwrap();
                if let Some(transform) = xrandr_output_captures.get(1) {
                    return Ok(transform.as_str().to_owned());
                } else {
                    return Ok("normal".to_owned());
                }
            }

            return Err(format!(
                "Unable to determine rotation state: display {} not found in xrandr output",
                display
            ));
        }
    }
}

struct Orientation {
    vector: (f32, f32),
    new_state: &'static str,
    x_state: &'static str,
    matrix: [&'static str; 9],
}

fn main() -> Result<(), String> {
    let mut new_state: &str;
    let mut path_x: String = "".to_string();
    let mut path_y: String = "".to_string();
    let mut path_z: String = "".to_string();
    let mut matrix: [&str; 9];
    let mut x_state: &str;

    let backend = if !String::from_utf8(Command::new("pidof").arg("sway").output().unwrap().stdout)
        .unwrap()
        .is_empty()
    {
        Backend::Sway
    } else if !String::from_utf8(Command::new("pidof").arg("Xorg").output().unwrap().stdout)
        .unwrap()
        .is_empty()
        || !String::from_utf8(Command::new("pidof").arg("X").output().unwrap().stdout)
            .unwrap()
            .is_empty()
    {
        Backend::Xorg
    } else {
        return Err("Unable to find Sway or Xorg procceses".to_owned());
    };

    let mut args = vec![
        Arg::with_name("oneshot")
            .long("oneshot")
            .short("O")
            .help("Instead of running continuously, just check the accelerometer and perform screen rotation if necessary once")
            .takes_value(false),
        Arg::with_name("sleep")
            .default_value("500")
            .long("sleep")
            .short("s")
            .value_name("SLEEP")
            .help("Set sleep millis")
            .takes_value(true),
        Arg::with_name("display")
            .default_value("eDP-1")
            .long("display")
            .short("d")
            .value_name("DISPLAY")
            .help("Set Display Device")
            .takes_value(true),
        Arg::with_name("touchscreen")
            .default_value("ELAN0732:00 04F3:22E1")
            .long("touchscreen")
            .short("i")
            .value_name("TOUCHSCREEN")
            .help("Set Touchscreen input Device (X11 only)")
            .min_values(1)
            .takes_value(true),
        Arg::with_name("threshold")
            .default_value("0.5")
            .long("threshold")
            .short("t")
            .value_name("THRESHOLD")
            .help("Set a rotation threshold between 0 and 1")
            .takes_value(true),
        Arg::with_name("invert-x")
            .long("invert-x")
            .short("X")
            .help("Invert readings from the HW x axis")
            .takes_value(false),
        Arg::with_name("invert-y")
            .long("invert-y")
            .short("Y")
            .help("Invert readings from the HW y axis")
            .takes_value(false),
        Arg::with_name("invert-z")
            .long("invert-z")
            .short("Z")
            .help("Invert readings from the HW z axis")
            .takes_value(false),
        Arg::with_name("xy")
            .default_value("xy")
            .long("xy")
            .value_name("XY")
            .help("Map hardware accelerometer axes to internal x and y respectively")
            .possible_values(&["xy", "yx", "zy", "yz", "xz", "zx"])
            .takes_value(true),
        Arg::with_name("normalization-factor")
            .default_value("1e6")
            .long("normalization-factor")
            .short("n")
            .value_name("NORMALIZATION_FACTOR")
            .help("Set factor for sensor value normalization")
            .takes_value(true),
        Arg::with_name("version")
            .long("version")
            .short("V")
            .value_name("VERSION")
            .help("Displays rot8 version")
            .takes_value(false)
    ];

    match backend {
        Backend::Sway => {
            args.push(
                Arg::with_name("keyboard")
                    .long("disable-keyboard")
                    .short("k")
                    .help("Disable keyboard for tablet modes (Sway only)")
                    .takes_value(false),
            );
        }
        Backend::Xorg => { /* Keyboard disabling in Xorg is not supported yet */ }
    }

    let cmd_lines = App::new("rot8").version(ROT8_VERSION).args(&args);

    let matches = cmd_lines.get_matches();

    if matches.is_present("version") {
        println!("{}", ROT8_VERSION);
        return Ok(());
    }

    let oneshot = matches.is_present("oneshot");
    let sleep = matches.value_of("sleep").unwrap_or("default.conf");
    let display = matches.value_of("display").unwrap_or("default.conf");
    let touchscreens: Vec<&str> = matches.values_of("touchscreen").unwrap().collect();
    let disable_keyboard = matches.is_present("keyboard");
    let threshold = matches.value_of("threshold").unwrap_or("default.conf");
    let old_state_owned = get_window_server_rotation_state(display, &backend)?;
    let mut old_state = old_state_owned.as_str();

    let flip_x = matches.is_present("invert-x");
    let flip_y = matches.is_present("invert-y");
    let flip_z = matches.is_present("invert-z");
    let mut xy = matches.value_of("xy").unwrap_or("xy").chars();
    let x_source = xy.next().unwrap();
    let y_source = xy.next().unwrap();

    let normalization_factor = matches.value_of("normalization-factor").unwrap_or("1e6");
    let normalization_factor = normalization_factor.parse::<f32>().unwrap_or(1e6);

    let keyboards = get_keyboards(&backend)?;

    for entry in glob("/sys/bus/iio/devices/iio:device*/in_accel_*_raw").unwrap() {
        match entry {
            Ok(path) => {
                if path.to_str().unwrap().contains("x_raw") {
                    path_x = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("y_raw") {
                    path_y = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("z_raw") {
                    path_z = path.to_str().unwrap().to_owned();
                } else {
                    panic!("Unknown accelerometer device path {:?}", path);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

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

        new_state = current_orient.new_state;
        x_state = current_orient.x_state;
        matrix = current_orient.matrix;

        if new_state != old_state {
            let keyboard_state = if new_state == "normal" {
                "enabled"
            } else {
                "disabled"
            };
            match backend {
                Backend::Sway => {
                    Command::new("swaymsg")
                        .arg("output")
                        .arg(display)
                        .arg("transform")
                        .arg(new_state)
                        .spawn()
                        .expect("Swaymsg rotate command failed to start")
                        .wait()
                        .expect("Swaymsg rotate command wait failed");
                    if disable_keyboard {
                        for keyboard in &keyboards {
                            //                            println!("swaymsg input {} events {}", keyboard, keyboard_state);
                            Command::new("swaymsg")
                                .arg("input")
                                .arg(keyboard)
                                .arg("events")
                                .arg(keyboard_state)
                                .spawn()
                                .expect("Swaymsg keyboard command failed to start")
                                .wait()
                                .expect("Swaymsg keyboard command wait failed");
                        }
                    }
                }
                Backend::Xorg => {
                    Command::new("xrandr")
                        .arg("-o")
                        .arg(x_state)
                        .spawn()
                        .expect("Xrandr rotate command failed to start")
                        .wait()
                        .expect("Xrandr rotate command wait failed");

                    // Support Touchscreen and Styli on some 2-in-1 devices
                    for touchscreen in &touchscreens {
                        Command::new("xinput")
                            .arg("set-prop")
                            .arg(touchscreen)
                            .arg("Coordinate Transformation Matrix")
                            .args(&matrix)
                            .spawn()
                            .expect("Xinput rotate command failed to start")
                            .wait()
                            .expect("Xinput rotate command wait failed");
                    }
                }
            }
            old_state = new_state;
        }

        if oneshot {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(sleep.parse::<u64>().unwrap_or(0)));
    }
}
