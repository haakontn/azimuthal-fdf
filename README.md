# Saturation of flames to multiple inputs at one frequency

This code was written as part of the article *Saturation of flames to multiple inputs at one frequency* by H. T. Nygård, G. Ghirardo, and N. A. Worth [1], extending the stochastic quaternion valued model first developed by Ghirardo and Gant [2, 3].

# Installation

The program requires the Rust compiler, which can be obtained from their official [homepage](https://www.rust-lang.org/).

## Dependencies

The dependencies of the project can be found in [Cargo.toml](Cargo.toml), but it should be noted that the [HDF5 crate](https://crates.io/crates/hdf5) ***requires the HDF5 library to be installed on the system already***.
It is possible to install it using conda, but see the full list of options [here](https://crates.io/crates/hdf5).

## Basic usage

After cloning this repository, the executable program can ran using the command
```bash
$ cargo run --release
```
where the `--release` option turns on more optimization than the standard `cargo build`.

To get a fresh list of all the command line options (also found [here](#command-line-options)), run the command
```bash
$ cargo run --release -- --help
```

To run the simulations presented in [1], use the following command
```bash
$ cargo run --release -- --experiment
```
which takes around **1 hour** to complete depending on computer hardware, and assuming the process has at least 5 physical cores.
To run a shorter demonstration (around 10-20 minutes depending on the system), on a single core, the following command can be used
```bash
$ cargo run --release -- --example
```

For custom settings, it is recommended to create .json files based on the .json file created by running
```bash
$ cargo run --release -- --export-default-settings
```
For the full list of options when exporting the default settings, see the output of 
```bash
$ cargo run --release -- --help
```
If there are two settings files, named `setting_1.json` and `setting_2.json`, execute the following command to 
```bash
$ cargo run --release -- --settings-files setting_1.json setting_2.json
```


## Command line options

Options can be specified in the following way
```bash
$ cargo run --release -- [OPTIONS]
```
The following options are available:
```
Usage: azimuthal_fdf [OPTIONS]

Options:
      --example
          Run example simulation
      --experiment
          Run the experiment simulation from the paper
  -e, --export-default-settings
          Export the default settings to JSON file
      --export-saturation <EXPORT_SATURATION>
          Choose which saturation function to export when performing using the '--export-default-settings' option [default: Tangent]
      --export-observer <EXPORT_OBSERVER>
          Choose which saturation function to export when performing using the '--export-default-settings' option [default: TimeSeries]
      --export-path <EXPORT_PATH>
          Set the output path for the '--export-default-settings' option [default: default_settings.json]
  -s, --settings-files [<SETTINGS_FILES>...]
          Path to the settings file(s) to run simulations for
  -h, --help
          Print help
  -V, --version
          Print version
```

## Custom calling functions

If more programmatic control is desired for setting up and running different simulations, this project can also be imported as a crate to write a `custom main.rs` file.
Please see the current [main.rs](src/main.rs) file for examples on how to set up a simulation from the different components.

# References

[1] H. T. Nygård, G. Ghirardo, N. A. Worth "*Saturation of flames to multiple inputs at one frequency*", Journal of Fluid Mechanics, *accepted for publication*.

[2] [G. Ghirardo, F. Gant, "*Background noise pushes azimuthal instabilities away from spinning states*", 2019, https://arxiv.org/abs/1904.00213](https://arxiv.org/abs/1904.00213)

[3] [G. Ghirardo, F. Gant, "*Averaging of thermoacoustic azimuthal instabilities*", Journal of Sound and Vibration, Volume 490, 2021, 115732,](https://www.sciencedirect.com/science/article/pii/S0022460X20305629)