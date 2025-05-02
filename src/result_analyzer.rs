use crate::{arg_paths::ArgReadablePath, stan_model::{CmdStanError, FileError, ParamError, WithParam}};
use std::{io::{BufReader, BufRead}, collections::HashMap};

pub trait AsResult {
    fn new_line(&mut self);
    fn set_value(&mut self, key: &str, val: &str) -> Result<(), ParamError>;
}

pub fn analyze_csv<T:AsResult>(csv_file: ArgReadablePath, res: &mut T) -> Result<(), CmdStanError> {
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
                    return Err(CmdStanError::File(FileError::BadFileFormat(
                        format!("Bad CSV Format: line {i} has more column than header"), 
                        csv_file.into()))
                    );
                }
                res.set_value(&args[j], val).map_err(CmdStanError::Param)?;
            }
        }
    }

    Ok(())
}

impl<T: WithParam+Default> AsResult for Vec<T> {
    fn new_line(&mut self) {
        self.push(T::default());
    }

    fn set_value(&mut self, key: &str, val: &str) -> Result<(), ParamError> {
        if let Some(item) = self.last_mut() {
            let _ = item.set_param_value(key, val);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct RawTable {
    keys: HashMap<String, usize>,
    values: Vec<Vec<f64>>,
    key_index: usize,  //default: 0
}

impl AsResult for RawTable {
    fn new_line(&mut self) {
        self.values.push(vec![0.0; self.key_index+1]);
    }

    fn set_value(&mut self, key: &str, val: &str) -> Result<(), ParamError> {
        if let Some(id) = self.keys.get(key) {
            if let Some(line) = self.values.last_mut() {
                line[*id] = val.parse::<f64>().map_err(|e| ParamError::ParseError(Box::new(e)))?;
            }
        } else {
            self.keys.insert(key.into(), self.key_index);
            self.key_index += 1;
            if let Some(line) = self.values.last_mut() {
                line.push(val.parse::<f64>().map_err(|e| ParamError::ParseError(Box::new(e)))?);
            }
        }
        Ok(())
    }
}

impl RawTable {
    pub fn new() -> Self {
        Self::default()
    }
}