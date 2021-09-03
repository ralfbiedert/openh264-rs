use openh264_sys2::{dsErrorFree, DECODING_STATE};
use std::fmt::{Debug, Display, Formatter};
use std::os::raw::{c_long, c_ulong};

/// Error struct if something goes wrong.
#[derive(Debug, Copy, Clone)]
pub struct Error {
    native: i64,
    decoding_state: DECODING_STATE,
    misc: Option<&'static str>,
}

impl Error {
    pub(crate) fn from_native(native: i64) -> Self {
        Error {
            native,
            decoding_state: dsErrorFree,
            misc: None,
        }
    }

    #[allow(unused)]
    pub(crate) fn from_decoding_state(decoding_state: DECODING_STATE) -> Self {
        Error {
            native: 0,
            decoding_state,
            misc: None,
        }
    }

    pub(crate) fn msg(msg: &'static str) -> Self {
        Error {
            native: 0,
            decoding_state: dsErrorFree,
            misc: Some(msg),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenH264 encountered an error (TODO: improve this message): ")?;
        <i64 as std::fmt::Display>::fmt(&self.native, f)?;
        <std::os::raw::c_int as std::fmt::Display>::fmt(&self.decoding_state, f)?;
        self.misc.fmt(f)?;
        Ok(())
    }
}

impl std::error::Error for Error {}

/// Helper trait to check the various error values produced by OpenH264.
pub(crate) trait NativeErrorExt {
    fn ok(self) -> Result<(), Error>;
}

impl NativeErrorExt for c_ulong {
    fn ok(self) -> Result<(), Error> {
        if self == 0 {
            Ok(())
        } else {
            Err(Error::from_native(self as i64))
        }
    }
}

impl NativeErrorExt for c_long {
    fn ok(self) -> Result<(), Error> {
        if self == 0 {
            Ok(())
        } else {
            Err(Error::from_native(self as i64))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Error;
    use openh264_sys2::dsRefListNullPtrs;

    #[test]
    fn errors_wont_panic() {
        dbg!(Error::from_native(1));
        dbg!(Error::from_decoding_state(dsRefListNullPtrs));
        dbg!(Error::msg("hello world"));

        println!("{}", Error::from_native(1));
        println!("{}", Error::from_decoding_state(dsRefListNullPtrs));
        println!("{}", Error::msg("hello world"));

        println!("{:?}", Error::from_native(1));
        println!("{:?}", Error::from_decoding_state(dsRefListNullPtrs));
        println!("{:?}", Error::msg("hello world"));

        println!("{:#?}", Error::from_native(1));
        println!("{:#?}", Error::from_decoding_state(dsRefListNullPtrs));
        println!("{:#?}", Error::msg("hello world"));
    }
}
