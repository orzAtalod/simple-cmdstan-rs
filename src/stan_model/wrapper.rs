use crate::{prelude::DataEntries, StanData};
use std::path::absolute;
use std::process::Command;
use crate::init::STAN_HOME_KEY;
use std::env::consts::OS;
use super::*;

#[derive(Debug, Clone)]
pub struct CmdStanModel<T, D: StanData = DataEntries> {
    pub model: T,
    pub workspace_path: ArgPath,
    pub model_name: String,
    pub data_path: Option<ArgPath>,
    pub data: Option<D>,
    pub compiled: bool,
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