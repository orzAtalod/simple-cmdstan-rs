use super::arg_tree::*;

DefArgTree!{<"Return the log density up to a constant and its gradients, given supplied parameters">ArgLogProb => {
    <"Input file (JSON or R dump)  of parameter values on unconstrained scale">unconstrained_params: ArgReadablePath = ArgReadablePath::ARG_DEFAULT,
    <"Input file (JSON or R dump)  of parameter values on constrained scale">constrained_params: ArgReadablePath = ArgReadablePath::ARG_DEFAULT,
    <"When true, include change-of-variables adjustment for constraining parameter transforms">jacobian: bool = true,
}}

ImplDefault!{ArgLogProb}

impl ArgThrough for ArgLogProb {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::LogProb)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("log_prob");
        arg_into!(self.{unconstrained_params, constrained_params, jacobian} in Self >> cmd);
        Ok(())
    }
}

impl ArgLogProb {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Input file (JSON or R dump)  of parameter values on unconstrained scale">(unconstrained_params:ArgReadablePath;);
        <"Input file (JSON or R dump)  of parameter values on constrained scale">(constrained_params:ArgReadablePath;);
        <"When true, include change-of-variables adjustment for constraining parameter transforms">(jacobian:bool;);
    }
}