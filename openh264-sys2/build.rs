use std::path::Path;
use walkdir::WalkDir;

fn ugly_import<P: AsRef<Path>>(x: P, extenstion: &str, exclude: &str) -> Vec<String> {
    WalkDir::new(x)
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| x.path().to_str().unwrap().ends_with(extenstion))
        .map(|x| x.path().to_str().unwrap().to_string())
        .filter(|x| !x.contains(exclude))
        .collect()
}

fn build_library(name: &str, root: &str, extra_inclues: &[&str]) {
    let mut debug = false;
    let mut opt_level = 3;

    if std::env::var("PROFILE").unwrap().contains("debug") {
        debug = true;
        opt_level = 0;
    }

    let mut cc_build = cc::Build::new();
    cc_build
        .include("upstream/codec/api/svc/")
        .include("upstream/codec/common/inc/")
        .cpp(true)
        .warnings(false)
        .opt_level(opt_level)
        .files(ugly_import(root, "cpp", "DllEntry.cpp")) // Otherwise fails when compiling on Linux
        .pic(true)
        // Upstream sets these two and if we don't we get segmentation faults on Linux and MacOS ... Happy times.
        .flag_if_supported("-fno-strict-aliasing")
        .flag_if_supported("-fstack-protector-all")
        .flag_if_supported("-fembed-bitcode")
        .flag_if_supported("-fno-common")
        .flag_if_supported("-undefined dynamic_lookup")
        .debug(debug);

    for include in extra_inclues {
        cc_build.include(include);
    }

    if cfg!(feature = "asm") {
        let target = std::env::var("TARGET").unwrap();

        let is_64bits = target.starts_with("x86_64") || target.starts_with("aarch64");
        let is_x86 = target.starts_with("x86_64") || target.starts_with("i686");
        let is_arm = target.starts_with("arm") || target.starts_with("arm7") || target.starts_with("aarch64");
        let is_windows = target.contains("windows");
        let is_unix = target.contains("apple") || target.contains("linux");

        let (extension, cpp_define, asm_dir, asm_define, exclude) = if is_x86 {
            let extension = ".asm";
            let cpp_define = "X86_ASM";
            let asm_dir = "x86";
            let exclusion = "asm_inc.asm";
            if is_windows && is_64bits {
                (extension, cpp_define, asm_dir, "WIN64", exclusion)
            } else if is_unix && is_64bits {
                (extension, cpp_define, asm_dir, "UNIX64", exclusion)
            } else {
                (extension, cpp_define, asm_dir, "X86_32", exclusion)
            }
        } else if is_arm {
            let extension = ".S";
            if is_64bits {
                (
                    extension,
                    "HAVE_NEON_AARCH64",
                    "arm64",
                    "HAVE_NEON_AARCH64",
                    "arm_arch64_common_macro.S",
                )
            } else {
                (extension, "HAVE_NEON", "arm", "HAVE_NEON", "arm_arch_common_macro.S")
            }
        } else {
            panic!("");
        };

        cc_build.define(cpp_define, None);

        if is_x86 {
            let objs = nasm_rs::Build::new()
                .include(format!("upstream/codec/common/{}/", asm_dir))
                .define(asm_define, None)
                .files(ugly_import(root, extension, exclude))
                .compile_objects()
                .unwrap();
            for obj in &objs {
                cc_build.object(obj);
            }
        } else {
            cc_build
                .include(format!("upstream/codec/common/{}/", asm_dir))
                .define(asm_define, None)
                .files(ugly_import(root, extension, exclude));
        }
    }

    cc_build.compile(format!("libopenh264_{}.a", name).as_str());

    println!("cargo:rustc-link-lib=static=openh264_{}", name);
}

fn main() {
    build_library("common", "upstream/codec/common", &[]);
    if cfg!(feature = "decoder") {
        build_library(
            "decoder",
            "upstream/codec/decoder",
            &["upstream/codec/decoder/core/inc/", "upstream/codec/decoder/plus/inc/"],
        );
    }
    if cfg!(feature = "encoder") {
        build_library(
            "processing",
            "upstream/codec/processing",
            &[
                "upstream/codec/processing/src/common/",
                "upstream/codec/processing/interface/",
            ],
        );
        build_library(
            "encoder",
            "upstream/codec/encoder",
            &[
                "upstream/codec/encoder/core/inc/",
                "upstream/codec/encoder/plus/inc/",
                "upstream/codec/processing/interface/",
            ],
        );
    }
}
