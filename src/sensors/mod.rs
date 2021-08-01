//! Sensors
//!
//! This is the abstraction that represents something like a gyroscope or other source of
//! desired rotation.

pub mod dummy;

#[cfg(target_os = "linux")]
pub mod linux_iio;

use crate::error::Result;
use crate::orientation::Rotation;

#[async_trait::async_trait]
pub trait Sensor {
    async fn get_rotation(self) -> Result<Rotation>;
}

#[async_trait::async_trait]
pub trait SensorDriver {
    type Output: Sensor + Sized;

    /// Create a sensor with given arguments.
    async fn create_sensor(&self, args: String) -> Result<Self::Output>;

    /// Lists available sensors that the driver knows of.
    async fn available_sensors(&self) -> Result<Vec<String>>;
}
