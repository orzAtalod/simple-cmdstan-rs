mod data_entries;
mod result_analyzer;
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
            if self.is_empty_array() {
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
                        if i != 0 {
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
                        if i != 0 {
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
                if i != 0 {
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

    /// impl StanData trait for every tuple (&str, T)
    /// the tuple will equal to { "{str}": T }
    /// # Examples
    /// ```
    /// let test = ("val", 5);
    /// assert_eq!(test.write_as_stan_data(), "{\n    \"val\": 5\n}");
    /// ```
    impl<T:Into<DataEntry>+Clone> StanData for (&str,T) {
        fn write_as_stan_data(&self) -> String {
            let mut result = "{\n".to_string();
            result.push_str(format!("    \"{}\": ",self.0).as_str());
            self.1.clone().into().write_to_stan_json(&mut result);
            result.push_str("\n}");
            result
        }
    }

    /// impl StanData trait for every tuple (char, &str, Vec<T>)
    /// char: the name of the vector size (size_name) (usually N)
    /// &str: the name of the vector (vec_name)
    /// Vec<T>: the vector of data entries
    /// translate to { "{size_name}": vec.len(), "{vec_name}": [vec] }
    impl<T:Into<DataEntry>+Clone> StanData for (char,&str,Vec<T>) {
        fn write_as_stan_data(&self) -> String {
            let mut result = "{\n".to_string();
            result.push_str(format!("    \"{}\": {},\n    \"{}\": [", self.0, self.2.len(), self.1).as_str());
            for (i,item) in self.2.iter().enumerate() {
                if i != 0 {
                    result.push_str(", ");
                }
                item.clone().into().write_to_stan_json(&mut result);
            }
            result.push_str("]\n}");
            result
        }
    }
}

mod stan_interface {
    use std::env::set_current_dir;
    use std::path::Path;
    pub fn stan_init(stan_home_path: &Path) -> Result<(), std::io::Error> {
        if !std::env::current_dir()?.ends_with("cmdstan") {
            set_current_dir(stan_home_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod stan_data_test {
    use crate::data_entries::core::*;
    use crate::stan_model::StanData;
    use std::fs::File;
    use std::io::{Write, Error};

    fn dump_stan_json<T:StanData>(data: &T, path: &str) -> Result<(), Error> {
        let mut output = File::create(path)?;
        write!(output, "{}", data.write_as_stan_data())?;
        Ok(())
    }

    #[test]
    fn base_test() {
        let mut x = DataEntries::new();
        x.add_entry("N", 5).add_entry("vec", vec![2,3,2,4,2]);
        dump_stan_json(&x, "D:\\experimental\\base_test.json").unwrap();
        assert_eq!(x.write_as_stan_data(),"{\n    \"N\": 5,\n    \"vec\": [2, 3, 2, 4, 2]\n}");
    }

    #[test]
    fn nested_array() {
        let mut x = DataEntries::new();
        x.add_entry("N", 2)
            .add_entry("M", 2)
            .add_entry("vec", vec![vec![1,2], vec![3,4]]);
        dump_stan_json(&x, "D:\\experimental\\nested_array.json").unwrap();
        assert_eq!(x.write_as_stan_data(),"{\n    \"N\": 2,\n    \"M\": 2,\n    \"vec\": [[1, 2], [3, 4]]\n}");
    }

    #[test]
    fn nested_empty_array() {
        let mut x = DataEntries::new();
        x.add_entry("N", 0).add_entry("M", 0).add_entry::<Vec<Vec<i32>>>("vec", vec![vec![]]);
        dump_stan_json(&x, "D:\\experimental\\nested_empty_array.json").unwrap();
        assert_eq!(x.write_as_stan_data(),"{\n    \"N\": 0,\n    \"M\": 0,\n    \"vec\": []\n}");
    }
}