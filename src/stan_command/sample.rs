use std::path::{PathBuf, Path};
use super::arg_tree::*;

#[derive(Debug, Clone)]
pub struct ArgSample {
    pub num_samples: u32,    //>0, default: 1000
    pub num_warmup: u32,     //>0, default: 1000
    pub save_warmup: bool,   //default: false
    pub thin: u32,           //>0, default: 1
    pub adapt: ArgSampleAdapt,
    pub algorithm: ArgSampleAlgorithm,
    pub num_chains: u32,     //>0, default: 1
}

#[derive(Debug, Clone)]
pub struct ArgSampleAdapt {
    pub engaged: bool,       //default: true
    pub gamma: f64,          //>0, default: 0.05
    pub delta: f64,          //[0.1,1.0], default:0.8
    pub kappa: f64,          //>0, default:0.75
    pub t0: f64,             //>0, default:10
    pub init_buffer: u32,    //>0, default:75
    pub term_buffer: u32,    //>0, default:50
    pub window: u32,         //>0, default:25
    pub save_metric: bool,   //default: false
}

#[derive(Debug, Clone)]
pub enum ArgSampleAlgorithm {
    Hmc(ArgSampleHmc),  //default
    FixedParam,
}

#[derive(Debug, Clone)]
pub struct ArgSampleHmc {
    pub engine: ArgSampleEngine,
    pub metric: ArgSampleMetric,
    pub metric_file: PathBuf, //input file, default:""
    pub stepsize: f64,        //>0, default:1
    pub stepsize_jitter: f64  //[0,1], default:0
}

#[allow(clippy::approx_constant, reason="the 6.28319 is hard coded in CmdStan")]
const DEFAULT_ENGINE_STATIC_VAL: f64 = 6.28319;
#[derive(Debug, Clone)]
pub enum ArgSampleEngine {
    Static(f64),          //>0, default: 6.28319
    Nuts(u32),            //default, >0, default: 10
}

#[derive(Debug, Clone, Default)]
pub enum ArgSampleMetric {
    UnitE,
    #[default]
    DiagE,  //default
    DenseE,
}

impl Default for ArgSample {
    fn default() -> Self {
        Self {
            num_samples: 1000,
            num_warmup: 1000,
            save_warmup: false,
            thin: 1,
            adapt: ArgSampleAdapt::default(),
            algorithm: ArgSampleAlgorithm::default(),
            num_chains: 1,
        }
    }
}

impl Default for ArgSampleAdapt {
    fn default() -> Self {
        Self { 
            engaged: true,
            gamma: 0.05,
            delta: 0.8,
            kappa: 0.75,
            t0: 10.0,
            init_buffer: 75,
            term_buffer: 50,
            window: 25,
            save_metric: false
        }
    }
}

impl Default for ArgSampleAlgorithm {
    fn default() -> Self {
        ArgSampleAlgorithm::Hmc(ArgSampleHmc::default())
    }
}

impl Default for ArgSampleHmc {
    fn default() -> Self {
        Self {
            engine: ArgSampleEngine::default(),
            metric: ArgSampleMetric::default(),
            metric_file: PathBuf::new(),
            stepsize: 1.0,
            stepsize_jitter: 0.0,
        }
    }
}

impl Default for ArgSampleEngine {
    fn default() -> Self {
        ArgSampleEngine::Nuts(10)
    }
}

impl ArgSampleAdapt {
    fn is_default(&self) -> bool {
        self.engaged                     &&
        (self.gamma-0.05).abs() <= EPS   &&
        (self.delta-0.8).abs() <= EPS    &&
        (self.kappa-0.75).abs() <= EPS   &&
        (self.t0-0.75).abs() <= EPS      &&
        self.init_buffer == 75           &&
        self.term_buffer == 50           &&
        self.window == 25                &&
        !self.save_metric
    }
}

impl ArgSampleAlgorithm {
    fn is_default(&self) -> bool {
        match self {
            Self::FixedParam => false,
            Self::Hmc(hmc) => hmc.is_default()
        }
    }
}

impl ArgSampleHmc {
    fn is_default(&self) -> bool {
        self.engine.is_default()                &&
        self.metric.is_default()                &&
        self.metric_file.as_os_str().is_empty() &&
        (self.stepsize-1.0).abs() <= EPS        &&
        self.stepsize_jitter.abs() <= EPS
    }
}

impl ArgSampleEngine {
    fn is_default(&self) -> bool {
        match self {
            Self::Nuts(x) => *x == 10,
            _ => false,
        }
    }
}

