use openh264_sys2::DECODING_STATE;

#[derive(Debug, Copy, Clone)]
pub struct Error {
    native: i64,
    decoding_state: DECODING_STATE,
    misc: Option<&'static str>,
}

impl Error {
    pub fn from_native(native: std::os::raw::c_ulong) -> Self {
        Error {
            native: native as i64,
            decoding_state: DECODING_STATE::dsErrorFree,
            misc: None,
        }
    }

    pub fn from_decoding_state(decoding_state: DECODING_STATE) -> Self {
        Error {
            native: 0,
            decoding_state,
            misc: None,
        }
    }

    pub fn msg(msg: &'static str) -> Self {
        Error {
            native: 0,
            decoding_state: DECODING_STATE::dsErrorFree,
            misc: Some(msg),
        }
    }
}

pub trait NativeErrorExt {
    fn ok(self) -> Result<(), Error>;
}

impl NativeErrorExt for std::os::raw::c_ulong {
    fn ok(self) -> Result<(), Error> {
        if self == 0 {
            Ok(())
        } else {
            Err(Error::from_native(self))
        }
    }
}

impl NativeErrorExt for i32 {
    fn ok(self) -> Result<(), Error> {
        if self == 0 {
            Ok(())
        } else {
            Err(Error::from_native(self as u32))
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
