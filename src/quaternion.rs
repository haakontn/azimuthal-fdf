use crate::Float;

/// Quaternion number.
///
/// Quaternion number represented using four real valued numbers.
/// Simplifies some of the calculations when calculating the
/// values of the governing equation.
#[derive(Copy, Clone, Debug)]
pub struct Quaternion {
    pub real: Float,
    pub imag_i: Float,
    pub imag_j: Float,
    pub imag_k: Float,
}

impl Quaternion {
    #[inline]
    pub fn new(real: Float, imag_i: Float, imag_j: Float, imag_k: Float) -> Self {
        Self {
            real,
            imag_i,
            imag_j,
            imag_k,
        }
    }
}

impl std::ops::Add for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn add(self, rhs: Quaternion) -> Self::Output {
        let real = self.real + rhs.real;
        let imag_i = self.imag_i + rhs.imag_i;
        let imag_j = self.imag_j + rhs.imag_j;
        let imag_k = self.imag_k + rhs.imag_k;

        Self::new(real, imag_i, imag_j, imag_k)
    }
}

impl std::ops::Mul<Float> for Quaternion {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Float) -> Self::Output {
        let real = self.real * rhs;
        let imag_i = self.imag_i * rhs;
        let imag_j = self.imag_j * rhs;
        let imag_k = self.imag_k * rhs;

        Self::new(real, imag_i, imag_j, imag_k)
    }
}

impl std::ops::Div<Float> for Quaternion {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Float) -> Self::Output {
        self * (1.0 / rhs)
    }
}
