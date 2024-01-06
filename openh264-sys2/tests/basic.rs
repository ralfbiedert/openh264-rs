use openh264_sys2::API;
use std::ptr::null_mut;

fn api_generic(api: impl API) {
    unsafe {
        let rval = api.WelsCreateDecoder(null_mut());
        let version = api.WelsGetCodecVersion();

        assert_eq!(rval, 1);
        assert_eq!(version.uMajor, 2);
    }
}

#[test]
fn api_source() {
    let api = openh264_sys2::source::APIEntry::new();
    api_generic(api);
}

#[test]
fn api_libloading() {
    let api = unsafe {
        openh264_sys2::libloading::APIEntry::new(r"C:\Users\rb\Downloads\openh264-2.4.0-win64.dll\openh264-2.4.0-win64.dll")
            .unwrap()
    };
    api_generic(api);
}
