mod model_error {
    use std::fmt::{self, Display, Formatter};
    use std::error::Error;
    use crate::stan_command::ArgError;

    #[derive(Debug)]
    pub enum ParamError {
        ParamNotFound(String),
        ParseError(std::string::ParseError),
    }

    impl Display for ParamError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                ParamError::ParamNotFound(s) => write!(f, "Parameter not found: {}", s),
                ParamError::ParseError(e) => write!(f, "Parameter parse error: {}", e),
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
        EnvVar(std::env::VarError)
    }

    pub enum CmdStanError {
        Arg(ArgError),
        Param(ParamError),
        File(FileError),
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

mod wrapper {
    use crate::{prelude::DataEntries, StanData};
    use std::path::absolute;
    use std::process::Command;
    use crate::init::STAN_HOME_KEY;
    use std::env::consts::OS;
    use super::*;

    #[derive(Debug, Clone)]
    pub struct CmdStanModel<T, D: StanData = DataEntries> {
        pub model: T,
        workspace_path: ArgPath,
        model_name: String,
        model_executable: Option<ArgPath>,
        data_path: Option<ArgPath>,
        data: Option<D>,
        compiled: bool,
    }

    impl<T, D: StanData> CmdStanModel<T, D> {
        pub fn set_executable_name(&mut self, name: ArgPath) -> &mut Self {
            self.model_executable = Some(name);
            self
        }

        fn executable_name(&self) -> ArgPath {
            if let Some(p) = &self.model_executable {
                p.clone()
            } else {
                let mut res = PathBuf::from(self.workspace_path.as_path()).join(&self.model_name);
                if OS == "windows" {
                    res.set_extension("exe");
                }
                ArgPath::Owned(res)
            }
        }

        fn set_default_executable_name(&mut self) -> &mut Self {
            if self.model_executable.is_none() {
                self.set_executable_name(self.executable_name());
            }
            self
        }

        pub fn set_compiled(&mut self) -> &mut Self {
            self.compiled = true;
            self
        }

        pub fn set_data_path(&mut self, path: ArgPath) -> &mut Self {
            self.data_path = Some(path);
            self
        }

        fn data_file_name(&self) -> ArgPath {
            if let Some(p) = &self.data_path {
                p.clone()
            } else {
                let mut res = PathBuf::from(self.workspace_path.as_path()).join(&self.model_name);
                res.set_extension("data.json");
                ArgPath::Owned(res)
            }
        }

        fn set_default_data_path(&mut self) -> &mut Self {
            if self.data_path.is_none() {
                self.set_data_path(self.data_file_name());
            }
            self
        }
    }

    impl<T:WithParam, D:StanData> WithParam for CmdStanModel<T, D> {
        fn get_param_name(&self) -> Vec<String> {
            self.model.get_param_name()
        }

        fn get_param_value(&self, name: &str) -> Option<String> {
            self.model.get_param_value(name)
        }

        fn set_param_value(&mut self, name: &str, value: &str) -> Result<&mut Self, ParamError> {
            self.model.set_param_value(name, value)?;
            Ok(self)
        }
    }

    impl<T, D:StanData> WithPath for CmdStanModel<T, D> {
        fn get_workspace_path(&self) -> ArgPath {
            self.workspace_path.clone()
        }

        fn get_model_name(&self) -> String {
            self.model_name.clone()
        }
    }

    impl<T, D:StanData> WithExecutable for CmdStanModel<T, D> {
        fn compile(&mut self) -> Result<(), FileError> {
            self.set_default_executable_name();
            if self.compiled {
                return Ok(());
            }

            let absolute_executable = absolute(self.executable_name().as_path()).map_err(FileError::FileSystem)?;
            let command = Command::new("make")
                .current_dir(std::env::var(STAN_HOME_KEY).map_err(FileError::EnvVar)?)
                .arg(absolute_executable)
                .output().map_err(FileError::FileSystem)?;

            if !command.status.success() {
                Err(FileError::Compilation(command))
            } else {
                self.compiled = true;
                Ok(())
            }
        }

        fn get_model_executable(&self) -> ArgPath {
            self.executable_name()
        }
    }

    impl<T, D:StanData> WithData for CmdStanModel<T, D> {
        fn dump_data(&mut self) -> Result<(), FileError> {
            self.set_default_data_path();
            if let Some(data) = &self.data {
                let dpath = self.data_file_name().into_writeable().map_err(FileError::FileSystem)?;
                dpath.write_once(&data.write_as_stan_data()).map_err(FileError::FileSystem)?;
                self.data = None;
                Ok(())
            } else {
                self.data_file_name().into_readable().map(|_|()).map_err(FileError::FileSystem)
            }
        }

        fn get_data_path(&self) -> ArgReadablePath {
            match self.data_file_name() {
                ArgPath::Borrowed(p) => ArgReadablePath::Borrowed(p),
                ArgPath::Owned(p) => ArgReadablePath::Owned(p)
            }
        }
    }
}

#[allow(unused_imports)]
pub use wrapper::CmdStanModel;