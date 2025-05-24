use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    /// Indicates we could not open a file as a shared library.
    #[cfg(feature = "libloading")]
    LibLoading(libloading::Error),

    /// Could not read data.
    Io(std::io::Error),

    /// The given hash was not amongst the known hashes we should load.
    InvalidHash(String),
}

#[cfg(feature = "libloading")]
impl From<libloading::Error> for Error {
    fn from(value: libloading::Error) -> Self {
        Self::LibLoading(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "libloading")]
            Error::LibLoading(x) => x.fmt(f),
            Error::Io(x) => x.fmt(f),
            Error::InvalidHash(x) => format!("Invalid hash: {x}").fmt(f),
            _ => "".fmt(f),
        }
    }
}

impl std::error::Error for Error {}
