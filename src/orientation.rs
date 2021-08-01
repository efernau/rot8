//! # Rotation
//!
//! We differentiate between absolute and relative rotations so that
//! there is type-based safety that it's harder to misuse.

use crate::error::{Error, Result};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AbsoluteOrientation {
    /// Zero degree Rotation; Don't rotate.
    Normal,
    /// 90 degree Clockwise Rotation; Screen "Up" will be on the right.
    RightUp,
    /// 180 degree Rotation; Screen will be flipped.
    Flipped,
    /// 270 degree Clockwise Rotation; Screen "Up" will be on the left side.
    LeftUp,
}

impl Rotation {
    /// Convert to clockwise degrees.
    pub fn to_degrees(&self) -> isize {
        match *self {
            Self::None => 0,
            Self::Clockwise90 => 90,
            Self::Clockwise180 => 180,
            Self::Clockwise270 => 270,
        }
    }

    /// Attempt conversion from degrees to rotation.
    /// Positive value is clockwise, negative is counter clockwise.
    pub fn from_degrees(cw_degrees: isize) -> Result<Self> {
        match cw_degrees % 360 {
            0 => Ok(Self::None),
            90 | -270 => Ok(Self::Clockwise90),
            180 | -180 => Ok(Self::Clockwise180),
            270 | -90 => Ok(Self::Clockwise270),
            other => Err(Error::InvalidDegrees(other)),
        }
    }
}

impl std::ops::Add<Rotation> for Rotation {
    type Output = Rotation;

    fn add(self, rhs: Rotation) -> Self::Output {
        Rotation::from_degrees(self.to_degrees() + rhs.to_degrees())
            .expect("adding 90 degree values should never fail")
    }
}

impl From<AbsoluteOrientation> for Rotation {
    fn from(abs: AbsoluteOrientation) -> Self {
        match abs {
            AbsoluteOrientation::Normal => Rotation::None,
            AbsoluteOrientation::RightUp => Rotation::Clockwise90,
            AbsoluteOrientation::Flipped => Rotation::Clockwise180,
            AbsoluteOrientation::LeftUp => Rotation::Clockwise270,
        }
    }
}

impl From<Rotation> for AbsoluteOrientation {
    fn from(rel: Rotation) -> Self {
        match rel {
            Rotation::None => AbsoluteOrientation::Normal,
            Rotation::Clockwise90 => AbsoluteOrientation::RightUp,
            Rotation::Clockwise180 => AbsoluteOrientation::Flipped,
            Rotation::Clockwise270 => AbsoluteOrientation::LeftUp,
        }
    }
}

impl std::ops::Add<Rotation> for AbsoluteOrientation {
    type Output = AbsoluteOrientation;

    fn add(self, rhs: Rotation) -> Self::Output {
        AbsoluteOrientation::from(Rotation::from(self) + rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_to_degrees() -> Result<()> {
        assert_eq!(Rotation::None.to_degrees(), 0);
        assert_eq!(Rotation::Clockwise90.to_degrees(), 90);
        assert_eq!(Rotation::Clockwise180.to_degrees(), 180);
        assert_eq!(Rotation::Clockwise270.to_degrees(), 270);
        Ok(())
    }

    #[test]
    fn degrees_to_rotation() -> Result<()> {
        // Clockwise degrees.
        assert_eq!(Rotation::from_degrees(0)?, Rotation::None);
        assert_eq!(Rotation::from_degrees(90)?, Rotation::Clockwise90);
        assert_eq!(Rotation::from_degrees(180)?, Rotation::Clockwise180);
        assert_eq!(Rotation::from_degrees(270)?, Rotation::Clockwise270);
        assert_eq!(Rotation::from_degrees(360)?, Rotation::None);

        // Counter-clockwise degrees.
        assert_eq!(Rotation::from_degrees(-90)?, Rotation::Clockwise270);
        assert_eq!(Rotation::from_degrees(-180)?, Rotation::Clockwise180);
        assert_eq!(Rotation::from_degrees(-270)?, Rotation::Clockwise90);
        assert_eq!(Rotation::from_degrees(-360)?, Rotation::None);

        // Test the modulo.
        assert_eq!(Rotation::from_degrees(720)?, Rotation::None);
        assert_eq!(Rotation::from_degrees(-720)?, Rotation::None);
        assert_eq!(Rotation::from_degrees(810)?, Rotation::Clockwise90);
        assert_eq!(Rotation::from_degrees(-810)?, Rotation::Clockwise270);

        // Test invalid input.
        assert!(Rotation::from_degrees(42).is_err());

        Ok(())
    }

    #[test]
    fn rotation_addition() -> Result<()> {
        assert_eq!(Rotation::None + Rotation::None, Rotation::None);
        assert_eq!(
            Rotation::Clockwise90 + Rotation::Clockwise90,
            Rotation::Clockwise180
        );
        assert_eq!(
            Rotation::Clockwise180 + Rotation::Clockwise180,
            Rotation::None
        );
        assert_eq!(
            Rotation::Clockwise270 + Rotation::Clockwise180,
            Rotation::Clockwise90
        );

        Ok(())
    }

    #[test]
    fn abs_plus_rel() -> Result<()> {
        assert_eq!(
            AbsoluteOrientation::Normal + Rotation::None,
            AbsoluteOrientation::Normal
        );
        assert_eq!(
            AbsoluteOrientation::Normal + Rotation::Clockwise90,
            AbsoluteOrientation::RightUp
        );
        assert_eq!(
            AbsoluteOrientation::RightUp + Rotation::Clockwise90,
            AbsoluteOrientation::Flipped
        );
        assert_eq!(
            AbsoluteOrientation::Flipped + Rotation::Clockwise90,
            AbsoluteOrientation::LeftUp
        );
        assert_eq!(
            AbsoluteOrientation::LeftUp + Rotation::Clockwise90,
            AbsoluteOrientation::Normal
        );
        assert_eq!(
            AbsoluteOrientation::Flipped + Rotation::Clockwise180,
            AbsoluteOrientation::Normal
        );
        Ok(())
    }
}
