use super::arg_tree::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ArgOptimize {
    pub algorithm: ArgOptimizeAlgorithm,
    pub jacobian: bool,         //defualt: false
    pub iter: u32,              //>0, default: 2000
    pub save_iterations: bool,  //default: false
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgOptimizeAlgorithm {
    Bfgs(ArgOptimizeBfgs),
    LBfgs(ArgOptimizeBfgs,u32),  //default, u32(history_size)>0, default: 5
    Newton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgOptimizeBfgs {
    pub init_alpha: f64,    //>0, default: 0.001
    pub tol_obj: f64,       //>0, default: 1e-12
    pub tol_rel_obj: f64,   //>0, default: 10000
    pub tol_grad:f64,       //>0, default: 1e-08
    pub tol_rel_grad: f64,  //>0, default: 1e+07
    pub tol_param: f64,     //>0, default: 1e-08
}

impl WithDefaultArg for ArgOptimize {
    const ARG_DEFAULT: Self = ArgOptimize {
        algorithm: ArgOptimizeAlgorithm::ARG_DEFAULT,
        jacobian: false,
        iter: 2000,
        save_iterations: false,
    };

    fn is_default(&self) -> bool {
        self.algorithm.is_default() &&
        self.jacobian == Self::ARG_DEFAULT.jacobian &&
        self.iter == Self::ARG_DEFAULT.iter &&
        self.save_iterations == Self::ARG_DEFAULT.save_iterations
    }
}

impl WithDefaultArg for ArgOptimizeAlgorithm {
    const ARG_DEFAULT: Self = Self::LBfgs(ArgOptimizeBfgs::ARG_DEFAULT, 5);
    fn is_default(&self) -> bool {
        match self {
            Self::LBfgs(bfgs, his_size) => *his_size==5 && bfgs.is_default(),
            _ => false
        }
    }
}

impl WithDefaultArg for ArgOptimizeBfgs {
    const ARG_DEFAULT: Self = ArgOptimizeBfgs {
        init_alpha: 0.001,
        tol_obj: 1e-12,
        tol_rel_obj: 10000.0,
        tol_grad: 1e-8,
        tol_rel_grad: 1e+7,
        tol_param: 1e-8
    };

    fn is_default(&self) -> bool {
        (self.init_alpha-Self::ARG_DEFAULT.init_alpha).abs()   <= EPS &&
        (self.tol_obj-Self::ARG_DEFAULT.tol_obj).abs()      <= EPS &&
        (self.tol_rel_obj-Self::ARG_DEFAULT.tol_rel_obj).abs()  <= EPS &&
        (self.tol_grad-Self::ARG_DEFAULT.tol_grad).abs()     <= EPS &&
        (self.tol_rel_grad-Self::ARG_DEFAULT.tol_rel_grad).abs() <= EPS &&
        (self.tol_param-Self::ARG_DEFAULT.tol_param).abs()    <= EPS
    }
}

ImplDefault!{ArgOptimize, ArgOptimizeAlgorithm, ArgOptimizeBfgs}

impl ArgThrough for ArgOptimize {
    fn arg_type(&self) -> Result<ArgType, ArgError> {    
        Ok(ArgType::Optimize)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("optimize");
        'algo: {
            if !self.algorithm.is_default() {
                let bfgs: &ArgOptimizeBfgs = match &self.algorithm {
                    ArgOptimizeAlgorithm::Newton => {
                        cmd.arg("algorithm=newton");
                        break 'algo;
                    },
                    ArgOptimizeAlgorithm::Bfgs(b) => {
                        cmd.arg("algorithm=bfgs");
                        b
                    }
                    ArgOptimizeAlgorithm::LBfgs(b, v) => {
                        cmd.arg("algorithm=lbfgs");
                        if *v!=5 {
                            cmd.arg(format!("history_size={}",*v));
                        }
                        b
                    }
                };

                arg_into!(bfgs.{init_alpha, tol_obj, tol_rel_obj, tol_grad, tol_rel_grad, tol_param} in ArgOptimizeBfgs >> cmd);
            }
        }

        arg_into!(self.{jacobian, iter, save_iterations} in Self >> cmd);
        Ok(())
    }
}