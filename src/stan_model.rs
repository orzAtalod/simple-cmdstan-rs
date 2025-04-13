use std::process::Output;

pub trait StanData {
    fn write_as_stan_data(&self) -> String;
}

pub trait StanResultAnalyzer {
    type AnalyzeResult: Sized;
    type Err: std::error::Error + From<StanError>;
    fn analyze(&self, output: std::process::Output, out_file: &str) -> Result<Self::AnalyzeResult, Self::Err>;
}

/// The Function `get_data_path` and `get_model_path` are only used after check_ready is true.
/// The get_model_path function should return the .exe file, so please ensure the model is complied in the check_ready function.
/// Default implementation of get_workspace_path is the same as get_model_path, but it will remove the last part of the path.
pub trait StanModel {
    fn check_ready(&self) -> bool;
    fn get_model_path(&self) -> String;
    fn get_data_path(&self) -> String;
    fn get_workspace_path(&self) -> String {
        let mut str = self.get_model_path();
        while !str.ends_with("/") && !str.ends_with("\\") && !str.ends_with(":") {
            str.pop();
        }
        return str;
    }
}

pub enum StanError {
    DataError(String),
    CompileIOError(std::io::Error),
    IoError(std::io::Error),
    CompileError(String),
    ModelIsNotReady,
}

/// a standard StanModel implementation.
mod std_stan_model {
    use std::fs::File;
    use super::{StanData, StanError, StanModel};
    use std::process::Command;

    pub struct StdStanModel<T: StanData> {
        dir: String,
        name: String,
        data: Option<T>,
        data_path: Option<String>,
        complied: bool,
    }

    impl<T: StanData> StdStanModel<T> {
        /// Create a new StanModel with the given directory and name.
        /// Every space in the begin or end of directory or name will be trimmed.
        /// If the directory does not end with a '/', it will be added.
        /// If the name ends with '.stan', it will be removed.
        pub fn new(dir: &str, name: &str) -> Self {
            let mut dir = dir.trim().to_string();
            if !dir.ends_with('/') {
                dir.push('/');
            }

            let mut complied= false;
            let mut name = name.trim().to_string();
            if name.ends_with(".stan") {
                name = name[..name.len() - 5].to_string();
            }
            else if name.ends_with(".exe") {
                name = name[..name.len() - 4].to_string();
                complied = true;
            }
            
            Self {
                dir: dir.to_string(),
                name: name.to_string(),
                data: None,
                data_path: None,
                complied,
            }
        }

        /// drop the old data_path.
        pub fn link_data(&mut self, data: T) -> &mut Self {
            self.data = Some(data);
            self.data_path = None;
            self
        }

        pub fn set_data_path(&mut self, path: &str) {
            self.data_path = Some(path.to_string());
        }

        /// try to create a json file corresponding to the data.
        /// if the data is None, return an error.
        /// if the data_path is None, create a file in the dir with the name of the model.
        pub fn write_stan_data(&mut self) -> Result<&mut Self, StanError> {
            if self.data.is_none() {
                return Err(StanError::DataError("No data provided".to_string()));
            }
            if self.data_path.is_none() {
                self.data_path = Some(format!("{}/{}.data.json", self.dir, self.name));
            }

            if let Some(data) = &self.data {
                let mut file = File::create(self.data_path.clone().unwrap()).map_err(|e| StanError::IoError(e))?;
                let content = data.write_as_stan_data();
                std::io::Write::write_all(&mut file, content.as_bytes()).map_err(|e| StanError::IoError(e))?;
                Ok(self)
            } else { unreachable!() } // hint the complier
        }

        pub fn complie(&mut self) -> Result<&mut Self, StanError> {
            if self.complied {
                return Ok(self);
            }
            
            let command = Command::new("make")
                .arg(format!("{}{}.exe", self.dir, self.name))
                .status().map_err(|e| StanError::CompileIOError(e))?;

            if !command.success() {
                return Err(StanError::CompileError("Compile failed".to_string()));
            } else {
                self.complied = true;
                Ok(self)    
            }
        }

