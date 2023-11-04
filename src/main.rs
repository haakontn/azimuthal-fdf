use std::path::PathBuf;
use std::time::SystemTime;

use azimuthal_fdf::hrr_integral::{self, DescribingFunction};
use azimuthal_fdf::observers::{self, Observer, ObserverTrait, SaveInfo};
use azimuthal_fdf::{Parameters, Saturation, SaveData, Settings};
use clap::{CommandFactory, Parser};
use rayon::prelude::*;

fn main() {
    let cli_arguments = CliParser::parse();

    if cli_arguments.export_default_settings {
        println!(
            "Preparing for export to file: {}",
            cli_arguments.export_path
        );
        let saturation = match cli_arguments.export_saturation.to_lowercase().as_str() {
            "exponential" => Saturation::Exponential(1.0),
            _ => Saturation::default(),
        };
        let observer = match cli_arguments.export_observer.to_lowercase().as_str() {
            "histogram" => Observer::Histogram(observers::HistogramObserver::default()),
            _ => Observer::default(),
        };
        // TODO Make this selectable
        let describing_function = DescribingFunction::default();

        let settings = Settings::new(
            Parameters::default(),
            saturation,
            observer,
            describing_function,
        );
        println!("Exporting to file...");
        if let Err(e) = settings.export(&PathBuf::from(&cli_arguments.export_path)) {
            println!("could not export the settings: {}", e);
            return;
        }
        println!("Success!");
    } else if cli_arguments.example {
        // Run an example simualation
        println!("Setting up simulation...");
        let mut settings = Settings::default();
        println!(
            "Results will be saved to: {}",
            settings.observer.save_info()
        );
        println!("Simulation started...");
        settings.run();

        match settings
            .observer
            .save(&settings.parameters, &settings.describing_function)
        {
            Ok(_) => println!(
                "Results were succesfully saved to: {}",
                settings.observer.save_info()
            ),
            Err(e) => println!("Could not save: {}", e),
        }
    } else if cli_arguments.experiment {
        // Run the simulations related to the reported experiments
        println!("Setting up simulations...");

        // Different gain factors (gain = gain_factor * damping)
        let gain_factors = [5.0, 3.75, 2.5, 1.25];

        // Set up the saving
        let path = PathBuf::from("experiment_simulation.hdf5");
        let mut save_infos = vec![SaveInfo::default(); 4];
        for (ind, gain_factor) in gain_factors.into_iter().enumerate() {
            let group = format!("gain_factor_{}", gain_factor);
            save_infos[ind] = SaveInfo::new(&path, &group);
        }

        // Set up how many threads to use for the computation
        let num_threads = build_rayon_pool(gain_factors.len());

        // Get the reference case damping
        let damping = Settings::default().parameters.damping;

        println!("Simulation started on {} threads...", num_threads);
        let start_time = SystemTime::now();
        let save_data: Vec<Option<SaveData>> = gain_factors
            .into_par_iter()
            .zip(save_infos)
            .map(|(gain_factor, save_info)| {
                // Need to create this inside the parallel iterator
                // for the RNG initialization to work properly
                let mut settings = Settings::default();

                // Set the gain
                settings.parameters.gain = gain_factor * damping;
                settings.parameters.noise = 0.06;

                // Set the time step
                let new_timestep = settings.parameters.get_timestep() / 2.0;
                if let Err(e) = settings.parameters.set_timestep(new_timestep) {
                    println!("{}", e);
                }

                // Set the saving information
                settings.observer.set_save_info(&save_info);

                // Set the length of the simulation
                settings.parameters.set_number_of_cycles(170_000.0).unwrap();

                let df = hrr_integral::ConventionalFDF::new();
                let describing_function = hrr_integral::DescribingFunction::Conventional(df);
                settings.describing_function = describing_function;

                run_settings(settings)
            })
            .collect();

        // Save the data outside of the parallel for-loop
        save(save_data, start_time);
    } else if !cli_arguments.settings_files.is_empty() {
        // Run the simulations related to the reported experiments
        println!("Loading the settings files...");

        // Load the settings from file
        if cli_arguments.settings_files.len() == 1 {
            // Keep it a bit general to allow for disabling rayon
            let mut all_settings: Vec<Settings> = Vec::new();
            for filepath in cli_arguments.settings_files {
                println!("Loading settings from: {}", filepath);
                match Settings::from_file(&filepath) {
                    Ok(settings) => all_settings.push(settings),
                    Err(e) => println!(
                        "{}\ncould not load settings {}, skipping simulation",
                        e, filepath
                    ),
                }
            }

            for settings in all_settings {
                let start_time = SystemTime::now();

                if let Some(save_data) = run_settings(settings) {
                    if let Err(e) = save_data.save() {
                        println!("could not save: {}", e);
                    }

                    if let Ok(elapsed_time) = save_data.finish_time.duration_since(start_time) {
                        let save_info = save_data.get_save_info();
                        println!(
                            "{}: {} took {} seconds",
                            save_info.get_path().to_string_lossy(),
                            save_info.get_group(),
                            elapsed_time.as_secs()
                        );
                    }
                }
            }
        } else {
            let start_time = SystemTime::now();
            let save_data: Vec<Option<SaveData>> = cli_arguments
                .settings_files
                .into_par_iter()
                .map(|filepath| {
                    println!("Loading settings from: {}", filepath);

                    match Settings::from_file(&filepath) {
                        Ok(settings) => run_settings(settings),
                        Err(e) => {
                            println!(
                                "{}\nCould not load {}, the simulation will be skipped",
                                e, filepath
                            );
                            None
                        }
                    }
                })
                .collect();

            // Save the data outside of the parallel for-loop
            save(save_data, start_time);
        }
    } else {
        // If no arguments are provided print the help information
        let mut cmd = CliParser::command();
        cmd.print_help().unwrap_or_default()
    }
}

