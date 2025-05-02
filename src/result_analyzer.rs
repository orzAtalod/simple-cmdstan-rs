use crate::{arg_paths::ArgReadablePath, stan_model::{CmdStanError, FileError, WithParam}};
use std::{io::{BufReader, BufRead}, collections::HashMap};

pub trait AsResult {
    fn new_line(&mut self) -> &mut Self;
    fn set_value(&mut self, key: &str, val: &str) -> Result<&mut Self, Box::<dyn std::error::Error>>;
}

pub fn analyze_csv<T:AsResult>(csv_file: ArgReadablePath, res: &mut T) -> Result<(), Box::<dyn std::error::Error>> {
    let file = std::fs::File::open(csv_file.as_path()).map_err(|e| CmdStanError::File(FileError::FileSystem(e)))?;
    let buf = BufReader::new(file);
    let mut args = Vec::new();

    for (i,line) in buf.lines().enumerate() {
        let line = line.map_err(|e| CmdStanError::File(FileError::FileSystem(e)))?;
        if line.starts_with("#") {
            continue;
        }
        let parts = line.split(',');

        if args.is_empty() {
            for arg in parts {
                let arg = arg.trim();
                args.push(arg.to_string());
            }
        } else {
            res.new_line();
            for (j,val) in parts.enumerate() {
                if j >= args.len() {
                    panic!("Bad CSV Format: line {} has more columns than header", i);
                }
                res.set_value(&args[j], val)?;
            }
        }
    }

    Ok(())
}

impl<T: WithParam+Default> AsResult for Vec<T> {
    fn new_line(&mut self) -> &mut Self {
        self.push(T::default());
        self
    }

    fn set_value(&mut self, key: &str, val: &str) -> Result<&mut Self, Box::<dyn std::error::Error>> {
        if let Some(item) = self.last_mut() {
            let _ = item.set_param_value(key, val);
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Default)]
struct RawTable {
    keys: HashMap<String, usize>,
    values: Vec<Vec<f64>>,
    key_index: usize,  //default: 0
}

impl AsResult for RawTable {
    fn new_line(&mut self) -> &mut Self {
        self.values.push(vec![0.0; self.key_index+1]);
        self
    }

    fn set_value(&mut self, key: &str, val: &str) -> Result<&mut Self, Box::<dyn std::error::Error>> {
        if let Some(id) = self.keys.get(key) {
            if let Some(line) = self.values.last_mut() {
                line[*id] = val.parse::<f64>()?;
            }
        } else {
            self.keys.insert(key.into(), self.key_index);
            self.key_index += 1;
            if let Some(line) = self.values.last_mut() {
                line.push(val.parse::<f64>()?);
            }
        }
        Ok(self)
    }
}

impl RawTable {
    pub fn new() -> Self {
        Self::default()
    }
}