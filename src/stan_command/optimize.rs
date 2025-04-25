use super::arg_tree::*;

DefArgTree!{ <"Point estimation">ArgOptimize => {
    <"Optimization algorithm"> algorithm: ArgOptimizeAlgorithm = ArgOptimizeAlgorithm::ARG_DEFAULT,
    <"When true, include change-of-variables adjustment for constraining parameter transforms">
        jacobian: bool = false,
    <"Total number of iterations"> iter: u32 = 2000,
    <"Stream optimization progress to output?"> save_iterations: bool = false
}}

DefArgTree!{ <"Optimization algorithm">ArgOptimizeAlgorithm = 
    Self::LBfgs(ArgOptimizeBfgs::ARG_DEFAULT, 5) => {
        <"BFGS with linesearch">Bfgs(ArgOptimizeBfgs),
        <"LBFGS with linesearch">LBfgs(ArgOptimizeBfgs, u32),
        Newton,
}}

DefArgTree!{ ArgOptimizeBfgs => {
    <"Line search step size for first iteration">init_alpha: f64 = 0.001,
    <"Convergence tolerance on absolute changes in objective function value">
        tol_obj: f64 = 1e-12,
    <"Convergence tolerance on relative changes in objective function value">
        tol_rel_obj: f64 = 10000.0,
    <"Convergence tolerance on the norm of the gradient">
        tol_grad: f64 = 1e-08,
    <"Convergence tolerance on the relative norm of the gradient">
        tol_rel_grad: f64 = 1e+07,
    <"Convergence tolerance on changes in parameter value">
        tol_param: f64 = 1e-08,
}}

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

impl ArgOptimize {
    pub fn new() -> Self {
        Self::default()
    }

    default_setter!{
        <"Optimization algorithm">(algorithm:ArgOptimizeAlgorithm;);
        <"When true, include change-of-variables adjustment for constraining parameter transforms">
            (jacobian:bool;);
        <"Total number of iterations">
            (iter:u32; iter==0 => "Optimize: iter cannot be 0".to_string());
        <"Stream optimization progress to output?">
            (save_iterations:bool;);
    }

    ///Optimization algorithm
    ///this function won't consume the algorithm structure
    pub fn set_algorithm_clone(&mut self, algorithm: &ArgOptimizeAlgorithm) -> &mut Self {
        self.algorithm = algorithm.clone();
        self
    }

    ///Optimization algorithm
    ///this function won't consume the algorithm structure
    pub fn with_algorithm_clone(mut self, algorithm: &ArgOptimizeAlgorithm) -> Self {
        self.algorithm = algorithm.clone();
        self
    }
}

impl ArgOptimizeBfgs {
    pub fn new() -> Self {
        Self::default()
    }

    default_setter!{
        <"Line search step size for first iteration">
            (init_alpha:f64; 
                init_alpha<=0.0 => format!("Optimize: except init_alpha > 0, found {}",init_alpha));

        <"Convergence tolerance on absolute changes in objective function value">
            (tol_obj:f64; 
                tol_obj<=0.0 => format!("Optimize: except tol_obj > 0, found {}",tol_obj));

        <"Convergence tolerance on relative changes in objective function value">
            (tol_rel_obj:f64; 
                tol_rel_obj<=0.0 => format!("Optimize: except tol_rel_obj > 0, found {}",tol_rel_obj));
        
        <"Convergence tolerance on the norm of the gradient">
            (tol_grad:f64; 
                tol_grad<=0.0 => format!("Optimize: except tol_grad > 0, found {}",tol_grad));
        
        <"Convergence tolerance on the relative norm of the gradient">
            (tol_rel_grad:f64; 
                tol_rel_grad<=0.0 => format!("Optimize: except tol_rel_grad > 0, found {}",tol_rel_grad));
        
        <"Convergence tolerance on changes in parameter value">
            (tol_param:f64; 
                tol_param<=0.0 => format!("Optimize: except tol_param > 0, found {}",tol_param));            
    }
}

impl ArgOptimizeAlgorithm {
    pub fn new() -> Self {
        Self::default()
    }

    /// get the &ArgOptimizeBfgs structure if the algorithm is BFGS/LBFGS
    /// 
    /// return None when the algorithm is Newton
    ///
    /// # Example
    /// 
    /// ```untest
    /// assert!(ArgOptimizeAlgorithm::new().get_bfgs().unwrap().tol_obj, 1e-12);
    /// ```
    pub fn get_bfgs(&self) -> Option<&ArgOptimizeBfgs> {
        match self {
            Self::Bfgs(b) => Some(b),
            Self::LBfgs(b, _) => Some(b),
            Self::Newton => None,
        }
    }

    /// get the &ArgOptimizeBfgs structure if the algorithm is BFGS/LBFGS
    /// 
    /// return None when the algorithm is Newton
    ///
    /// # Example
    /// 
    /// ```untest
    /// let x = ArgOptimizeAlgorithm::new();
    /// x.get_bfgs_mut().unwrap().set_tol_obj(1e-11);
    /// assert!(x.get_bfgs().unwrap().tol_obj, 1e-11);
    /// ```
    pub fn get_bfgs_mut(&mut self) -> Option<&mut ArgOptimizeBfgs> {
        match self {
            Self::Bfgs(b) => Some(b),
            Self::LBfgs(b, _) => Some(b),
            Self::Newton => None,
        }        
    }

