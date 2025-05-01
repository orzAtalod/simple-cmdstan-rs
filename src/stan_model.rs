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

mod wrapper {
    use crate::stan_command::WithDefaultArg;
    use crate::{prelude::DataEntries, StanData};
    use std::path::{absolute, Path};
    use std::process::Command;
    use crate::init::STAN_HOME_KEY;
    use std::{env::consts::OS, ffi::OsStr};
    use super::*;

    #[derive(Debug, Clone)]
    pub struct CmdStanModel<T, D: StanData = DataEntries> {
        pub model: T,
        workspace_path: ArgPath,
        model_name: String,
        data_path: Option<ArgPath>,
        data: Option<D>,
        compiled: bool,
    }

    impl<T, D: StanData> CmdStanModel<T, D> {
        fn executable_name(&self) -> ArgPath {
            let mut res = PathBuf::from(self.workspace_path.as_path()).join(&self.model_name);
            if OS == "windows" {
                res.set_extension("exe");
            }
            ArgPath::Owned(res)
        }

        pub fn set_compiled(&mut self) -> &mut Self {
            self.compiled = true;
            self
        }

        pub fn set_data_path(&mut self, path: ArgPath) -> &mut Self {
            self.data_path = Some(path);
            self
        }

        pub fn set_data(&mut self, data: D) -> &mut Self {
            self.data = Some(data);
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

    use std::collections::HashMap;
    struct CmdStanModelBuilder<T, D:StanData = DataEntries> {
        model: T,
        workspace_path: ArgPath,
        model_name: String,
        data_path: Option<ArgPath>,
        data: Option<D>,
        compiled: bool,
        complie_arg: HashMap<String, String>,
    }

    impl<T:Default, D:StanData> Default for CmdStanModelBuilder<T, D> {
        fn default() -> Self {
            Self {
                model: T::default(),
                workspace_path: ArgPath::ARG_DEFAULT,
                model_name: String::new(),
                data_path: None,
                data: None,
                compiled: false,
                complie_arg: HashMap::new(),
            }
        }
    }

    impl<T:Default, D:StanData> CmdStanModelBuilder<T, D> {
        fn new() -> Self {
            Self::default()
        }

        fn new_with_path(model_path: &Path) -> Result<Self, FileError> {
            Self::default().with_path(model_path)
        }
    }

    impl<T, D:StanData> CmdStanModelBuilder<T, D> {
        pub fn new_with_model(model: T) -> Self {
            Self {
                model,
                workspace_path: ArgPath::ARG_DEFAULT,
                model_name: String::new(),
                data_path: None,
                data: None,
                compiled: false,
                complie_arg: HashMap::new(),
            }
        }

        pub fn with_path(mut self, model_path: &Path) -> Result<Self, FileError> {
            let mut model_path = PathBuf::from(model_path);
            if model_path.extension().is_none() && OS == "windows" {
                model_path.set_extension("exe");
            }
            if model_path.extension().is_none() || model_path.extension() == Some(OsStr::new("exe")) {
                self.compiled = true;
            }

            if let Some(file_name) = model_path.file_name() {
                self.model_name = String::from(file_name.to_string_lossy());
            } else {
                return Err(FileError::InvalidPath("No file name founded".to_string(), model_path));
            }

            if let Some(p) = model_path.parent() {
                self.workspace_path = ArgPath::Owned(p.into())
            } else {
                return Err(FileError::InvalidPath("No parent founded".to_string(), model_path));
            }
            Ok(self)
        }

        pub fn with_complie_arg(mut self, arg: &str, argv: &str) -> Self {
            self.complie_arg.insert(arg.into(), argv.into());
            self
        }

        pub fn with_data_path(mut self, data_path: &Path) -> Self {
            self.data_path = Some(ArgPath::Owned(data_path.into()));
            self
        }

        pub fn with_data(mut self, data: D) -> Self {
            self.data = Some(data);
            self
        }

        pub fn with_model_name(mut self, name: &str) -> Self {
            self.model_name = String::from(name);
            self
        }

        pub fn with_workspace_path(mut self, workspace_path: &Path) -> Self {
            self.workspace_path = ArgPath::Owned(workspace_path.into());
            self
        }

        fn executable_name(&self) -> ArgPath {
            let mut res = PathBuf::from(self.workspace_path.as_path()).join(&self.model_name);
            if OS == "windows" {
                res.set_extension("exe");
            }
            ArgPath::Owned(res)
        }

        pub fn build(self) -> Result<CmdStanModel<T, D>, FileError> {
            let mut compiled = self.compiled;
            if !self.complie_arg.is_empty() {
                let absolute_executable = absolute(self.executable_name().as_path()).map_err(FileError::FileSystem)?;
                let mut command = Command::new("make");
                command.current_dir(std::env::var(STAN_HOME_KEY).map_err(FileError::EnvVar)?)
                    .arg(absolute_executable);

                for (key, val) in self.complie_arg.into_iter() {
                    command.arg(format!("{}={}",key,val));
                }

                let output = command.output().map_err(FileError::FileSystem)?;   
                if !output.status.success() {
                    return Err(FileError::Compilation(output))
                }

                compiled = true;
            }

            Ok(CmdStanModel {
                model: self.model,
                workspace_path: self.workspace_path,
                model_name: self.model_name,
                data_path: self.data_path,
                data: self.data,
                compiled,
            })
        }
    }
}

#[allow(unused_imports)]
pub use wrapper::CmdStanModel;