use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "libloading")]
    LibLoading(libloading::Error),
}

#[cfg(feature = "libloading")]
impl From<::libloading::Error> for Error {
    fn from(value: libloading::Error) -> Self {
        Self::LibLoading(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "libloading")]
            Error::LibLoading(x) => x.fmt(f),
            _ => ().fmt(f),
        }
    }
}

impl std::error::Error for Error {}
