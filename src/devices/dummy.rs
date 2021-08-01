//! Dummy driver.
//!
//! This is purely for testing or debugging.
//! It logs changes.

use super::{Rotator, RotatorCreator};
use crate::error::Result;
use crate::orientation::{AbsoluteOrientation, Rotation};

use async_std::sync::Mutex;

pub struct DummyCreator {}

pub struct DummyDriver {
    state: Mutex<AbsoluteOrientation>,
}

#[async_trait::async_trait]
impl RotatorCreator for DummyCreator {
    type Output = DummyDriver;

    async fn create_rotator(&self, args: String) -> Result<Self::Output> {
        Ok(DummyDriver {
            state: Mutex::new(AbsoluteOrientation::from(Rotation::from_degrees(
                args.parse::<isize>().unwrap(),
            )?)),
        })
    }

    // Return a list of valid offsets to start with.
    async fn available_rotators(&self) -> Result<Vec<String>> {
        Ok(vec!["0", "90", "180", "270"]
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>())
    }
}

#[async_trait::async_trait]
impl Rotator for DummyDriver {
    async fn get_current_orientation(&self) -> Result<AbsoluteOrientation> {
        let lock = self.state.lock().await;
        Ok(*lock)
    }

    async fn set_orientation(&self, orientation: AbsoluteOrientation) -> Result<()> {
        let mut lock = self.state.lock().await;
        let current: AbsoluteOrientation = *lock;
        println!(
            "driver.dummy: Rotating from {:?} to {:?}.",
            current, orientation
        );
        *lock = orientation;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[async_std::test]
    async fn create_rotatable() -> Result<()> {
        let driver = DummyCreator {};
        assert_eq!(
            driver.available_rotators().await?,
            ["0", "90", "180", "270"]
                .iter()
                .map(|s| String::from(*s))
                .collect::<Vec<String>>()
        );

        for rotator_arg in driver.available_rotators().await? {
            let rotator = driver.create_rotator(rotator_arg.clone()).await?;
            assert_eq!(
                rotator.get_current_orientation().await?,
                AbsoluteOrientation::from(Rotation::from_degrees(
                    rotator_arg.parse::<isize>().unwrap()
                )?)
            )
        }

        Ok(())
    }

    #[async_std::test]
    async fn set_orientation() -> Result<()> {
        let driver = DummyCreator {};
        let rotator = driver.create_rotator(String::from("0")).await?;

        assert_eq!(
            rotator.get_current_orientation().await?,
            AbsoluteOrientation::Normal
        );

        rotator
            .set_orientation(AbsoluteOrientation::RightUp)
            .await?;
        assert_eq!(
            rotator.get_current_orientation().await?,
            AbsoluteOrientation::RightUp
        );

        rotator
            .set_orientation(AbsoluteOrientation::Flipped)
            .await?;
        assert_eq!(
            rotator.get_current_orientation().await?,
            AbsoluteOrientation::Flipped
        );

        rotator.set_orientation(AbsoluteOrientation::LeftUp).await?;
        assert_eq!(
            rotator.get_current_orientation().await?,
            AbsoluteOrientation::LeftUp
        );

        Ok(())
    }
}
