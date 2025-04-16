pub trait StanData {
    fn write_as_stan_data(&self) -> String;
}

pub trait StanResultAnalyzer {
    type AnalyzeResult: Sized;
    type Err: std::error::Error + From<stan_error::StanError>;
    fn analyze(&self, output: std::process::Output, out_file: &std::path::Path) -> Result<Self::AnalyzeResult, Self::Err>;
}

/// The Function `get_data_path` and `get_model_path` are only used after check_ready is true.
/// The get_model_path function should return the .exe file, so please ensure the model is complied in the check_ready function.
/// Default implementation of get_workspace_path is the same as get_model_path, but it will remove the last part of the path.
pub trait StanModel {
    fn check_ready(&self) -> bool;
    fn get_model_excutable(&self) -> std::path::PathBuf;
    fn get_data_path(&self) -> std::path::PathBuf;
    fn get_workspace_path(&self) -> std::path::PathBuf {
        self.get_model_excutable().parent().unwrap().to_path_buf()
    }
}

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
    
    impl Into<StanError> for std::io::Error {
        fn into(self) -> StanError {
            StanError::IoError(self)
        }
    }
    
    impl std::error::Error for StanError {}    
}
/// a standard StanModel implementation.
mod std_stan_model {
    use std::fs::File;
    use super::{StanData, stan_error::StanError, StanModel};
    use std::process::Command;
    use std::path::{Path,PathBuf};
    use std::ffi::OsStr;
    use std::io::prelude::*;
    use std::env::consts::OS;

    pub struct StdStanModel<T: StanData> {
        dir: PathBuf,
        name: PathBuf,
        data: Option<T>,
        data_path: Option<PathBuf>,
        complied: bool,
    }

