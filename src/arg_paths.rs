use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{prelude::*, Error};

pub mod core {
    use super::*;
    use std::fmt::Display;
    use crate::stan_command::WithDefaultArg;
    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgPath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    impl WithDefaultArg for ArgPath {
        const ARG_DEFAULT: Self = Self::Borrowed("");
    }

    impl Display for ArgPath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Borrowed(path) => write!(f, "{}", *path),
                Self::Owned(path) => write!(f, "{}", path.to_string_lossy())
            }
        }
    }

    impl ArgPath {
        pub fn as_path(&self) -> &Path {
            match self {
                Self::Borrowed(path) => Path::new(path),
                Self::Owned(path) => path.as_path(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgWritablePath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    impl WithDefaultArg for ArgWritablePath {
        const ARG_DEFAULT: Self = Self::Borrowed("");
    }

    impl Display for ArgWritablePath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Borrowed(path) => write!(f, "{}", *path),
                Self::Owned(path) => write!(f, "{}", path.to_string_lossy())
            }
        }
    }

    impl ArgWritablePath {
        pub fn as_path(&self) -> &Path {
            match self {
                Self::Borrowed(path) => Path::new(path),
                Self::Owned(path) => path.as_path(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ArgReadablePath {
        Borrowed(&'static str),
        Owned(PathBuf),
    }

    impl WithDefaultArg for ArgReadablePath {
        const ARG_DEFAULT: Self = Self::Borrowed("");
    }

    impl Display for ArgReadablePath {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Borrowed(path) => write!(f, "{}", *path),
                Self::Owned(path) => write!(f, "{}", path.to_string_lossy())
            }
        }
    }
    
    impl ArgReadablePath {
        pub fn as_path(&self) -> &Path {
            match self {
                Self::Borrowed(path) => Path::new(path),
                Self::Owned(path) => path.as_path(),
            }
        }
    }

    impl From<&'static str> for ArgPath {
        fn from(path: &'static str) -> Self {
            Self::Borrowed(path)
        }
    }

    impl From<PathBuf> for ArgPath {
        fn from(path: PathBuf) -> Self {
            Self::Owned(path)
        }
    }

    impl From<ArgPath> for PathBuf {
        fn from(path: ArgPath) -> Self {
            match path {
                ArgPath::Borrowed(path) => PathBuf::from(path),
                ArgPath::Owned(path) => path,
            }
        }
    }

    impl From<ArgReadablePath> for PathBuf {
        fn from(path: ArgReadablePath) -> Self {
            match path {
                ArgReadablePath::Borrowed(path) => PathBuf::from(path),
                ArgReadablePath::Owned(path) => path,
            }
        }
    }

    impl From<ArgWritablePath> for PathBuf {
        fn from(path: ArgWritablePath) -> Self {
            match path {
                ArgWritablePath::Borrowed(path) => PathBuf::from(path),
                ArgWritablePath::Owned(path) => path,
            }
        }
    }

    impl From<ArgReadablePath> for ArgWritablePath {
        fn from(path: ArgReadablePath) -> Self {
            match path {
                ArgReadablePath::Borrowed(path) => ArgWritablePath::Borrowed(path),
                ArgReadablePath::Owned(path) => ArgWritablePath::Owned(path),
            }
        }
    }
}

pub use core::{ArgPath, ArgWritablePath, ArgReadablePath};

impl ArgPath {
    pub fn verify_file_readable(&self) -> Result<(), Error> {
        match self {
            ArgPath::Borrowed(path) => std::fs::File::open(path).map(|_|()),
            ArgPath::Owned(path) => std::fs::File::open(path).map(|_|()),
        }
    }

    pub fn into_readable(self) -> Result<ArgReadablePath, Error> {
        self.verify_file_readable()?;
        match self {
            ArgPath::Borrowed(path) => Ok(ArgReadablePath::Borrowed(path)),
            ArgPath::Owned(path) => Ok(ArgReadablePath::Owned(path)),
        }
    }

    pub fn verify_file_writeable(&self) -> Result<(), Error> {
        let path = match self {
            ArgPath::Borrowed(path) => Path::new(path),
            ArgPath::Owned(path) => path,
        };
        if let Some(parent_path) = path.parent() {
            std::fs::create_dir_all(parent_path)?;
        }
        std::fs::OpenOptions::new()
                .create(true)
                .append(true) // 避免 truncate 覆盖
                .open(path)
                .map(|_|())
    }

    pub fn into_writeable(self) -> Result<ArgWritablePath, Error> {
        self.verify_file_writeable()?;
        match self {
            ArgPath::Borrowed(path) => Ok(ArgWritablePath::Borrowed(path)),
            ArgPath::Owned(path) => Ok(ArgWritablePath::Owned(path)),
        }
    }

    pub fn extend_default_file(&mut self, default_name: &str) -> &mut Self {
        match self {
            ArgPath::Borrowed(path) => {
                let mut path = PathBuf::from(*path);
                if path.extension().is_none() {
                    path.push(default_name);
                    *self = ArgPath::Owned(path);
                }
            },
            ArgPath::Owned(path) => {
                if path.extension().is_none() {
                    path.push(default_name);
                }
            },
        }
        self
    }
}

impl ArgWritablePath {
    pub fn write_once(&self, text: &str) -> Result<&Self, Error> {
        let path: &Path = match self {
            ArgWritablePath::Borrowed(p) => Path::new(p),
            ArgWritablePath::Owned(p) => p,
        };

        let mut file= File::create(path)?;
        file.write_all(text.as_bytes())?;
        Ok(self)
    }
}