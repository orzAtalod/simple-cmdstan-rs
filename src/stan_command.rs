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
}