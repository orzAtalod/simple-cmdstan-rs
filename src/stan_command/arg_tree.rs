use std::{ffi::{OsStr, OsString}, process::Command};
pub const EPS: f64 = f64::EPSILON * 10.0;
pub use paste::paste;
pub use super::{ArgError, ArgType, WithDefaultArg, ArgThrough};
pub use crate::arg_paths::{ArgWritablePath, ArgReadablePath};

pub fn args_combine(name: &str, val: &OsStr) -> OsString {
    let mut res = OsString::new();
    res.push(name);
    res.push("=");
    res.push(val);
    res
}

pub fn arg_if_not_default<T:std::fmt::Display+PartialEq>(cmd: &mut Command, arg_name: &str, arg_val: &T, arg_default: T) {
    if *arg_val != arg_default {
        cmd.arg(format!("{}={}",arg_name, arg_val));
    }
}

macro_rules! arg_into (
    ($struct_name:ident.{$($member_name:ident),+} in $struct_type:ty >> $command:expr) => {
        $(arg_if_not_default($command, stringify!($member_name), &$struct_name.$member_name, <$struct_type>::ARG_DEFAULT.$member_name);)+
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