use openh264_sys2::{dsErrorFree, DECODING_STATE};
use std::fmt::{Debug, Display, Formatter};
use std::num::TryFromIntError;

/// Error struct if something goes wrong.
#[derive(Debug)]
pub struct Error {
    native: i64,
    decoding_state: DECODING_STATE,
    misc: Option<String>,
    backtrace: Option<std::backtrace::Backtrace>,
}

impl Error {
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn from_native(native: i64) -> Self {
        Self {
            native,
            decoding_state: dsErrorFree,
            misc: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        }
    }

    #[allow(unused)]
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn from_decoding_state(decoding_state: DECODING_STATE) -> Self {
        Self {
            native: 0,
            decoding_state,
            misc: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        }
    }

    /// Creates a new [`Error`] with a custom message.
    #[must_use]
    pub fn msg(msg: &str) -> Self {
        Self {
            native: 0,
            decoding_state: dsErrorFree,
            misc: Some(msg.to_string()),
            backtrace: Some(std::backtrace::Backtrace::capture()),
        }
    }

    /// Creates a new [`Error`] with a custom message.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn msg_string(msg: String) -> Self {
        Self {
            native: 0,
            decoding_state: dsErrorFree,
            misc: Some(msg),
            backtrace: Some(std::backtrace::Backtrace::capture()),
        }
    }

    /// Returns the backtrace, if available.
    #[allow(clippy::missing_const_for_fn)]
    pub const fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        self.backtrace.as_ref()
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self::msg_string(format!("Could not covert value: {value}"))
    }
}

impl From<openh264_sys2::Error> for Error {
    fn from(value: openh264_sys2::Error) -> Self {
        Self::msg_string(format!("open264-sys error: {value}"))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenH264 encountered an error. Native:")?;
        <i64 as std::fmt::Display>::fmt(&self.native, f)?;
        f.write_str(". Decoding State:")?;
        <std::os::raw::c_int as std::fmt::Display>::fmt(&self.decoding_state, f)?;
        f.write_str(". User Message:")?;
        self.misc.fmt(f)?;

        {
            f.write_str(". Backtraces enabled.")?;
        }
        Ok(())
    }
}

/// Helper trait to check the various error values produced by OpenH264.
pub trait NativeErrorExt {
    fn ok(self) -> Result<(), Error>;
}

macro_rules! impl_native_error {
    ($t:ty) => {
        impl NativeErrorExt for $t {
            #[allow(clippy::cast_lossless)]
            fn ok(self) -> Result<(), Error> {
                if self == 0 {
                    Ok(())
                } else {
                    Err(Error::from_native(self as i64))
                }
            }
        }
    };
}

impl_native_error!(u64);
impl_native_error!(i64);
impl_native_error!(i32);

impl std::error::Error for Error {}

#[cfg(test)]
mod test {
    use crate::Error;
    use openh264_sys2::dsRefListNullPtrs;

    #[test]
    #[allow(unused_must_use)]
    fn errors_wont_panic() {
        format!("{}", Error::from_native(1));
        format!("{}", Error::from_decoding_state(dsRefListNullPtrs));
        format!("{}", Error::msg("hello world"));

        format!("{:?}", Error::from_native(1));
        format!("{:?}", Error::from_decoding_state(dsRefListNullPtrs));
        format!("{:?}", Error::msg("hello world"));

        format!("{:#?}", Error::from_native(1));
        format!("{:#?}", Error::from_decoding_state(dsRefListNullPtrs));
        format!("{:#?}", Error::msg("hello world"));
    }

    #[test]
    fn backtrace_works() {
        _ = Error::from_native(1).backtrace.expect("Must have backtrace");
    }
}
