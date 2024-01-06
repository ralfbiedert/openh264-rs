use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    LibLoading(libloading::Error),
}

impl From<::libloading::Error> for Error {
    fn from(value: libloading::Error) -> Self {
        Self::LibLoading(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LibLoading(x) => x.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
