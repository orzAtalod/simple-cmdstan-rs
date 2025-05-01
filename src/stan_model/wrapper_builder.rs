use std::collections::HashMap;
use crate::stan_command::WithDefaultArg;
use crate::{prelude::DataEntries, StanData};
use std::path::{absolute, Path};
use std::process::Command;
use crate::init::STAN_HOME_KEY;
use std::{env::consts::OS, ffi::OsStr};
use super::*;
use super::wrapper::CmdStanModel;
pub struct CmdStanModelBuilder<T, D:StanData = DataEntries> {
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