impl ArgSampleMetric {
    fn is_default(&self) -> bool {
        matches!(self, Self::DiagE)
    }
}

impl ArgThrough for ArgSample {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::Sample)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("sample");
        if self.num_samples != 1000 {
            cmd.arg(format!("num_samples={}", self.num_samples));
        }
        if self.num_warmup != 1000 {
            cmd.arg(format!("num_warmup={}", self.num_warmup));
        }
        if self.save_warmup {
            cmd.arg("save_warmup=true");
        }
        if self.thin != 1 {
            cmd.arg(format!("thin={}",self.thin));
        }
        if !self.adapt.is_default() {
            cmd.arg("adapt");
            if !self.adapt.engaged {
                cmd.arg("engaged=false");
            }
            if (self.adapt.gamma-0.05).abs() > EPS {
                cmd.arg(format!("gamma={}",self.adapt.gamma));
            }
            if (self.adapt.delta-0.8).abs() > EPS {
                cmd.arg(format!("delta={}",self.adapt.delta));
            }
            if (self.adapt.kappa-0.75).abs() > EPS {
                cmd.arg(format!("kappa={}",self.adapt.kappa));
            }
            if (self.adapt.t0-10.0).abs() > EPS {
                cmd.arg(format!("t0={}",self.adapt.t0));
            }
            if self.adapt.init_buffer != 75 {
                cmd.arg(format!("init_buffer={}",self.adapt.init_buffer));
            }
            if self.adapt.term_buffer != 50 {
                cmd.arg(format!("term_buffer={}",self.adapt.term_buffer));
            }
            if self.adapt.window != 25 {
                cmd.arg(format!("window={}",self.adapt.window));
            }
            if self.adapt.save_metric {
                cmd.arg("save_metric=true");
            }
        }
        if !self.algorithm.is_default() {
            match &self.algorithm {
                ArgSampleAlgorithm::FixedParam => {
                    cmd.arg("algorithm=fixed_param");
                }
                ArgSampleAlgorithm::Hmc(hmc) => {
                    cmd.arg("algorithm=hmc");
                    match hmc.engine {
                        ArgSampleEngine::Static(x) => {
                            cmd.arg("engine=static");
                            if (x-DEFAULT_ENGINE_STATIC_VAL).abs() > EPS {
                                cmd.arg(format!("int_time={x}"));
                            }
                        }
                        ArgSampleEngine::Nuts(x) => {
                            if x != 10 {
                                cmd.arg("engine=nuts");
                                cmd.arg(format!("max_depth={x}"));
                            }
                        }
                    }
                    match hmc.metric {
                        ArgSampleMetric::DenseE => {
                            cmd.arg("metric=dense_e");
                        }
                        ArgSampleMetric::UnitE => {
                            cmd.arg("metric=unit_e");
                        }
                        _ => {}
                    }
                    if !hmc.metric_file.as_os_str().is_empty() {
                        cmd.arg(args_combine("metric_file", hmc.metric_file.as_os_str()));
                    }
                    if (hmc.stepsize-1.0).abs() > EPS {
                        cmd.arg(format!("stepsize={}",hmc.stepsize));
                    }
                    if hmc.stepsize_jitter.abs() > EPS {
                        cmd.arg(format!("stepsize_jitter={}",hmc.stepsize_jitter));
                    }
                }
            }
        }
        if self.num_chains != 1 {
            cmd.arg(format!("num_chains={}",self.num_chains));
        }
        Ok(())
    }
}

/*
    num_samples: u32,    //>0, default: 1000
    num_warmup: u32,     //>0, default: 1000
    save_warmup: bool,   //default: false
    thin: u32,           //>0, default: 1
    pub adapt: ArgSampleAdapt,
    pub algorithm: ArgSampleAlgorithm,
    num_chains: u32,     //>0, default: 1
*/
impl ArgSample {
    pub fn new() -> ArgSample {
        Self::default()
    }

    pub fn set_num_samples(&mut self, num_sample: u32) -> Result<&mut Self, ArgError> {
        if num_sample == 0 {
            Err(ArgError::BadArgumentValue("Sample: num of samples could not be 0".to_string()))
        } else {
            self.num_samples = num_sample;
            Ok(self)
        }
    }

    pub fn set_num_warmup(&mut self, num_warmup: u32) -> Result<&mut Self, ArgError> {
        if num_warmup == 0 {
            Err(ArgError::BadArgumentValue("Sample: num of warmup could not be 0".to_string()))
        } else {
            self.num_warmup = num_warmup;
            Ok(self)
        }
    }

