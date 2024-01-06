use openh264_sys2::{DynamicAPI, API};
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
#[cfg(feature = "source")]
fn api_source() {
    use openh264_sys2::source::APIEntry;
    let api = APIEntry::new();
    api_generic(api);
}

#[test]
#[ignore]
#[cfg(feature = "libloading")]
fn api_libloading() {
    use openh264_sys2::libloading::APIEntry;
    let api = unsafe { APIEntry::new(r"C:\Users\rb\Downloads\openh264-2.4.0-win64.dll\openh264-2.4.0-win64.dll").unwrap() };
    api_generic(api);
}

#[test]
#[cfg(feature = "source")]
fn api_dynamic() {
    let api = DynamicAPI::from_source();
    api_generic(api);
}
