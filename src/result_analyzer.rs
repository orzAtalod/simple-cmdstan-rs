pub enum ResultAnalyzerError {
    ParameterNotFound,
    NoArgument,
    ParameterSamplesHasDifferentLength,
}

/// standard StanResultAnalyzer implementation for Sample and Optimize.
pub mod stan_result_analyzer {
    use crate::prelude::*;
    use std::{collections::HashMap, io::BufReader, io::BufRead};
    use std::path::Path;
    pub use raw_table::{RawTable, RawTableAnalyzer};
    pub use sample_analyzer::{SampleResult, SampleResultAnalyzer};
    pub use optimize_analyzer::{OptimizeResult, OptimizeResultAnalyzer};

    mod raw_table {
        use super::*;
        pub struct RawTable {
            pub args: Vec<String>,
            pub argv: Vec<Vec<f64>>,
        }

        pub struct RawTableAnalyzer;

        impl StanResultAnalyzer for RawTableAnalyzer {
            type AnalyzeResult = RawTable;
            type Err = StanError;

            fn analyze(&self, _: std::process::Output, out_file: &Path) -> Result<Self::AnalyzeResult, Self::Err> {
                let mut res = RawTable {
                    args: Vec::new(),
                    argv: Vec::new(),
                };
        
                let file = std::fs::File::open(out_file).map_err(|e| StanError::IoError(e))?;
                let buf = BufReader::new(file);
        
                for (i,line) in buf.lines().enumerate() {
                    let line = line.map_err(|e| StanError::IoError(e))?;
                    if line.starts_with("#") {
                        continue;
                    }
                    let parts = line.split(',');
        
                    if res.args.is_empty() {
                        for arg in parts {
                            let arg = arg.trim();
                            res.args.push(arg.to_string());
                            res.argv.push(Vec::new());
                        }
                    } else {
                        for (j,val) in parts.enumerate() {
                            if j >= res.args.len() {
                                panic!("Bad CSV Format: line {} has more columns than header", i);
                            }
                            res.argv[j].push(val.parse().unwrap());
                        }
                    }
                }
        
                Ok(res)
            }
        }
    }

    mod sample_analyzer {
        use super::*;
        
        #[derive(Debug)]
        pub struct SampleResult {
            pub samples: HashMap<String, Vec<f64>>,
            pub length: usize,
        }

        pub struct SampleResultAnalyzer;

        impl StanResultAnalyzer for SampleResultAnalyzer {
            type AnalyzeResult = SampleResult;
            type Err = StanError;

            /// Panic when the csv format is not correct.
            fn analyze(&self, ot: std::process::Output, out_file: &Path) -> Result<Self::AnalyzeResult, Self::Err> {
                let rt = RawTableAnalyzer.analyze(ot, out_file)?;
                let mut res = SampleResult {
                    samples: HashMap::new(),
                    length: rt.argv[0].len(),
                };

                for (name, vec) in rt.args.into_iter().zip(rt.argv.into_iter()) {
                    res.samples.insert(name, vec);
                }

                Ok(res)
            }
        }
    }

    mod optimize_analyzer {
        use super::*;
        #[derive(Debug)]
        pub struct OptimizeResult {
            pub parameters: HashMap<String, f64>,
            pub log_likelihood: f64,
        }
    
        pub struct OptimizeResultAnalyzer;
    
        impl StanResultAnalyzer for OptimizeResultAnalyzer {
            type AnalyzeResult = OptimizeResult;
            type Err = StanError;
    
            fn analyze(&self, ot: std::process::Output, out_file: &Path) -> Result<Self::AnalyzeResult, Self::Err> {
                let rt = RawTableAnalyzer.analyze(ot, out_file)?;

                let mut res = OptimizeResult {
                    parameters: HashMap::new(),
                    log_likelihood: 0.0,
                };

                for (k, v) in rt.args.into_iter().zip(rt.argv.into_iter()) {
                    if v.len() == 0 {
                        panic!("Bad CSV Format: no value for {}", k);
                    }
                    if k == "lp__" {
                        res.log_likelihood = *v.last().unwrap();
                    }
                    res.parameters.insert(k, *v.last().unwrap());
                }

                Ok(res)
            }
        }
    }
}


/// This module provides a way to analyze and manipulate sample results.
/// Use MappedSampleResult to create a new sample result based on the provided sample result.
/// The MappedSampleResult impl Distribution trait, so you can sample from posterior.
mod sample_analyzer {
    use std::collections::HashMap;
    use super::ResultAnalyzerError;
    use rand_distr::Distribution;
    use rand::Rng;
    struct MappedSampleResult {
        pub samples: Vec<f64>,
    }

    impl MappedSampleResult {
        pub fn new<F>(sample_result: HashMap<String, Vec<f64>>, arg_list: Vec<&str>, arg_func: F) -> Result<Self, ResultAnalyzerError> 
        where F: Fn(Vec<f64>) -> f64 {
            if arg_list.is_empty() {
                return Err(ResultAnalyzerError::NoArgument);
            }

            let mut samples = Vec::new();
            for arg in arg_list {
                let sample = sample_result.get(arg).ok_or(ResultAnalyzerError::ParameterNotFound)?;
                samples.push(sample);
            };
            let len = samples[0].len();

            let mut result = Vec::new();
            for i in 0..len {
                let mut params= Vec::new();
                for sample in samples.iter() {
                    if i >= samples.len() {
                        return Err(ResultAnalyzerError::ParameterSamplesHasDifferentLength);
                    }
                    params.push(sample[i]);
                };
                result.push(arg_func(params));
            };

            Ok(MappedSampleResult { samples: result })
        }
    }

    impl Distribution<f64> for MappedSampleResult {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
            let index = rng.random_range(0..self.samples.len());
            self.samples[index]
        }
    }
}