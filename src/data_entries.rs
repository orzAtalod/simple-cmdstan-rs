pub mod data_entry {
    pub use num::Complex;
    use impl_trait_for_tuples::*;
    #[derive(Debug, Clone, PartialEq)]
    pub enum DataEntry {
        Int(i32),
        Real(f64),
        Complex((f64, f64)),
        Array(Vec<DataEntry>),
        Tuple(Vec<DataEntry>),
    }

    impl Into<DataEntry> for i32 {
        fn into(self) -> DataEntry {
            DataEntry::Int(self)
        }
    }

    impl Into<DataEntry> for f64 {
        fn into(self) -> DataEntry {
            DataEntry::Real(self)
        }
    }

    impl<T> Into<DataEntry> for Complex<T> where T:Into<f64> {
        fn into(self) -> DataEntry {
            DataEntry::Complex((self.re.into(), self.im.into()))
        }
    }
    
    impl<T> Into<DataEntry> for Vec<T> where T:Into<DataEntry> {
        fn into(self) -> DataEntry {
            DataEntry::Array(self.into_iter().map(|x| x.into()).collect())
        }
    }

    #[impl_for_tuples(5)]
    impl Into<DataEntry> for Tuple {
        fn into(self) -> DataEntry {
            let mut res: Vec<DataEntry> = Vec::new();
            for_tuples!( #( res.push(Tuple.into()); )* );
            DataEntry::Tuple(res)
        }
    }

    impl DataEntry {
        pub fn create_from<T:Into<DataEntry>>(item: T) -> DataEntry {
            item.into()
        }

        pub fn create_from_complex(r: f64, i: f64) -> DataEntry {
            DataEntry::Complex((r, i))
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct DataEntries {
        pub datas: Vec<(String, DataEntry)>,
    }

    impl DataEntries {
        pub fn new() -> DataEntries {
            DataEntries { datas: Vec::new() }
        }

        pub fn add_entry<T:Into<DataEntry>>(&mut self, name: &str, entry: T) -> &mut Self {
            self.datas.push((name.to_string(), entry.into()));
            self
        }
    }
}

pub mod data_collections {
    use super::data_entry::*;
    use std::{collections::HashMap, iter::from_fn, fmt::Display};

    #[derive(Debug, Clone)]
    pub enum DataCollectionError {
        AddEntryError(String),
    }

    impl Display for DataCollectionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DataCollectionError::AddEntryError(msg) => write!(f, "AddEntryError: {}", msg),
            }
        }
    }

    impl std::error::Error for DataCollectionError {}

    #[derive(Debug, Clone)]
    pub struct DataCollection {
        pub entires: DataEntries,
        indexs: HashMap<String, usize>,
    }

    #[derive(Debug, Clone)]
    pub struct DataCollectionUncompleted {
        entries: DataEntries,
        indexs: HashMap<String, usize>,
        uncompleted_array: Vec<DataEntry>,
        uncompleted_data_name: String,
    }

    impl DataCollection {
        pub fn new() -> DataCollection {
            DataCollection {
                entires: DataEntries::new(),
                indexs: HashMap::new(),
            }
        }

        pub fn add_entry<T: Into<DataEntry>>(&mut self, name: &str, entry: T) -> &mut Self {
            self.entires.add_entry(name, entry);
            self.indexs.insert(name.to_string(), self.entires.datas.len() - 1);
            self
        }

        pub fn get_entry(&self, name: &str) -> Option<&DataEntry> {
            if let Some(index) = self.indexs.get(name) {
                Some(&self.entires.datas[*index].1)
            } else {
                None
            }
        }

        pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut DataEntry> {
            if let Some(index) = self.indexs.get(name) {
                Some(&mut self.entires.datas[*index].1)
            } else {
                None
            }
        }

        pub fn open_array(self, name: &str) -> DataCollectionUncompleted {
            DataCollectionUncompleted {
                entries: self.entires, 
                indexs: self.indexs, 
                uncompleted_array: Vec::new(), 
                uncompleted_data_name: name.to_string()
            }
        }

        /// used to add a batch of entries.
        /// **Important**: it will clone the entries, so it's not suitable for big sheet.
        /// # example:
        /// ```
        /// let x = DataCollection::new();
        /// x.add_entries(&["N","y"], &[1,2]).unwrap();
        /// let y = DataEntries::new();
        /// y.add_entry("N",1).add_entry("y",2);
        /// assert_eq!(x.entries.write_as_stan_data(), y.write_as_stan_data());
        /// ```
        pub fn add_entries<T:Into<DataEntry>+Clone>(&mut self, name: &[&str], entries: &[T]) -> Result<&mut Self, DataCollectionError> {
            if name.len() != entries.len() {
                return Err(DataCollectionError::AddEntryError(
                    format!("Name and entries length mismatch, Name.len()={}, entries.len()={}",name.len(),entries.len()).to_string()));
            }
            for (i, entry) in entries.into_iter().enumerate() {
                self.add_entry(name[i], entry.clone());
            }
            Ok(self)
        }

        /// take ownership of entries which should be a vector to avoid clone
        pub fn add_entries_and_consume<T:Into<DataEntry>>(&mut self, name: &[&str], entries: Vec<T>) -> Result<&mut Self, DataCollectionError> {
            if name.len() != entries.len() {
                return Err(DataCollectionError::AddEntryError(
                    format!("Name and entries length mismatch, Name.len()={}, entries.len()={}",name.len(),entries.len()).to_string()));
            }
            for (i, entry) in entries.into_iter().enumerate() {
                self.add_entry(name[i], entry);
            }
            Ok(self)
        }

        // grammar sugar for add_entries
        pub fn add_entry_from_func<T,F>(&mut self, name: &str, iter_n: usize, func: F) -> &mut Self
        where
            F: Fn() -> T,
            T: Into<DataEntry>,
        {
            self.add_entry(name, from_fn(|| Some(func().into())).take(iter_n).collect::<Vec<_>>())
        }

        // The function returns n results collected in a n-length vector
        // The vector will be recombined into n iter_n-length array which will be added to the data collection
        // If the function needs to return different type of results, consider changing them to DataEntry first.
        pub fn add_entries_from_func<T,F>(&mut self, name: &[&str], iter_n: usize, func: F) -> Result<&mut Self, DataCollectionError>
        where
            F: Fn() -> Vec<T>,
            T: Into<DataEntry>,
        {
            let n = name.len();
            let mut res = Vec::<Vec::<DataEntry>>::with_capacity(n);
            for _ in 0..n {
                res.push(Vec::with_capacity(iter_n));
            }
            for funcall_res in from_fn(|| Some(func())).take(iter_n) {
                if funcall_res.len() != n {
                    return Err(DataCollectionError::AddEntryError(
                        format!("Name and function returns length mismatch, Name.len()={}, entries.len()={}",name.len(),funcall_res.len()).to_string()));
                }
                for(i, item) in funcall_res.into_iter().enumerate() {
                    res[i].push(item.into());
                }
            }
            self.add_entries_and_consume(name, res)
        }
    }

    impl DataCollectionUncompleted {
        pub fn add_item<T: Into<DataEntry>>(&mut self, entry: T) -> &mut Self {
            self.uncompleted_array.push(entry.into());
            self
        }

        pub fn close_array(mut self) -> DataCollection {
            let new_entry = DataEntry::Array(self.uncompleted_array);
            self.entries.add_entry(&self.uncompleted_data_name, new_entry);
            DataCollection {
                entires: self.entries,
                indexs: self.indexs,
            }
        }
    }
}

