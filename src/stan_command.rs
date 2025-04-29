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
#[allow(unused_imports)]
pub use arg_path::{ArgPath, ArgWritablePath, ArgReadablePath};

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

mod arg_path {
    use std::path::PathBuf;
    use std::fmt::Display;
    use super::WithDefaultArg;
    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgPath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    impl WithDefaultArg for ArgPath {
        const ARG_DEFAULT: Self = Self::Borrowed("");
    }

    impl Display for ArgPath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Borrowed(path) => write!(f, "{}", *path),
                Self::Owned(path) => write!(f, "{}", path.to_string_lossy())
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgWritablePath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    impl WithDefaultArg for ArgWritablePath {
        const ARG_DEFAULT: Self = Self::Borrowed("");
    }

    impl Display for ArgWritablePath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Borrowed(path) => write!(f, "{}", *path),
                Self::Owned(path) => write!(f, "{}", path.to_string_lossy())
            }
        }
    }


    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgReadablePath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    impl WithDefaultArg for ArgReadablePath {
        const ARG_DEFAULT: Self = Self::Borrowed("");
    }

    impl Display for ArgReadablePath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Borrowed(path) => write!(f, "{}", *path),
                Self::Owned(path) => write!(f, "{}", path.to_string_lossy())
            }
        }
    }

    impl From<&'static str> for ArgPath {
        fn from(path: &'static str) -> Self {
            Self::Borrowed(path)
        }
    }

    impl From<PathBuf> for ArgPath {
        fn from(path: PathBuf) -> Self {
            Self::Owned(path)
        }
    }

    impl From<ArgPath> for PathBuf {
        fn from(path: ArgPath) -> Self {
            match path {
                ArgPath::Borrowed(path) => PathBuf::from(path),
                ArgPath::Owned(path) => path,
            }
        }
    }

    impl From<ArgReadablePath> for PathBuf {
        fn from(path: ArgReadablePath) -> Self {
            match path {
                ArgReadablePath::Borrowed(path) => PathBuf::from(path),
                ArgReadablePath::Owned(path) => path,
            }
        }
    }
    
    impl From<ArgWritablePath> for PathBuf {
        fn from(path: ArgWritablePath) -> Self {
            match path {
                ArgWritablePath::Borrowed(path) => PathBuf::from(path),
                ArgWritablePath::Owned(path) => path,
            }
        }
    }

    use std::io::Error;
    impl ArgPath {
        pub fn verify_file_readable(&self) -> Result<(), Error> {
            match self {
                ArgPath::Borrowed(path) => std::fs::File::open(path).map(|_|()),
                ArgPath::Owned(path) => std::fs::File::open(path).map(|_|()),
            }
        }

        pub fn into_readable(self) -> Result<ArgReadablePath, Error> {
            self.verify_file_readable()?;
            match self {
                ArgPath::Borrowed(path) => Ok(ArgReadablePath::Borrowed(path)),
                ArgPath::Owned(path) => Ok(ArgReadablePath::Owned(path)),
            }
        }

        pub fn verify_file_writeable(&self) -> Result<(), Error> {
            let path = match self {
                ArgPath::Borrowed(path) => PathBuf::from(path),
                ArgPath::Owned(path) => path.clone(),
            };
            if let Some(parent_path) = path.parent() {
                std::fs::create_dir_all(parent_path)?;
            }
            std::fs::OpenOptions::new()
                    .create(true)
                    .append(true) // 避免 truncate 覆盖
                    .open(path)
                    .map(|_|())
        }

        pub fn into_writeable(self) -> Result<ArgWritablePath, Error> {
            self.verify_file_writeable()?;
            match self {
                ArgPath::Borrowed(path) => Ok(ArgWritablePath::Borrowed(path)),
                ArgPath::Owned(path) => Ok(ArgWritablePath::Owned(path)),
            }
        }

        pub fn extend_default_file(&mut self, default_name: &str) -> &mut Self {
            match self {
                ArgPath::Borrowed(path) => {
                    let mut path = PathBuf::from(*path);
                    if path.extension().is_none() {
                        path.push(default_name);
                        *self = ArgPath::Owned(path);
                    }
                },
                ArgPath::Owned(path) => {
                    if path.extension().is_none() {
                        path.push(default_name);
                    }
                },
            }
            self
        }
    }
}