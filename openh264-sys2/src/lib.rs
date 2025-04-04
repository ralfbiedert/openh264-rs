//!
//! [![Latest Version]][crates.io]
//! [![docs]][docs.rs]
//! ![BSD-2]
//! [![Rust](https://img.shields.io/badge/rust-1.65%2B-blue.svg?maxAge=3600)](https://github.com/ralfbiedert/openh264-rust)
//!
//!
//! This low-level crate used by [openh264](https://crates.io/crates/openh264)
//! contains
//!
//! - a fully self-contained version of OpenH264
//! - alternatively, a libloading wrapper around precompiled OpenH264 binaries
//! - `unsafe` Rust bindings
//! - build logic that should work "out of the box" on most platforms (sans bugs)
//!
//! [Latest Version]: https://img.shields.io/crates/v/openh264-sys2.svg
//! [crates.io]: https://crates.io/crates/openh264-sys2
//! [BSD-2]: https://img.shields.io/badge/license-BSD2-blue.svg
//! [docs]: https://docs.rs/openh264-sys2/badge.svg
//! [docs.rs]: https://docs.rs/openh264-sys2/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

mod error;

/// Generated bindings for OpenH264.
mod generated {
    pub mod consts;
    #[cfg(feature = "libloading")]
    pub mod fns_libloading;
    #[cfg(feature = "source")]
    pub mod fns_source;
    pub mod types;
}

pub use self::generated::consts::*;
pub use self::generated::types::*;
pub use error::Error;
use std::os::raw::{c_int, c_long};

/// Abstraction over `source` or `libloading` APIs.
#[rustfmt::skip]
#[allow(clippy::missing_safety_doc)]
pub trait API {
    unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> ::std::os::raw::c_int;
    unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder);
    unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> ::std::os::raw::c_int;
    unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> ::std::os::raw::c_long;
    unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder);
    unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version;
    unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version);
}

/// API surface via libloading.
///
/// While this is no legal advice, the idea is that using this API might be covered by Cisco's [promise to cover MPEG-LA license costs](https://www.openh264.org/).
/// The big downside is you will have to download binary blobs from Cisco during installation. From [their FAQ](https://www.openh264.org/faq.html) (copied 2024-01-06, emphasis ours):
///
/// - **Q: If I use the source code in my product, and then distribute that product on my own, will Cisco cover the MPEG LA licensing fees which I'd otherwise have to pay?**
///
///     A: No. Cisco is only covering the licensing fees for its own binary module, and products or projects that utilize it **must download it at the time the product or project is installed on the user's computer or device**. Cisco will not be liable for any licensing fees incurred by other parties.
///
/// In addition, note that this might not cover _all_ possible license claims:
///
/// - **Q: Is Cisco guaranteeing that it will pay other licensing fees for H.264, should additional patent holders assert claims in the future?**
///
///     A: Cisco is providing no such guarantee. We are only covering the royalties that would apply to the binary module under MPEG LA's AVC/H.264 patent pool.
#[cfg(feature = "libloading")]
pub mod libloading {
    pub use crate::generated::fns_libloading::*;
    use crate::{ISVCDecoder, ISVCEncoder, OpenH264Version, SDecoderCapability};
    use std::os::raw::{c_int, c_long};

    #[rustfmt::skip]
    impl super::API for APILoader {
        unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> c_int { APILoader::WelsCreateSVCEncoder(self, ppEncoder) }
        unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) { APILoader::WelsDestroySVCEncoder(self, pEncoder) }
        unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> c_int { APILoader::WelsGetDecoderCapability(self, pDecCapability) }
        unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> c_long { APILoader::WelsCreateDecoder(self, ppDecoder) }
        unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) { APILoader::WelsDestroyDecoder(self, pDecoder) }
        unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version { APILoader::WelsGetCodecVersion(self) }
        unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) {APILoader::WelsGetCodecVersionEx(self, pVersion) }
    }
}

/// API surface using built-in source.
///
/// This API surface should _just work_ once compiled. Depending on your commercial, legal and geographic situation, and the H.264 features you use,
/// this might or might not come with an elevated patent risk.
#[cfg(feature = "source")]
pub mod source {
    use crate::{ISVCDecoder, ISVCEncoder, OpenH264Version, SDecoderCapability};
    use std::os::raw::{c_int, c_long};

    #[derive(Debug, Default)]
    pub struct APILoader {}

    #[rustfmt::skip]
    #[allow(clippy::missing_safety_doc)]
    impl APILoader {
        pub const fn new() -> Self { Self {} }
        pub unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> ::std::os::raw::c_int { crate::generated::fns_source::WelsCreateSVCEncoder(ppEncoder) }
        pub unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) { crate::generated::fns_source::WelsDestroySVCEncoder(pEncoder) }
        pub unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> ::std::os::raw::c_int { crate::generated::fns_source::WelsGetDecoderCapability(pDecCapability) }
        pub unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> ::std::os::raw::c_long { crate::generated::fns_source::WelsCreateDecoder(ppDecoder) }
        pub unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) { crate::generated::fns_source::WelsDestroyDecoder(pDecoder) }
        pub unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version { crate::generated::fns_source::WelsGetCodecVersion() }
        pub unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) { crate::generated::fns_source::WelsGetCodecVersionEx(pVersion) }
    }

    #[rustfmt::skip]
    #[allow(clippy::missing_safety_doc)]
    impl super::API for APILoader {
        unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> c_int { APILoader::WelsCreateSVCEncoder(self, ppEncoder) }
        unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) { APILoader::WelsDestroySVCEncoder(self, pEncoder) }
        unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> c_int { APILoader::WelsGetDecoderCapability(self, pDecCapability) }
        unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> c_long { APILoader::WelsCreateDecoder(self, ppDecoder) }
        unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) { APILoader::WelsDestroyDecoder(self, pDecoder) }
        unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version { APILoader::WelsGetCodecVersion(self) }
        unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) { APILoader::WelsGetCodecVersionEx(self, pVersion) }
    }
}

