#[macro_use]
pub mod arg_tree;
mod sample;
mod optimize;
mod common_arg;
mod variational;

pub mod stan_command_core {
    use crate::prelude::*;
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
        /// 
        /// args: like "data", "output", "random", etc.
        /// 
        /// argv: like "file=data.json", "seed=121", etc.
        /// 
        pub fn add_args(&mut self, args: &str, argv: Option<&str>) -> &mut Self {
            let args = args.trim().to_string();
            let argv = argv.map(|s| s.trim().to_string());
            self.command_args.insert(args, argv);
            self
        }

        /// panic when not inited.
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

            let out_value = command.output().map_err( StanError::CompileIOError)?;
            if !out_value.status.success() {
                return Err(StanError::CompileError("Stan command failed".to_string()).into());
            }

            analyzer.analyze(out_value, &output_file)
        }
    }
}

#[cfg(test)]
mod test_command {
    use crate::prelude::*;
    use std::path::Path;
    const PATHS: [&str;3] = [".conda\\Library\\bin\\cmdstan", "examples\\bernoulli\\", "bernoulli.stan"];
    
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
        cmd.add_args("output", Some("file=examples\\bernoulli\\outputs\\output1.csv"));
        let res1 = cmd.execute(SampleResultAnalyzer {}).unwrap();
        let mut cmd = StanCommand::new(&stm, StanCommandType::Sample).unwrap();
        cmd.add_args("output", Some("file=examples\\bernoulli\\outputs\\output2.csv"));
        cmd.add_args("random", Some("seed=20060626"));
        let res2 = cmd.execute(SampleResultAnalyzer {}).unwrap();
        println!("res1.len={}, res2.len={}", res1.length, res2.length);
        assert_eq!(res1.length, res2.length);
    }

    #[test]
    #[should_panic]
    #[ignore = "should be tested in single-thread"]
    fn test_panic_without_init() {
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]), Path::new("bernoulli3.stan"));
        stm.set_data_path("examples\\bernoulli\\bernoulli.data.json");
        stm.complie().unwrap();
        let mut cmd = StanCommand::new(&stm, StanCommandType::Sample).unwrap();
        let _ = cmd.execute(SampleResultAnalyzer {}).unwrap();
    }
}