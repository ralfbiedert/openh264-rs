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
