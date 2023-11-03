//! Describes the azimuthal mode in terms of the four state space parameters.

use crate::{Float, FRAC_PI_4, PI};
use serde::{Deserialize, Serialize};

/// Acoustic or heat release rate mode.
///
/// Acoustic or heat release rate mode expressed in terms of the four
/// state space variables: amplitude, orientation angle (azimuthal location
/// of the anti-node), the temporal phase, and the nature angle.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Mode {
    pub amplitude: Float,
    pub orientation_angle: Float,
    pub phase: Float,
    pub nature_angle: Float,
}

impl Mode {
    /// Create a new instance of [`Mode`]
    pub fn new(
        amplitude: Float,
        orientation_angle: Float,
        phase: Float,
        nature_angle: Float,
    ) -> Self {
        Self {
            amplitude,
            orientation_angle,
            phase,
            nature_angle,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.amplitude >= 0.0 && self.nature_angle.abs() <= PI / 4.0
    }
}

impl Default for Mode {
    /// Default value (standing mode with unit amplitude, zero phase and orientation angle).
    fn default() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0)
    }
}

impl From<SystemMode> for Mode {
    #[inline]
    fn from(value: SystemMode) -> Self {
        let amplitude = value.a();
        let orientation_angle = value.nth0();
        let phase = value.phi();
        let nature_angle = value.chi();

        Self::new(amplitude, orientation_angle, phase, nature_angle)
    }
}

/// Acoustic or heat release rate mode as expressed in the model.
///
/// Acoustic or heat release rate mode as expressed in the model.
/// The same parameters as in [`Mode`] are used to describe the
/// given mode, except the amplitude is instead expressed as the
/// natural logarithm of the amplitude, and the nature angle is
/// expressed as the tangent of twice the nature nature angle.
/// This ensures they stay within the physical bounds
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SystemMode {
    pub ln_a: Float,
    pub nth0: Float,
    pub phi: Float,
    pub tan_2chi: Float,
}

impl From<Mode> for SystemMode {
    #[inline]
    fn from(value: Mode) -> Self {
        Self::new(
            value.amplitude,
            value.orientation_angle,
            value.phase,
            value.nature_angle,
        )
    }
}

impl Default for SystemMode {
    /// Default value (standing mode with unit amplitude, zero phase and orientation angle).
    fn default() -> Self {
        Self::from(Mode::default())
    }
}

impl SystemMode {
    /// Create a new instance of [`Mode`]
    pub fn new(a: Float, nth0: Float, phi: Float, chi: Float) -> Self {
        // Check the input is in the valid range
        if a <= 0.0 {
            panic!("impossible to have non-negative amplitudes");
        }

        if chi.abs() > FRAC_PI_4 {
            panic!("impossible to have nature angle magnitudes above pi/4");
        }

        SystemMode {
            ln_a: a.ln(),
            nth0,
            phi,
            tan_2chi: (2.0 * chi).tan(),
        }
    }

    /// Calculate the local amplitude at a single location `theta`.
    #[inline]
    pub fn local_amplitude(&self, theta: Float, mode_order: u32) -> Float {
        let n = mode_order as Float;

        let cos = (n * theta - self.nth0()).cos() * self.chi().cos();
        let sin = (n * theta - self.nth0()).sin() * self.chi().sin();

        self.a() * (cos.powi(2) + sin.powi(2)).sqrt()
    }

    /// Returns the amplitude of the mode.
    #[inline]
    pub fn a(&self) -> Float {
        self.ln_a.exp()
    }

    /// Returns the orientation angle of the mode.
    #[inline]
    pub fn nth0(&self) -> Float {
        self.nth0
    }

    /// Returns the temporal phase of the mode.
    #[inline]
    pub fn phi(&self) -> Float {
        self.phi
    }

    /// Returns the nature angle of the acoustic mode.
    #[inline]
    pub fn chi(&self) -> Float {
        0.5 * self.tan_2chi.atan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amplitude() {
        let a = 1.3;
        let mode = SystemMode::new(a, 0.0, 0.0, 0.0);

        assert_eq!(mode.a(), a);
    }

    #[test]
    #[should_panic]
    fn invalid_amplitude_panic() {
        SystemMode::new(-0.4, 0.0, 0.0, 0.0);
    }

    #[test]
    fn chi() {
        let chi = 0.14;
        let mode = SystemMode::new(1.0, 0.0, 0.0, chi);

        assert_eq!(mode.chi(), chi);
    }

    #[test]
    #[should_panic]
    fn invalid_chi_panic() {
        SystemMode::new(1.0, 0.0, 0.0, -1.0);
    }
}
