use std::{ffi::{OsStr, OsString}, path::{Path, PathBuf}, process::Command};
pub const EPS: f64 = f64::EPSILON * 10.0;
use paste::paste;

mod arg_error {
    use std::{error::Error, fmt::Display};

    #[derive(Debug)]
    pub enum ArgError {
        NotValidArgTreeType(String),
        BadPath(String),
        BadArgumentValue(String),
        FileSystemError(std::io::Error),
    }

    impl Display for ArgError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NotValidArgTreeType(s) => write!(f, "{s}"),
                Self::BadPath(s) => write!(f, "{s}"),
                Self::BadArgumentValue(s) => write!(f, "{s}"),
                Self::FileSystemError(e) => write!(f, "file system error: {e}"),
            }
        }
    }

    impl Error for ArgError {}
}

pub use arg_error::ArgError;

pub enum ArgType {
    Sample,
    Optimize,
    Variational,
    Diagnose,
    GenerateQuantities,
    Pathfinder,
    LogProb,
    Laplace,
}

pub trait ArgThrough {
    fn arg_type(&self) -> Result<ArgType, ArgError>;
    fn arg_through(&self, cmd: &mut Command) -> Result<(), ArgError>;
}

pub fn args_combine(name: &str, val: &OsStr) -> OsString {
    let mut res = OsString::new();
    res.push(name);
    res.push("=");
    res.push(val);
    res
}

pub fn verify_file_readable(path: &Path) -> Result<(), ArgError> {
    std::fs::File::open(path).map(|_|()).map_err(ArgError::FileSystemError)
}

pub fn verify_file_writeable(path: &Path) -> Result<(), ArgError> {
    if let Some(parten_path) = path.parent() {
        std::fs::create_dir_all(parten_path).map_err(ArgError::FileSystemError)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true) // 避免 truncate 覆盖
        .open(path)
        .map(|_| ())
        .map_err(ArgError::FileSystemError)
}

pub fn verify_or_default(path: &Path, default: &str) -> Result<PathBuf, ArgError> {
    if path.extension().is_none() {
        let mut path = path.to_path_buf();
        path.push(default);
        verify_file_writeable(&path)?;
        Ok(path)
    } else {
        verify_file_writeable(path)?;
        Ok(path.to_path_buf())
    }
}

pub fn arg_if_not_default<T:std::fmt::Display+PartialEq>(cmd: &mut Command, arg_name: &str, arg_val: T, arg_default: T) {
    if arg_val != arg_default {
        cmd.arg(format!("{}={}",arg_name, arg_val));
    }
}

pub fn arg_if_not_default_f64(cmd: &mut Command, arg_name: &str, arg_val: f64, arg_default: f64) {
    if (arg_val - arg_default).abs() > EPS {
        cmd.arg(format!("{}={}",arg_name, arg_val));
    }
}

pub trait WithDefaultArg {
    const ARG_DEFAULT: Self;
    fn is_default(&self) -> bool;
}

macro_rules! arg_into (
    ($struct_name:ident.{$($member_name:ident),+} in $struct_type:ty >> $command:expr) => {
        $(arg_if_not_default($command, stringify!($member_name), $struct_name.$member_name, <$struct_type>::ARG_DEFAULT.$member_name);)+
    };
);

macro_rules! ImplDefault {
    ($struct_type:ty) => {
        impl Default for $struct_type {
            fn default() -> Self {
                Self::ARG_DEFAULT
            }
        }
    };

    ($struct_type:ty,$($struct_types:ty),+) => {
        impl Default for $struct_type {
            fn default() -> Self {
                Self::ARG_DEFAULT
            }
        }
        ImplDefault!{$($struct_types),+}
    };
}

macro_rules! default_setter {
    ($(<$doc:literal>)?($member:ident:$member_type:ty;)$(;)?) => { 
        paste! {
            $(#[doc=concat!($doc)])?
            pub fn [<set_ $member>](&mut self, $member: $member_type) -> &mut Self {
                self.$member = $member;
                self
            }

            pub fn [<with_ $member>](mut self, $member: $member_type) -> Self {
                self.$member = $member;
                self
            }
        } 
    };
    
    ($(<$doc:literal>)?($member:ident:$member_type:ty; $($expect:expr => $else_val: expr),+)$(;)?) => { 
        paste! {
            pub fn [<set_ $member>](&mut self, $member: $member_type) -> Result<&mut Self, ArgError> {
                $(if $expect {
                    return Err(ArgError::BadArgumentValue($else_val))
                });*
                self.$member = $member;
                Ok(self)
            }

            pub fn [<with_ $member>](mut self, $member: $member_type) -> Result<Self, ArgError> {
                $(if $expect {
                    return Err(ArgError::BadArgumentValue($else_val))
                });*
                self.$member = $member;
                Ok(self)
            }
        } 
    };

    ($(<$doc:literal>)?($member:ident:$member_type:ty;); $($(<$docs:literal>)?($members:ident:$member_types:ty; $($expects:expr => $else_vals:expr),*));+$(;)?) => { 
        paste! {
            pub fn [<set_ $member>](&mut self, $member: $member_type) -> &mut Self {
                self.$member = $member;
                self
            }

            pub fn [<with_ $member>](mut self, $member: $member_type) -> Self {
                self.$member = $member;
                self
            }
        }
        default_setter!{$($(<$docs>)?($members:$member_types; $($expects => $else_vals),*));+}
    };

    ($(<$doc:literal>)?($member:ident:$member_type:ty; $($expect:expr => $else_val: expr),+); $($(<$docs:literal>)?($members:ident:$member_types:ty; $($expects:expr => $else_vals: expr),*));+$(;)?) => { 
        paste! {
            pub fn [<set_ $member>](&mut self, $member: $member_type) -> Result<&mut Self, ArgError> {
                $(if $expect {
                    return Err(ArgError::BadArgumentValue($else_val))
                });*
                self.$member = $member;
                Ok(self)
            }

            pub fn [<with_ $member>](mut self, $member: $member_type) -> Result<Self, ArgError> {
                $(if $expect {
                    return Err(ArgError::BadArgumentValue($else_val))
                });*
                self.$member = $member;
                Ok(self)
            }
        } 
        default_setter!{$($(<$docs>)?($members:$member_types; $($expects => $else_vals),*));+}
    };
}

//test
struct Foo {
    c1: i32,
    c2: u32,
    c3: f64,
    c4: i32,
}

impl Foo {
    default_setter!{
        <"111">
        (c1:i32;);
        <"222">
        (c3:f64; c3<0.0 => "Sample: c3 cannot below zero".to_string(), 
            c3>18.0 => "Sample: c3 cannot greater than 18".to_string());
    }

    default_setter!{
        (c2:u32; c2==0 => "Sample: c2 cannot be zero".to_string());
        (c4:i32; c4<0 => "Sample: c3 cannot below zero".to_string(), 
            c4>18 => "Sample: c3 cannot greater than 18".to_string());
    }
}