use openh264_sys2::API;
use std::ptr::null_mut;

#[allow(dead_code)]
pub fn api_generic(api: impl API) {
    unsafe {
        let rval = api.WelsCreateDecoder(null_mut());
        let version = api.WelsGetCodecVersion();

        assert_eq!(rval, 1);
        assert_eq!(version.uMajor, 2);
    }
}
