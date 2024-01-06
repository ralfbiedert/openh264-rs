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

/// Generated bindings for OpenH264.
mod generated {
    pub mod consts;
    pub mod fns_libloading;
    pub mod fns_source;
    pub mod types;
}

/// Abstraction over `source` or `libloading` APIs.
#[rustfmt::skip]
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
/// While this is no legal advice, the idea is that using this API might be covered by Cicos' [promise to cover MPEG-LA license costs](https://www.openh264.org/).
/// The big downside is you will have to download pre-build libraries from Cisco during installation. From [their FAQ](https://www.openh264.org/faq.html) (copied 2024-01-06), emphasis ours:
///
/// Q: If I use the source code in my product, and then distribute that product on my own, will Cisco cover the MPEG LA licensing fees which I'd otherwise have to pay?
/// A: No. Cisco is only covering the licensing fees for its own binary module, and products or projects that utilize it **must download it at the time the product or project is installed on the user's computer or device**. Cisco will not be liable for any licensing fees incurred by other parties.
///
/// In addition, note that this might not cover _all_ possible license claims:
///
/// Q: Is Cisco guaranteeing that it will pay other licensing fees for H.264, should additional patent holders assert claims in the future?
/// A: Cisco is providing no such guarantee. We are only covering the royalties that would apply to the binary module under MPEG LA's AVC/H.264 patent pool.
pub mod libloading {
    pub use crate::generated::fns_libloading::*;
    use crate::{ISVCDecoder, ISVCEncoder, OpenH264Version, SDecoderCapability};
    use std::os::raw::{c_int, c_long};

    #[rustfmt::skip]
    impl super::API for APIEntry {
        unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> c_int { APIEntry::WelsCreateSVCEncoder(self, ppEncoder) }
        unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) { APIEntry::WelsDestroySVCEncoder(self, pEncoder) }
        unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> c_int { APIEntry::WelsGetDecoderCapability(self, pDecCapability) }
        unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> c_long { APIEntry::WelsCreateDecoder(self, ppDecoder) }
        unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) { APIEntry::WelsDestroyDecoder(self, pDecoder) }
        unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version { APIEntry::WelsGetCodecVersion(self) }
        unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) {APIEntry::WelsGetCodecVersionEx(self, pVersion) }
    }
}

/// API surface using built-in source.
///
/// This API surface should _just work_ once compiled. Depending on your commercial, legal and geographic situation, and the H.264 features you use,
/// this might or might not come with an elevated patent risk.
pub mod source {
    use crate::{ISVCDecoder, ISVCEncoder, OpenH264Version, SDecoderCapability};
    use std::os::raw::{c_int, c_long};

    pub struct APIEntry {}

    #[rustfmt::skip]
    impl APIEntry {
        pub fn new() -> Self { Self {} }
        pub unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> ::std::os::raw::c_int { crate::generated::fns_source::WelsCreateSVCEncoder(ppEncoder) }
        pub unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) { crate::generated::fns_source::WelsDestroySVCEncoder(pEncoder) }
        pub unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> ::std::os::raw::c_int { crate::generated::fns_source::WelsGetDecoderCapability(pDecCapability) }
        pub unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> ::std::os::raw::c_long { crate::generated::fns_source::WelsCreateDecoder(ppDecoder) }
        pub unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) { crate::generated::fns_source::WelsDestroyDecoder(pDecoder) }
        pub unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version { crate::generated::fns_source::WelsGetCodecVersion() }
        pub unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) { crate::generated::fns_source::WelsGetCodecVersionEx(pVersion) }
    }

    #[rustfmt::skip]
    impl super::API for APIEntry {
        unsafe fn WelsCreateSVCEncoder(&self, ppEncoder: *mut *mut ISVCEncoder) -> c_int { APIEntry::WelsCreateSVCEncoder(self, ppEncoder) }
        unsafe fn WelsDestroySVCEncoder(&self, pEncoder: *mut ISVCEncoder) { APIEntry::WelsDestroySVCEncoder(self, pEncoder) }
        unsafe fn WelsGetDecoderCapability(&self, pDecCapability: *mut SDecoderCapability) -> c_int { APIEntry::WelsGetDecoderCapability(self, pDecCapability) }
        unsafe fn WelsCreateDecoder(&self, ppDecoder: *mut *mut ISVCDecoder) -> c_long { APIEntry::WelsCreateDecoder(self, ppDecoder) }
        unsafe fn WelsDestroyDecoder(&self, pDecoder: *mut ISVCDecoder) { APIEntry::WelsDestroyDecoder(self, pDecoder) }
        unsafe fn WelsGetCodecVersion(&self) -> OpenH264Version { APIEntry::WelsGetCodecVersion(self) }
        unsafe fn WelsGetCodecVersionEx(&self, pVersion: *mut OpenH264Version) { APIEntry::WelsGetCodecVersionEx(self, pVersion) }
    }
}

pub use self::generated::consts::*;
pub use self::generated::types::*;
