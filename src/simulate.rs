use crate::azimuthal_mode::SystemMode;
use crate::hrr_integral::HeatReleaseRate;
use crate::observers::ObserverTrait;
use crate::{Float, Quaternion, Settings};

impl Settings {
    pub fn run(&mut self) {
        // Save the initial mode
        let initial_mode = SystemMode::from(self.parameters.initial_mode);
        let initial_hrr_mode = self.describing_function.mode(&initial_mode);
        self.observer.log(&initial_mode, &initial_hrr_mode, 0.0);

        // Allocate variables for the mode
        let mut mode = initial_mode;

        // Make some shorthand notation
        let dt = self.parameters.get_timestep();

        for ind in 1..(self.parameters.get_total_steps() + 1) {
            // Find the right hand side of the discrete equation
            let rhs = self.get_rhs(&mode);

            // Update the solution
            self.update_mode(&mut mode, &rhs);

            // Save the mode at set intervals
            if (ind % self.parameters.get_skip_per_save()) == 0 {
                let time = (ind as Float) * dt;
                let hrr_mode = self.describing_function.mode(&mode);
                self.observer.log(&mode, &hrr_mode, time);
            }

            // Print progress to user at set intervals
            if (ind % (1000 * self.parameters.get_steps_per_cycle())) == 0 {
                println!(
                    "{}/{}",
                    ((ind as Float) * dt) as usize,
                    self.parameters.get_number_of_cycles()
                );
            }
        }
    }

    #[inline]
    fn deterministic_stochastic(&self, mode: &SystemMode) -> Quaternion {
        let nd_noise_sq = self.parameters.noise.powi(2) / (4.0 * mode.a().powi(2));
        let real = nd_noise_sq;
        let imag_i = 0.0;
        let imag_j = 0.0;
        let imag_k = nd_noise_sq * mode.tan_2chi;

        Quaternion::new(real, imag_i, imag_j, imag_k)
    }

    #[inline]
    fn get_rhs(&mut self, mode: &SystemMode) -> Quaternion {
        let dt = self.parameters.get_timestep();

        // Calculate the relative noise
        let relative_noise = self.parameters.noise / (mode.a() * Float::sqrt(2.0));

        // First, get the deterministic part (without dt)
        let hrr_integral = self.describing_function.integral(mode, self);
        let rhs_deterministic = hrr_integral + self.deterministic_stochastic(mode);

        // Obtain the stochastic part
        let rhs_stochastic = self.rng.get_random() * relative_noise;

        // Now multiply the determninistic part by dt and the stochastic part by sqrt(dt)
        rhs_deterministic * dt + rhs_stochastic * dt.sqrt()
    }

    #[inline]
    fn update_mode(&self, mode: &mut SystemMode, right_hand_side: &Quaternion) {
        // Introduce some sharthands
        let rhs = right_hand_side;
        let chi = mode.chi();
        let tan_2chi = mode.tan_2chi;

        // Update the mode
        mode.ln_a += rhs.real;
        mode.nth0 += rhs.imag_i - tan_2chi * rhs.imag_j;
        mode.phi += rhs.imag_j / (2.0 * chi).cos();
        mode.tan_2chi += -2.0 * rhs.imag_k / (2.0 * chi).cos().powi(2);
    }
}