/// Shorthand for checking whether there is a save conflict and run the simulation.
#[inline]
fn run_settings(mut settings: Settings) -> Option<SaveData> {
    match settings.observer.save_info().is_valid() {
        Ok(_) => {
            println!("Results will be saved to {}", settings.observer.save_info())
        }
        Err(e) => {
            println! {"{}\nSave conflict, skipping simulation {}", e, settings.observer.save_info().get_group()}
            return None;
        }
    }

    settings.run();

    let mut save_data = SaveData::from(settings);
    save_data.finish_time = SystemTime::now();

    Some(save_data)
}

/// Shorthand for saving the [`SaveData`] from the different simulations
#[inline]
fn save(save_data: Vec<Option<SaveData>>, start_time: SystemTime) {
    for sd in save_data {
        if let Some(sd) = sd {
            if let Err(e) = sd.save() {
                println!("could not save: {}", e);
            }

            if let Ok(elapsed_time) = sd.finish_time.duration_since(start_time) {
                let si = sd.get_save_info();
                println!(
                    "{}: {} took {} seconds",
                    si.get_path().to_string_lossy(),
                    si.get_group(),
                    elapsed_time.as_secs()
                );
            }
        }
    }
}

#[inline]
fn build_rayon_pool(number_of_jobs: usize) -> usize {
    let max_threads = num_cpus::get_physical() - 1;
    let num_threads = number_of_jobs.min(max_threads);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    num_threads
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
#[command(long_about = None)]
pub struct CliParser {
    /// Run example simulation
    #[arg(long, action)]
    example: bool,

    /// Run the experiment simulation from the paper.
    #[arg(long, action)]
    experiment: bool,

    /// Export the default settings to JSON file.
    #[arg(short, long, action)]
    export_default_settings: bool,

    /// Choose which saturation function to export when performing
    /// using the '--export-default-settings' option
    #[arg(long, default_value_t = String::from("Tangent"))]
    export_saturation: String,

    /// Choose which saturation function to export when performing
    /// using the '--export-default-settings' option
    #[arg(long, default_value_t = String::from("TimeSeries"))]
    export_observer: String,

    /// Set the output path for the '--export-default-settings' option
    #[arg(long, default_value_t = String::from("default_settings.json"))]
    export_path: String,

    /// Path to the settings file(s) to run simulations for
    #[arg(short, long, num_args(0..))]
    settings_files: Vec<String>,
}
