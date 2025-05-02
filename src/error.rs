use std::{fmt::{Display, Formatter, self}, error::Error};
pub use {data_collection_error::DataCollectionError, arg_error::ArgError, param_error::ParamError, file_error::FileError, cmd_stan_error::CmdStanError};

mod data_collection_error {
    use super::*;
    #[derive(Debug, Clone)]
    pub enum DataCollectionError {
        AddEntryError(String),
    }

    impl Display for DataCollectionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DataCollectionError::AddEntryError(msg) => write!(f, "AddEntryError: {}", msg),
            }
        }
    }

    impl std::error::Error for DataCollectionError {}
}

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

mod param_error {
    use super::*;

    #[derive(Debug)]
    pub enum ParamError {
        ParamNotFound(String),
        ParseError(Box<dyn Error>),
    }

    impl Display for ParamError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                ParamError::ParamNotFound(s) => write!(f, "Parameter not found: {s}"),
                ParamError::ParseError(e) => write!(f, "Parameter parse error: {e}"),
            }
        }
    }

    impl Error for ParamError  {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                ParamError::ParamNotFound(_) => None,
                ParamError::ParseError(e) => Some(e.as_ref()),
            }
        }
    }
}

mod file_error {
    use super::*;
    use std::path::PathBuf;
    use crate::init::STAN_HOME_KEY;
    #[derive(Debug)]
    pub enum FileError {
        FileSystem(std::io::Error),
        Compilation(std::process::Output),
        EnvVar(std::env::VarError),
        InvalidPath(String, PathBuf),
        BadFileFormat(String, PathBuf),
    }

    impl Display for FileError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                FileError::Compilation(e) => write!(f, "complation error {e:?}"),
                FileError::EnvVar(e) => write!(f, "cannot find {STAN_HOME_KEY} : {e}"),
                FileError::FileSystem(e) => write!(f, "file system error: {e}"),
                FileError::InvalidPath(s, p) => write!(f, "invalid filename: {s} {p:?}"),
                FileError::BadFileFormat(s, p) => write!(f, "invalid file: {s} {p:?}"),
            }
        }
    }

    impl Error for FileError  {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                FileError::Compilation(_) => None,
                FileError::EnvVar(e) => Some(e),
                FileError::FileSystem(e) => Some(e),
                FileError::InvalidPath(_,_) => None,
                FileError::BadFileFormat(_,_) => None,
            }
        }
    }
}

mod cmd_stan_error {
    use super::*;
    #[derive(Debug)]
    pub enum CmdStanError {
        Arg(ArgError),
        Param(ParamError),
        File(FileError),
    }

    impl Display for CmdStanError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::Arg(e) => write!(f, "{e}"),
                Self::File(e) => write!(f, "{e}"),
                Self::Param(e) => write!(f, "{e}"),
            }
        }
    }

    impl Error for CmdStanError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Arg(e) => Some(e),
                Self::File(e) => Some(e),
                Self::Param(e) => Some(e),
            }
        }
    }
}