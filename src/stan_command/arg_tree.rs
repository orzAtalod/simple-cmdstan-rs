use std::{ffi::{OsStr, OsString}, process::Command};

pub enum ArgError {
    NoArgTree,
    NotValidArgTreeType(String),
    BadPath(String),
    BadArgumentValue(String),
    FileSystemError(std::io::Error),
}

pub enum ArgType {
    Sample,
    Optimize,
    Variational,
    Diagnose,
    GenerateQuantities,
    Pathfinder,
    LogProb,
    Laplace,
}

pub trait ArgThrough {
    fn arg_type(&self) -> Result<ArgType, ArgError>;
    fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError>;
}

pub struct ArgTreeLinks {
    trees: Vec<Box<dyn ArgThrough>>,
}

impl ArgThrough for ArgTreeLinks {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        if let Some(at) = self.trees.first() {
            at.arg_type()
        } else {
            Err(ArgError::NoArgTree)
        }
    }

    fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
        for at in self.trees.iter() {
            at.arg_through(cmd)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct WithCommonArgs<T:ArgThrough>  {
    pub root: T,
    pub id: ArgID,
    pub data: ArgData,
    pub init: ArgInit,
    pub random: ArgRandom,
    pub output: ArgOutput,
    pub num_threads: ArgNumThreads,
}

impl<T:ArgThrough> ArgThrough for WithCommonArgs<T> {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        self.root.arg_type()
    }

    fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
        self.root.arg_through(cmd)?;
        self.id.arg_through(cmd)?;
        self.data.arg_through(cmd)?;
        self.init.arg_through(cmd)?;
        self.random.arg_through(cmd)?;
        self.output.arg_through(cmd)?;
        self.num_threads.arg_through(cmd)?;
        Ok(())
    }
}

impl<T:ArgThrough> WithCommonArgs<T> {
    pub fn new(root: T) -> Self {
        Self {
            root,
            id: ArgID::new(),
            data: ArgData::new(),
            init: ArgInit::new(),
            random: ArgRandom::new(),
            output: ArgOutput::new(),
            num_threads: ArgNumThreads::new(),
        }
    }
}

impl<T:ArgThrough+Default> Default for WithCommonArgs<T> {
    fn default() -> Self {
        Self {
            root: T::default(),
            id: ArgID::new(),
            data: ArgData::new(),
            init: ArgInit::new(),
            random: ArgRandom::new(),
            output: ArgOutput::new(),
            num_threads: ArgNumThreads::new(),
        }
    }
}

pub fn args_combine(name: &str, val: &OsStr) -> OsString {
    let mut res = OsString::new();
    res.push(name);
    res.push("=");
    res.push(val);
    res
}

pub fn verify_file_readable(path: &std::path::Path) -> Result<(), ArgError> {
    std::fs::File::open(path).map(|_|()).map_err(ArgError::FileSystemError)
}

pub fn verify_file_writeable(path: &std::path::Path) -> Result<(), ArgError> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true) // 避免 truncate 覆盖
        .open(path)
        .map(|_| ())
        .map_err(ArgError::FileSystemError)
}

mod common_arg_trees {
    use std::path::{Path, PathBuf};
    use super::*;

    mod arg_id {
        use super::*;
        const DEFAULT_ID: u32 = 1;
        #[derive(Debug, PartialEq, Eq)]
        pub struct ArgID {
            id: u32,
        }
    
        impl ArgThrough for ArgID {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgID is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if self.id!=DEFAULT_ID { cmd.arg(format!("id={}", self.id)); }
                Ok(())
            }
        }
    
        impl ArgID {
            pub fn new() -> ArgID {
                ArgID{ id: DEFAULT_ID }
            }
    
            pub fn set_id(&mut self, new_id: u32) -> &mut Self {
                self.id = new_id;
                self
            }
        }
    
        impl Default for ArgID {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub use arg_id::*;

    mod arg_data {
        use super::*;
        #[derive(Debug, PartialEq, Eq)]
        pub struct ArgData {
            file: PathBuf,
        }
    
        impl ArgThrough for ArgData {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgData is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if !self.path_is_empty() {
                    cmd.arg("data");
                    cmd.arg(args_combine("file", self.file.as_os_str()));
                }
                Ok(())
            }
        }
    
        impl ArgData {
            pub fn new() -> ArgData {
                Self{ file: PathBuf::new() }
            }
    
