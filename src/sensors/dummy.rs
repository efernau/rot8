use super::{Sensor, SensorDriver};
use crate::error::Result;
use crate::orientation::Rotation;

pub struct DummyCreator {}

pub struct DummySensor {
    rot: Rotation,
}

#[async_trait::async_trait]
impl SensorDriver for DummyCreator {
    type Output = DummySensor;

    async fn create_sensor(&self, args: String) -> Result<Self::Output> {
        Ok(DummySensor {
            rot: Rotation::from_degrees(args.parse::<isize>().unwrap())?,
        })
    }

    async fn available_sensors(&self) -> Result<Vec<String>> {
        Ok(vec!["0", "90", "180", "270"]
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>())
    }
}

#[async_trait::async_trait]
impl Sensor for DummySensor {
    async fn get_rotation(self) -> Result<Rotation> {
        Ok(self.rot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn create_sensor() -> Result<()> {
        let driver = DummyCreator {};
        assert_eq!(
            driver.available_sensors().await?,
            ["0", "90", "180", "270"]
                .iter()
                .map(|s| String::from(*s))
                .collect::<Vec<String>>()
        );

        for sensor_arg in driver.available_sensors().await? {
            let sensor = driver.create_sensor(sensor_arg.clone()).await?;
            assert_eq!(
                sensor.get_rotation().await?,
                Rotation::from_degrees(sensor_arg.parse::<isize>().unwrap())?
            )
        }

        Ok(())
    }
}
