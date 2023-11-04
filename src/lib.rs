pub mod azimuthal_mode;
mod fourier;
pub mod hrr_integral;
pub mod observers;
mod parameters;
mod quaternion;
mod saturation;
mod settings;
mod simulate;

pub use fourier::Fourier;
pub use parameters::{Parameters, ParametersError};
pub use quaternion::Quaternion;
pub use saturation::Saturation;
pub use settings::{SaveData, Settings};

/// Floating point precision.
pub type Float = f64;
/// The value of pi at the given floating point precision.
pub const PI: Float = std::f64::consts::PI;
/// The value of pi/4 at the given floating point precision.
pub const FRAC_PI_4: Float = std::f64::consts::FRAC_PI_4;