            pub fn set_data_path(&mut self, path: &Path) -> Result<&mut Self, ArgError> {
                verify_file_readable(path)?;
                self.file = path.to_path_buf();
                Ok(self)
            }
    
            pub fn path_is_empty(&self) -> bool {
                self.file.as_os_str().is_empty()
            }
        }
    
        impl Default for ArgData {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub use arg_data::*;

    mod arg_init {
        use super::*;
        use crate::prelude::DataEntry;
        use std::collections::hash_map::HashMap;
        use std::f64::EPSILON;
        use std::fs::File;
        use std::io::Write;

        const RANGE_DEFAULT: f64 = 2.0;

        #[derive(Debug, PartialEq)]
        pub enum ArgInit {
            ByRange(f64),
            ByPath(PathBuf),
            ByValue((HashMap<String, DataEntry>, PathBuf)),
        }

        impl ArgThrough for ArgInit {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgInit is not a valid root arg".to_string()))
            }

            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if let Self::ByRange(x) = self {
                    if (x-RANGE_DEFAULT).abs() <= 10.0*EPSILON { //default value
                        return Ok(())
                    }
                }

                match self {
                    Self::ByRange(val) => {
                        cmd.arg(format!("init={}",val));
                    }
                    Self::ByPath(val) => {
                        cmd.arg(args_combine("init", val.as_os_str()));
                    }
                    Self::ByValue((params,file)) => {
                        let mut param_init = String::new();

                        param_init.push_str("{\n");
                        for (i,(name, initval)) in params.iter().enumerate() {
                            if i>0 {
                                param_init.push_str(",\n");
                            }
                            param_init.push_str(&format!("    \"{name}\": "));
                            initval.write_to_stan_json(&mut param_init);
                        }
                        param_init.push_str("\n}");

                        let init_path: &Path;
                        if file.as_os_str().is_empty() {
                            init_path = &Path::new("init_params_setup.json");
                        } else {
                            init_path = &file;
                        }

                        let mut file= File::create(init_path).map_err(|e| ArgError::FileSystemError(e))?;
                        file.write(param_init.as_bytes()).map_err(|e| ArgError::FileSystemError(e))?;
                        cmd.arg(args_combine("init", init_path.as_os_str()));
                    }
                };
                Ok(())
            }
        }

        impl ArgInit {
            pub fn new() -> Self {
                ArgInit::ByRange(RANGE_DEFAULT)
            }

            pub fn set_init_by_range(&mut self, r: f64) -> &mut Self {
                *self = Self::ByRange(r);
                self
            }

            pub fn set_init_by_path(&mut self, file: &Path) -> Result<&mut Self, ArgError> {
                verify_file_readable(file)?;
                *self = Self::ByPath(file.to_path_buf());
                Ok(self)
            }

            pub fn set_init_by_param<T: Into<DataEntry>>(&mut self, param: &str, val: T) -> &mut Self {
                if let Self::ByValue((p,f)) = self {
                    p.insert(param.to_string(), val.into());
                } else {
                    let mut par = HashMap::new();
                    par.insert(param.to_string(), val.into());
                    *self = Self::ByValue((par, PathBuf::new()));
                }
                self
            }

            pub fn target_init_by_param_path(&mut self, file: &Path) -> Result<&mut Self, ArgError> {
                verify_file_writeable(file)?;
                if let Self::ByValue((p, f)) = self {
                    *f = file.to_path_buf();
                } else {
                    *self = Self::ByValue((HashMap::new(), file.to_path_buf()));
                }
                Ok(self)
            }
        }

        impl Default for ArgInit {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub use arg_init::*;

    mod arg_random {
        use super::*;
        #[derive(Debug, PartialEq)]
        pub struct ArgRandom {
            seed: Option<u32>,
        }

        impl ArgThrough for ArgRandom {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgRandom is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if let Some(x) = self.seed {
                    cmd.arg("random");
                    cmd.arg(format!("seed={}", x));
                }
                Ok(())
            }
        }
    
        impl ArgRandom {
            pub fn new() -> ArgRandom {
                ArgRandom { seed: None }
            }
    
            pub fn set_seed(&mut self, new_seed: Option<u32>) -> &mut Self {
                self.seed = new_seed;
                self
            }
        }
    
        impl Default for ArgRandom {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub use arg_random::*;

