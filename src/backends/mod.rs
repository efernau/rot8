use wayland_client::protocol::wl_output::Transform;

use crate::Orientation;

pub trait DisplayManager {
    fn change_rotation_state(&mut self, new_state: &Orientation) -> ();
    fn get_rotation_state(&mut self) -> Result<Transform, String>;
}

pub mod sway;
pub mod wlroots;
pub mod xorg;
