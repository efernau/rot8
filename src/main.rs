extern crate clap;
extern crate glob;
extern crate regex;

use clap::{App, Arg};
use glob::glob;
use serde::Deserialize;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

enum Backend {
    Sway,
    Xorg,
}

#[derive(Deserialize)]
struct SwayOutput {
    name: String,
    transform: String,
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
            )
            .to_owned());
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
            for xrandr_output_line in raw_rotation_state.split("\n") {
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
            )
            .to_owned());
        }
    }
}

fn main() -> Result<(), String> {
    let mut new_state: &str;
    let mut path_x: String = "".to_string();
    let mut path_y: String = "".to_string();
    let mut matrix: [&str; 9];
    let mut x_state: &str;

    let backend = if String::from_utf8(Command::new("pidof").arg("sway").output().unwrap().stdout)
        .unwrap()
        .len()
        >= 1
    {
        Backend::Sway
    } else if String::from_utf8(Command::new("pidof").arg("Xorg").output().unwrap().stdout)
        .unwrap()
        .len()
        >= 1
    {
        Backend::Xorg
    } else {
        return Err("Unable to find Sway or Xorg procceses".to_owned());
    };

    let matches = App::new("rot8")
        .version("0.1.1")
        .arg(
            Arg::with_name("sleep")
                .default_value("500")
                .long("sleep")
                .value_name("SLEEP")
                .help("Set sleep millis")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("display")
                .default_value("eDP-1")
                .long("display")
                .value_name("DISPLAY")
                .help("Set Display Device")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("touchscreen")
                .default_value("ELAN0732:00 04F3:22E1")
                .long("touchscreen")
                .value_name("TOUCHSCREEN")
                .help("Set Touchscreen Device (X11)")
                .takes_value(true),
        )
        .get_matches();
    let sleep = matches.value_of("sleep").unwrap_or("default.conf");
    let display = matches.value_of("display").unwrap_or("default.conf");
    let touchscreen = matches.value_of("touchscreen").unwrap_or("default.conf");
    let old_state_owned = get_window_server_rotation_state(display, &backend)?;
    let mut old_state = old_state_owned.as_str();

    for entry in glob("/sys/bus/iio/devices/iio:device*/in_accel_*_raw").unwrap() {
        match entry {
            Ok(path) => {
                if path.to_str().unwrap().contains("x_raw") {
                    path_x = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("y_raw") {
                    path_y = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("z_raw") {
                    continue;
                } else {
                    panic!("Unknown accelerometer device path {:?}", path);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    loop {
        let x_raw = fs::read_to_string(path_x.as_str()).unwrap();
        let y_raw = fs::read_to_string(path_y.as_str()).unwrap();
        let x = x_raw.trim_end_matches('\n').parse::<i32>().unwrap_or(0);
        let y = y_raw.trim_end_matches('\n').parse::<i32>().unwrap_or(0);

        if x < -500000 {
            if y > 500000 {
                new_state = "180";
                x_state = "normal";
                matrix = ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"];
            } else {
                new_state = "90";
                x_state = "right";
                matrix = ["0", "-1", "1", "1", "0", "0", "0", "0", "1"];
            }
        } else if x > 500000 {
            if y > 500000 {
                new_state = "180";
                x_state = "inverted";
                matrix = ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"];
            } else {
                new_state = "270";
                x_state = "left";
                matrix = ["0", "1", "0", "-1", "0", "1", "0", "0", "1"];
            }
        } else {
            if y > 500000 {
                new_state = "180";
                x_state = "inverted";
                matrix = ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"];
            } else {
                new_state = "normal";
                x_state = "normal";
                matrix = ["1", "0", "0", "0", "1", "0", "0", "0", "1"];
            }
        }

        if new_state != old_state {
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
                }
                Backend::Xorg => {
                    Command::new("xrandr")
                        .arg("-o")
                        .arg(x_state)
                        .spawn()
                        .expect("Xrandr rotate command failed to start")
                        .wait()
                        .expect("Xrandr rotate command wait failed");

                    Command::new("xinput")
                        .arg("set-prop")
                        .arg(touchscreen)
                        .arg("Coordinate")
                        .arg("Transformation")
                        .arg("Matrix")
                        .args(&matrix)
                        .spawn()
                        .expect("Xinput rotate command failed to start")
                        .wait()
                        .expect("Xinput rotate command wait failed");
                }
            }
            old_state = new_state;
        }
        thread::sleep(Duration::from_millis(sleep.parse::<u64>().unwrap_or(0)));
    }
}
