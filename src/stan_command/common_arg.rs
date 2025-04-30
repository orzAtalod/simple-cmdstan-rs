use super::arg_tree::*;
use std::process::Command;

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
            id: ArgID::ARG_DEFAULT,
            data: ArgData::ARG_DEFAULT,
            init: ArgInit::ARG_DEFAULT,
            random: ArgRandom::ARG_DEFAULT,
            output: ArgOutput::ARG_DEFAULT,
            num_threads: ArgNumThreads::ARG_DEFAULT,
        }
    }
}

impl<T:PartialEq+ArgThrough> PartialEq for WithCommonArgs<T> {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root &&
        self.id == other.id &&
        self.data == other.data &&
        self.init == other.init &&
        self.random == other.random &&
        self.output == other.output &&
        self.num_threads == other.num_threads
    }
}

impl<T:ArgThrough+WithDefaultArg> WithDefaultArg for WithCommonArgs<T> {
    const ARG_DEFAULT: Self = Self {
        root: T::ARG_DEFAULT,
        id: ArgID::ARG_DEFAULT,
        data: ArgData::ARG_DEFAULT,
        init: ArgInit::ARG_DEFAULT,
        random: ArgRandom::ARG_DEFAULT,
        output: ArgOutput::ARG_DEFAULT,
        num_threads: ArgNumThreads::ARG_DEFAULT,
    };
}

impl<T:ArgThrough+Default> Default for WithCommonArgs<T> {
    fn default() -> Self {
        Self {
            root: T::default(),
            id: ArgID::ARG_DEFAULT,
            data: ArgData::ARG_DEFAULT,
            init: ArgInit::ARG_DEFAULT,
            random: ArgRandom::ARG_DEFAULT,
            output: ArgOutput::ARG_DEFAULT,
            num_threads: ArgNumThreads::ARG_DEFAULT,
        }
    }
}

mod common_arg_trees {
    use super::*;

    mod arg_id {
        use super::*;
        DefArgTree!{<"Unique process identifier">ArgID => {
            <"Unique process identifier">id: u32 = 1,
        }}
    
        impl ArgThrough for ArgID {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgID is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                arg_into!(self.{id} in Self >> cmd);
                Ok(())
            }
        }
    
        impl ArgID {
            pub fn new() -> ArgID {
                Self::ARG_DEFAULT
            }

