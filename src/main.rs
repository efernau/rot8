extern crate glob;
extern crate clap;
use clap::{Arg, App};
use std::fs;
use std::thread;
use std::time::Duration;
use std::process::Command;
use glob::glob;

fn main() {
    let mut mode = "";
    let mut old_state = "normal";
    let mut new_state: &str;
    let mut path_x: String = "".to_string();
    let mut path_y: String = "".to_string();
    let mut matrix: [&str;9];
    let mut x_state: &str;

    let sway_pid = String::from_utf8(Command::new("pidof")
                            .arg("sway")
                            .output()
                            .unwrap()
                            .stdout).unwrap();

    let x_pid = String::from_utf8(Command::new("pidof")
                            .arg("x")
                            .output()
                            .unwrap()
                            .stdout).unwrap();

    if sway_pid.len() >= 1  {
        mode = "sway";
    }
    if x_pid.len() >= 1  {
        mode = "x";
    }

    let matches = App::new("rot8")
                        .version("0.1.1")
                        .arg(Arg::with_name("sleep")
                                .default_value("500")
                                .long("sleep")
                                .value_name("SLEEP")
                                .help("Set sleep millis")
                                .takes_value(true))
                        .arg(Arg::with_name("display")
                                .default_value("eDP-1")
                                .long("display")
                                .value_name("DISPLAY")
                                .help("Set Display Device")
                                .takes_value(true))
                        .arg(Arg::with_name("touchscreen")
                                .default_value("ELAN0732:00 04F3:22E1")
                                .long("touchscreen")
                                .value_name("TOUCHSCREEN")
                                .help("Set Touchscreen Device (X11)")
                                .takes_value(true))             
                        .get_matches(); 
    let sleep = matches.value_of("sleep").unwrap_or("default.conf");
    let display = matches.value_of("display").unwrap_or("default.conf");
    let touchscreen = matches.value_of("touchscreen").unwrap_or("default.conf"); 

    for entry in  glob("/sys/bus/iio/devices/iio:device*/in_accel_*_raw").unwrap(){
        match entry  {
            Ok(path) => {
                if path.to_str().unwrap().contains("x_raw"){
                    path_x = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("y_raw"){
                    path_y = path.to_str().unwrap().to_owned();
                } else if path.to_str().unwrap().contains("z_raw"){
                    continue;
                } else {
                    println!("{:?}", path);
                    panic!();
                }
            },
            Err(e) => println!("{:?}",e)
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
            }
            else {

              new_state = "90";
              x_state = "left";
              matrix = ["0", "-1", "1", "1", "0", "0", "0", "0", "1"];
            }
        } else if x > 500000 {
            if y > 500000 {
                new_state = "180";
                x_state = "inverted";
                matrix = ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"];
            }
            else {
                new_state = "270";
                x_state = "right";
                matrix = ["0", "1", "0", "-1", "0", "1", "0", "0", "1"];
            }
        } else {
            if y > 500000 {
                new_state = "180";
                x_state = "inverted";
                matrix = ["-1", "0", "1", "0", "-1", "1", "0", "0", "1"];
            }
            else {
                new_state = "normal";
                x_state = "normal";
                matrix = ["1", "0", "0", "0", "1", "0", "0", "0", "1"];
            }
        }

        if new_state != old_state {
            if mode == "sway"  {
                Command::new("swaymsg")
                        .arg("output")
                        .arg(display)
                        .arg("transform")
                        .arg(new_state)
                        .spawn()
                        .expect("rotate command failed to start");

                old_state = new_state;
            }
            if mode == "x"  {
                Command::new("xrandr")
                        .arg("-o")
                        .arg(x_state)
                        .spawn()
                        .expect("rotate command failed to start");

                Command::new("xinput")
                        .arg("set-prop")
                        .arg(touchscreen)
                        .arg("Coordinate")
                        .arg("Transformation")
                        .arg("Matrix")
                        .args(&matrix)
                        .spawn()
                        .expect("rotate command failed to start");

                old_state = new_state;
            }
        }
        thread::sleep(Duration::from_millis(sleep));
    }
}


