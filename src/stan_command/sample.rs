use std::path::{PathBuf, Path};
use super::arg_tree::*;

/*
sample
  Bayesian inference with Markov Chain Monte Carlo
  Valid subarguments: num_samples, num_warmup, save_warmup, thin, adapt, algorithm, num_chains

  num_samples=<int>
    Number of sampling iterations
    Valid values: 0 < num_samples
    Defaults to 1000

  num_warmup=<int>
    Number of warmup iterations
    Valid values: 0 < num_warmup
    Defaults to 1000

  save_warmup=<boolean>
    Stream warmup samples to output?
    Valid values: [0, 1, false, true]
    Defaults to false

  thin=<int>
    Period between saved samples
    Valid values: 0 < thin
    Defaults to 1

  adapt
    Warmup Adaptation
    Valid subarguments: engaged, gamma, delta, kappa, t0, init_buffer, term_buffer, window, save_metric

    engaged=<boolean>
      Adaptation engaged?
      Valid values: [0, 1, false, true]
      Defaults to true

    gamma=<double>
      Adaptation regularization scale
      Valid values: 0 < gamma
      Defaults to 0.05

    delta=<double>
      Adaptation target acceptance statistic
      Valid values: 0.100000 <= delta <= 1.000000
      Defaults to 0.8

    kappa=<double>
      Adaptation relaxation exponent
      Valid values: 0 < kappa
      Defaults to 0.75

    t0=<double>
      Adaptation iteration offset
      Valid values: 0 < t0
      Defaults to 10

    init_buffer=<unsigned int>
      Width of initial fast adaptation interval
      Valid values: 0 < init_buffer
      Defaults to 75

    term_buffer=<unsigned int>
      Width of final fast adaptation interval
      Valid values: 0 < term_buffer
      Defaults to 50

    window=<unsigned int>
      Initial width of slow adaptation interval
      Valid values: 0 < window
      Defaults to 25

    save_metric=<boolean>
      Save metric as JSON?
      Valid values: [0, 1, false, true]
      Defaults to false

  algorithm=<list element>
    Sampling algorithm
    Valid values: hmc, fixed_param
    Defaults to hmc

    hmc
      Hamiltonian Monte Carlo
      Valid subarguments: engine, metric, metric_file, stepsize, stepsize_jitter

      engine=<list element>
        Engine for Hamiltonian Monte Carlo
        Valid values: static, nuts
        Defaults to nuts

        static
          Static integration time
          Valid subarguments: int_time

          int_time=<double>
            Total integration time for Hamiltonian evolution, default is 2 * pi
            Valid values: 0 < int_time
            Defaults to 6.28319

        nuts
          The No-U-Turn Sampler
          Valid subarguments: max_depth

          max_depth=<int>
            Maximum tree depth
            Valid values: 0 < max_depth
            Defaults to 10

      metric=<list element>
        Geometry of base manifold
        Valid values: unit_e, diag_e, dense_e
        Defaults to diag_e

        unit_e
          Euclidean manifold with unit metric

        diag_e
          Euclidean manifold with diag metric

        dense_e
          Euclidean manifold with dense metric

      metric_file=<string>
        Input file with precomputed Euclidean metric
        Valid values: All
        Defaults to

      stepsize=<double>
        Step size for discrete evolution
        Valid values: 0 < stepsize
        Defaults to 1

      stepsize_jitter=<double>
        Uniformly random jitter of the stepsize, in percent
        Valid values: 0.000000 <= stepsize_jitter <= 1.000000
        Defaults to 0

    fixed_param
      Fixed Parameter Sampler

  num_chains=<int>
    Number of chains
    Valid values: 0 < num_chains
    Defaults to 1
*/

DefArgTree!{<"Bayesian inference with Markov Chain Monte Carlo"> ArgSample => {
    <"Number of sampling iterations">num_samples: u32 = 1000,    //>0, default: 1000
    <"Number of warmup iterations">num_warmup: u32 = 1000,     //>0, default: 1000
    <"Stream warmup samples to output?">save_warmup: bool = false,   //default: false
    <"Period between saved samples">thin: u32 = 1,           //>0, default: 1
    <"Warmup Adaptation">adapt: ArgSampleAdapt = ArgSampleAdapt::ARG_DEFAULT, 
    <"Sampling algorithm">algorithm: ArgSampleAlgorithm = ArgSampleAlgorithm::ARG_DEFAULT,
    <"Number of chains">num_chains: u32 = 1,     //>0, default: 1
}}

DefArgTree!{<"Warmup Adaptation">ArgSampleAdapt => {
    <"Adaptation engaged?">engaged: bool = true,       //default: true
    <"Adaptation regularization scale">gamma: f64 = 0.05,          //>0, default: 0.05
    <"Adaptation target acceptance statistic">delta: f64 = 0.8,          //[0.1,1.0], default:0.8
    <"Adaptation relaxation exponent">kappa: f64 = 0.75,          //>0, default:0.75
    <"Adaptation iteration offset">t0: f64 = 10.0,             //>0, default:10
    <"Width of initial fast adaptation interval">init_buffer: u32 = 75,    //>0, default:75
    <"Width of final fast adaptation interval">term_buffer: u32 = 50,    //>0, default:50
    <"Initial width of slow adaptation interval">window: u32 = 25,         //>0, default:25
    <"Save metric as JSON?">save_metric: bool = false,
}}

