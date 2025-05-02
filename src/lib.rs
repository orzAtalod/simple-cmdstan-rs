mod data_entries;
mod result_analyzer;
mod stan_model;
mod stan_command;
mod arg_paths;
mod error;

pub trait StanData {
    fn write_as_stan_data(&self) -> String;
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

pub use init::stan_init;
mod init {
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

    // importants
    pub use super::StanError;
    pub use super::stan_init;

    // structs
    pub use crate::data_entries::data_entry::DataEntry;
    pub use crate::data_entries::data_entry::DataEntries;
    pub use crate::data_entries::data_collections::DataCollection;
}