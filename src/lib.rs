mod data_entries;
mod result_analyzer;
mod stan_model;
mod stan_command;

pub trait StanData {
    fn write_as_stan_data(&self) -> String;
}

pub trait StanResultAnalyzer {
    type AnalyzeResult: Sized;
    type Err: std::error::Error + From<crate::StanError>;
    fn analyze(&self, output: std::process::Output, out_file: &std::path::Path) -> Result<Self::AnalyzeResult, Self::Err>;
}

pub trait StanModel {
    fn check_ready(&self) -> bool;
    fn get_model_excutable(&self) -> std::path::PathBuf;
    fn get_data_path(&self) -> std::path::PathBuf;
    fn get_workspace_path(&self) -> std::path::PathBuf {
        self.get_model_excutable().parent().unwrap().to_path_buf()
    }
}

pub use stan_error::StanError;
mod stan_error {
    #[derive(Debug)]
    pub enum StanError {
        DataError(String),
        CompileIOError(std::io::Error),
        IoError(std::io::Error),
        CompileError(String),
        ModelIsNotReady,
        BadParameter(String),
    }
    
    impl std::fmt::Display for StanError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                StanError::DataError(msg) => write!(f, "Data error: {msg}"),
                StanError::CompileIOError(e) => write!(f, "Compile IO error: {e}"),
                StanError::IoError(e) => write!(f, "IO error: {e}"),
                StanError::CompileError(msg) => write!(f, "Compile error: {msg}"),
                StanError::ModelIsNotReady => write!(f, "Model is not ready"),
                StanError::BadParameter(args) => write!(f, "Bad parameter for {args}")
            }
        }
    }
    
    impl From<std::io::Error> for StanError {
        fn from(e: std::io::Error) -> Self {
            StanError::IoError(e)
        }
    }

    impl std::error::Error for StanError {}    
}

pub use stan_interface::stan_init;
mod stan_interface {
    use std::path::Path;
    pub const STAN_HOME_KEY: &str = "STAN_HOME_RS";
    pub fn stan_init(stan_home_path: &Path) -> Result<(), std::io::Error> {
        unsafe {
            std::env::set_var(STAN_HOME_KEY, stan_home_path.as_os_str());
        }
        Ok(())
    }
}

pub mod prelude {
    // traits
    pub use super::StanData;
    pub use super::StanResultAnalyzer;
    pub use super::StanModel;

    // importants
    pub use super::StanError;
    pub use super::stan_init;

    // structs
    pub use crate::stan_model::std_stan_model::StdStanModel;
    pub use crate::stan_command::stan_command_core::StanCommand;
    pub use crate::stan_command::stan_command_core::StanCommandType;
    pub use crate::data_entries::data_entry::DataEntry;
    pub use crate::data_entries::data_entry::DataEntries;
    pub use crate::data_entries::data_collections::DataCollection;
    pub use crate::result_analyzer::ResultAnalyzerError;
    pub use crate::result_analyzer::stan_result_analyzer::{
        RawTable, RawTableAnalyzer, SampleResult, SampleResultAnalyzer, OptimizeResult, OptimizeResultAnalyzer
    };
}