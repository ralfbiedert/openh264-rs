use walkdir::WalkDir;

fn ugly_cpp_import(x: &str) -> Vec<String> {
    WalkDir::new(x)
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| x.path().to_str().unwrap().ends_with("cpp"))
        .map(|x| x.path().to_str().unwrap().to_string())
        // Otherwise fails when compiling on Linux
        .filter(|x| !x.contains("DllEntry.cpp"))
        .collect()
}

fn main() {
    cc::Build::new()
        .include("upstream/codec/api/svc/")
        .include("upstream/codec/common/inc/")
        .include("upstream/codec/decoder/core/inc/")
        .include("upstream/codec/decoder/plus/inc/")
        .include("upstream/codec/processing/interface/")
        .files(ugly_cpp_import("upstream/codec/common"))
        .files(ugly_cpp_import("upstream/codec/decoder"))
        .cpp(true)
        .warnings(false)
        .compile("libopenh264_decode.a");

    cc::Build::new()
        .include("upstream/codec/api/svc/")
        .include("upstream/codec/common/inc/")
        .include("upstream/codec/encoder/core/inc/")
        .include("upstream/codec/encoder/plus/inc/")
        .include("upstream/codec/processing/interface/")
        .include("upstream/codec/processing/src/common/")
        .files(ugly_cpp_import("upstream/codec/encoder"))
        .files(ugly_cpp_import("upstream/codec/processing"))
        .cpp(true)
        .warnings(false)
        .compile("libopenh264_encode.a");

    println!("cargo:rustc-link-lib=static=openh264_encode");
    println!("cargo:rustc-link-lib=static=openh264_decode");
    println!("cargo:rerun-if-env-changed=XXXXXXXXXXXXXXX");
}
