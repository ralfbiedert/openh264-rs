use crate::common::api_generic;
use openh264_sys2::reference_dll_name;

mod common;

#[test]
#[cfg(all(target_os = "windows", target_arch = "x86_64", feature = "libloading"))]
fn api_loader() {
    use openh264_sys2::libloading::APILoader;
    let file = format!("./tests/reference/{}", reference_dll_name());
    let api = unsafe { APILoader::new(file).unwrap() };
    api_generic(api);
}

