[![DOI](https://zenodo.org/badge/713919474.svg)](https://zenodo.org/doi/10.5281/zenodo.10071466)

# Saturation of flames to multiple inputs at one frequency

This code was written as part of the article *Saturation of flames to multiple inputs at one frequency* by H. T. Nygård, G. Ghirardo, and N. A. Worth [1], extending the stochastic quaternion valued model first developed by Ghirardo and Gant [2, 3] by introducing the Azimuthal Flame Describing Function (AFDF) [4].

# Installation

The program requires the Rust compiler, which can be obtained from their official [homepage](https://www.rust-lang.org/).

## Dependencies

The dependencies of the project can be found in [Cargo.toml](Cargo.toml), but it should be noted that the [HDF5 crate](https://crates.io/crates/hdf5) ***requires the HDF5 library to be installed on the system already***.
It is possible to install it using conda, but see the full list of options [here](https://crates.io/crates/hdf5).
**For a step by step minimal example of using `conda` to install the HDF5 library and compile this program, please see [this section](#minimal-example-using-the-conda-hdf5-library).**

## Basic usage

After cloning this repository, the executable program can be compiled and ran using the command
```console
cargo run --release
```
where the `--release` option turns on more optimization than the standard `cargo build`.

To get a fresh list of all the command line options (also found [here](#command-line-options)), run the command
```console
cargo run --release -- --help
```

To run the simulations presented in [1], use the following command
```console
cargo run --release -- --experiment
```
which takes around **1 hour** to complete depending on computer hardware, and assuming the process has at least 5 physical cores.
To run a shorter demonstration (around 10-20 minutes depending on the system), on a single core, the following command can be used
```console
cargo run --release -- --example
```

For custom settings, it is recommended to create .json files based on the .json file created by running
```console
cargo run --release -- --export-default-settings
```
For the full list of options when exporting the default settings, see [here](#command-line-options) or the output of 
```console
cargo run --release -- --help
```
If there are two settings files, named `setting_1.json` and `setting_2.json`, execute the following command to run both simulations
```console
cargo run --release -- --settings-files setting_1.json setting_2.json
```
The documentation can be compiled and opened in a browser with the following command
```console
cargo doc --open
```

## Command line options

Options can be specified in the following way
```bash
cargo run --release -- [OPTIONS]
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

## Minimal example using the `conda` HDF5 library


After [installing Rust](https://www.rust-lang.org/), download and install `conda`, either through [Anaconda](https://www.anaconda.com/download) or [Miniconda](https://docs.conda.io/projects/miniconda/en/latest/).
Then, create an environment, called `afdf` for this demonstration, where the [h5py module](https://anaconda.org/anaconda/h5py) is installed.
This can achieved with the following command in a terminal (*nix based systems) or Anaconda Powershell (Windows):
```console
conda create --name afdf h5py
``` 
After the virtual environment has been created, make sure to activate it
```console
conda activate afdf
```
While the environment is active, print the PATH environment variable:

>**Bash:**
>```bash
>$PATH
>```
>**Windows Powershell**
>```powershell
>$env:PATH
>```
The first item in the list should contain the path to the active environment, which should look similar to this
>***nix**
>```bash
>/some/path/envs/afdf
>```
> **Windows**
>```powershell
>C:\some\path\envs\afdf
>```
It should be noted that the path of interest ends with `envs` and the name of the environment, which is `afdf` in this example.
Before compiling this repository, the environment variable `HDF5_DIR` should be set ([see the HDF5 documentation here](https://crates.io/crates/hdf5)) to the above path (the root of the conda environment)
> **Bash:**
>```bash
> export HDF5_DIR=/some/path/envs/afdf
>```
> **Windows Powershell**
> ```powershell
> $env:HDF5_DIR='C:\some\path\envs\afdf'
>```
Finally, this repository can be compiled using the commands listed in the [Basic usage section](#basic-usage). A good starting point is to run
```console
cargo run --release -- --help
```

**NOTE:** When recompiling the repository, the external dependencies, such as HDF5, will typically not be recompiled unless `cargo clean` has been run since last time the repository was compiled. However, if the HDF5 crate has to be recompiled, make sure the HDF5_DIR path environment variable is set before compilation if you followed this example.



# References

[1] H. T. Nygård, G. Ghirardo, N. A. Worth "*Saturation of flames to multiple inputs at one frequency*", Journal of Fluid Mechanics, *accepted for publication*.

[2] [G. Ghirardo, F. Gant, "*Background noise pushes azimuthal instabilities away from spinning states*", 2019, https://arxiv.org/abs/1904.00213](https://arxiv.org/abs/1904.00213)

[3] [G. Ghirardo, F. Gant, "*Averaging of thermoacoustic azimuthal instabilities*", Journal of Sound and Vibration, Volume 490, 2021, 115732](https://www.sciencedirect.com/science/article/pii/S0022460X20305629)

[4] [Håkon T. Nygård, Giulio Ghirardo, Nicholas A. Worth, "*Azimuthal flame response and symmetry breaking in a forced annular combustor*", Combustion and Flame, Volume 233, 2021, 111565](https://www.sciencedirect.com/science/article/pii/S0010218021003084)