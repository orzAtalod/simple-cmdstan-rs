/// a standard StanModel implementation.
pub mod std_stan_model {
    use std::fs::File;
    use crate::prelude::*;
    use std::process::Command;
    use std::path::{Path,PathBuf,absolute};
    use std::io::prelude::*;
    use std::env::consts::OS;
    use crate::stan_interface::STAN_HOME_KEY;

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
                let mut file = File::create(self.data_path.clone().unwrap()).map_err(StanError::IoError)?;
                let content = data.write_as_stan_data();
                file.write_all( content.as_bytes()).map_err(StanError::IoError)?;
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
            
            let absolute_excutable = absolute(self.get_excutable_name()).map_err( StanError::IoError)?;
            let command = Command::new("make")
                .current_dir(std::env::var(STAN_HOME_KEY).unwrap())
                .arg(absolute_excutable)
                .status().map_err(StanError::CompileIOError)?;

            if !command.success() {
                Err(StanError::CompileError("Compile failed".to_string()))
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

#[cfg(test)]
mod stan_model_test {
    use crate::prelude::*;
    use std::path::Path;
    const PATHS: [&str;3] = [".conda\\Library\\bin\\cmdstan", "examples\\bernoulli\\", "bernoulli.stan"];
    
    #[test]
    fn test_init() {
        stan_init(Path::new(PATHS[0])).unwrap();
    }

    #[test]
    fn test_complie() {
        stan_init(Path::new(PATHS[0])).unwrap();
        let mut stm = StdStanModel::<DataEntries>::new(Path::new(PATHS[1]),Path::new("bernoulli2.stan"));
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
}