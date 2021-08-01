//! Dummy driver.
//!
//! This is purely for testing or debugging.
//! It logs changes.

use super::{Rotator, RotatorCreator};
use crate::error::Result;
use crate::orientation::AbsoluteOrientation;

use async_std::process::Command;

use lazy_static::lazy_static;
use regex::Regex;

pub struct XRandRCreator {}

pub struct XRandRDriver {
    output: String,
}

#[async_trait::async_trait]
impl RotatorCreator for XRandRCreator {
    type Output = XRandRDriver;

    async fn create_rotator(&self, args: String) -> Result<Self::Output> {
        // TODO: handle initial rotation offset
        Ok(XRandRDriver { output: args })
    }

    // Return a list of valid offsets to start with.
    async fn available_rotators(&self) -> Result<Vec<String>> {
        lazy_static! {
            static ref XRANDR_MONITORS: Regex = Regex::new("(.+) connected ").unwrap();
        }

        let raw_xrandr = String::from_utf8(Command::new("xrandr").arg("-q").output().await?.stdout)
            .expect("failed to decode utf-8 from xrandr output");
        let monitors = Regex::captures_iter(&XRANDR_MONITORS, &raw_xrandr)
            .map(|cap| String::from(&cap[1]))
            .collect::<Vec<String>>();
        Ok(monitors)
    }
}

#[async_trait::async_trait]
impl Rotator for XRandRDriver {
    async fn get_current_orientation(&self) -> Result<AbsoluteOrientation> {
        lazy_static! {
            static ref XRANDR_MONITOR_ORIENTATION: Regex =
                Regex::new(r"(.+) connected .+ .+ (.+) \(").unwrap();
        }

        let raw_xrandr =
            String::from_utf8(Command::new("xrandr").arg("--query").output().await?.stdout)
                .expect("failed to decode utf-8 from xrandr output");
        let orientation = Regex::captures_iter(&XRANDR_MONITOR_ORIENTATION, &raw_xrandr)
            .map(|cap| (String::from(&cap[1]), cap.get(2).map(|c| c.as_str())))
            .find_map(|cap| if cap.0 == self.output { cap.1 } else { None });

        match orientation {
            None => Ok(AbsoluteOrientation::Normal),
            Some("left") => Ok(AbsoluteOrientation::LeftUp),
            Some("inverted") => Ok(AbsoluteOrientation::Flipped),
            Some("right") => Ok(AbsoluteOrientation::RightUp),
            Some(_) => Ok(AbsoluteOrientation::Normal),
        }
    }

    async fn set_orientation(&self, orientation: AbsoluteOrientation) -> Result<()> {
        println!("xrandr: rotating {} to {:?}", self.output, orientation);
        Command::new("xrandr")
            .arg("--output")
            .arg(self.output.clone())
            .arg("--rotate")
            .arg(match orientation {
                AbsoluteOrientation::Normal => "normal",
                AbsoluteOrientation::RightUp => "right",
                AbsoluteOrientation::Flipped => "inverted",
                AbsoluteOrientation::LeftUp => "left",
            })
            .output()
            .await?;
        Ok(())
    }
}