mod json_interface {
    use super::data_entry::*;
    use super::data_collections::*;
    use crate::StanData;

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

        pub fn write_to_stan_json(&self, res: &mut String) {
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

    impl StanData for DataCollection {
        fn write_as_stan_data(&self) -> String {
            self.entires.write_as_stan_data()
        }
    }
}

#[cfg(test)]
mod stan_data_test {
    use crate::prelude::*;
    use std::fs::File;
    use std::io::{Write, Error};
    use num::Complex;

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

    #[test]
    fn test_tuple() {
        let mut x = DataEntries::new();
        x.add_entry("N", 2).add_entry("vec", vec![(1,2),(3,4)]);
        dump_stan_json(&x, "D:\\experimental\\tuple.json").unwrap();
        assert_eq!(x.write_as_stan_data(),"{\n    \"N\": 2,\n    \"vec\": [{\"1\": 1, \"2\": 2}, {\"1\": 3, \"2\": 4}]\n}");
    }

    #[test]
    fn test_complex() {
        let mut x = DataEntries::new();
        x.add_entry("N", 2).add_entry("vec", vec![Complex::new(1,2), Complex::new(3,4)]);
        dump_stan_json(&x, "D:\\experimental\\complex.json").unwrap();
        assert_eq!(x.write_as_stan_data(),"{\n    \"N\": 2,\n    \"vec\": [[1, 2], [3, 4]]\n}");
    }

