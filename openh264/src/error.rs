use openh264_sys2::DECODING_STATE;
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
            decoding_state: DECODING_STATE::dsErrorFree,
            misc: None,
        }
    }

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
            decoding_state: DECODING_STATE::dsErrorFree,
            misc: Some(msg),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("OpenH264 encountered an error (TODO: improve this message): ")?;
        <i64 as std::fmt::Display>::fmt(&self.native, f)?;
        self.decoding_state.fmt(f)?;
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

impl NativeErrorExt for DECODING_STATE {
    fn ok(self) -> Result<(), Error> {
        if self == DECODING_STATE::dsErrorFree {
            Ok(())
        } else {
            Err(Error::from_decoding_state(self))
        }
    }
}
