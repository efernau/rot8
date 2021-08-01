//! Device traits.
//!
//! We have two different types of devices we care about:
//!  - Display
//!  - Absolute pointing devices, like Touchscreens and Digitizers.
//!
//!  The other devices probably don't need to be rotated,
//!  as a mouse doesn't change movement direction based on screen orientation.

mod dummy;

use crate::error::*;
use crate::orientation::AbsoluteOrientation;

#[async_trait::async_trait]
pub trait Rotator {
    async fn get_current_orientation(&self) -> Result<AbsoluteOrientation>;
    async fn set_orientation(&self) -> Result<AbsoluteOrientation>;
}

#[async_trait::async_trait]
pub trait RotatorCreator {
    type Output: Rotator + Sized;
    
    /// Create a rotatable device with the given arguments.
    async fn create_rotator(&self, args: String) -> Result<Self::Output>;

    /// List the found rotatable devices.
    async fn available_rotators(&self) -> Result<Vec<String>>;
}