    impl<T: StanData> StdStanModel<T> {
        /// Create a new StanModel with the given directory and name.
        /// if the name ends with extension that is excutable, the complied field will be set ture, otherwise false.
        /// the extension will be removed.
        pub fn new(dir: &Path, name: &Path) -> Self {
            let mut complied= false;
            let mut name = name.to_path_buf();
            if let Some(ext) = name.extension() {
                if ext == "exe" || ext == "run" || ext == "bin" || ext == "app" {
                    complied = true;
                }
            }
            name.set_extension("");
            
            Self {
                dir: dir.to_path_buf(),
                name,
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

        pub fn set_data_path(&mut self, path: &str) -> &mut Self {
            self.data_path = Some(Path::new(path).to_path_buf());
            self
        }

        /// try to create a json file corresponding to the data.
        /// if the data is None, return an error.
        /// if the data_path is None, create a file in the dir with the name of the model.
        pub fn write_stan_data(&mut self) -> Result<&mut Self, StanError> {
            if self.data.is_none() {
                return Err(StanError::DataError("No data provided".to_string()));
            }
            if self.data_path.is_none() {
                let mut res = self.dir.clone();
                res.push(&self.name);
                res.set_extension("data.json");
                self.data_path = Some(res);
            }

            if let Some(data) = &self.data {
                let mut file = File::create(self.data_path.clone().unwrap()).map_err(|e| StanError::IoError(e))?;
                let content = data.write_as_stan_data();
                file.write_all( content.as_bytes()).map_err(|e| StanError::IoError(e))?;
                Ok(self)
            } else { unreachable!() } // hint the complier
        }

        fn get_excutable_name(&self) -> PathBuf {
            let mut res = self.dir.join(&self.name);
            if OS == "windows" {
                res.set_extension("exe");
            }
            res
        }

        pub fn complie(&mut self) -> Result<&mut Self, StanError> {
            if self.complied {
                return Ok(self);
            }
            
            let command = Command::new("make")
                .arg(self.get_excutable_name())
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

        fn get_data_path(&self) -> PathBuf {
            self.data_path.clone().unwrap()
        }

        fn get_model_excutable(&self) -> PathBuf {
            self.get_excutable_name()
        }

        fn get_workspace_path(&self) -> PathBuf {
            self.dir.clone()
        }
    }
}

mod stan_command {
    use super::{StanData, StanResultAnalyzer, StanModel, stan_error::StanError};
    use std::collections::HashMap;
    use std::process::Command;
    use std::path::{Path,PathBuf};

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
            let mut command = Command::new(self.model.get_model_excutable());

            match self.command_type {
                StanCommandType::Sample => command.arg("sample"),
                StanCommandType::Optimize => command.arg("optimize"),
                StanCommandType::Other(ref s) => command.arg(s),
            };

            command.arg("data");
            if self.command_args.contains_key("data") {
                if let Some(Some(data)) = self.command_args.get("data") {
                    command.arg(data);
                } else {
                    return Err(StanError::BadParameter("data".to_string()).into());
                }
            } else {
                command.arg(format!("file={}", self.model.get_data_path().display()));
            }

            let output_file: PathBuf;
            command.arg("output");
            if self.command_args.contains_key("output") {
                if let Some(Some(output_str)) = self.command_args.get("output") {
                    command.arg(output_str);
                    output_file = Path::new(&output_str[5..]).to_path_buf();
                } else {
                    return Err(StanError::BadParameter("output".to_string()).into());
                }
            } else {
                output_file = self.model.get_workspace_path().join("output.csv");
                command.arg(format!("file={}", output_file.display()));
            }

            for (key, value) in &self.command_args {
                if key != "data" && key != "output" {
                    command.arg(key);
                    if let Some(v) = value {
                        command.arg(v);
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
mod stan_result_analyzer {
    use super::{StanResultAnalyzer, stan_error::StanError};
    use std::{collections::HashMap, io::Read};
    use std::path::Path;

    #[derive(Debug)]
    pub struct SampleResult {
        pub samples: HashMap<String, Vec<f64>>,
        pub length: usize,
    }

    pub struct SampleResultAnalyzer {}

    impl StanResultAnalyzer for SampleResultAnalyzer {
        type AnalyzeResult = SampleResult;
        type Err = StanError;

        /// Panic when the csv format is not correct.
        fn analyze(&self, _: std::process::Output, out_file: &Path) -> Result<Self::AnalyzeResult, Self::Err> {
            let mut res = SampleResult {
                samples: HashMap::new(),
                length: 0,
            };

            let mut file = std::fs::File::open(out_file).map_err(|e| StanError::IoError(e))?;
            let mut reader= String::new();
            file.read_to_string(&mut reader).map_err(|e| StanError::IoError(e))?;

            let mut arg_index = Vec::new();

            for (i,line) in reader.lines().enumerate() {
                if line.starts_with("#") {
                    continue;
                }

                let parts = line.split(',');
                if arg_index.is_empty() {
                    for arg in parts {
                        arg_index.push(arg.to_string());
                    }
                    for arg in arg_index.iter() {
                        res.samples.insert(arg.clone(), Vec::new());
                    }
                } else {
                    for (i,argv) in parts.enumerate() {
                        if i >= arg_index.len() {
                            panic!("Bad CSV Format: line {} has more columns than header", i);
                        }
                        res.samples.get_mut(arg_index[i].as_str()).unwrap().push(argv.parse().unwrap());
                    }
                    res.length += 1;
                }
            }

            Ok(res)
        }
    }

    #[derive(Debug)]
    pub struct OptimizeResult {
        pub parameters_iter: HashMap<String, Vec<f64>>,
        pub parameters: HashMap<String, f64>,
        pub log_likelihood: f64,
    }

    pub struct OptimizeResultAnalyzer {}

    impl StanResultAnalyzer for OptimizeResultAnalyzer {
        type AnalyzeResult = OptimizeResult;
        type Err = StanError;

        fn analyze(&self, _: std::process::Output, out_file: &Path) -> Result<Self::AnalyzeResult, Self::Err> {
            let mut res = OptimizeResult {
                parameters_iter: HashMap::new(),
                parameters: HashMap::new(),
                log_likelihood: 0.0,
            };

            let mut file = std::fs::File::open(out_file).map_err(|e| StanError::IoError(e))?;
            let mut reader= String::new();
            file.read_to_string(&mut reader).map_err(|e| StanError::IoError(e))?;

            let mut arg_index = Vec::new();

            for (i,line) in reader.lines().enumerate() {
                if line.starts_with("#") {
                    continue;
                }

                let parts = line.split(',');
                if arg_index.is_empty() {
                    for arg in parts {
                        arg_index.push(arg.to_string());
                    }
                    for arg in arg_index.iter() {
                        res.parameters_iter.insert(arg.clone(), Vec::new());
                    }
                } else {
                    for (i,argv) in parts.enumerate() {
                        if i >= arg_index.len() {
                            panic!("Bad CSV Format: line {} has more columns than header", i);
                        }
                        res.parameters_iter.get_mut(arg_index[i].as_str()).unwrap().push(argv.parse().unwrap());
                    }
                }
            }

            for arg in arg_index.iter() {
                if let Some(v) = res.parameters_iter.get(arg) {
                    if v.len() > 0 {
                        res.parameters.insert(arg.clone(), *(v.last().unwrap()));
                    } else {
                        panic!("Bad CSV Format: no value for {}", arg);
                    }
                } else {
                    panic!("Bad CSV Format: no value for {}", arg);
                }
            }

            res.log_likelihood = *res.parameters_iter.get("lp__").unwrap().last().unwrap();

            Ok(res)
        }
    }
}

#[cfg(test)]
mod stan_model_test {
    use crate::{data_entries::core::DataEntries, stan_interface::stan_init, stan_model::StanModel};
    use std::fs::File;
    use std::path::{Path,PathBuf};
    use super::stan_result_analyzer::SampleResultAnalyzer;
    use super::std_stan_model::*;
    use super::stan_command::{StanCommand,StanCommandType};
    use std::io::Read;
    const PATHS: [&str;3] = [".conda\\Library\\bin\\cmdstan", "examples\\bernoulli\\", "bernoulli.stan"];
    
    #[test]
    fn test_init() {
        stan_init(Path::new(PATHS[0])).unwrap();
    }

    #[test]
    fn test_complie() {
        stan_init(Path::new(PATHS[0])).unwrap();
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]),Path::new(PATHS[2]));
        println!("{}",stm.get_model_excutable().display());
        stm.complie().unwrap();
        assert!(!stm.check_ready());
    }

    #[test]
    fn test_dump_data() {
        stan_init(Path::new(PATHS[0])).unwrap();
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]), Path::new("bernoulli2.exe"));
        let mut de = DataEntries::new();
        de.add_entry("N", 10);
        de.add_entry("y", vec![0,1,0,0,0,0,0,0,0,1]);
        stm.link_data(de);
        stm.write_stan_data().unwrap();
    }

