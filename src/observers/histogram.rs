use std::path::PathBuf;

use super::{ObserverTrait, SaveInfo};
use crate::{azimuthal_mode::SystemMode, DescribingFunction, Float, Parameters, PI};
use hdf5;
use ndarray;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HistogramObserver {
    pub save_info: SaveInfo,
    amplitude_limit: Float,
    max_amplitude_limit: Float,
    nbins: usize,

    #[serde(skip)]
    a: Vec<usize>,

    #[serde(skip)]
    nth0: Vec<usize>,

    #[serde(skip)]
    phi: Vec<usize>,

    #[serde(skip)]
    chi: Vec<usize>,

    #[serde(skip)]
    chi_q: Vec<usize>,

    #[serde(skip)]
    num_values: usize,
}

impl HistogramObserver {
    pub fn new(
        output_filepath: &PathBuf,
        group_name: Option<&str>,
        nbins: usize,
        a_lim: Float,
    ) -> HistogramObserver {
        // Set up the save info
        let mut save_info = SaveInfo::default();
        save_info.set_path(output_filepath);
        if let Some(group) = group_name {
            save_info.set_group(group);
        }

        HistogramObserver {
            save_info,
            amplitude_limit: a_lim,
            max_amplitude_limit: 10.0 * a_lim,
            nbins,
            a: vec![0; nbins],
            nth0: vec![0; nbins],
            phi: vec![0; nbins],
            chi: vec![0; nbins],
            chi_q: vec![0; nbins],
            num_values: 0,
        }
    }

    pub fn set_amplitude_limit(&mut self, amplitude_limit: Float) {
        self.amplitude_limit = amplitude_limit;
    }

    pub fn set_nbins(&mut self, nbins: usize) {
        // Resize the vectors, this assumes that there
        // are no data in the histogram from before
        assert_eq!(self.num_values, 0);

        self.nbins = nbins;

        self.a.resize(nbins, 0);
        self.nth0.resize(nbins, 0);
        self.phi.resize(nbins, 0);
        self.chi.resize(nbins, 0);
        self.chi_q.resize(nbins, 0);
    }

    /// Create [`HistogramObserver`] observer from a JSON string.
    pub fn from_str(json_string: &str) -> Result<Self, serde_json::Error> {
        // Load the main struct without the histogram storage from
        // the JSON based string
        let mut histogram: Self = serde_json::from_str(json_string)?;

        // Set up the bins correctly
        histogram.set_nbins(histogram.nbins);

        Ok(histogram)
    }

    // Extend the amplitude range
    fn expand_amplitude_range(&mut self, new_amplitude: Float) {
        let extension_factor = (new_amplitude / self.amplitude_limit).floor() as usize;
        // Check that the amplitude limit does not grow too large
        if self.amplitude_limit * extension_factor as Float > self.max_amplitude_limit {
            panic!(
                "the amplitude has grown too large, amplitude = {}",
                new_amplitude
            )
        }

        self.a.resize((1 + extension_factor) * self.a.len(), 0);
        self.amplitude_limit *= (1 + extension_factor) as Float;
    }
}

impl ObserverTrait for HistogramObserver {
    #[inline]
    fn log(&mut self, acoustic_mode: &SystemMode, hrr_mode: &SystemMode, _time: Float) {
        // Check if the amplitude range needs to be expanded
        if acoustic_mode.a() >= self.amplitude_limit {
            self.expand_amplitude_range(acoustic_mode.a());
        }

        // Calculate the bin index for each state space parameter and then
        let a_bin = get_index(acoustic_mode.a(), self.amplitude_limit, &self.a);
        self.a[a_bin] += 1;

        let nth0_bin = get_index(acoustic_mode.nth0(), 2.0 * PI, &self.nth0);
        self.nth0[nth0_bin] += 1;

        let phi_bin = get_index(acoustic_mode.phi(), 2.0 * PI, &self.phi);
        self.phi[phi_bin] += 1;

        let chi_bin = get_index(acoustic_mode.chi(), PI / 2.0, &self.chi);
        self.chi[chi_bin] += 1;

        let chi_q_bin = get_index(hrr_mode.chi(), PI / 2.0, &self.chi_q);
        self.chi_q[chi_q_bin] += 1;

        // Update the total number of values
        self.num_values += 1;
    }

    fn save(
        &self,
        setup: &Parameters,
        _describing_function: &DescribingFunction,
    ) -> hdf5::Result<()> {
        // Open the file if it alreay exist, or else create it
        let file = hdf5::File::append(&self.save_info.path)?;

        // Load the group name, or use the default
        let group_name = &self.save_info.group;
        let group = file.create_group(group_name)?;

        // Save the actual (non-normalised) histograms
        super::write_dataset(&group, &self.a, "amplitude")?;
        super::write_dataset(&group, &self.nth0, "ntheta_0")?;
        super::write_dataset(&group, &self.phi, "phi")?;
        super::write_dataset(&group, &self.chi, "chi")?;
        super::write_dataset(&group, &self.chi_q, "chi_q")?;

        // Calculate the bin edges
        let a_edges = get_bin_edges(0.0, self.amplitude_limit, self.a.len());
        let nth0_edges = get_bin_edges(-PI, PI, self.nth0.len());
        let phi_edges = get_bin_edges(-PI, PI, self.phi.len());
        let chi_edges = get_bin_edges(-PI / 4.0, PI / 4.0, self.chi.len());
        let chi_q_edges = get_bin_edges(-PI / 4.0, PI / 4.0, self.chi_q.len());

        // Save the bin edges in a subgroup
        let edge_group = group.create_group("bin_edges")?;
        super::write_dataset(&edge_group, &a_edges, "amplitude")?;
        super::write_dataset(&edge_group, &nth0_edges, "ntheta_0")?;
        super::write_dataset(&edge_group, &phi_edges, "phi")?;
        super::write_dataset(&edge_group, &chi_edges, "chi")?;
        super::write_dataset(&edge_group, &chi_q_edges, "chi_q")?;

        // Save the number of values
        super::save_attr(&group, &ndarray::arr0(self.num_values), "number_of_values")?;
        // Save the setup as an attribute
        super::save_parameters_as_attribute_json(&group, setup)
    }
}

impl std::fmt::Display for HistogramObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_string = serde_json::to_string_pretty(self).unwrap_or_default();
        write!(f, "HistogramObserver: {}", data_string)
    }
}

impl Default for HistogramObserver {
    fn default() -> Self {
        let output_filepath = PathBuf::from("simulation_histogram.hdf5");

        let nbins = 100;
        let a_lim = 10.0;

        Self::new(&output_filepath, None, nbins, a_lim)
    }
}

impl From<SaveInfo> for HistogramObserver {
    fn from(value: SaveInfo) -> Self {
        let mut ho = Self::default();
        ho.save_info = value;

        ho
    }
}

#[inline]
fn get_index(num: Float, limit: Float, bin_vec: &Vec<usize>) -> usize {
    if num > limit + Float::EPSILON {
        println!("Number: {}\t Limit: {}", num, limit);
    }
    if num >= limit {
        return bin_vec.len() - 1;
    }

    Float::floor((modulo(num, limit) / limit) * bin_vec.len() as Float) as usize
}

#[inline]
fn modulo(num: Float, limit: Float) -> Float {
    ((num % limit) + limit) % limit
}

fn get_bin_edges(min: Float, max: Float, len: usize) -> Vec<Float> {
    // There are `len` number of intervals, meaning there should be
    // `len + 1` values for the edges
    let bin_length = (max - min) / len as Float;

    (0..=len).map(|ind| bin_length * ind as Float).collect()
}
