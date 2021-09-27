use std::path::Path;
use walkdir::WalkDir;

// if this gets more complicated, we might want to inline the list of files available in meson
fn glob_import<P: AsRef<Path>>(root: P, extenstion: &str, exclude: &str) -> Vec<String> {
    WalkDir::new(root)
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| x.path().to_str().unwrap().ends_with(extenstion))
        .map(|x| x.path().to_str().unwrap().to_string())
        .filter(|x| !x.contains(exclude))
        .collect()
}

fn add_openh264_lib(name: &str, root: &str, includes: &[&str]) {
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
        .files(glob_import(root, ".cpp", "DllEntry.cpp")) // Otherwise fails when compiling on Linux
        .pic(true)
        // Upstream sets these two and if we don't we get segmentation faults on Linux and MacOS ... Happy times.
        .flag_if_supported("-fno-strict-aliasing")
        .flag_if_supported("-fstack-protector-all")
        .flag_if_supported("-fembed-bitcode")
        .flag_if_supported("-fno-common")
        .flag_if_supported("-undefined dynamic_lookup")
        .debug(debug);

    for include in includes {
        cc_build.include(include);
    }

    #[cfg(feature = "asm")]
    {
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

        if is_x86 {
            if let Ok(objs) = nasm_rs::Build::new()
                .include(format!("upstream/codec/common/{}/", asm_dir))
                .define(asm_define, None)
                .files(glob_import(root, extension, exclude))
                .compile_objects()
            {
                cc_build.define(cpp_define, None);
                for obj in &objs {
                    cc_build.object(obj);
                }
            } else {
                println!(
                    "cargo:warning=failed to build asm files, please check that NASM is available on the path and is at a recent version"
                );
            }
        } else {
            cc_build
                .include(format!("upstream/codec/common/{}/", asm_dir))
                .define(asm_define, None)
                .files(glob_import(root, extension, exclude));
        }
    }

    cc_build.compile(format!("libopenh264_{}.a", name).as_str());

    println!("cargo:rustc-link-lib=static=openh264_{}", name);
}

fn main() {
    add_openh264_lib("common", "upstream/codec/common", &[]);
    add_openh264_lib(
        "processing",
        "upstream/codec/processing",
        &[
            "upstream/codec/processing/src/common/",
            "upstream/codec/processing/interface/",
        ],
    );

    #[cfg(feature = "decoder")]
    add_openh264_lib(
        "decoder",
        "upstream/codec/decoder",
        &["upstream/codec/decoder/core/inc/", "upstream/codec/decoder/plus/inc/"],
    );

    #[cfg(feature = "encoder")]
    add_openh264_lib(
        "encoder",
        "upstream/codec/encoder",
        &[
            "upstream/codec/encoder/core/inc/",
            "upstream/codec/encoder/plus/inc/",
            "upstream/codec/processing/interface/",
        ],
    );

    #[cfg(not(any(feature = "decoder", feature = "encoder")))]
    panic!("at least one of 'decoder' or 'encoder' feature must be enabled");
}
