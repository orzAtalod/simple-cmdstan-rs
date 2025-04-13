pub mod core {
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
    use super::core::*;
    use std::{collections::HashMap, iter::from_fn, fmt::Display, error::Error};

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

        pub fn open_array(mut self, name: &str) -> DataCollectionUncompleted {
            DataCollectionUncompleted {
                entries: self.entires, 
                indexs: self.indexs, 
                uncompleted_array: Vec::new(), 
                uncompleted_data_name: name.to_string()
            }
        }

        //clone the entires
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

        //take ownership of entries which should be a vector to avoid clone
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

        //The function returns n results collected in a n-length vector
        //The vector will be recombined into n iter_n-length array which will be added to the data collection
        //If the function needs to return different type of results, consider changing them to DataEntry first.
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