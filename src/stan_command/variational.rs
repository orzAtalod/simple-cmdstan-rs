use super::arg_tree::*;

DefArgTree!{<"Variational inference">ArgVariational => {
    <"Variational inference algorithm">algorithm: ArgVariationalAlgorithm = ArgVariationalAlgorithm::ARG_DEFAULT,
    <"Maximum number of ADVI iterations.">iter: u32 = 10000,
    <"Number of Monte Carlo draws for computing the gradient.">grad_samples: u32 = 1,
    <"Number of Monte Carlo draws for estimate of ELBO.">elbo_samples: u32 = 100,
    <"Stepsize scaling parameter.">eta: f64 = 1.0,
    <"Eta Adaptation for Variational Inference">adapt: ArgVariationalAdapt = ArgVariationalAdapt::ARG_DEFAULT,
    <"Relative tolerance parameter for convergence.">tol_rel_obj: f64 = 0.01,
    <"Number of iterations between ELBO evaluations">eval_elbo: u32 = 100,
    <"Number of approximate posterior output draws to save.">output_samples: u32 = 1000,
}}

DefArgTree!{<"Variational inference algorithm">ArgVariationalAlgorithm = Self::Meanfield => {
    <"mean-field approximation">Meanfield,
    <"full-rank covariance">Fullrank,
}}

DefArgTree!{<"Eta Adaptation for Variational Inference">ArgVariationalAdapt => {
    <"Boolean flag for eta adaptation.">engaged: bool = true,
    <"Number of iterations for eta adaptation.">iter: u32 = 50,
}}

ImplDefault!{ArgVariational, ArgVariationalAlgorithm, ArgVariationalAdapt}

impl ArgThrough for ArgVariational {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::Variational)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("variational");
        if !self.algorithm.is_default() {
            cmd.arg("algorithm=fullrank");
        }
        if !self.adapt.is_default() {
            cmd.arg("adapt");
            let v = &self.adapt;
            arg_into!(v.{engaged, iter} in ArgVariationalAdapt >> cmd);
        }
        arg_into!(self.{iter, grad_samples, elbo_samples, eta, tol_rel_obj, eval_elbo, output_samples} in Self >> cmd);
        Ok(())
    }
}

impl ArgVariational {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Variational inference algorithm">(algorithm: ArgVariationalAlgorithm;);
        <"Maximum number of ADVI iterations.">
            (iter: u32; iter==0 => "Variational: iter cannot be 0".to_string());
        <"Number of Monte Carlo draws for computing the gradient.">
            (grad_samples: u32; grad_samples==0 => "Variational: grad_samples cannot be 0".to_string());
        <"Stepsize scaling parameter.">
            (eta: f64; eta<=0.0 => format!("Variational: expected eta>0, found {}",eta));
        <"Eta Adaptation for Variational Inference">(adapt: ArgVariationalAdapt;);
        <"Relative tolerance parameter for convergence.">
            (tol_rel_obj: f64; tol_rel_obj<=0.0 => format!("Variational: expected tol_rel_obj>0, found {}", tol_rel_obj));
        <"Number of iterations between ELBO evaluations">
            (eval_elbo: u32; eval_elbo==0 => "Variational: eval_elbo cannot be 0".to_string());
        <"Number of approximate posterior output draws to save.">
            (output_samples: u32; output_samples==0 => "Variational: output_samples cannot be 0".to_string());
    }
}

impl ArgVariationalAlgorithm {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    pub fn is_manifile(&self) -> bool {
        matches!(self, Self::Meanfield)
    }

    pub fn is_fullrank(&self) -> bool {
        !self.is_manifile()
    }
}

impl ArgVariationalAdapt {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Boolean flag for eta adaptation.">(engaged: bool;);
        <"Number of iterations for eta adaptation.">
            (iter: u32; iter==0 => "Variational: adapt.iter cannot be 0".to_string());
    }
}