mod wrapper;
mod wrapper_builder;

mod model_error {
    use std::fmt::{self, Display, Formatter};
    use std::error::Error;
    use std::path::PathBuf;
    use crate::stan_command::ArgError;
    use crate::init::STAN_HOME_KEY;

    #[derive(Debug)]
    pub enum ParamError {
        ParamNotFound(String),
        ParseError(std::string::ParseError),
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
                ParamError::ParseError(e) => Some(e),
            }
        }
    }

    #[derive(Debug)]
    pub enum FileError {
        FileSystem(std::io::Error),
        Compilation(std::process::Output),
        EnvVar(std::env::VarError),
        InvalidPath(String, PathBuf),
    }

    impl Display for FileError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                FileError::Compilation(e) => write!(f, "complation error {e:?}"),
                FileError::EnvVar(e) => write!(f, "cannot find {STAN_HOME_KEY} : {e}"),
                FileError::FileSystem(e) => write!(f, "file system error: {e}"),
                FileError::InvalidPath(s, p) => write!(f, "invalid filename: {s} {p:?}"),
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
            }
        }
    }

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
use model_error::CmdStanError;
pub use model_error::{ParamError, FileError};

use crate::{arg_paths::{ArgPath, ArgReadablePath}, stan_command::{arg_into, ArgThrough, StanResult}};
use std::path::PathBuf;

pub trait WithParam {
    fn get_param_name(&self) -> Vec<String>;
    fn get_param_value(&self, name: &str) -> Option<String>;
    fn set_param_value(&mut self, name: &str, value: &str) -> Result<&mut Self, ParamError>;
}

pub trait WithPath {
    fn get_workspace_path(&self) -> ArgPath;
    fn get_model_name(&self) -> String;

    fn get_model_path(&self) -> ArgPath {
        let mut p = PathBuf::from(self.get_workspace_path());
        p.push(self.get_model_name());
        p.set_extension("stan");
        ArgPath::Owned(p)
    }
}

pub trait WithExecutable {
    fn get_model_executable(&self) -> ArgPath;

    /// see: crate::stan_command::arg_into
    fn arg_into<T:ArgThrough+Clone>(&mut self, arg_tree: &T) -> Result<StanResult<T>, CmdStanError> {
        self.compile().map_err(CmdStanError::File)?;
        arg_into(arg_tree, &self.get_model_executable()).map_err(CmdStanError::Arg)
    }

    /// called before every get_model_excutable()
    /// 
    /// return Ok(()) if the compile is successful, otherwise return Err(ArgError)
    fn compile(&mut self) -> Result<(), FileError> {
        Ok(())
    }
}

pub trait WithData {
    fn get_data_path(&self) -> ArgReadablePath;

    /// called before every get_data_path()
    fn dump_data(&mut self) -> Result<(), FileError> {
        Ok(())
    }
}