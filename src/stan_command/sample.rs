use super::arg_tree::*;
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

impl ArgSampleAlgorithm {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    /// Set self to Self::FixedParam
    pub fn set_to_fixed_param(&mut self) -> &mut Self {
        *self = Self::FixedParam;
        self
    }

    /// Set self to Self::Hmc(hmc) with the given hmc parameter.
    /// 
    /// Use x.set_hmc(some_hmc.clone()) to avoid move.
    pub fn set_hmc(&mut self, hmc: ArgSampleHmc) -> &mut Self {
        *self = Self::Hmc(hmc);
        self
    }

    /// return Some(&mut hmc) if self is Hmc(hmc), else None
    /// 
    /// ```no_run
    /// let mut x = ArgSampleAlgorithm::new();
    /// x.get_mut_hmc().unwrap().set_stepsize(0.1);
    /// let y = ArgSampleAlgorithm::Hmc(ArgSampleHmc::new().with_stepsize(0.1));
    /// assert!(x == y);
    /// ```
    pub fn get_mut_hmc(&mut self) -> Option<&mut ArgSampleHmc> {
        match self {
            Self::Hmc(hmc) => Some(hmc),
            _ => None,
        }
    }

    /// return Some(&hmc) if self is Hmc(hmc), else None
    /// 
    /// use this function to seek the hmc parameter.
    pub fn get_hmc(&self) -> Option<&ArgSampleHmc> {
        match self {
            Self::Hmc(hmc) => Some(hmc),
            _ => None,
        }
    }

    /// return &hmc if self is Hmc(hmc), else panic
    pub fn expect_hmc(&self) -> &ArgSampleHmc {
        match self {
            Self::Hmc(hmc) => hmc,
            _ => panic!("Expected HMC, found {:?}", self),
        }
    }
}

impl ArgSampleHmc {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Engine for Hamiltonian Monte Carlo">(engine: ArgSampleEngine;);
        <"Geometry of base manifold. Use ArgSampleMetric::UnitE or something to set the exact value.">(metric: ArgSampleMetric;);
        <"Input file with precomputed Euclidean metric">(metric_file: ArgReadablePath;);
        <"Step size for discrete evolution">(stepsize: f64; stepsize<=0.0 => format!("Sample: set_stepsize expected stepsize>0, found {}",stepsize));
        <"Uniformly random jitter of the stepsize, in percent">(stepsize_jitter: f64; !(0.0..=1.0).contains(&stepsize_jitter) => format!("Sample: set_stepsize_jitter expected jitter in [0,1], found {}",stepsize_jitter));
    }
}

impl ArgSampleEngine {
    /// Default: Self::Nuts(10) (The No-U-Turn Sampler with max_depth 10)
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    /// Return the default static engine with int_time 6.2839
    /// 
    /// ```no_run
    /// let x = ArgSampleHmc::new().with_engine(ArgSampleEngine::default_static_engine()); 
    /// let y = ArgSampleHmc::new().with_engine(ArgSampleEngine::Static(6.28319)); //or (will warning)
    /// assert!(x == y);
    /// ```
    pub const fn default_static_engine() -> Self {
        Self::Static(DEFAULT_ENGINE_STATIC_VAL)
    }

    /// Set the engine to static with default int_time 6.28319
    /// 
    /// Static engine: Static integration time
    /// 
    /// int_time: Total integration time for Hamiltonian evolution, default is 2 * pi (6.28319)
    pub fn set_static_engine(&mut self) -> &mut Self {
        *self = Self::Static(DEFAULT_ENGINE_STATIC_VAL);
        self
    }

    /// Set the engine to static with given int_time
    /// 
    /// #Errors
    /// 
    /// when `int_time<=0` return BadArgumentValue error `Sample: set_static_int_time expected int_time>0, found {int_time}`
    pub fn set_static_int_time(&mut self, int_time: f64) -> Result<&mut Self, ArgError> {
        if int_time > 0.0 {
            *self = Self::Static(int_time);
            Ok(self)
        } else {
            Err(ArgError::BadArgumentValue(format!("Sample: set_static_int_time expected int_time>0, found {}",int_time).to_string()))
        }
    }

    /// Set the engine to nuts with default max_depth 10
    pub fn set_nuts_engine(&mut self) -> &mut Self {
        *self = Self::ARG_DEFAULT;
        self
    }

    /// Set the engine to nuts with given max_depth
    /// 
    /// #Errors
    /// 
    /// when `max_depth==0` return BadArgumentValue error `Sample: max_depth of set_nuts_max_depth could not be 0`
    pub fn set_nuts_max_depth(&mut self, max_depth: u32) -> Result<&mut Self, ArgError> {
        if max_depth == 0 {
            Err(ArgError::BadArgumentValue("sample: max_depth of set_nuts_max_depth could not be 0".to_string()))
        } else {
            *self = Self::Nuts(max_depth);
            Ok(self)
        }
    }

    /// if the engine is static, return Some(int_time), else None
    pub fn get_int_time(&self) -> Option<f64> {
        match self {
            Self::Static(x) => Some(*x),
            _ => None,
        }
    }

    /// if the engine is nuts, return Some(max_depth), else None
    pub fn get_max_depth(&self) -> Option<u32> {
        match self {
            Self::Nuts(x) => Some(*x),
            _ => None,
        }
    }
}

impl ArgSampleMetric {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }
}