use super::arg_tree::*;

DefArgTree!{<"Sample from a Laplace approximation">ArgLaplace => {
    <"A specification of a mode on the constrained scale for all model parameters, either in JSON or CSV format">mode: ArgReadablePath = ArgReadablePath::ARG_DEFAULT,
    <"When true, include change-of-variables adjustment for constraining parameter transforms">jacobian: bool = true,
    <"Number of draws from the laplace approximation">draws: u32 = 1000,
    <"If true, calculate the log probability of the model at each draw">calculate_lp: bool = true,
}}

ImplDefault!{ArgLaplace}

impl ArgThrough for ArgLaplace {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::Laplace)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("laplace");
        arg_into!(self.{mode, jacobian, draws, calculate_lp} in Self >> cmd);
        Ok(())
    }
}

impl ArgLaplace {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"A specification of a mode on the constrained scale for all model parameters, either in JSON or CSV format">(mode:ArgReadablePath;);
        <"When true, include change-of-variables adjustment for constraining parameter transforms">(jacobian:bool;);
        <"Number of draws from the laplace approximation">(draws:u32; draws==0 => "Laplace: draws cannot be 0".to_string());
        <"If true, calculate the log probability of the model at each draw">(calculate_lp:bool;);
    }
}