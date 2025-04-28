use super::arg_tree::*;
DefArgTree!{<"Pathfinder algorithm">ArgPathfinder => {
    <"Line search step size for first iteration">init_alpha: f64 = 0.001,
    <"Convergence tolerance on absolute changes in objective function value">tol_obj: f64 = 1e-12,
    <"Convergence tolerance on relative changes in objective function value">tol_rel_obj: f64 = 10000.0,
    <"Convergence tolerance on the norm of the gradient">tol_grad: f64 = 1e-8,
    <"Convergence tolerance on the relative norm of the gradient">tol_rel_grad: f64 = 1e7,
    <"Convergence tolerance on changes in parameter value">tol_param: f64 = 1e-8,
    <"Amount of history to keep for L-BFGS">history_size: u32 = 5,
    <"Number of draws from PSIS sample">num_psis_draws: u32 = 1000,
    <"Number of single pathfinders">num_paths: u32 = 4,
    <"Output single-path pathfinder draws as CSV">save_single_paths: bool = false,
    <"If true, perform psis resampling on samples returned from individual pathfinders. If false, returns num_paths * num_draws samples">psis_resample: bool = true,
    <"If true, individual pathfinders lp calculations are calculated and returned with the output. If false, each pathfinder will only  calculate the lp values needed for the elbo calculation. If false, psis resampling cannot be performed and the algorithm returns num_paths * num_draws samples. The output will still contain any lp values used when calculating ELBO scores within LBFGS iterations.">calculate_lp: bool = true,
    <"Maximum number of LBFGS iterations">max_lbfgs_iters: u32 = 1000,
    <"Number of approximate posterior draws">num_draws: u32 = 1000,
    <"Number of Monte Carlo draws to evaluate ELBO">num_elbo_draws: u32 = 25
}}

ImplDefault!{ArgPathfinder}

impl ArgThrough for ArgPathfinder {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::Pathfinder)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("pathfinder");
        arg_into!(self.{init_alpha, tol_obj, tol_rel_obj, tol_grad, tol_rel_grad, tol_param, history_size, num_psis_draws, num_paths, save_single_paths, psis_resample, calculate_lp, max_lbfgs_iters, num_draws, num_elbo_draws} in Self >> cmd);
        Ok(())
    }
}

impl ArgPathfinder {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Line search step size for first iteration">(init_alpha:f64; init_alpha<=0.0 => format!("Pathfinder: expected init_alpha>0, found {}",init_alpha));
        <"Convergence tolerance on absolute changes in objective function value">(tol_obj:f64; tol_obj<=0.0 => format!("Pathfinder: expected tol_obj>0, found {}",tol_obj));
        <"Convergence tolerance on relative changes in objective function value">(tol_rel_obj:f64; tol_rel_obj<=0.0 => format!("Pathfinder: expected tol_rel_obj>0, found {}",tol_rel_obj));
        <"Convergence tolerance on the norm of the gradient">(tol_grad:f64; tol_grad<=0.0 => format!("Pathfinder: expected tol_grad>0, found {}",tol_grad));
        <"Convergence tolerance on the relative norm of the gradient">(tol_rel_grad:f64; tol_rel_grad<=0.0 => format!("Pathfinder: expected tol_rel_grad>0, found {}",tol_rel_grad));
        <"Convergence tolerance on changes in parameter value">(tol_param:f64; tol_param<=0.0 => format!("Pathfinder: expected tol_param>0, found {}",tol_param));
        <"Amount of history to keep for L-BFGS">(history_size:u32; history_size==0 => "Pathfinder: history_size cannot be 0".to_string());
        <"Number of draws from PSIS sample">(num_psis_draws:u32; num_psis_draws==0 => "Pathfinder: num_psis_draws cannot be 0".to_string());
        <"Number of single pathfinders">(num_paths:u32; num_paths==0 => "Pathfinder: num_paths cannot be 0".to_string());
        <"Maximum number of LBFGS iterations">(max_lbfgs_iters:u32; max_lbfgs_iters==0 => "Pathfinder: max_lbfgs_iters cannot be 0".to_string());
        <"Number of approximate posterior draws">(num_draws:u32; num_draws==0 => "Pathfinder: num_draws cannot be 0".to_string());
        <"Number of Monte Carlo draws to evaluate ELBO">(num_elbo_draws:u32; num_elbo_draws==0 => "Pathfinder: num_elbo_draws cannot be 0".to_string());
        <"Output single-path pathfinder draws as CSV">(save_single_paths:bool;);
        <"If true, perform psis resampling on samples returned from individual pathfinders. If false, returns num_paths * num_draws samples">
            (psis_resample:bool;);
        <"If true, individual pathfinders lp calculations are calculated and returned with the output. If false, each pathfinder will only  calculate the lp values needed for the elbo calculation. If false, psis resampling cannot be performed and the algorithm returns num_paths * num_draws samples. The output will still contain any lp values used when calculating ELBO scores within LBFGS iterations.">
            (calculate_lp:bool;);
    }
}