use std::process::Command;

use serde::Deserialize;
use serde_json::Value;
use wayland_client::Connection;

use crate::Orientation;

use super::{wlroots::WaylandLoop, AppLoop};

#[derive(Deserialize)]
struct SwayOutput {
    name: String,
    transform: String,
}

pub struct SwayLoop {
    wayland_loop: WaylandLoop,
    manage_keyboard: bool,
}

impl SwayLoop {
    pub fn new(conn: Connection, target_display: &str, manage_keyboard: bool) -> SwayLoop {
        SwayLoop {
            wayland_loop: WaylandLoop::new(conn, target_display),
            manage_keyboard,
        }
    }

    fn get_keyboards() -> Vec<String> {
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
        let deserialized: Vec<Value> =
            serde_json::from_str(&raw_inputs).expect("Unable to deserialize swaymsg JSON output");
        for output in deserialized {
            let input_type = output["type"].as_str().unwrap();
            if input_type == "keyboard" {
                keyboards.push(output["identifier"].to_string());
            }
        }

        keyboards
    }
}
impl AppLoop for SwayLoop {
    fn tick_always(&mut self) -> () {
        self.wayland_loop.tick_always();
    }
    fn tick(&mut self, new_state: &Orientation) {
        self.wayland_loop.tick(new_state);

        if !self.manage_keyboard {
            return;
        }

        let keyboard_state = if new_state.new_state == "normal" {
            "enabled"
        } else {
            "disabled"
        };
        for keyboard in &SwayLoop::get_keyboards() {
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

    fn get_rotation_state(&self, display: &str) -> Result<String, String> {
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

        Err(format!(
            "Unable to determine rotation state: display {} not found in 'swaymsg -t get_outputs'",
            display
        ))
    }
}
