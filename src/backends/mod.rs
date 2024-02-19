use crate::Orientation;
use wayland_client::protocol::wl_output::Transform;

pub trait DisplayManager {
    /// Change the orientation of the target display.
    fn change_rotation_state(&mut self, new_state: &Orientation);

    /// Get the current transformation of the target display.
    fn get_rotation_state(&mut self) -> Result<Transform, String>;
}

pub mod hyprland;
pub mod sway;
pub mod wlroots;
pub mod xorg;
