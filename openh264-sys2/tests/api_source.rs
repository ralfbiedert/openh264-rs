use crate::common::api_generic;
use openh264_sys2::DynamicAPI;

mod common;

#[test]
#[cfg(feature = "source")]
fn api_loader() {
    use openh264_sys2::source::APILoader;
    let api = APILoader::new();
    api_generic(api);
}

#[test]
#[cfg(feature = "source")]
fn dynamic_api() {
    let api = DynamicAPI::from_source();
    api_generic(api);
}

#[test]
#[cfg(feature = "source")]
fn codec_version_matches_vendored_source() {
    use openh264_sys2::{OpenH264Version, source::APILoader};

    // SAFETY: The statically linked version query takes no pointers or other inputs.
    let version = unsafe { APILoader::WelsGetCodecVersion() };

    assert_eq!(
        version,
        OpenH264Version {
            uMajor: 2,
            uMinor: 7,
            uRevision: 0,
            uReserved: 2610,
        }
    );
}
