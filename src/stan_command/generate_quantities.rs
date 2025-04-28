use super::arg_tree::*;

DefArgTree!{<"Generate quantities of interest">ArgGenerate => {
    <"Input file of sample of fitted parameter values for model conditioned on data">fitted_params: ArgReadablePath = ArgReadablePath::ARG_DEFAULT,
    <"Number of chains">num_chains: u32 = 1,
}}

ImplDefault!{ArgGenerate}

impl ArgGenerate {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Number of chains">(num_chains:u32; num_chains==0 => "Generate: num_chains cannot be 0".to_string());
        <"Input file of sample of fitted parameter values for model conditioned on data">(fitted_params:ArgReadablePath;);
    }
}

impl ArgThrough for ArgGenerate {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::GenerateQuantities)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("generate_quantities");
        arg_into!(self.{num_chains, fitted_params} in Self >> cmd);
        Ok(())
    }
}