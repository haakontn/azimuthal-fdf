use std::path::PathBuf;
use std::str::FromStr;

use super::{ObserverTrait, SaveInfo};
use crate::azimuthal_mode::SystemMode;
use crate::hrr_integral::{DescribingFunction, HeatReleaseRate};
use crate::{Float, Parameters};
use hdf5;
use serde::{Deserialize, Serialize};

/// Time series observer.
///
/// Logging time series data at set intervals
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeSeriesObserver {
    pub save_info: SaveInfo,

    #[serde(skip)]
    modes: Vec<SystemMode>,

    #[serde(skip)]
    time: Vec<Float>,
}

impl TimeSeriesObserver {
    pub fn new(output_filepath: &PathBuf, group_name: Option<&str>) -> TimeSeriesObserver {
        // Set up the save information (path and group name)
        let mut save_info = SaveInfo::default();
        save_info.set_path(output_filepath);
        if let Some(group) = group_name {
            save_info.set_group(group);
        }

        TimeSeriesObserver {
            save_info,
            modes: Vec::new(),
            time: Vec::new(),
        }
    }

    /// Create [`TimeSeriesObserver`] observer with a given capacity.
    pub fn with_capacity(capacity: usize) -> TimeSeriesObserver {
        TimeSeriesObserver {
            save_info: SaveInfo::default(),
            modes: Vec::with_capacity(capacity),
            time: Vec::with_capacity(capacity),
        }
    }

    /// Reserves capacity for storing the time series.
    ///
    /// Reserves storage capacity for the time series such
    /// that the number of elements is at least `additional`or larger.
    pub fn reserve(&mut self, additional: usize) {
        self.modes.reserve(additional);
        self.time.reserve(additional);
    }
}

impl Default for TimeSeriesObserver {
    fn default() -> Self {
        let output_filepath = PathBuf::from("simulation_time_series.hdf5");

        Self::new(&output_filepath, None)
    }
}

impl From<SaveInfo> for TimeSeriesObserver {
    fn from(value: SaveInfo) -> Self {
        Self::new(&value.path, Some(&value.group))
    }
}

impl std::fmt::Display for TimeSeriesObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_string = serde_json::to_string(self).unwrap_or_default();
        write!(f, "TimeSeriesObserver: {}", data_string)
    }
}

impl FromStr for TimeSeriesObserver {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ObserverTrait for TimeSeriesObserver {
    #[inline]
    fn log(&mut self, acoustic_mode: &SystemMode, _hrr_mode: &SystemMode, time: Float) {
        self.modes.push(*acoustic_mode);
        self.time.push(time);
    }

    fn save(
        &self,
        parameters: &Parameters,
        describing_function: &DescribingFunction,
    ) -> hdf5::Result<()> {
        // Open the file
        let file = hdf5::File::append(&self.save_info.path)?;

        // Obtain the group to save the data to
        // Load the group name, or use the default
        let group = file.create_group(&self.save_info.group)?;

        // Save the time of each sample
        super::write_dataset(&group, &self.time, "time")?;

        // Convert the data into individual vectors
        let a: Vec<Float> = self.modes.iter().map(|mode| mode.a()).collect();
        super::write_dataset(&group, &a, "amplitude")?;

        let nth0: Vec<Float> = self.modes.iter().map(|mode| mode.nth0()).collect();
        super::write_dataset(&group, &nth0, "ntheta_0")?;

        let phi: Vec<Float> = self.modes.iter().map(|mode| mode.phi()).collect();
        super::write_dataset(&group, &phi, "phi")?;

        let chi: Vec<Float> = self.modes.iter().map(|mode| mode.chi()).collect();
        super::write_dataset(&group, &chi, "chi")?;

        let mut chi_q = Vec::with_capacity(chi.len());
        for acoustic_mode in &self.modes {
            let hrr_mode = describing_function.mode(&acoustic_mode);
            chi_q.push(hrr_mode.chi());
        }
        super::write_dataset(&group, &chi_q, "chi_q")?;

        super::save_parameters_as_attribute_json(&group, parameters)
    }
}
