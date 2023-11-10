use super::HeatReleaseRate;
use crate::azimuthal_mode::SystemMode;
use crate::{Float, Fourier, Quaternion, Settings};
use serde::{Deserialize, Serialize};

/// Simplified version of the Azimuthal Flame Describing Function (AFDF).
///
/// This is the simplified version of the azimuthal describing function,
/// which is presented in the main part of the paper. This assumes the
/// main difference from the conventional flame describing function is
/// the nature angle dependence of the heat release rate mode.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct AFDFSimplified {
    pub gain_ratio_r: Float,
}

impl AFDFSimplified {
    pub fn new(gain_ratio_r: Float) -> Self {
        if gain_ratio_r < 0.0 {
            panic!("gain_ratio_r should be positive!")
        }

        Self { gain_ratio_r }
    }
}

impl Default for AFDFSimplified {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl HeatReleaseRate for AFDFSimplified {
    fn integral(&self, acoustic_mode: &SystemMode, setup: &Settings) -> Quaternion {
        let hrr_mode = self.mode(acoustic_mode);
        // let aq = local_amplitudes(&hrr_mode, &setup.parameters);
        let gain_values = super::saturated_gain(&hrr_mode, acoustic_mode, setup);

        // Calculate the Fourier components
        let n = setup.parameters.mode_order;
        let thetas = setup.parameters.get_thetas();
        let fourier0 = Fourier::coefficient(thetas, &gain_values, 0, hrr_mode.nth0());
        let fourier2n = Fourier::coefficient(thetas, &gain_values, 2 * n, hrr_mode.nth0());

        // Make shorthand for the Fourier coefficient components
        let n0 = fourier0.amplitude;
        let n2n = fourier2n.amplitude;
        let theta2n = fourier2n.phase;

        // Nature angle difference between HRR and acoustic mode
        let delta_chi = hrr_mode.chi() - acoustic_mode.chi();
        let chi = acoustic_mode.chi();
        // let a = acoustic_mode.a();

        let alpha = setup.parameters.damping;
        // let sigma = setup.parameters.noise;
        // First element
        let n = n as Float;
        let mut real = 0.5 * n2n * (2.0 * n * theta2n).cos() * (2.0 * chi + delta_chi).cos();
        real += n0 * delta_chi.cos() - alpha;
        // real += sigma.powi(2) / (4.0 * a.powi(2));

        // Second element
        let imag_i = 0.5 * (n2n * (2.0 * n * theta2n).sin() * (2.0 * chi + delta_chi).cos());

        // Third element
        let imag_j = 0.5 * (-n2n * (2.0 * n * theta2n).sin() * (2.0 * chi + delta_chi).sin());

        // Fourth element
        let mut imag_k = 0.5 * n2n * (2.0 * n * theta2n).cos() * (2.0 * chi + delta_chi).sin();
        imag_k += -n0 * delta_chi.sin();
        // imag_k += (sigma.powi(2) / (4.0 * a.powi(2))) * acoustic_mode.tan_2chi;

        Quaternion {
            real,
            imag_i,
            imag_j,
            imag_k,
        }
    }

    fn mode(&self, acoustic_mode: &SystemMode) -> SystemMode {
        // Orientation angle and nature angle are assumed to
        // be the same as for the acoustic mode
        let mut mode = *acoustic_mode;

        // Shorthand for the acoustic nature angle
        let chi = acoustic_mode.chi();

        // Modified amplitude A_{q}
        let rsq = self.gain_ratio_r.powi(2);
        let amp_factor = (1.0 + (rsq - 1.0) / (rsq + 1.0) * (2.0 * chi).sin()).sqrt();
        let amplitude = acoustic_mode.a() * amp_factor;

        // Nature angle
        let r = self.gain_ratio_r;
        let num = (r - 1.0) * chi.cos() + (r + 1.0) * chi.sin();
        let den = (r + 1.0) * chi.cos() + (r - 1.0) * chi.sin();
        let nature_angle = (num / den).atan();

        mode.ln_a = amplitude.ln();
        mode.tan_2chi = (2.0 * nature_angle).tan();

        mode
    }
}
