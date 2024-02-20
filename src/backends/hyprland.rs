use std::process::{Command, Stdio};

use wayland_client::protocol::wl_output::Transform;

use crate::Orientation;

use super::{wlroots::WaylandBackend, DisplayManager};

// TODO: remove when https://github.com/hyprwm/Hyprland/pull/3544 gets merged
// as per the wiki: https://wiki.hyprland.org/Configuring/Monitors/#rotating
pub struct HyprlandBackend {
    wayland_backend: WaylandBackend,
}

impl HyprlandBackend {
    pub fn new(wayland_backend: WaylandBackend) -> Self {
        HyprlandBackend {
            wayland_backend,
        }
    }

    // https://wiki.hyprland.org/Configuring/Monitors/#rotating
    fn get_transform_idx(transform: Transform) -> Option<u8> {
        match transform {
            Transform::Normal => Some(0),
            Transform::_90 => Some(1),
            Transform::_180 => Some(2),
            Transform::_270 => Some(3),
            Transform::Flipped => Some(4),
            Transform::Flipped90 => Some(5),
            Transform::Flipped180 => Some(6),
            Transform::Flipped270 => Some(7),
            _ => None,
        }
    }
}

impl DisplayManager for HyprlandBackend {
    fn change_rotation_state(&mut self, new_state: &Orientation) {
        self.wayland_backend.change_rotation_state(new_state);
        let transform_idx = HyprlandBackend::get_transform_idx(new_state.wayland_state)
            .expect("unknown transform");
        Command::new("hyprctl")
            .arg("keyword")
            .arg("input:touchdevice:transform")
            .arg(transform_idx.to_string())
            .stdout(Stdio::null())
            .spawn()
            .expect("hyprctl touchdevice transform command failed to start")
            .wait()
            .expect("hyprctl touchdevice transform command wait failed");
        Command::new("hyprctl")
            .arg("keyword")
            .arg("input:tablet:transform")
            .arg(transform_idx.to_string())
            .stdout(Stdio::null())
            .spawn()
            .expect("hyprctl tablet transform command failed to start")
            .wait()
            .expect("hyprctl tablet transform command wait failed");
    }

    fn get_rotation_state(&mut self) -> Result<Transform, String> {
        self.wayland_backend.get_rotation_state()
    }
}
