//!
//! [![Latest Version]][crates.io]
//! [![docs]][docs.rs]
//! ![BSD-2]
//! [![Rust](https://img.shields.io/badge/rust-1.53%2B-blue.svg?maxAge=3600)](https://github.com/ralfbiedert/openh264-rust)
//!
//!
//! Low level crate used by [openh264](https://crates.io/crates/openh264).
//!
//! [Latest Version]: https://img.shields.io/crates/v/openh264-sys2.svg
//! [crates.io]: https://crates.io/crates/openh264-sys2
//! [BSD-2]: https://img.shields.io/badge/license-BSD2-blue.svg
//! [docs]: https://docs.rs/openh264-sys2/badge.svg
//! [docs.rs]: https://docs.rs/openh264-sys2/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod generated;

pub use generated::*;