    pub fn set_num_chains(&mut self, num_chains: u32) -> Result<&mut Self, ArgError> {
        if num_chains == 0 {
            Err(ArgError::BadArgumentValue("Sample: num of chains could not be 0".to_string()))
        } else {
            self.num_chains = num_chains;
            Ok(self)
        }
    }

    pub fn set_save_warmup(&mut self, save_warmup: bool) -> &mut Self {
        self.save_warmup = save_warmup;
        self
    }

    pub fn set_thin(&mut self, thin: u32) -> Result<&mut Self, ArgError> {
        if thin == 0 {
            Err(ArgError::BadArgumentValue("Sample: period between saved samples could not be 0".to_string()))
        } else {
            self.thin = thin;
            Ok(self)
        }
    }

    pub fn set_adapt(&mut self, adapt: ArgSampleAdapt) -> &mut Self {
        self.adapt = adapt;
        self
    }

    pub fn set_adapt_clone(&mut self, adapt: &ArgSampleAdapt) -> &mut Self {
        self.adapt = adapt.clone();
        self
    }

    pub fn set_algorithm(&mut self, algo: ArgSampleAlgorithm) -> &mut Self {
        self.algorithm = algo;
        self
    }

    pub fn set_algorithm_clone(&mut self, algo: &ArgSampleAlgorithm) -> &mut Self {
        self.algorithm = algo.clone();
        self
    }
}

/*
    pub engaged: bool,       //default: true
    pub gamma: f64,          //>0, default: 0.05
    pub delta: f64,          //[0.1,1.0], default:0.8
    pub kappa: f64,          //>0, default:0.75
    pub t0: f64,             //>0, default:10
    pub init_buffer: u32,    //>0, default:75
    pub term_buffer: u32,    //>0, default:50
    pub window: u32,         //>0, default:25
    pub save_metric: bool,   //default: false
*/
impl ArgSampleAdapt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_engaged(&mut self, engaged: bool) -> &mut Self {
        self.engaged = engaged;
        self
    }

    pub fn set_gamma(&mut self, gamma: f64) -> Result<&mut Self, ArgError> {
        if gamma>0.0 {
            self.gamma = gamma;
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_gamma expected gamma>0, found {}", gamma).to_string()))
        }
    }

    pub fn set_delta(&mut self, delta: f64) -> Result<&mut Self, ArgError> {
        if (0.1..=1.0).contains(&delta) {
            self.delta = delta;
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_delta expected delta in [0.1,1.0], found {}", delta).to_string()))
        }
    }

    pub fn set_kappa(&mut self, kappa: f64) -> Result<&mut Self, ArgError> {
        if kappa>0.0 {
            self.kappa = kappa;
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_kappa expected kappa>0, found {}", kappa).to_string()))
        }
    }

    pub fn set_t0(&mut self, t0: f64) -> Result<&mut Self, ArgError> {
        if t0>0.0 {
            self.t0 = t0;
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_t0 expected t0>0, found {}", t0).to_string()))
        }
    }

    pub fn set_init_buffer(&mut self, buffer: u32) -> Result<&mut Self, ArgError> {
        if buffer==0 {
            Err(ArgError::BadArgumentValue("Sample: init buffer could not be 0".to_string()))
        } else {
            self.init_buffer = buffer;
            Ok(self)
        }
    }

    pub fn set_term_buffer(&mut self, buffer: u32) -> Result<&mut Self, ArgError> {
        if buffer==0 {
            Err(ArgError::BadArgumentValue("Sample: term buffer could not be 0".to_string()))
        } else {
            self.term_buffer = buffer;
            Ok(self)
        }
    }

    pub fn set_window(&mut self, window: u32) -> Result<&mut Self, ArgError> {
        if window==0 {
            Err(ArgError::BadArgumentValue("Sample: window length could not be 0".to_string()))
        } else {
            self.window = window;
            Ok(self)
        }
    }

    pub fn set_save_metric(&mut self, save_metric: bool) -> &mut Self {
        self.save_metric = save_metric;
        self
    }
}