            default_setter!{
                <"Unique process identifier">(id:u32;);
            }
        }
    }
    pub use arg_id::*;

    mod arg_data {
        use super::*;
        DefArgTree!{<"Input data options">ArgData => {
            <"Input data file">file: ArgReadablePath = ArgReadablePath::ARG_DEFAULT,
        }}
    
        impl ArgThrough for ArgData {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgData is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {                
                if self.file != ArgReadablePath::ARG_DEFAULT {
                    cmd.arg("data");
                    cmd.arg(args_combine("file", self.file.as_path().as_os_str()));
                }
                Ok(())
            }
        }
    
        impl ArgData {
            pub fn new() -> ArgData {
                Self::ARG_DEFAULT
            }
    
            default_setter!{
                <"Input data file">(file:ArgReadablePath;);
            }
        }
    }
    pub use arg_data::*;

    mod arg_init {
        use super::*;
        use crate::prelude::DataEntry;
        use std::collections::hash_map::HashMap;

        #[derive(Debug, PartialEq)]
        pub enum ArgInit {
            Range(f64),
            Path(ArgReadablePath),
            ParamValue((HashMap<String, DataEntry>, ArgWritablePath)),
        }

        impl ArgThrough for ArgInit {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgInit is not a valid root arg".to_string()))
            }

            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if self.is_default() {
                    return Ok(());
                }

                match self {
                    Self::Range(val) => {
                        cmd.arg(format!("init={}",val));
                    }
                    Self::Path(val) => {
                        cmd.arg(format!("init={}",val));
                    }
                    Self::ParamValue((params,file)) => {
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

                        file.write_once(&param_init).map_err(ArgError::FileSystemError)?;

                        cmd.arg(args_combine("init", file.as_path().as_os_str()));
                    }
                };
                Ok(())
            }
        }

        impl WithDefaultArg for ArgInit {
            const ARG_DEFAULT: Self = Self::Range(2.0);
        }

        impl ArgInit {
            pub fn new() -> Self {
                Self::ARG_DEFAULT
            }

            pub fn set_init_by_range(&mut self, r: f64) -> &mut Self {
                *self = Self::Range(r);
                self
            }

            pub fn set_init_by_path(&mut self, file: ArgReadablePath) -> Result<&mut Self, ArgError> {
                *self = Self::Path(file);
                Ok(self)
            }

            pub fn set_init_by_param<T: Into<DataEntry>>(&mut self, param: &str, val: T) -> &mut Self {
                if let Self::ParamValue((p,_)) = self {
                    p.insert(param.to_string(), val.into());
                } else {
                    let mut par = HashMap::new();
                    par.insert(param.to_string(), val.into());
                    *self = Self::ParamValue((par, ArgWritablePath::Borrowed("init_params_setup.json")));
                }
                self
            }

            pub fn target_init_by_param_path(&mut self, file: ArgWritablePath) -> &mut Self {
                if let Self::ParamValue((_, f)) = self {
                    *f = file;
                } else {
                    *self = Self::ParamValue((HashMap::new(), file));
                }
                self
            }
        }
    }
    pub use arg_init::*;

    mod arg_random {
        use super::*;
        DefArgTree!{<"Random number generator options">ArgRandom => {
            <"Random number generator seed">seed: Option<u32> = None,
        }}

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
                Self::ARG_DEFAULT
            }

            default_setter!{
                <"Random number generator seed">(seed:Option<u32>;);
            }
        }
    }
    pub use arg_random::*;

    mod arg_num_threads {
        use super::*;
        DefArgTree!{<"Number of threads">ArgNumThreads => {
            <"Number of threads">threads: u32 = 1,
        }}
    
        impl ArgThrough for ArgNumThreads {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgNumThreads is not a valid root arg".to_string()))
            }
    
            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if !self.is_default() { cmd.arg(format!("num_threads={}", self.threads)); }
                Ok(())
            }
        }
    
        impl ArgNumThreads {
            pub fn new() -> ArgNumThreads {
                Self::ARG_DEFAULT
            }
    
            default_setter!{
                <"Number of threads">(threads:u32;);
            }
        }
    }
    pub use arg_num_threads::*;

    mod arg_output {
        use super::*;
        DefArgTree!{<"File output options">ArgOutput => {
            <"Output file">file: ArgWritablePath = ArgWritablePath::Borrowed("output.csv"),
            <"Auxiliary output file for diagnostic information">diagnostic_file: ArgWritablePath = ArgWritablePath::ARG_DEFAULT,
            <"Number of iterations between screen updates">refresh: u32 = 100,
            <"The number of significant figures used for the output CSV files">sig_figs: i32 = -1,
            <"File to store profiling information">profile_file: ArgWritablePath = ArgWritablePath::ARG_DEFAULT,
            <"Save cmdstan config">save_cmdstan_config: bool = false,
        }}

        impl ArgThrough for ArgOutput {
            fn arg_type(&self) -> Result<ArgType, ArgError> {
                Err(ArgError::NotValidArgTreeType("ArgOutput is not a valid root arg".to_string()))
            }

            fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError> {
                if self.is_default() {
                    return Ok(());
                }
                cmd.arg("output");
                arg_into!(self.{file, diagnostic_file, refresh, sig_figs, profile_file, save_cmdstan_config} in Self >> cmd);
                Ok(())
            }
        }

        impl ArgOutput {
            pub fn new() -> ArgOutput {
                Self::ARG_DEFAULT
            }

            default_setter!{
                <"Output file">(file:ArgWritablePath;);
                <"Auxiliary output file for diagnostic information">(diagnostic_file:ArgWritablePath;);
                <"Number of iterations between screen updates">(refresh:u32;);
                <"The number of significant figures used for the output CSV files">
                    (sig_figs:i32; !(0..=18).contains(&sig_figs) && sig_figs!=-1 => {
                        format!("argument output->sig_figs requires 0 <= integer <= 18 or -1, received {}",sig_figs)
                    });
                <"File to store profiling information">(profile_file:ArgWritablePath;);
                <"Save cmdstan config">(save_cmdstan_config:bool;);
            }
        }
    }
    pub use arg_output::*;

    ImplDefault!{ArgID, ArgData, ArgInit, ArgRandom, ArgNumThreads, ArgOutput}
}

pub use common_arg_trees::{ArgID, ArgData, ArgInit, ArgRandom, ArgNumThreads, ArgOutput};