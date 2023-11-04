use crate::azimuthal_mode::Mode;
use crate::Float;
use serde::{Deserialize, Serialize};

/// Possible errors for [`Parameters`].
#[derive(Clone, Debug)]
pub enum ParametersError {
    Timestep,
    Saving,
    NegativeNumber,
    Mode,
}

impl std::error::Error for ParametersError {}

impl std::fmt::Display for ParametersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::Timestep => "timestep has to satisfy 0.0 < dt < 1.0".to_owned(),
            Self::Saving => "saves_per_cycle >= 2 required to avoid undersampling".to_owned(),
            Self::NegativeNumber => "negative value where positive value was expected".to_owned(),
            Self::Mode => "invalid initial mode".to_owned(),
        };

        write!(f, "error setting the parameters: {}", msg)
    }
}

/// Parameters describing the system to be simulated.
///
/// Parameters describing the system in terms of `damping`, `gain`,
/// and `noise`. Additionally, the AFDF parameter `r` describes the
/// degree of asymmetry in the system, with `r = 1.0` being the
/// conventional FDF case (symmetric).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub damping: Float,
    pub gain: Float,
    pub noise: Float,
    pub mode_order: u32,
    pub number_of_burners: u32,
    pub initial_mode: Mode,
    timestep: Float,
    number_of_cycles: Float,
    saves_per_cycle: usize,

    #[serde(skip)]
    skip_per_save: usize,

    #[serde(skip)]
    total_steps: usize,

    #[serde(skip)]
    steps_per_cycle: usize,

    #[serde(skip)]
    num_steps_to_save: usize,

    #[serde(skip)]
    thetas: Vec<Float>,
}

impl Parameters {
    /// Create a new instance of [`Parameters`]
    pub fn new(
        damping: Float,
        gain: Float,
        noise: Float,
        mode_order: u32,
        number_of_burners: u32,
        initial_mode: Mode,
        timestep: Float,
        number_of_cycles: Float,
        saves_per_cycle: usize,
    ) -> Result<Self, ParametersError> {
        if timestep <= 0.0 || timestep > 1.0 {
            return Err(ParametersError::Timestep);
        }
        if saves_per_cycle < 2 {
            return Err(ParametersError::Saving);
        }

        // Insert the user provided data, use default values for the rest
        let mut parameters = Self {
            damping,
            gain,
            noise,
            mode_order,
            number_of_burners,
            initial_mode,
            timestep,
            number_of_cycles,
            saves_per_cycle,
            skip_per_save: 0,
            total_steps: 0,
            steps_per_cycle: 0,
            num_steps_to_save: 0,
            thetas: Vec::new(),
        };

        // Initialize the rest of the variables
        // Doing it this way is conenient for later
        parameters.init()?;

        Ok(parameters)
    }

    /// Initialize the private fields in [`Parameters`].
    ///
    /// This is always ran as part of [`Parameters::new`], but it is also
    /// used by [`crate::Settings`] when loading from file. It should
    /// never be required to be used manually by the user.
    pub fn init(&mut self) -> Result<(), ParametersError> {
        // Set up the theta locations (assumed equidistantly spaced)
        let dtheta = 2.0 * crate::PI / (self.number_of_burners as Float);

        self.thetas = (0..self.number_of_burners)
            .map(|ind| dtheta * (ind as Float))
            .collect();

        self.set_timestep(self.timestep)
    }

    /// Set the time step (and update dependent parameters).
    pub fn set_timestep(&mut self, new_timestep: Float) -> Result<(), ParametersError> {
        // Can only have positive time steps
        if new_timestep <= 0.0 || new_timestep > 1.0 {
            return Err(ParametersError::Timestep);
        }
        if self.number_of_cycles < 0.0 {
            return Err(ParametersError::NegativeNumber);
        }

        let saves_per_cycle = self.saves_per_cycle;
        self.total_steps = (self.number_of_cycles / new_timestep).ceil() as usize;
        self.steps_per_cycle = (1.0 / new_timestep).round() as usize;
        self.skip_per_save = self.steps_per_cycle / saves_per_cycle;
        self.num_steps_to_save = self.total_steps / self.skip_per_save;

        self.timestep = new_timestep;

        Ok(())
    }

    #[inline]
    pub fn get_timestep(&self) -> Float {
        self.timestep
    }

    /// Set the total number of cycles (and update dependent parameters).
    pub fn set_number_of_cycles(&mut self, number_of_cycles: Float) -> Result<(), ParametersError> {
        self.number_of_cycles = number_of_cycles;

        // Recalculate the other relevant parameters
        // This should not be able to fail under regular circumstances
        self.set_timestep(self.timestep)
    }

    #[inline]
    pub fn get_number_of_cycles(&self) -> Float {
        self.number_of_cycles
    }

    /// Set the number of saving points per cycle (and update depdendent parameters).
    pub fn set_saves_per_cycle(&mut self, saves_per_cycle: usize) -> Result<(), ParametersError> {
        if saves_per_cycle < 2 {
            return Err(ParametersError::Saving);
        }

        self.saves_per_cycle = saves_per_cycle;
        self.set_timestep(self.timestep)
    }

    #[inline]
    pub fn get_saves_per_cycle(&self) -> usize {
        self.saves_per_cycle
    }

    /// Set the initial [`Mode`] of the simulation.
    pub fn set_initial_mode(&mut self, mode: Mode) {
        self.initial_mode = mode;
    }

    /// Get the azimuthal locations of the burners.
    #[inline]
    pub fn get_thetas<'a>(&'a self) -> &'a [Float] {
        &self.thetas
    }

    #[inline]
    pub fn get_num_steps_to_save(&self) -> usize {
        self.num_steps_to_save
    }

    #[inline]
    pub fn get_total_steps(&self) -> usize {
        self.total_steps
    }

    #[inline]
    pub fn get_steps_per_cycle(&self) -> usize {
        self.steps_per_cycle
    }

    #[inline]
    pub fn get_skip_per_save(&self) -> usize {
        self.skip_per_save
    }
}

impl Default for Parameters {
    fn default() -> Self {
        let gain = 0.16 / crate::PI;
        let damping = gain * 0.2;
        let noise = 0.06;
        let mode_order = 1;
        let dt = 1e-4;
        let number_of_cycles = 52000.0;
        let number_of_burners = 12;
        let saves_per_cycle = 50;
        let initial_mode = Mode::default();

        Parameters::new(
            damping,
            gain,
            noise,
            mode_order,
            number_of_burners,
            initial_mode,
            dt,
            number_of_cycles,
            saves_per_cycle,
        )
        .unwrap()
    }
}

impl std::fmt::Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json_string = serde_json::to_string(self).unwrap_or_default();

        write!(f, "{}", json_string)
    }
}