        pub fn set_complied(&mut self) -> &mut Self {
            self.complied = true;
            self
        }
    }

    impl<T:StanData> StanModel for StdStanModel<T> {
        fn check_ready(&self) -> bool {
            self.complied && self.data_path.is_some()
        }

        fn get_data_path(&self) -> String {
            if let Some(path) = &self.data_path {
                path.clone()
            } else {
                panic!("Data path is not set")
            }
        }

        fn get_model_path(&self) -> String {
            if self.complied {
                format!("{}{}.exe", self.dir, self.name)
            } else {
                panic!("Model is not complied")
            }
        }

        fn get_workspace_path(&self) -> String {
            self.dir.clone()
        }
    }
}

mod stan_command {
    use super::{StanData, StanResultAnalyzer, StanModel, StanError};
    use std::collections::HashMap;
    use std::process::Command;

    pub enum StanCommandType {
        Sample,
        Optimize,
        Other(String)
    }
    pub struct StanCommand<'a,T:StanModel> {
        model: &'a T,
        command_type: StanCommandType,
        command_args: HashMap<String, Option<String>>,
    }

    impl<'a,T:StanModel> StanCommand<'a,T> {
        pub fn new<'b:'a>(model: &'b T, command: StanCommandType) -> Result<Self, StanError> {
            if !model.check_ready() {
                return Err(StanError::ModelIsNotReady);
            }
            Ok(StanCommand {
                model,
                command_type: command,
                command_args: HashMap::new(),
            })
        }
    
        /// add a command line argument to the command.
        /// args: like "data", "output", "random", etc.
        /// argv: like "file=data.json", "seed=121", etc.
        pub fn add_args(&mut self, args: &str, argv: Option<&str>) -> &mut Self {
            let args = args.trim().to_string();
            let argv = argv.map(|s| s.trim().to_string());
            self.command_args.insert(args, argv);
            self
        }

        pub fn execute<R:StanResultAnalyzer>(&mut self, analyzer: R) -> Result<R::AnalyzeResult, R::Err> {
            let mut command = Command::new(format!(".\\{}", self.model.get_model_path()));

            match self.command_type {
                StanCommandType::Sample => command.arg("sample"),
                StanCommandType::Optimize => command.arg("optimize"),
                StanCommandType::Other(ref s) => command.arg(s),
            };

            if self.command_args.contains_key("data") {
                if let Some(Some(data)) = self.command_args.get("data") {
                    command.arg(format!("data {}", data));
                } else {
                    return Err(StanError::DataError("data file is not set".to_string()).into());
                }
            } else {
                command.arg(format!("data file={}", self.model.get_data_path()));
            }

            let output_file: String;
            if self.command_args.contains_key("output") {
                if let Some(Some(output)) = self.command_args.get("output") {
                    command.arg(format!("output {}", output));
                    output_file = output.clone();
                } else {
                    return Err(StanError::DataError("output file is not set".to_string()).into());
                }
            } else {
                output_file = format!("{}output.csv", self.model.get_workspace_path()).to_string();
                command.arg(format!("output file={output_file}"));
            }

            for (key, value) in &self.command_args {
                if key != "data" && key != "output" {
                    if let Some(v) = value {
                        command.arg(format!("{} {}", key, v));
                    } else {
                        command.arg(key);
                    }
                }
            }

            let out_value = command.output().map_err(|e| StanError::CompileIOError(e))?;
            if !out_value.status.success() {
                return Err(StanError::CompileError("Stan command failed".to_string()).into());
            }

            analyzer.analyze(out_value, &output_file)
        }
    }

}

/// standard StanResultAnalyzer implementation for Sample and Optimize.
mod stan_result {
    use std::collections::HashMap;

    pub enum StanResult {
        McMc(McmcResult),
        Optimize(OptimizeResult),
        Other(String),
    }

    #[derive(Debug)]
    pub struct McmcResult {
        pub samples: HashMap<String, Vec<f64>>,
        pub length: usize,
    }

    pub struct OptimizeResult {

    }
}