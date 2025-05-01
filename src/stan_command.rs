#[macro_use]
pub mod arg_tree;
mod sample;
mod optimize;
mod common_arg;
mod variational;
mod diagnose;
mod generate_quantities;
mod pathfinder;
mod log_prob;
mod laplace;

use std::process::Command;
pub use arg_error::ArgError;
use arg_tree::{ArgPath, ArgReadablePath};

mod arg_error {
    use std::{error::Error, fmt::Display};

    #[derive(Debug)]
    pub enum ArgError {
        NotValidArgTreeType(String),
        BadArgumentValue(String),
        FileSystemError(std::io::Error),
    }

    impl Display for ArgError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NotValidArgTreeType(s) => write!(f, "{s}"),
                Self::BadArgumentValue(s) => write!(f, "{s}"),
                Self::FileSystemError(e) => write!(f, "file system error: {e}"),
            }
        }
    }

    impl Error for ArgError {}
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    Sample,
    Optimize,
    Variational,
    Diagnose,
    GenerateQuantities,
    Pathfinder,
    LogProb,
    Laplace,
}

pub trait WithDefaultArg : PartialEq+Sized {
    const ARG_DEFAULT: Self;
    fn is_default(&self) -> bool {
        *self == Self::ARG_DEFAULT
    }
}

pub trait ArgThrough {
    fn arg_type(&self) -> Result<ArgType, ArgError>;
    fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError>;
    fn get_output_path(&self) -> Result<ArgPath, ArgError> {
        Err(ArgError::NotValidArgTreeType("no ArgOutput found in arg_tree, if this's costomized arg_tree structure, please impl this function.".to_string()))
    }
}

pub struct StanResult<T: ArgThrough> {
    pub arg_tree: T,
    pub output_path: ArgReadablePath,
    pub output: std::process::Output,
}

/// Generates a `StanResult` from an argument tree (`arg_tree`) and an executable model path.
///
/// This function allows you to use a customized argument tree. However, you must ensure that the
/// argument tree implements the `ArgThrough` trait and provides the `.get_output_path()` method
/// to determine the output path for the result.
///
/// This function is fundamental to the operation of the system and will be wrapped in higher-level
/// utilities for easier usage in other modules.
///
/// # Arguments
/// - `arg_tree`: A reference to an object that implements the `ArgThrough` trait, which is responsible
///   for providing command line arguments for the Stan model.
/// - `model_path`: The path to the executable model file. This should be a valid path to the model
///   binary that will be executed.
///
/// # Returns
/// - `Result<StanResult<T>, ArgError>`: Returns a `StanResult` containing the output of the model execution
///   or an `ArgError` if there is an issue during the process (e.g., invalid argument tree, file system 
///   errors).
///
/// # Example
/// This example demonstrates how to use `arg_into` with a standard argument tree and model path:
///
/// ```no-run
/// let common_args = WithCommonArgs::new(ArgSample::new());
/// common_args.data.set_file(ArgReadablePath::from("examples/bernoulli/bernoulli.data.json")).unwrap();
/// let model_path = ArgReadablePath::from("bernoulli2.exe");
/// let output = arg_into(common_args, model_path).unwrap();
/// ```
///
/// In this example:
/// - A standard argument tree (`common_args`) is created using `WithCommonArgs::new`.
/// - The `data` field of `common_args` is set to a file path using the `set_file` method.
/// - The model executable is specified by `model_path`.
/// - The `arg_into` function is called, which generates the `StanResult` containing the model's output.
///
/// # Notes
/// - Ensure that your custom argument tree (`arg_tree`) implements the `ArgThrough` trait and correctly
///   defines the `.get_output_path()` method.
/// - The function uses the `Command` struct to execute the model, so the model must be executable
///   and properly set up in your environment.
pub fn arg_into<T:ArgThrough+Clone>(arg_tree: &T, model_path: &ArgReadablePath) -> Result<StanResult<T>, ArgError> {
    let output_path = arg_tree.get_output_path()?; // check wether the arg_tree is valid
    let mut cmd = Command::new(model_path.as_path());
    arg_tree.arg_through(&mut cmd)?;
    let output = cmd.output().map_err(ArgError::FileSystemError)?;
    Ok(StanResult {
        arg_tree: arg_tree.clone(),
        output_path: output_path.into_readable().map_err(ArgError::FileSystemError)?,
        output,
    })
}