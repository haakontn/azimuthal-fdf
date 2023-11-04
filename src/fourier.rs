use crate::Float;

/// Fourier series component.
#[derive(Debug)]
pub struct Fourier {
    pub amplitude: Float,
    pub phase: Float,
}

impl Fourier {
    fn new(amplitude: Float, phase: Float) -> Option<Self> {
        if amplitude < 0.0 {
            return None;
        }

        Some(Self { amplitude, phase })
    }

    /// Calculate specific Fourier component for a given signal
    ///
    /// Calculates the Fourier component of order `order` for the signal `signal`
    /// sampled at the discrete spatial locations given by `thetas`, using the
    /// following definitions:
    ///
    /// amplitude = sum_{theta} sig(theta) * cos(i * (theta - phase - ntheta0))
    ///         0 = sum_{theta} sig(theta) * sin(i * (theta - phase - ntheta0))
    ///
    /// where `ntheta0` is a given reference angle. The function then calculates
    /// the value of `amplitude` and `phase`, which are both real valued variables.
    ///
    /// # Special case
    ///
    /// The angle thetai is not defined for the zeroth Fourier coefficient
    /// (`order = 0`), and is given as Float::NAN in that case. This can be tested
    /// for with `Fourier.phase.is_nan()`
    ///
    /// # Example
    ///
    /// Create a synthetic signal with known coefficients, and get the
    /// coefficients used to create the signal back.
    ///
    /// ```
    /// use azimuthal_fdf::{Float, PI};
    /// use azimuthal_fdf::Fourier;
    ///
    /// let n_points = 12;
    /// let dtheta = 2.0 * PI / (n_points as Float);
    ///
    /// // Set up the parameters to create the signal
    /// let n2 = 0.2;
    /// let n0 = 0.9;
    /// let order = 2;
    /// let nth0 = 0.5;
    /// let thi = 0.18;
    ///
    /// // Create signal and measurement locations
    /// let thetas: Vec<Float> = (0..n_points).map(|i| (i as Float) * dtheta).collect();
    /// let sig: Vec<Float> = (0..n_points)
    ///     .map(|i| n0 + n2 * ((order as Float) * ((dtheta * (i as Float)) - thi) - nth0).cos())
    ///     .collect();
    ///
    /// // Extract the zeroth component
    /// let fourier = Fourier::coefficient(&thetas, &sig, 0, nth0);
    /// println!("Zeroth Fourier coefficient: {:?}", fourier);
    ///
    /// assert!((n0 - fourier.amplitude).abs() < Float::EPSILON);
    /// assert!(fourier.phase.is_nan());
    ///
    /// // Extract the non-zero component
    /// let fourier = Fourier::coefficient(&thetas, &sig, order, nth0);
    /// println!("Non-zero Fourier coefficient: {:?}", fourier);
    ///
    /// assert!((n2 - fourier.amplitude).abs() < 10.0 * Float::EPSILON);
    /// assert!((thi - fourier.phase).abs() < 10.0 * Float::EPSILON);
    /// ```
    pub fn coefficient(thetas: &[Float], signal: &[Float], order: u32, ntheta0: Float) -> Self {
        // Treat the special case of wanting the zeroth coefficient
        if order == 0 {
            // Corresponds to the mean response, and the phase is not well defined
            return Fourier {
                amplitude: signal.iter().sum::<Float>() / (signal.len() as Float),
                phase: Float::NAN,
            };
        }

        // Cast the order to a floating point number
        let forder = order as Float;

        // Calcluate the sine and cosine terms
        let mut sin_term: Float = 0.0;
        let mut cos_term: Float = 0.0;
        for (&th, &s) in thetas.iter().zip(signal) {
            let idth = (forder * th) - ntheta0;

            sin_term += s * idth.sin();
            cos_term += s * idth.cos();
        }

        let pre_factor: Float = if order == thetas.len() as u32 / 2 {
            1.0
        } else {
            2.0
        };
        let n_terms = thetas.len() as Float;
        let sin_term = pre_factor * sin_term / n_terms;
        let cos_term = pre_factor * cos_term / n_terms;

        // Calculate the magnitude and the angle used in the
        // definition of the Fourier coefficients here
        let amplitude = (sin_term.powi(2) + cos_term.powi(2)).sqrt();

        // Calculate the value of thetai
        let phase = Float::atan2(sin_term, cos_term) / forder;

        // The amplitude is non-negative by definition,
        // allowing for the use of unwrap without the possibility
        // of causing a panic in the program
        return Self::new(amplitude, phase).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::PI;

    #[test]
    fn zeroth_fourier_coeff() {
        let amplitude = 0.75;
        let order = 2;
        let forder = order as Float;

        // Set up the measurement points
        const NPOINTS: usize = 12;
        let dtheta = 2.0 * PI / (NPOINTS as Float);
        let thetas: Vec<Float> = (0..NPOINTS).map(|i| (i as Float) * dtheta).collect();

        // Create the signal to get the coefficient(s) for
        let nth0 = 0.25;
        let thi = 0.5;
        let mut amps = [0.0; NPOINTS];
        for (ind, th) in thetas.iter().enumerate() {
            amps[ind] = amplitude + 0.2 * (forder * (th - thi) - nth0).cos();
        }

        let coeff = Fourier::coefficient(&thetas, &amps, 0, 0.0);

        assert!((coeff.amplitude - amplitude).abs() < Float::EPSILON);
    }

    #[test]
    fn nonzero_fourier_coeff() {
        let amplitude = 0.36;
        let order = 2;
        let forder = order as Float;

        // Set up the measurement points
        const NPOINTS: usize = 12;
        let dtheta = 2.0 * PI / (NPOINTS as Float);
        let thetas: Vec<Float> = (0..NPOINTS).map(|i| (i as Float) * dtheta).collect();

        // Create the signal to get the coefficient(s) for
        let ntheta0 = 0.4;
        let thi = 0.18;
        let mut amps = [0.0; NPOINTS];
        for (ind, th) in thetas.iter().enumerate() {
            amps[ind] = amplitude + 0.2 * (forder * (th - thi) - ntheta0).cos();
        }

        let coeff = Fourier::coefficient(&thetas, &amps, order, ntheta0);

        // Need to tolerate slightly higher inaccuracy than maching precision
        // since there are several trigonometric functions, etc.
        let precision = 10.0 * Float::EPSILON;

        assert!((coeff.amplitude - amplitude) < precision);
        assert!((coeff.phase - thi).abs() <= precision);
    }
}
