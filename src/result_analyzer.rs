use crate::{arg_paths::ArgReadablePath, error::{CmdStanError, FileError, ParamError}};
use std::{collections::HashMap, io::{BufRead, BufReader}};

pub trait AsResult {
    fn new_line(&mut self);
    fn set_value(&mut self, key: &str, val: &str) -> Result<(), ParamError>;
}

pub fn analyze_csv<T:AsResult>(csv_file: ArgReadablePath, res: &mut T) -> Result<(), CmdStanError> {
    let file = std::fs::File::open(csv_file.as_path()).map_err(|e| CmdStanError::File(FileError::FileSystem(e)))?;
    let buf = BufReader::new(file);
    let mut args = Vec::new();

    for (line_number, line) in buf.lines().enumerate() {
        let line = line.map_err(|e| CmdStanError::File(FileError::FileSystem(e)))?;
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        let parts = line.split(',');

        if args.is_empty() {
            for arg in parts {
                let arg = arg.trim();
                args.push(arg.to_string());
            }
        } else {
            if parts.clone().count() != args.len() {
                return Err(CmdStanError::File(FileError::BadFileFormat(
                    format!("Bad CSV Format: line {} has more or less columns than header", line_number+1), 
                    csv_file.into()))
                );
            }
            res.new_line();
            for (arg,val) in args.iter().zip(parts) {
                res.set_value(arg, val).map_err(CmdStanError::Param)?;
            }
        }
    }

    Ok(())
}

mod param_stream {
    use std::ops::{Deref, DerefMut};
    use crate::stan_model::WithParam;
    use super::*;
    #[derive(Debug, Default, Clone)]
    pub struct ParamStream<T: WithParam+Default>(Vec<T>);

    impl<T: WithParam+Default> Deref for ParamStream<T> {
        type Target = Vec<T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: WithParam+Default> DerefMut for ParamStream<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T: WithParam+Default> From<Vec<T>> for ParamStream<T> {
        fn from(value: Vec<T>) -> Self {
            Self(value)
        }
    }

    impl<T: WithParam+Default> From<ParamStream<T>> for Vec<T> {
        fn from(value: ParamStream<T>) -> Self {
            value.0
        }
    }

    impl<T: WithParam+Default> IntoIterator for ParamStream<T> {
        type Item = T;
        type IntoIter = std::vec::IntoIter<T>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl<'a, T: WithParam+Default> IntoIterator for &'a ParamStream<T> {
        type Item = &'a T;
        type IntoIter = std::slice::Iter<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    impl<'a, T: WithParam+Default> IntoIterator for &'a mut ParamStream<T> {
        type Item = &'a mut T;
        type IntoIter = std::slice::IterMut<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            self.0.iter_mut()
        }
    }

    impl<T: WithParam+Default> FromIterator<T> for ParamStream<T> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            ParamStream(iter.into_iter().collect())
        }
    }

    impl<T: WithParam+Default> Extend<T> for ParamStream<T> {
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            self.0.extend(iter);
        }
    }

    impl<T: WithParam+Default> AsResult for ParamStream<T> {
        fn new_line(&mut self) {
            self.push(T::default());
        }

        fn set_value(&mut self, key: &str, val: &str) -> Result<(), ParamError> {
            if let Some(item) = self.last_mut() {
                if let Err(ParamError::ParseError(e)) = item.set_param_value(key, val) {
                    return Err(ParamError::ParseError(e));
                }
            }
            Ok(())
        }
    }
}
#[allow(unused_imports)]
pub use param_stream::ParamStream;

#[derive(Debug, Clone, Default)]
struct RawTable {
    keys: HashMap<String, usize>,
    values: Vec<Vec<f64>>,
    key_index: usize,  //default: 0
}

impl AsResult for RawTable {
    fn new_line(&mut self) {
        self.values.push(vec![0.0; self.key_index]);
    }

    fn set_value(&mut self, key: &str, val: &str) -> Result<(), ParamError> {
        let id = match self.keys.get(key) {
            Some(id) => *id,
            None => {
                self.keys.insert(key.into(), self.key_index);
                for lines in &mut self.values {
                    lines.push(0.0);
                }
                self.key_index += 1;
                self.key_index - 1
            }
        };

        if let Some(line) = self.values.last_mut() {
            line[id] = val.parse::<f64>()
                          .map_err(|e| ParamError::ParseError(Box::new(e)))?;
        }
        Ok(())
    }
}

impl RawTable {
    pub fn new() -> Self {
        Self::default()
    }
}