use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::StandardNormal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time;

use crate::hrr_integral::DescribingFunction;
use crate::observers::{Observer, ObserverTrait, SaveInfo};
use crate::{Float, Parameters, ParametersError, Quaternion, Saturation};

/// Struct containing most of the data from [`Settings`] for saving purposes.
///
/// The [`ThreadRng`] has some specific requirements for moving it in and out of
/// different threads. Therefore, this struct is used to pass (most of) the data
/// from the simulation out of any parallel for loops
#[derive(Debug)]
pub struct SaveData {
    parameters: Parameters,
    observer: Observer,
    describing_function: DescribingFunction,
    pub finish_time: time::SystemTime,
}

impl From<Settings> for SaveData {
    fn from(value: Settings) -> Self {
        Self {
            parameters: value.parameters,
            observer: value.observer,
            describing_function: value.describing_function,
            finish_time: time::SystemTime::now(),
        }
    }
}

impl SaveData {
    pub fn save(&self) -> hdf5::Result<()> {
        self.observer
            .save(&self.parameters, &self.describing_function)
    }

    pub fn get_save_info(&self) -> SaveInfo {
        self.observer.save_info()
    }
}

/// All the settings of the simulation.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub parameters: Parameters,
    pub saturation: Saturation,
    pub observer: Observer,
    pub describing_function: DescribingFunction,

    #[serde(skip)]
    pub rng: RNG,
}

impl Clone for Settings {
    /// Clone everything except the RNG state(!)
    fn clone(&self) -> Self {
        let parameters = self.parameters.clone();
        let saturation = self.saturation.clone();
        let observer = self.observer.clone();
        let describing_function = self.describing_function.clone();

        Self::new(parameters, saturation, observer, describing_function)
    }
}

impl Settings {
    pub fn new(
        parameters: Parameters,
        saturation: Saturation,
        mut observer: Observer,
        describing_function: DescribingFunction,
    ) -> Self {
        // Allocate space for the observer
        observer.reserve(parameters.get_num_steps_to_save());

        Self {
            parameters,
            observer,
            saturation,
            describing_function,
            rng: RNG::default(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let buffer = BufReader::new(File::open(path)?);

        let mut user_settings: Self = serde_json::from_reader(buffer)?;

        // Calculate all the values that are not included in the JSON
        user_settings.parameters.init()?;
        // Reserve space for the observer
        user_settings
            .observer
            .reserve(user_settings.parameters.get_num_steps_to_save());

        Ok(user_settings)
    }

    pub fn export(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self)?;

        Ok(())
    }

    /// Set the observer.
    pub fn set_observer(&mut self, observer: Observer) {
        self.observer = observer;

        self.observer
            .reserve(self.parameters.get_num_steps_to_save());
    }

    /// Set the saturation model.
    pub fn set_saturation(&mut self, saturation: Saturation) {
        self.saturation = saturation;
    }

    /// Set the time step.
    pub fn set_timestep(&mut self, dt: Float) -> Result<(), ParametersError> {
        self.parameters.set_timestep(dt)
    }

    /// Transfer ownership of the observer.
    pub fn get_observer(self) -> Observer {
        self.observer
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct RNG {
    rng: ThreadRng,
}

impl RNG {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    pub fn get_random(&mut self) -> Quaternion {
        let real = self.rng.sample(StandardNormal);
        let imag_i = self.rng.sample(StandardNormal);
        let imag_j = self.rng.sample(StandardNormal);
        let imag_k = self.rng.sample(StandardNormal);

        return Quaternion {
            real,
            imag_i,
            imag_j,
            imag_k,
        };
    }
}

impl Default for RNG {
    fn default() -> Self {
        Self::new()
    }
}
