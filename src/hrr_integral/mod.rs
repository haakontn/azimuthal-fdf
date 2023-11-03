mod conventional;
mod simplified;

use crate::azimuthal_mode::SystemMode;
use crate::{Float, Parameters, Quaternion, Settings};
pub use conventional::ConventionalFDF;
use serde::{Deserialize, Serialize};
pub use simplified::AFDFSimplified;

pub trait HeatReleaseRate {
    fn integral(&self, acoustic_mode: &SystemMode, setup: &Settings) -> Quaternion;
    fn mode(&self, acoustic_mode: &SystemMode) -> SystemMode;
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum DescribingFunction {
    Conventional(ConventionalFDF),
    Simplified(AFDFSimplified),
}

impl HeatReleaseRate for DescribingFunction {
    fn integral(&self, acoustic_mode: &SystemMode, setup: &Settings) -> Quaternion {
        match self {
            Self::Conventional(hrr) => hrr.integral(acoustic_mode, setup),
            Self::Simplified(hrr) => hrr.integral(acoustic_mode, setup),
        }
    }

    fn mode(&self, acoustic_mode: &SystemMode) -> SystemMode {
        match self {
            Self::Conventional(hrr) => hrr.mode(acoustic_mode),
            Self::Simplified(hrr) => hrr.mode(acoustic_mode),
        }
    }
}

impl Default for DescribingFunction {
    fn default() -> Self {
        Self::Simplified(AFDFSimplified::new(1.6))
    }
}

// Calculate the local amplitude at each flame location.
#[inline]
pub fn local_amplitudes(mode: &SystemMode, parameters: &Parameters) -> Vec<Float> {
    let mode_order = parameters.mode_order;
    parameters
        .get_thetas()
        .iter()
        .map(|&theta| mode.local_amplitude(theta, mode_order))
        .collect()
}

/// Calculated the saturated gain values.
fn saturated_gain(
    hrr_mode: &SystemMode,
    acoustic_mode: &SystemMode,
    setup: &Settings,
) -> Vec<Float> {
    let local_amplitudes = local_amplitudes(&hrr_mode, &setup.parameters);
    let saturation_factor = setup.saturation.factor(&local_amplitudes);

    let ref_gain = setup.parameters.gain;
    let gain = ref_gain * hrr_mode.a() / acoustic_mode.a();

    saturation_factor.into_iter().map(|sf| gain * sf).collect()
}