/// Convenience wrapper around `libloading` and `source` API surfaces.
///
/// This type mainly exists to prevent infecting the rest of the OpenH264 crate with generics. The dispatch overhead
/// in contrast to H.264 computation is absolutely negligible.
pub enum DynamicAPI {
    #[cfg(feature = "source")]
    Source(source::APILoader),

    #[cfg(feature = "libloading")]
    Libloading(libloading::APILoader),
}

impl DynamicAPI {
    /// Creates an OpenH264 API using the built-in source if available.
    #[cfg(feature = "source")]
    pub const fn from_source() -> Self {
        let api = crate::source::APILoader::new();
        Self::Source(api)
    }

    /// Creates an OpenH264 API via the provided shared library.
    ///
    /// In order for this to have any (legal) use, you should download the library from
    /// Cisco [**during installation**](https://www.openh264.org/faq.html), and then
    /// pass the file-system path in here.
    ///
    /// # Errors
    ///
    /// Can fail if the library could not be loaded, e.g., it does not exist.
    ///
    /// # Safety
    ///
    /// Will cause UB if the provided path does not match the current platform and version.
    #[cfg(feature = "libloading")]
    pub unsafe fn from_blob_path_unchecked(path: impl AsRef<std::ffi::OsStr>) -> Result<Self, Error> {
        let api = unsafe { libloading::APILoader::new(path)? };
        Ok(Self::Libloading(api))
    }

    /// Creates an OpenH264 API via the provided shared library if the library is well-known.
    ///
    /// In order for this to have any (legal) use, you should download the library from
    /// Cisco [**during installation**](https://www.openh264.org/faq.html), and then
    /// pass the file-system path in here.
    ///
    /// This function also checks the file's SHA against a list of well-known libraries we can load.
    ///
    /// # Errors
    ///
    /// Can fail if the library could not be loaded, e.g., it does not exist. It can also fail if the file's SHA does
    /// not match against a list of well-known versions we can load.
    #[cfg(feature = "libloading")]
    pub fn from_blob_path(path: impl AsRef<std::ffi::OsStr>) -> Result<Self, Error> {
        use sha2::Digest;
        use std::fmt::Write;

        let bytes = std::fs::read(path.as_ref())?;

        // Get SHA of blob at given path.
        let sha256 = sha2::Sha256::digest(bytes).iter().fold(String::new(), |mut acc, byte| {
            write!(&mut acc, "{:02x}", byte).unwrap(); // Unless we're out of memory this should never panic.
            acc
        });

        // Check all known hashes if we should load this library.
        // TODO: We might also want to verify this matches our architecture, but then again libloading should catch that.
        let hash_is_well_known = include_str!("blobs/hashes.txt")
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .any(|x| x == sha256);

        if !hash_is_well_known {
            return Err(Error::InvalidHash(sha256));
        }

        unsafe { Self::from_blob_path_unchecked(path) }
    }
}

#[allow(unreachable_patterns)]
#[allow(unused)]
impl API for DynamicAPI {
    unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> c_int {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsCreateSVCEncoder(ppEncoder),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsCreateSVCEncoder(ppEncoder),
            _ => panic!("No API enabled"),
        }
    }

    unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsDestroySVCEncoder(pEncoder),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsDestroySVCEncoder(pEncoder),
            _ => panic!("No API enabled"),
        }
    }

    unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> c_int {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsGetDecoderCapability(pDecCapability),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsGetDecoderCapability(pDecCapability),
            _ => panic!("No API enabled"),
        }
    }

    unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> c_long {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsCreateDecoder(ppDecoder),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsCreateDecoder(ppDecoder),
            _ => panic!("No API enabled"),
        }
    }

    unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsDestroyDecoder(pDecoder),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsDestroyDecoder(pDecoder),
            _ => panic!("No API enabled"),
        }
    }

    unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsGetCodecVersion(),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsGetCodecVersion(),
            _ => panic!("No API enabled"),
        }
    }

    unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) {
        match self {
            #[cfg(feature = "source")]
            DynamicAPI::Source(api) => api.WelsGetCodecVersionEx(pVersion),
            #[cfg(feature = "libloading")]
            DynamicAPI::Libloading(api) => api.WelsGetCodecVersionEx(pVersion),
            _ => panic!("No API enabled"),
        }
    }
}

/// Helper function that should always give the name of the latest supported and
/// included DLL file, used by unit tests.
#[doc(hidden)]
#[cfg(all(target_os = "windows", target_arch = "x86_64", feature = "libloading"))]
pub fn reference_dll_name() -> &'static str {
    include_str!("../tests/reference/reference.txt")
}