    /// set the bfgs params
    /// 
    /// - if the algorithm is Bfgs, the bfgs params will be replaced by the input;
    /// - if the algorithm is LBfgs, the bfgs params part of the LBfgs will be replaced, and the history_size part will stay same.
    /// - if the algorithm is Newton, switch to Bfgs with the given input param
    ///
    pub fn set_bfgs(&mut self, bfgs: ArgOptimizeBfgs) -> &mut Self {
        *self = match self {
            Self::Bfgs(_) => Self::Bfgs(bfgs),
            Self::LBfgs(_, h) => Self::LBfgs(bfgs, *h),
            Self::Newton => Self::Bfgs(bfgs),
        };
        self
    }

    /// set the bfgs params, consume self and return a new self
    /// 
    /// - if the algorithm is Bfgs, the bfgs params will be replaced by the input;
    /// - if the algorithm is LBfgs, the bfgs params part of the LBfgs will be replaced, and the history_size part will stay same;
    /// - if the algorithm is Newton, switch to Bfgs with the given input param;
    ///
    pub fn with_bfgs(self, bfgs: ArgOptimizeBfgs) -> Self {
        match self {
            Self::Bfgs(_) => Self::Bfgs(bfgs),
            Self::LBfgs(_, h) => Self::LBfgs(bfgs, h),
            Self::Newton => Self::Bfgs(bfgs),
        }
    }

    /// get the history size if the algorithm is LBFGS
    /// 
    /// return None when the algorithm is not LBFGS
    ///
    /// # Example
    /// 
    /// ```untest
    /// assert!(ArgOptimizeAlgorithm::new().get_history_size().unwrap(), 5);
    /// ```
    pub fn get_history_size(&self) -> Option<u32> {
        match self {
            Self::LBfgs(_, h) => Some(*h),
            _ => None,
        }
    }

    /// set the amount of history to keep for L-BFGS
    /// 
    /// - if the algorithm is Bfgs, it will be transformed to LBfgs with the given history_size; (with a clone)
    /// - if the algorithm is LBfgs, the history_size will be changed;
    /// - if the algorithm is Newton, switch to Bfgs with the given input param; (with a clone)
    /// 
    /// # Errors
    /// 
    /// when recieved `history_size==0`, returns BadArgumentValue `"Optimize: history_size cannot be 0"`
    pub fn set_history_size(&mut self, history_size: u32) -> Result<&mut Self, ArgError> {
        if history_size == 0 {
            return Err(ArgError::BadArgumentValue("Optimize: history_size cannot be 0".to_string()));
        }
        match self {
            Self::Bfgs(b) => {
                *self = Self::LBfgs(b.clone(), history_size)
            },
            Self::LBfgs(_, h) => {
                *h = history_size
            }
            Self::Newton => {
                *self = Self::LBfgs(ArgOptimizeBfgs::ARG_DEFAULT, history_size)
            }
        };
        Ok(self)
    }

    /// set the amount of history to keep for L-BFGS, consume self and return a new one
    /// 
    /// - if the algorithm is Bfgs, it will be transformed to LBfgs with the given history_size; (with a clone)
    /// - if the algorithm is LBfgs, the history_size will be changed;
    /// - if the algorithm is Newton, switch to Bfgs with the given input param; (with a clone)
    /// 
    /// # Errors
    /// 
    /// when recieved `history_size==0`, returns BadArgumentValue `"Optimize: history_size cannot be 0"`
    pub fn with_history_size(self, history_size: u32) -> Result<Self, ArgError> {
        if history_size == 0 {
            return Err(ArgError::BadArgumentValue("Optimize: history_size cannot be 0".to_string()));
        }
        Ok(match self {
            Self::Bfgs(b) => Self::LBfgs(b, history_size),
            Self::LBfgs(b, _) => Self::LBfgs(b, history_size),
            Self::Newton => Self::LBfgs(ArgOptimizeBfgs::ARG_DEFAULT, history_size),
        })
    }

    /// drop the history_size of the params, turning self to bfgs
    /// 
    /// Clone when the algorithm is Self::LBfgs
    /// 
    /// if self is Self::Newton, this function will give it a default bfgs arguments
    pub fn drop_history_size(&mut self) -> &mut Self {
        match self {
            Self::LBfgs(b, _) => *self = Self::Bfgs(b.clone()),
            Self::Newton => *self = Self::Bfgs(ArgOptimizeBfgs::ARG_DEFAULT),
            _ => {},
        }
        self
    }

    /// drop the history_size of the params, turning self to bfgs. Consume self and return a new one.
    /// 
    /// if self is Self::Newton, this function will give it a default bfgs arguments
    pub fn without_history_size(self) -> Self {
        match self {
            Self::Bfgs(b) => Self::Bfgs(b),
            Self::LBfgs(b, _) => Self::Bfgs(b),
            Self::Newton => Self::Bfgs(ArgOptimizeBfgs::ARG_DEFAULT),
        }
    }

    /// set the algorithm to Newton's method.
    pub fn set_newton(&mut self) -> &mut Self {
        *self = Self::Newton;
        self
    }

    /// set the algorithm to Newton's method, consume self and return a new one
    pub fn with_newton(self) -> Self {
        Self::Newton
    }
}