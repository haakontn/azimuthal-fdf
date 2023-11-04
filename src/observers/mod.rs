//! Observers used for logging the [`crate::azimuthal_mode::Mode`].

mod histogram;
mod timeseries;

use std::path::PathBuf;

pub use histogram::HistogramObserver;
pub use timeseries::TimeSeriesObserver;

use crate::azimuthal_mode::SystemMode;
use crate::hrr_integral::DescribingFunction;
use crate::{Float, Parameters};
use hdf5::{H5Type, Location};
use ndarray::{arr0, ArrayView};
use serde::{Deserialize, Serialize};

/// Possible errors for [`SaveInfo`].
#[derive(Debug)]
pub enum ObserverError {
    DirectoryNotFound(SaveInfo),
    GroupAlreadyExist(SaveInfo),
}

impl std::error::Error for ObserverError {}

impl std::fmt::Display for ObserverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::DirectoryNotFound(save_info) => {
                format!(
                    "Could not find directory of \"{}\"",
                    save_info.path.to_string_lossy()
                )
            }
            Self::GroupAlreadyExist(save_info) => {
                format!(
                    "Group \"{}\" in file \"{}\" already exists",
                    save_info.group,
                    save_info.path.to_string_lossy()
                )
            }
        };

        write!(f, "ObserverError: {}", msg)
    }
}

/// Observer for logging data during the simulation.
pub trait ObserverTrait: std::fmt::Display {
    /// Log the current state of the system.
    fn log(&mut self, acoustic_mode: &SystemMode, hrr_mode: &SystemMode, time: Float);
    /// Save the observed data to file.
    fn save(
        &self,
        parameters: &Parameters,
        describing_function: &DescribingFunction,
    ) -> hdf5::Result<()>;
}

/// Wrapper for the structs implementing [`ObserverTrait`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Observer {
    TimeSeries(TimeSeriesObserver),
    Histogram(HistogramObserver),
}

impl Observer {
    pub fn new_timeseries(save_info: SaveInfo, capacity: usize) -> Self {
        let mut tso = TimeSeriesObserver::from(save_info);
        tso.reserve(capacity);

        Self::TimeSeries(tso)
    }

    pub fn new_histogram(save_info: SaveInfo, amplitude_limit: Float) -> Self {
        let mut ho = HistogramObserver::from(save_info);
        ho.set_amplitude_limit(amplitude_limit);

        Self::Histogram(ho)
    }

    pub fn valid_path(&self) -> Result<(), ObserverError> {
        match self {
            Self::TimeSeries(obs) => obs.save_info.is_valid(),
            Self::Histogram(obs) => obs.save_info.is_valid(),
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        if let Self::TimeSeries(obs) = self {
            obs.reserve(additional);
        }
    }

    pub fn set_save_info(&mut self, save_info: &SaveInfo) {
        match self {
            Self::TimeSeries(obs) => obs.save_info = save_info.clone(),
            Self::Histogram(obs) => obs.save_info = save_info.clone(),
        };
    }

    pub fn save_info(&self) -> SaveInfo {
        match self {
            Self::TimeSeries(obs) => obs.save_info.clone(),
            Self::Histogram(obs) => obs.save_info.clone(),
        }
    }
}

impl ObserverTrait for Observer {
    #[inline]
    fn log(&mut self, acoustic_mode: &SystemMode, hrr_mode: &SystemMode, time: Float) {
        match self {
            Self::TimeSeries(obs) => obs.log(acoustic_mode, hrr_mode, time),
            Self::Histogram(obs) => obs.log(acoustic_mode, hrr_mode, time),
        }
    }

    fn save(
        &self,
        parameters: &Parameters,
        describing_function: &DescribingFunction,
    ) -> hdf5::Result<()> {
        match self {
            Self::TimeSeries(obs) => obs.save(parameters, describing_function),
            Self::Histogram(obs) => obs.save(parameters, describing_function),
        }
    }
}

impl std::fmt::Display for Observer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serde_string = serde_json::to_string(self).unwrap_or_default();

        write!(f, "{}", serde_string)
    }
}

