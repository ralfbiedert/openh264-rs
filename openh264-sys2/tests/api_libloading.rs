#![cfg(all(target_os = "linux", target_arch = "x86_64", feature = "libloading"))]

mod common;

use common::api_generic;
use openh264_sys2::{API, DynamicAPI, reference_library_name};

fn reference_library_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/reference")
        .join(reference_library_name())
}

#[test]
fn api_loader() {
    use openh264_sys2::libloading::APILoader;

    let api = unsafe { APILoader::new(reference_library_path()).unwrap() };
    api_generic(api);
}

#[test]
fn dynamic_api() {
    let api = DynamicAPI::from_blob_path(reference_library_path()).unwrap();
    let version = unsafe { api.WelsGetCodecVersion() };

    assert_eq!((version.uMajor, version.uMinor, version.uRevision), (2, 7, 0));
    api_generic(api);
}
