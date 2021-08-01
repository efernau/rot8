//! Dummy driver.
//!
//! This is purely for testing or debugging.
//! It logs changes.

use super::{Rotator, RotatorCreator};
use crate::orientation::{Rotation, AbsoluteOrientation};
use crate::error::Result;

use async_std::sync::Mutex;

pub struct DummyCreator {}

pub struct DummyDriver {
    state: Mutex<Box<AbsoluteOrientation>>
}

#[async_trait::async_trait]
impl RotatorCreator for DummyCreator {
    type Output = DummyDriver;

    async fn create_rotator(&self, args: String) -> crate::error::Result<Self::Output> {
        Ok(DummyDriver {
            state: Mutex::new(Box::new(AbsoluteOrientation::from(Rotation::from_degrees(args.parse::<isize>().unwrap())?)))
        })
    }

    // Return a list of valid offsets to start with.
    async fn available_rotators(&self) -> crate::error::Result<Vec<String>> {
        Ok(vec!["0", "90", "180", "270"]
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>())

    }
}

#[async_trait::async_trait]
impl Rotator for DummyDriver {
    async fn get_current_orientation(&self) -> crate::error::Result<AbsoluteOrientation> {
        let lock = self.state.lock().await;
        Ok(**lock)
    }

    async fn set_orientation(&self) -> crate::error::Result<AbsoluteOrientation> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn create_rotatable() -> Result<()> {
        let driver = DummyCreator {};
        assert_eq!(driver.available_rotators().await?, ["0", "90", "180", "270"]
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>());

        for rotator_arg in driver.available_rotators().await? {
            let rotator = driver.create_rotator(rotator_arg.clone()).await?;
            assert_eq!(rotator.get_current_orientation().await?, AbsoluteOrientation::from(Rotation::from_degrees(rotator_arg.parse::<isize>().unwrap())?))
        }

        Ok(())
    }
}