    #[test]
    fn test_tup_dat() {
        let datx = ("N",3);
        dump_stan_json(&datx, "D:\\experimental\\complex.json").unwrap();
        assert_eq!(datx.write_as_stan_data(), "{\n    \"N\": 3\n}");
    }

    #[test]
    fn test_multi_tup_dat() {
        let datvec = ('N', "vec", vec![1,2,3]);
        dump_stan_json(&datvec, "D:\\experimental\\datvec.json").unwrap();
        assert_eq!(datvec.write_as_stan_data(), "{\n    \"N\": 3,\n    \"vec\": [1, 2, 3]\n}");
    }

    fn setup() -> (DataCollection, DataEntries) {
        let mut dc = DataCollection::new();
        dc.add_entries(&["N","y"], &[1,2]).unwrap();
        let mut dd = DataEntries::new();
        dd.add_entry("N", 1).add_entry("y", 2);
        (dc,dd)
    }
    
    #[test]
    fn test_add_entries() {
        let (dc, dd) = setup();
        assert_eq!(dc.entires.write_as_stan_data(), dd.write_as_stan_data());
    }

    #[test]
    fn test_get_entry() {
        let (dc, _) = setup();
        assert_eq!(*dc.get_entry("N").unwrap(), DataEntry::Int(1));
        assert_eq!(*dc.get_entry("y").unwrap(), DataEntry::Int(2));
        assert_eq!(dc.get_entry("vec"), None);
    }

    #[test]
    fn test_get_mut_entry() {
        let (mut dc, _) = setup();
        *dc.get_entry_mut("N").unwrap() = 3.into();
        assert_eq!(*dc.get_entry("N").unwrap(), DataEntry::Int(3));
        assert_eq!(dc.get_entry_mut("vec"), None);
    }

    #[test]
    fn test_open_array() {
        let (dc, mut dd) = setup();
        let mut dc = dc.open_array("vec");
        dc.add_item(1).add_item(3).add_item(2);
        let dc = dc.close_array();
        dd.add_entry("vec", vec![1,3,2]);
        assert_eq!(dc.entires.write_as_stan_data(), dd.write_as_stan_data());
    }

    #[test]
    fn test_add_entry_from_func() {
        let (mut dc, mut dd) = setup();
        dc.add_entry_from_func("var", 3, || 1);
        dd.add_entry("var", vec![1,1,1]);
        assert_eq!(dc.entires.write_as_stan_data(), dd.write_as_stan_data());
    }

    #[test]
    fn test_add_entries_from_func() {
        let (mut dc, mut dd) = setup();
        dc.add_entries_from_func(&["var1","var2","var3"], 3, || vec![1,2,3]).unwrap();
        dd.add_entry("var1", vec![1,1,1]);
        dd.add_entry("var2", vec![2,2,2]);
        dd.add_entry("var3", vec![3,3,3]);
        assert_eq!(dc.entires.write_as_stan_data(), dd.write_as_stan_data());
    }

    #[test]
    #[should_panic]
    fn test_add_entries_from_func_panic() {
        let (mut dc, _) = setup();
        dc.add_entries_from_func(&["var1","var2","var3"], 3, || vec![1,2,3,4]).unwrap();        
    }
}