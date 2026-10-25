/// Units used for trigonometric inputs and inverse trigonometric results.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum AngleMode {
    /// Angles are measured in radians.
    #[default]
    Radians,
    /// Angles are measured in degrees.
    Degrees,
}

impl AngleMode {
    pub(crate) fn input_radians(self, angle: f64) -> f64 {
        match self {
            Self::Radians => angle,
            Self::Degrees => angle.to_radians(),
        }
    }

    pub(crate) fn output_angle(self, radians: f64) -> f64 {
        match self {
            Self::Radians => radians,
            Self::Degrees => radians.to_degrees(),
        }
    }
}
