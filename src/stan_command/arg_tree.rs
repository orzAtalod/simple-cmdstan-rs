use std::{ffi::{OsStr, OsString}, path::{Path, PathBuf}, process::Command};
pub const EPS: f64 = f64::EPSILON * 10.0;
pub use paste::paste;

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

pub trait WithDefaultArg : PartialEq+Sized {
    const ARG_DEFAULT: Self;
    fn is_default(&self) -> bool {
        *self == Self::ARG_DEFAULT
    }
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
            $(#[doc=$doc])?
            pub fn [<set_ $member>](&mut self, $member: $member_type) -> &mut Self {
                self.$member = $member;
                self
            }

            $(#[doc=$doc])?
            pub fn [<with_ $member>](mut self, $member: $member_type) -> Self {
                self.$member = $member;
                self
            }
        } 
    };
    
    ($(<$doc:literal>)?($member:ident:$member_type:ty; $($expect:expr => $else_val: expr),+)$(;)?) => { 
        paste! {
            #[doc=concat!(
                $($doc,"\n\n",)?
                "# Errors\n\n"
                $(,"- when `", stringify!($expect), "` returns BadArgumentValue `", stringify!($else_val), "`\n")+
            )]
            pub fn [<set_ $member>](&mut self, $member: $member_type) -> Result<&mut Self, ArgError> {
                $(if $expect {
                    return Err(ArgError::BadArgumentValue($else_val));
                })*
                self.$member = $member;
                Ok(self)
            }

            #[doc=concat!(
                $($doc,"\n\n",)?
                "# Errors\n\n"
                $(,"- when `", stringify!($expect), "` returns BadArgumentValue `", stringify!($else_val), "`\n")+
            )]
            pub fn [<with_ $member>](mut self, $member: $member_type) -> Result<Self, ArgError> {
                $(if $expect {
                    return Err(ArgError::BadArgumentValue($else_val));
                })*
                self.$member = $member;
                Ok(self)
            }
        } 
    };

    ($(<$doc:literal>)?($member:ident:$member_type:ty;);
        $($(<$docs:literal>)?($members:ident:$member_types:ty; $($expects:expr => $else_vals:expr),*));+$(;)?) => { 
        default_setter!{$(<$doc>)?($member:$member_type;)}
        default_setter!{$($(<$docs>)?($members:$member_types; $($expects => $else_vals),*));+}
    };

    ($(<$doc:literal>)?($member:ident:$member_type:ty; $($expect:expr => $else_val:expr),+);
        $($(<$docs:literal>)?($members:ident:$member_types:ty; $($expects:expr => $else_vals: expr),*));+$(;)?) => { 
        default_setter!{$(<$doc>)?($member:$member_type; $($expect => $else_val),+)}
        default_setter!{$($(<$docs>)?($members:$member_types; $($expects => $else_vals),*));+}
    };
}

macro_rules! DefArgTree {
    //struct
    ($(<$name_doc:literal>)?$name:ident => {$($(<$arg_doc:literal>)?$arg_name:ident:$arg_type:ty = $default:expr),+$(,)?}) => {
        $(#[doc=$name_doc])?
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(
                #[doc=concat!(
                    $($arg_doc,"\n\n",)?
                    "Default:`",
                    stringify!($default),
                    "`"
                )]
                pub $arg_name: $arg_type
            ),+
        }

        impl WithDefaultArg for $name {
            const ARG_DEFAULT: Self = Self {
                $(
                    $arg_name: $default
                ),+
            };
        }
    };

    //enum
    ($(<$name_doc:literal>)?$name:ident = $default:expr => {$($(<$arg_doc:literal>)?$arg_name:ident$(($($arg_type:ty),+))?),+$(,)?}) => {
        #[doc=concat!(
            $($name_doc,"\n\n",)?
            "Default:`",
            stringify!($default),
            "`"
        )]
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $(
                $(#[doc=$arg_doc])?
                $arg_name$(($($arg_type),+))?
            ),+
        }

        impl WithDefaultArg for $name {
            const ARG_DEFAULT: Self = $default;
        }
    };
}

pub use arg_path::{ArgPath, EMPTY_ARG_PATH};
mod arg_path {
    use super::*;
    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgPath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    pub const EMPTY_ARG_PATH: ArgPath = ArgPath::Borrowed("");
}