DefArgTree!{<"Sampling algorithm">ArgSampleAlgorithm = Self::Hmc(ArgSampleHmc::ARG_DEFAULT) => {
    <"Hamiltonian Monte Carlo">Hmc(ArgSampleHmc),
    <"Fixed Parameter Sampler">FixedParam,
}}

DefArgTree!{<"Hamiltonian Monte Carlo">ArgSampleHmc => {
    <"Engine for Hamiltonian Monte Carlo">engine: ArgSampleEngine = ArgSampleEngine::ARG_DEFAULT, //default
    <"Geometry of base manifold">metric: ArgSampleMetric = ArgSampleMetric::DiagE, //default
    <"Input file with precomputed Euclidean metric">metric_file: ArgReadablePath = ArgReadablePath::ARG_DEFAULT, //input file, default:""
    <"Step size for discrete evolution">stepsize: f64 = 1.0,        //>0, default:1
    <"Uniformly random jitter of the stepsize, in percent">stepsize_jitter: f64 = 0.0,  //[0,1], default:0
}}

#[allow(clippy::approx_constant, reason="the 6.28319 is hard coded in CmdStan")]
const DEFAULT_ENGINE_STATIC_VAL: f64 = 6.28319;
DefArgTree!{<"Engine for Hamiltonian Monte Carlo">ArgSampleEngine = Self::Nuts(10) => {
    <"Static integration time">Static(f64),          //>0, default: 6.28319
    <"The No-U-Turn Sampler">Nuts(u32),            //default, >0, default: 10
}}

DefArgTree!{<"Geometry of base manifold">ArgSampleMetric = Self::DiagE => {
    <"Euclidean manifold with unit metric">UnitE,
    <"Euclidean manifold with diag metric">DiagE, 
    <"Euclidean manifold with dense metric">DenseE,
}}

ImplDefault!(ArgSample, ArgSampleAdapt, ArgSampleAlgorithm, ArgSampleHmc, ArgSampleEngine, ArgSampleMetric);

impl ArgThrough for ArgSample {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::Sample)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("sample");
        arg_into!(self.{num_samples, num_warmup, save_warmup, thin, num_chains} in ArgSample >> cmd);

        if !self.adapt.is_default() {
            cmd.arg("adapt");
            let adpt = &self.adapt;
            arg_into!(adpt.{engaged, gamma, delta, kappa, t0, init_buffer, term_buffer, window, save_metric} in ArgSampleAdapt >> cmd);
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
                    arg_into!(hmc.{metric_file, stepsize, stepsize_jitter} in ArgSampleHmc >> cmd);
                }
            }
        }
        Ok(())
    }
}

impl ArgSample {
    pub fn new() -> ArgSample {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Number of sampling iterations">(num_samples:u32; num_samples==0 => "Sample: num_samples could not be 0".to_string());
        <"Number of warmup iterations">(num_warmup: u32; num_warmup==0 => "Sample: num_warmup could not be 0".to_string());
        <"Stream warmup samples to output?">(save_warmup: bool;);
        <"Period between saved samples">(thin: u32; thin==0 => "Sample: period between saved samples could not be 0".to_string());
        <"Warmup Adaptation">(adapt: ArgSampleAdapt;); 
        <"Sampling algorithm">(algorithm: ArgSampleAlgorithm;);
        <"Number of chains">(num_chains: u32; num_chains==0 => "Sample: num of chains could not be 0".to_string());
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
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Adaptation engaged?">(engaged: bool;);
        <"Adaptation regularization scale">(gamma: f64; gamma<=0.0 => format!("Sample: set_gamma expected gamma>0, found {}",gamma));
        <"Adaptation target acceptance statistic">(delta: f64; !(0.1..=1.0).contains(&delta) => format!("Sample: set_delta expected delta in [0.1,1.0], found {}",delta));
        <"Adaptation relaxation exponent">(kappa: f64; kappa<=0.0 => format!("Sample: set_kappa expected kappa>0, found {}",kappa));
        <"Adaptation iteration offset">(t0: f64; t0<=0.0 => format!("Sample: set_t0 expected t0>0, found {}",t0));
        <"Width of initial fast adaptation interval">(init_buffer: u32; init_buffer==0 => format!("Sample: set_init_buffer expected init_buffer>0, found {}",init_buffer));
        <"Width of final fast adaptation interval">(term_buffer: u32; term_buffer==0 => format!("Sample: set_term_buffer expected term_buffer>0, found {}",term_buffer));
        <"Initial width of slow adaptation interval">(window: u32; window==0 => format!("Sample: set_window expected window>0, found {}",window));
        <"Save metric as JSON?">(save_metric: bool;);
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

    pub fn set_metric_file(&mut self, file: ArgReadablePath) -> Result<&mut Self, ArgError> {
        self.metric_file = file;
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