impl Default for Observer {
    fn default() -> Self {
        Self::TimeSeries(TimeSeriesObserver::default())
    }
}

/// Information of where the results will be saved.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SaveInfo {
    path: PathBuf,
    group: String,
}

impl SaveInfo {
    pub fn new(path: &PathBuf, group: &str) -> Self {
        Self {
            path: path.to_owned(),
            group: group.to_owned(),
        }
    }

    pub fn set_path(&mut self, new_path: &PathBuf) {
        self.path = new_path.to_owned();
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_group(&mut self, new_group: &str) {
        self.group = new_group.to_owned();
    }

    pub fn get_group(&self) -> &str {
        &self.group
    }

    pub fn is_valid(&self) -> Result<(), ObserverError> {
        // First, check if the directory exists
        let directory = std::path::Path::new(&self.path).parent();
        if let None = directory {
            return Err(ObserverError::DirectoryNotFound(self.to_owned()));
        }

        // Check if the file already exists
        let file = hdf5::File::open(&self.path);
        let file = match file {
            Ok(file) => file,
            // If the directory exist, but the file does not exist yet
            // it should be fine to save the results in the intended location
            Err(_) => return Ok(()),
        };

        // If the file exists already, make sure the group does not already exist
        match file.group(&self.group) {
            // The group already exists, it will not overwrite the results
            Ok(_) => Err(ObserverError::GroupAlreadyExist(self.to_owned())),
            // The group does not already exist, it will write the results in the desired location
            Err(_) => Ok(()),
        }
    }
}

impl Default for SaveInfo {
    fn default() -> Self {
        let path = PathBuf::from("simulation_data.hdf5");
        let group = "data";

        Self::new(&path, group)
    }
}

impl std::fmt::Display for SaveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "path: {}, group: {}",
            self.path.to_str().unwrap_or_default(),
            self.group
        )
    }
}

#[allow(dead_code)]
/// Save the [`Parameters`] as a HDF5 attribute of the group
fn save_parameters_as_attribute(group: &Location, parameters: &Parameters) -> hdf5::Result<()> {
    // Save the relevant bits of the parameters
    save_attr(group, &arr0(parameters.damping), "damping")?;
    save_attr(group, &arr0(parameters.gain), "gain")?;
    save_attr(group, &arr0(parameters.noise), "noise")?;
    save_attr(group, &arr0(parameters.mode_order), "mode_order")?;
    save_attr(group, &arr0(parameters.get_timestep()), "dt")?;
    save_attr(
        group,
        &arr0(parameters.get_number_of_cycles()),
        "number_of_cycles",
    )?;
    save_attr(
        group,
        &arr0(parameters.number_of_burners),
        "number_of_burners",
    )
}

fn save_parameters_as_attribute_json(
    group: &Location,
    parameters: &Parameters,
) -> hdf5::Result<()> {
    let save_string = parameters.to_string();
    save_str_attr(group, &save_string, "parameters")
}

/// Save `data` as an HDF5 attribute.
fn save_attr<'d, A, T, D>(group: &Location, data: A, name: &str) -> hdf5::Result<()>
where
    A: Into<ArrayView<'d, T, D>>,
    T: H5Type,
    D: ndarray::Dimension,
{
    let builder = group.new_attr_builder();
    builder.with_data(data).create(name)?;

    Ok(())
}

/// Save string `value` as a HDF5 attribute.
fn save_str_attr(location: &Location, value: &str, name: &str) -> hdf5::Result<()> {
    // Code found here: https://users.rust-lang.org/t/add-string-attribute-using-hdf5-rust/68744/8
    let attr = location
        .new_attr::<hdf5::types::VarLenUnicode>()
        .create(name)?;
    let value_: hdf5::types::VarLenUnicode = value.parse().unwrap();

    attr.write_scalar(&value_)
}

/// Write regular dataset to a [`hdf5::Group`].
fn write_dataset(
    group: &hdf5::Group,
    vec: &Vec<impl hdf5::H5Type>,
    name: &str,
) -> hdf5::Result<hdf5::Dataset> {
    let builder = group.new_dataset_builder();
    let ds = builder.with_data(vec.as_slice()).create(name)?;

    Ok(ds)
}
