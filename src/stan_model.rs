mod wrapper;
mod wrapper_builder;

use crate::{arg_paths::{ArgPath, ArgReadablePath}, stan_command::{arg_into, ArgThrough, StanResult}, error::{ParamError, FileError, CmdStanError}};
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