    #[test]
    fn test_model_ready() {
        stan_init(Path::new(PATHS[0])).unwrap();
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]), Path::new("bernoulli2.exe"));
        assert!(!stm.check_ready());
        stm.set_data_path("examples\\bernoulli\\bernoulli.data.json");
        assert!(stm.check_ready());
    }

    #[test]
    fn test_command_sample() {
        stan_init(Path::new(PATHS[0])).unwrap();
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]), Path::new("bernoulli.exe"));
        stm.set_data_path("examples\\bernoulli\\bernoulli.data.json");
        let mut cmd = StanCommand::new(&stm, StanCommandType::Sample).unwrap();
        let res = cmd.execute(SampleResultAnalyzer {}).unwrap();
        println!("ends with {} samples.", res.length);
        assert!(res.length == 1000);
        assert!(res.samples.contains_key("lp__"));
        assert!(res.samples.contains_key("theta"));
        assert!(!res.samples.contains_key("alpha"));
    }

    #[test]
    fn test_command_sample_with_arg() {
        stan_init(Path::new(PATHS[0])).unwrap();
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]), Path::new("bernoulli.exe"));
        stm.set_data_path("examples\\bernoulli\\bernoulli.data.json");
        
        let mut cmd = StanCommand::new(&stm, StanCommandType::Sample).unwrap();
        cmd.add_args("random", Some("seed=20060626"));
        cmd.add_args("output", Some("file=output1.csv"));
        let res1 = cmd.execute(SampleResultAnalyzer {}).unwrap();
        let mut cmd = StanCommand::new(&stm, StanCommandType::Sample).unwrap();
        cmd.add_args("output", Some("file=output2.csv"));
        cmd.add_args("random", Some("seed=20060626"));
        let res2 = cmd.execute(SampleResultAnalyzer {}).unwrap();
        assert_eq!(res1.length, res2.length);
    }
}