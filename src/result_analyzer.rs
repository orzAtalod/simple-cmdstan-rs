enum ResultAnalyzerError {
    ParameterNotFound,
    NoArgument,
    ParameterSamplesHasDifferentLength,
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