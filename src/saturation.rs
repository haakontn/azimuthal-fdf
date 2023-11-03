use crate::Float;
use serde::{Deserialize, Serialize};

/// Saturation factor of different forms.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Saturation {
    /// Saturation based on the tangent function.
    Tangent(Float),
    /// Exponential saturation.
    Exponential(Float),
}

impl Saturation {
    /// Get the saturation factor based on local amplitude
    ///
    /// The saturation factor is a function which approaches
    /// unity in the low amplitude limit (no saturation), and
    /// it should approach zero in the high amplitude limit
    #[inline]
    pub fn factor(&self, local_amplitudes: &[Float]) -> Vec<Float> {
        match self {
            Self::Tangent(kappa) => local_amplitudes
                .iter()
                .map(|&a| 2.0 / (1.0 + (1.0 + (kappa * a).powi(2)).sqrt()))
                .collect(),
            Self::Exponential(kappa) => local_amplitudes
                .iter()
                .map(|&a| (-kappa * a).exp())
                .collect(),
        }
    }
}

impl Default for Saturation {
    fn default() -> Self {
        Self::Tangent(6.0)
    }
}

impl std::fmt::Display for Saturation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // If it can't parse the Enum to string, just return and empty string.
        // There is no need to panic the whole program just because this
        // feature used for printing (and saving the settings) fails
        let serde_string = serde_json::to_string(self).unwrap_or_default();

        write!(f, "{}", serde_string)
    }
}