/*
    Hmc(ArgSampleHmc),  //default
    FixedParam,
*/
impl ArgSampleAlgorithm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_to_fixed_param(&mut self) -> &mut Self {
        *self = Self::FixedParam;
        self
    }

    pub fn set_hmc(&mut self, hmc: ArgSampleHmc) -> &mut Self {
        *self = Self::Hmc(hmc);
        self
    }

    pub fn set_hmc_clone(&mut self, hmc: &ArgSampleHmc) -> &mut Self {
        *self = Self::Hmc(hmc.clone());
        self
    }

    pub fn get_mut_hmc(&mut self) -> Option<&mut ArgSampleHmc> {
        match self {
            Self::Hmc(hmc) => Some(hmc),
            _ => None,
        }
    }

    pub fn get_hmc(&self) -> Option<&ArgSampleHmc> {
        match self {
            Self::Hmc(hmc) => Some(hmc),
            _ => None,
        }
    }
}


/*
    pub engine: ArgSampleEngine,
    pub metric: ArgSampleMetric,
    pub metric_file: PathBuf, //input file, default:""
    pub stepsize: f64,        //>0, default:1
    pub stepsize_jitter: f64  //[0,1], default:0
*/
impl ArgSampleHmc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_engine(&mut self, engine: ArgSampleEngine) -> &mut Self {
        self.engine = engine;
        self
    }

    pub fn set_engine_clone(&mut self, engine: &ArgSampleEngine) -> &mut Self {
        self.engine = engine.clone();
        self
    }

    pub fn set_metric(&mut self, metric: ArgSampleMetric) -> &mut Self {
        self.metric = metric;
        self
    }

    pub fn set_metric_clone(&mut self, metric: &ArgSampleMetric) -> &mut Self {
        self.metric = metric.clone();
        self
    }

    pub fn set_metric_unit_e(&mut self) -> &mut Self {
        self.metric = ArgSampleMetric::UnitE;
        self
    }

    pub fn set_metric_diag_e(&mut self) -> &mut Self {
        self.metric = ArgSampleMetric::DiagE;
        self
    }

    pub fn set_metric_dense_e(&mut self) -> &mut Self {
        self.metric = ArgSampleMetric::DenseE;
        self
    }

    pub fn set_metric_file(&mut self, file: &Path) -> Result<&mut Self, ArgError> {
        verify_file_readable(file)?;
        self.metric_file = file.to_path_buf();
        Ok(self)
    }

    pub fn set_stepsize(&mut self, stepsize: f64) -> Result<&mut Self, ArgError> {
        if stepsize > 0.0 {
            self.stepsize = stepsize;
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_stepsize expected stepsize>0, found {}",stepsize).to_string()))
        }
    }

    pub fn set_stepsize_jitter(&mut self, stepsize_jitter: f64) -> Result<&mut Self, ArgError> {
        if (0.0..=1.0).contains(&stepsize_jitter) {
            self.stepsize_jitter = stepsize_jitter;
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_stepsize_jitter expected jitter in [0,1], found {}",stepsize_jitter).to_string()))
        }
    }
}

/*
    Static(f64),          //>0, default: 6.28319
    Nuts(u32),            //default, >0, default: 10
*/
impl ArgSampleEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_static_engine(&mut self) -> &mut Self {
        *self = Self::Static(DEFAULT_ENGINE_STATIC_VAL);
        self
    }

    pub fn set_static_int_time(&mut self, int_time: f64) -> Result<&mut Self, ArgError> {
        if int_time > 0.0 {
            *self = Self::Static(int_time);
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_static_int_time expected int_time>0, found {}",int_time).to_string()))
        }
    }

    pub fn set_nuts_engine(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }

    pub fn set_nuts_max_depth(&mut self, max_depth: u32) -> Result<&mut Self, ArgError> {
        if max_depth == 0 {
            Err(ArgError::BadArgumentValue("sample: max_depth of set_nuts_max_depth could not be 0".to_string()))
        } else {
            *self = Self::Nuts(max_depth);
            Ok(self)
        }
    }

    pub fn get_int_time(&self) -> Option<f64> {
        match self {
            Self::Static(x) => Some(*x),
            _ => None,
        }
    }

    pub fn get_max_depth(&self) -> Option<u32> {
        match self {
            Self::Nuts(x) => Some(*x),
            _ => None,
        }
    }
}

/*    
    UnitE,
    DiagE,  //default
    DenseE, 
*/
impl ArgSampleMetric {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_unit_e(&mut self) -> &mut Self {
        *self = Self::UnitE;
        self
    }

    pub fn set_diag_e(&mut self) -> &mut Self {
        *self = Self::DiagE;
        self
    }

    pub fn set_dense_e(&mut self) -> &mut Self {
        *self = Self::DenseE;
        self
    }
}