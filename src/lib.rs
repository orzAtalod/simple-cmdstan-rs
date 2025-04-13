mod data_entries;
mod stan_model;

mod json_interface {
    use crate::data_entries::core::*;
    use crate::stan_model::StanData;
    use std::fs::File;
    use std::io::{Write, Error};

    impl DataEntry {
        fn is_empty_array(&self) -> bool {
            match self {
                DataEntry::Array(arr) => {
                    match arr.len() {
                        0 => true,
                        1 => arr[0].is_empty_array(),
                        _ => false,
                    }
                },
                _ => false,
            }
        }

        fn write_to_stan_json(&self, res: &mut String) {
            if(self.is_empty_array()) {
                res.push_str("[]"); // flat every [[[]]]-like structure to [] as documented in CmdStan website.
                return;
            }
            match self {
                DataEntry::Int(i) => res.push_str(&i.to_string()),
                DataEntry::Real(r) => res.push_str(&r.to_string()),
                DataEntry::Complex((r, i)) => {
                    res.push_str(&format!("[{}, {}]", r, i));
                }
                DataEntry::Array(arr) => {
                    res.push('[');
                    for (i, item) in arr.iter().enumerate() {
                        if i > 0 {
                            res.push(',');
                            res.push(' ');
                        }
                        item.write_to_stan_json(res);
                    }
                    res.push(']');
                }
                DataEntry::Tuple(tup) => {
                    res.push('{');
                    for (i, item) in tup.iter().enumerate() {
                        if i > 0 {
                            res.push(',');
                            res.push(' ');
                        }
                        res.push_str(&format!("\"{}\": ", i+1));
                        item.write_to_stan_json(res);
                    }
                    res.push('}');
                }
            }
        }
    }

    impl StanData for DataEntries {
        fn write_as_stan_data(&self) -> String {
            let mut result = "{\n".to_string();
            for (i, (name, entry)) in self.datas.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                    result.push('\n');
                }
                result.push_str("    ");
                result.push_str(&format!("\"{}\": ", name));
                entry.write_to_stan_json(&mut result);
            }
            result.push_str("\n}");
            result
        }
    }

    impl<T:Into<DataEntry>+Clone> StanData for (&str,T) {
        fn write_as_stan_data(&self) -> String {
            let mut result = "{\n".to_string();
            result.push_str(format!("    \"{}\": ",self.0).as_str());
            self.1.clone().into().write_to_stan_json(&mut result);
            result.push_str("\n}");
            result
        }
    }

    /// char: the name of the vector size (size_name) (usually N)
    /// &str: the name of the vector
    /// Vec<T>: the vector of data entries
    /// translate to { "{size_name}": vec.len(), "{name}": [vec] }
    impl<T:Into<DataEntry>+Clone> StanData for (char,&str,Vec<T>) {
        fn write_as_stan_data(&self) -> String {
            let mut result = "{\n".to_string();
            result.push_str(format!("    \"{}\": {},\n    \"{}\": [", self.0, self.2.len(), self.1).as_str());
            for (i,item) in self.2.iter().enumerate() {
                if(i != 0) {
                    result.push_str(", ");
                }
                item.clone().into().write_to_stan_json(&mut result);
            }
            result.push_str("]\n}");
            result
        }
    }

    pub fn dump_stan_json<T:StanData>(data: &T, path: &str) -> Result<(), Error> {
        let mut output = File::create(path)?;
        write!(output, "{}", data.write_as_stan_data())?;
        Ok(())
    }
}

mod stan_interface {
    use std::process::Command;
    pub fn stan_init(stan_home_path: &str) -> Result<(), std::io::Error> {
        Command::new("cd").arg(stan_home_path).status()?;
        Ok(())
    }
}