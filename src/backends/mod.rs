use crate::Orientation;

pub trait AppLoop {
    fn tick_always(&mut self) -> ();
    fn tick(&mut self, new_state: &Orientation) -> ();
    fn get_rotation_state(&self, display: &str) -> Result<String, String>;
}

pub mod sway;
pub mod wlroots;
pub mod xorg;
