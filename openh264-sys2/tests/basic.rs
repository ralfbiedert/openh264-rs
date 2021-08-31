use std::ptr::null_mut;

#[test]
fn can_call_simple_decoder() {
    unsafe {
        let rval = openh264_sys2::WelsCreateDecoder(null_mut());
        let version = openh264_sys2::WelsGetCodecVersion();

        assert_eq!(rval, 1);
        assert_eq!(version.uMajor, 2);
    }
}