    mod arg_num_threads {
        use super::*;
        const DEFAULT_THREADS: u32 = 1;
        #[derive(Debug,PartialEq,Eq)]
        pub struct ArgNumThreads {
            threads: u32,
        }
    
        impl ArgThrough for ArgNumThreads {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgNumThreads is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if self.threads!=DEFAULT_THREADS { cmd.arg(format!("num_threads={}", self.threads)); }
                Ok(())
            }
        }
    
        impl ArgNumThreads {
            pub fn new() -> ArgNumThreads {
                ArgNumThreads { threads: DEFAULT_THREADS }
            }
    
            pub fn set_id(&mut self, new_ts: u32) -> &mut Self {
                self.threads = new_ts;
                self
            }
        }
    
        impl Default for ArgNumThreads {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub use arg_num_threads::*;

    mod arg_output {
        use super::*;
        #[derive(Debug)]
        pub struct ArgOutput {
            file: PathBuf,
            diagnostic_file: PathBuf,
            refresh: u32,
            sig_figs: i32,
            profile_file: PathBuf,
            save_cmdstan_config: bool,
        }

        impl ArgThrough for ArgOutput {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgOutput is not a valid root arg".to_string()))
            }

            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if self.is_default() {
                    return Ok(());
                }
                cmd.arg("output");
                if self.file.as_os_str() != "output.csv" {
                    cmd.arg(args_combine("file", self.file.as_os_str()));
                }
                if !self.diagnostic_file.as_os_str().is_empty() {
                    cmd.arg(args_combine("diagnostic_file", self.diagnostic_file.as_os_str()));
                }
                if self.refresh != 100 {
                    cmd.arg(format!("refresh={}",self.refresh));
                }
                if self.sig_figs != -1 {
                    cmd.arg(format!("sig_fig={}",self.sig_figs));
                }
                if !self.profile_file.as_os_str().is_empty() {
                    cmd.arg(args_combine("profile_file", self.profile_file.as_os_str()));
                }
                if self.save_cmdstan_config {
                    cmd.arg("save_cmdstan_config=true");
                }
                Ok(())
            }
        }

        impl ArgOutput {
            pub fn new() -> ArgOutput {
                ArgOutput {
                    file: "output.csv".to_string().into(),
                    diagnostic_file: PathBuf::new(),
                    refresh: 100,
                    sig_figs: -1,
                    profile_file: PathBuf::new(),
                    save_cmdstan_config: false
                }
            }

            fn is_default(&self) -> bool {
                if self.file.as_os_str() != "output.csv" { return false; }
                if !self.diagnostic_file.as_os_str().is_empty() { return false; }
                if self.refresh != 100 { return false; }
                if self.sig_figs != -1 { return false; }
                if !self.profile_file.as_os_str().is_empty() { return false; }
                if self.save_cmdstan_config { return false; }
                true
            }

            pub fn target_output_file(&mut self, path: &Path) -> Result<&mut Self, ArgError> {
                verify_file_writeable(path)?;
                self.file = path.to_path_buf();
                Ok(self)
            }

            pub fn target_diagnostic_file(&mut self, path: &Path) -> Result<&mut Self, ArgError> {
                verify_file_writeable(path)?;
                self.diagnostic_file = path.to_path_buf();
                Ok(self)
            }

            pub fn set_refresh(&mut self, reftime: u32) -> &mut Self {
                self.refresh = reftime;
                self
            }

            pub fn set_sig_figs(&mut self, sig_figs: i32) -> Result<&mut Self, ArgError> {
                if sig_figs < -1 || sig_figs > 18 {
                    return Err(ArgError::BadArgumentValue(
                        format!("argument output->sig_figs requires 0 <= integer <= 18 or -1, recieved {}",sig_figs).to_string()));
                }
                self.sig_figs = sig_figs;
                Ok(self)
            }

            pub fn target_profile_file(&mut self, path: &Path) -> Result<&mut Self, ArgError> {
                verify_file_writeable(path)?;
                self.profile_file = path.to_path_buf();
                Ok(self)
            }

            pub fn set_save_cmdstan_config(&mut self, val: bool) -> &mut Self {
                self.save_cmdstan_config = val;
                self
            }
        }

        impl Default for ArgOutput {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub use arg_output::*;
}

use common_arg_trees::{ArgID, ArgData, ArgInit, ArgRandom, ArgNumThreads, ArgOutput};