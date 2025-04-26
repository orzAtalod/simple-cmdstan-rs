/*
diagnose
  Model diagnostics
  Valid subarguments: test

  test=<list element>
    Diagnostic test
    Valid values: gradient
    Defaults to gradient

    gradient
      Check model gradient against finite differences
      Valid subarguments: epsilon, error

      epsilon=<double>
        Finite difference step size
        Valid values: 0 < epsilon
        Defaults to 1e-6

      error=<double>
        Error threshold
        Valid values: 0 < error
        Defaults to 1e-6
 */
use super::arg_tree::*;

DefArgTree!{<"Model diagnostics">ArgDiagnose = Self::Gradient(ArgDiagnoseGradient::ARG_DEFAULT) => {
    <"Check model gradient against finite differences">Gradient(ArgDiagnoseGradient),
}}

DefArgTree!{<"Check model gradient against finite differences">ArgDiagnoseGradient => {
    <"Finite difference step size">epsilon: f64 = 1e-6,
    <"Error threshold">error: f64 = 1e-6
}}

ImplDefault!{ArgDiagnose, ArgDiagnoseGradient}

impl ArgThrough for ArgDiagnose {
    fn arg_type(&self) -> Result<ArgType, ArgError> {
        Ok(ArgType::Diagnose)
    }

    fn arg_through(&self, cmd: &mut std::process::Command) -> Result<(), ArgError> {
        cmd.arg("diagnose");
        match self {
            Self::Gradient(g) => {
                cmd.arg("test=gradient");
                arg_into!(g.{epsilon, error} in ArgDiagnoseGradient >> cmd)
            }
        }
        Ok(())
    }
}

impl ArgDiagnose {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    pub fn set_gradient(&mut self, gradient: ArgDiagnoseGradient) -> &mut Self {
        *self = Self::Gradient(gradient);
        self
    }

    pub fn with_gradient(&mut self, gradient: ArgDiagnoseGradient) -> Self {
        Self::Gradient(gradient)
    }

    pub fn get_gradient(&self) -> &ArgDiagnoseGradient {
        match self {
            Self::Gradient(g) => g
        }
    }

    pub fn get_mut_gradient(&mut self) -> &mut ArgDiagnoseGradient {
        match self {
            Self::Gradient(g) => g
        }
    }
}

impl ArgDiagnoseGradient {
    pub fn new() -> Self {
        Self::ARG_DEFAULT
    }

    default_setter!{
        <"Finite difference step size">(epsilon: f64; epsilon<=0.0 => format!("Diagnose: expected epsilon>0, found {}",epsilon));
        <"Error threshold">(error: f64; error<=0.0 => format!("Diagnose: expected error>0, found {}", error));
    }
}