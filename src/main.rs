extern crate glob;
extern crate input_sys;

use std::fs;
use std::thread;
use std::time::Duration;
use std::process::Command;
use glob::glob;

fn main() {
    let mut old_state = "normal";
    let mut new_state: &str;
    let mut path_x: String = "".to_string();
    let mut path_y: String = "".to_string();
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
            }
            else {
            
              new_state = "90";
            }
        } else if x > 500000 {
            if y > 500000 {
                new_state = "180";
            }
            else {
                new_state = "270";
            }
        } else {
            if y > 500000 {
                new_state = "180";
            }
            else {
                new_state = "normal";
            }
        }

        if new_state != old_state {

            Command::new("swaymsg")
                     .arg("output")
                     .arg("eDP-1")
                     .arg("transform")
                     .arg(new_state)
                     .spawn()
                     .expect("rotate command failed to start");       

            old_state = new_state;
        }   
        thread::sleep(Duration::from_millis(1000));
    }

    
}


