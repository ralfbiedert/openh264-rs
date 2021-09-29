#![cfg_attr(feature = "backtrace", feature(backtrace))]
#![cfg_attr(docsrs, feature(doc_cfg))]
//!
//! [![Latest Version]][crates.io]
//! [![docs]][docs.rs]
//! ![BSD-2]
//! [![Rust](https://img.shields.io/badge/rust-1.53%2B-blue.svg?maxAge=3600)](https://github.com/ralfbiedert/openh264-rust)
//! [![Rust](https://github.com/ralfbiedert/openh264-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/ralfbiedert/openh264-rust/actions/workflows/rust.yml)
//!
//! # OpenH264 Rust API
//!
//! Idiomatic and low-level bindings for [OpenH264](https://github.com/cisco/openh264), converting between these two in Rust:
//!
//! ![sample_image](https://media.githubusercontent.com/media/ralfbiedert/openh264-rust/master/gfx/title2.jpg)
//!
//!
//! ## Example API
//!
//! **Decode** some H.264 bitstream to YUV:
//! ```rust
//! use openh264::decoder::Decoder;
//! use openh264::nal_units;
//!
//! # use openh264::Error;
//! # fn main() -> Result<(), Error> {
//! let h264_in = include_bytes!("../tests/data/multi_512x512.h264");
//! let mut decoder = Decoder::new()?;
//!
//! // Split H.264 into NAL units and decode each.
//! for packet in nal_units(h264_in) {
//!     let yuv = decoder.decode(packet)?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//!
//! And **encode** the same YUV back to H.264:
//! ```rust
//! # use openh264::decoder::Decoder;
//! # use openh264::Error;
//! use openh264::encoder::{Encoder, EncoderConfig};
//! # fn main() -> Result<(), Error> {
//! # let mut decoder = Decoder::new()?;
//! # let mut rgb_out = vec![0; 512 * 512 * 3];
//! # let h264_in = include_bytes!("../tests/data/multi_512x512.h264");
//! # let yuv = decoder.decode(&h264_in[..])?;
//!
//! let config = EncoderConfig::new(512, 512);
//! let mut encoder = Encoder::with_config(config)?;
//!
//! // Encode YUV back into H.264.
//! let bitstream = encoder.encode(&yuv)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Platform Support
//!
//! Test results on various platforms:
//!
//! | Platform | Compiled | Unit Tested |
//! | --- | --- | --- |
//! | `x86_64-pc-windows-msvc` | ✅ | ✅ |
//! | `x86_64-unknown-linux-gnu` | ✅ | ✅ |
//! | `x86_64-apple-darwin` | ✅ | ✅ |
//! | `aarch64-linux-android` | 🆗<sup>1</sup>  | - |
//! | `wasm32-unknown-unknown` | ❌<sup>1,2</sup> | - |
//!
//! ✅ works out of the box;
//! 🆗 the usual shenanigans required;
//! ❌ not supported.
//!
//! <sup>1</sup> via `cargo build --target <platform>`, [needs `CXX` set](https://cheats.rs/#cross-compilation) and `libc++_shared.so`. <br/>
//! <sup>2</sup> unclear if could ever work, investigation welcome
//!
//!
//! ## Performance
//!
//! Tested on a i9-9900K, Windows 10, single threaded de- and encoding:
//!
//! ```text
//! -- Default --
//! test decode_yuv_single_1920x1080     ... bench:   9,243,380 ns/iter (+/- 497,200)
//! test decode_yuv_single_512x512_cabac ... bench:   1,841,775 ns/iter (+/- 53,211)
//! test decode_yuv_single_512x512_cavlc ... bench:   2,076,030 ns/iter (+/- 7,287)
//! test encode_1920x1080_from_yuv       ... bench:  38,657,620 ns/iter (+/- 793,310)
//! test encode_512x512_from_yuv         ... bench:   6,420,605 ns/iter (+/- 1,003,485)
//!
//! -- Feature `asm` --
//! test decode_yuv_single_1920x1080     ... bench:   4,265,260 ns/iter (+/- 89,438)
//! test decode_yuv_single_512x512_cabac ... bench:     901,025 ns/iter (+/- 21,902)
//! test decode_yuv_single_512x512_cavlc ... bench:   1,618,880 ns/iter (+/- 53,713)
//! test encode_1920x1080_from_yuv       ... bench:  13,455,160 ns/iter (+/- 862,042)
//! test encode_512x512_from_yuv         ... bench:   4,011,700 ns/iter (+/- 2,094,471)
//!
//! -- Color Conversion --
//! test convert_yuv_to_rgb_1920x1080    ... bench:   7,226,290 ns/iter (+/- 110,871)
//! test convert_yuv_to_rgb_512x512      ... bench:     907,340 ns/iter (+/- 28,296)
//! ```
//!
//! ## Compile Features
//!
//! - `decoder` - Enable the decoder. Used by default.
//! - `encoder` - Enable the encoder. Used by default.
//! - `backtrace` - Enable backtraces on errors (requires nightly)
//! - `asm` - Enable assembly. Only supported on `x86` and `ARM`, requires `nasm` installed.
//!
//! ## FAQ
//!
//! - **How does `openh264-sys2` differ from `openh264-sys`?**
//!
//!   We directly ship OpenH264 source code and provide simple, hand-crafted compilation via `cc` in `build.rs`. Our`openh264-sys2` crate should compile via `cargo build` out of the box on most platforms, and cross-compile via `cargo build --target ...` as
//!   long as the environment variable `CXX` is properly set.
//!
//!
//! - **I need to fix an important OpenH264 security hole, how can I update the library?**
//!
//!   Cisco's OpenH264 library is contained in `openh264-sys2/upstream`. Updating is (almost, see below) as simple as [pulling their latest source](https://github.com/cisco/openh264),
//!   copying it into that directory, and manually removing all "resource" files. We probably should have a script to strip that folder automatically ...
//!
//!
//! - **I heard Rust is super-safe, will this make decoding my videos safe too?**
//!
//!   No. Below a thin Rust layer we rely on a _very complex_ C library, and an equally complex standard. Apart from Rust being a
//!   much nicer language to work with, depending on this  project will give you _no_ additional safety guarantees as far as video
//!   handling is concerned. FYI, this is _not_ making a statement about OpenH264, but about the realities of securing +50k lines
//!   of C against attacks.
//!
//!
//! - **Feature X is missing or broken, will you fix it?**
//!
//!   Right now I only have time to implement what I need. However, I will gladly accept PRs either extending the APIs, or fixing bugs; see below.
//!
//!
//! - **Decoder::decode() returned an error, is this a bug?**
//!
//!   Maybe. Probably not. Some encoders can write data OpenH264 doesn't understand, and if _all_ frames fail this could either
//!   be your encoder doing exotic things, OpenH264 not having implemented a certain feature, or
//!   us having a bug.
//!
//!   If only _some_ frames fail the most likely reasons are your endoder injecting _some_ special
//!   packets or transmission errors. In other words, unless you have a very controlled setup you should not terminate on
//!   the first error(s), but simply continue decoding and hope for the decoder to recover.
//!
//!   FWIW, we consider OpenH264's `h264dec` the reference decoder. If you can get it to emit YUV it would be a bug
//!   if we can't. However, any stream / frame it fails on is pretty much a _wontfix_ for us.
//!
//! ## OpenH264 Patches Applied
//!
//! Ideally the embedded upstream should be pristine. That said, the following
//! patches have been applied to fix Valgrind issues and crashes on some platforms:
//!
//! - `decoder.cpp` - removed `if (pCtx->pDstInfo) pCtx->pDstInfo->iBufferStatus = 0;` which seems to write to previously deallocated memory.
//!
//! Help with upstreaming them would be appreciated.
//!
//!
//! ## Contributing
//!
//! PRs are very welcome. Feel free to submit PRs and fixes right away. You can open issues if you want to discuss things, but due to time restrictions on my side the project will have to rely on people contributing.
//!
//! Especially needed:
//!
//! - [ ] BT.601 / BT.709 YUV <-> RGB Conversion
//! - [ ] Faster YUV to RGB conversion
//! - [ ] Have script to automatically update / import OpenH264 source (or submodule?)
//! - [ ] WASM investigation (either patch, or evidence it can't be fixed)
//! - [ ] Submit patches upstream
//! - [ ] Feedback which platforms successfully built on
//!
//!
//! ## Changelog
//!
//! - **v0.2** - Added encoder; `asm` feature for 2x - 3x speed boost.
//! - **v0.1** - Initial release, decoder only.
//!
//! ## License
//!
//! - OpenH264 core library is [BSD-2](openh264-sys2/upstream/LICENSE), Cisco.
//! - Wrapper code is [BSD-2](https://opensource.org/licenses/BSD-2-Clause), Ralf Biedert.
//!
//! [Latest Version]: https://img.shields.io/crates/v/openh264.svg
//! [crates.io]: https://crates.io/crates/openh264
//! [BSD-2]: https://img.shields.io/badge/license-BSD2-blue.svg
//! [docs]: https://docs.rs/openh264/badge.svg
//! [docs.rs]: https://docs.rs/openh264/

mod error;
mod utils;

pub mod formats;

#[cfg(feature = "decoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "decoder")))]
pub mod decoder;

#[cfg(feature = "encoder")]
#[cfg_attr(docsrs, doc(cfg(feature = "encoder")))]
pub mod encoder;

pub use error::Error;
pub use utils::nal_units;
