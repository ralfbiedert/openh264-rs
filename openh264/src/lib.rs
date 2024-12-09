#![cfg_attr(docsrs, feature(doc_cfg))]
//!
//! [![Latest Version]][crates.io]
//! [![docs]][docs.rs]
//! ![BSD-2]
//! [![Rust](https://img.shields.io/badge/rust-1.65%2B-blue.svg?maxAge=3600)](https://github.com/ralfbiedert/openh264-rust)
//! [![Rust](https://github.com/ralfbiedert/openh264-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/ralfbiedert/openh264-rust/actions/workflows/rust.yml)
//!
//! # OpenH264 Rust API
//!
//! Idiomatic and low-level bindings for [OpenH264](https://github.com/cisco/openh264), converting between these two in Rust:
//!
//! ![sample_image](https://media.githubusercontent.com/media/ralfbiedert/openh264-rs/master/gfx/title3.jpg)
//!
//!
//! ## Example API
//!
//! **Decode** some H.264 bitstream to YUV:
//! ```rust
//! use openh264::decoder::Decoder;
//! use openh264::nal_units;
//!
//! # use openh264::{Error, OpenH264API};
//! # fn main() -> Result<(), Error> {
//! let h264_in = include_bytes!("../tests/data/multi_512x512.h264");
//! let mut decoder = Decoder::new()?;
//!
//! // Split H.264 into NAL units and decode each.
//! for packet in nal_units(h264_in) {
//!     // On the first few frames this may fail, so you should check the result
//!     // a few packets before giving up.
//!     let maybe_some_yuv = decoder.decode(packet);
//! }
//! # Ok(())
//! # }
//! ```
//!
//!
//! And **encode** the same YUV back to H.264:
//! ```rust
//! # use openh264::decoder::Decoder;
//! # use openh264::{Error, OpenH264API};
//! use openh264::encoder::Encoder;
//! # fn main() -> Result<(), Error> {
//! # let mut decoder = Decoder::new()?;
//! # let mut rgb_out = vec![0; 512 * 512 * 3];
//! # let h264_in = include_bytes!("../tests/data/multi_512x512.h264");
//! # let yuv = decoder.decode(&h264_in[..])?.ok_or_else(|| Error::msg("Must have image"))?;
//!
//! let mut encoder = Encoder::new()?;
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
//! | `x86_64-pc-windows-gnu` | ✅ | ✅ |
//! | `x86_64-unknown-linux-gnu` | ✅ | ✅ |
//! | `x86_64-apple-darwin` | ✅ | ✅ |
//! | `i686-unknown-linux-gnu` | ✅ | ✅ |
//! | `i686-pc-windows-msvc` | ✅ | ✅ |
//! | `i686-pc-windows-gnu` | ✅ | ✅ |
//! | `armv7-unknown-linux-gnueabihf` | ✅ | - |
//! | `aarch64-unknown-linux-gnu` | ✅ | - |
//! | `aarch64-apple-darwin` | ✅ | - |
//! | `aarch64-pc-windows-msvc` | ✅ | - |
//! | `aarch64-linux-android` | 🆗<sup>1</sup>  | - |
//! | `wasm32-unknown-unknown` | ❌<sup>2</sup> | - |
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
//! Tested on a Ryzen 9 7950X3D, Windows 11, single threaded de- and encoding:
//!
//! ```text
//! -- Default --
//! test decode_yuv_single_1920x1080     ... bench:   5,696,370.00 ns/iter (+/- 1,892,038.50)
//! test decode_yuv_single_512x512_cabac ... bench:   1,103,065.00 ns/iter (+/- 49,763.50)
//! test decode_yuv_single_512x512_cavlc ... bench:   1,358,595.00 ns/iter (+/- 52,667.00)
//! test encode_1920x1080_from_yuv       ... bench:  23,720,860.00 ns/iter (+/- 1,610,097.00)
//! test encode_512x512_from_yuv         ... bench:   3,954,905.00 ns/iter (+/- 566,698.00)
//!
//! -- If `nasm` available --
//! test decode_yuv_single_1920x1080     ... bench:   2,799,800.00 ns/iter (+/- 291,731.25)
//! test decode_yuv_single_512x512_cabac ... bench:     532,370.00 ns/iter (+/- 33,115.00)
//! test decode_yuv_single_512x512_cavlc ... bench:   1,038,490.00 ns/iter (+/- 56,953.25)
//! test encode_1920x1080_from_yuv       ... bench:   8,178,290.00 ns/iter (+/- 1,325,363.50)
//! test encode_512x512_from_yuv         ... bench:   1,828,287.50 ns/iter (+/- 190,976.50)
//!
//! -- Color Conversion if "target-cpu=native" --
//! test convert_yuv_to_rgb_1920x1080    ... bench:   1,510,065.00 ns/iter (+/- 25,921.00)
//! test convert_yuv_to_rgb_512x512      ... bench:     187,495.00 ns/iter (+/- 2,758.75)
//! ```
//!
//! ## Compile Features
//!
//! - `source` - Uses the bundled OpenH264 source; works out of the box (default).
//! - `libloading` - You'll need to provide Cisco's prebuilt library.
//!
//!
//! ## FAQ
//!
//! - **How does `openh264-sys2` differ from `openh264-sys`?**
//!
//!   We directly ship OpenH264 source code and provide simple, hand-crafted compilation via `cc` in `build.rs`. Our`openh264-sys2` crate should compile via `cargo build` out of the box on most platforms, and cross-compile via `cargo build --target ...` as
//!   long as the environment variable `CXX` is properly set.
//!
//! - **Which exact OpenH264 version does this use?**
//!
//!   See [this file](https://github.com/ralfbiedert/openh264-rust/tree/master/openh264-sys2/upstream/VERSION) for the upstream URL and Git hash used on latest master.
//!
//! - **I need to fix an important OpenH264 security hole, how can I update the library?**
//!
//!   Cisco's OpenH264 library is contained in `openh264-sys2/upstream`. Updating is as simple as [pulling their latest source](https://github.com/cisco/openh264),
//!   and running `update_openh264.sh` (and, if APIs changed, `regen-bindings.bat`).
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
//! - **Can I get a performance boost?**
//!
//!   - Make sure you have the command `nasm` somewhere in your PATH for your current platform (should be a single, standalone
//!     executable you don't even need to install). If found by `build.rs` it should be used automatically for an up to 3x speed
//!     boost for encoding / decoding.
//!   - Also compile your project with `target-cpu=native` for a 3x speed boost for YUV-to-RGB conversion (e.g., check
//!     our `.cargo/config.toml` how you can easily do that for your project. Note this only works if you are an application,
//!     not a library wrapping us).
//!
//!
//! - **Decoder::decode() returned an error, is this a bug?**
//!
//!   Maybe. Probably not. Some encoders can write data OpenH264 doesn't understand, and if _all_ frames fail this could either
//!   be your encoder doing exotic things, OpenH264 not having implemented a certain feature, or
//!   us having a bug.
//!
//!   If only _some_ frames fail the most likely reasons are your encoder injecting _some_ special
//!   packets or transmission errors. In other words, unless you have a controlled setup you should not terminate on
//!   the first error(s), but simply continue decoding and hope for the decoder to recover.
//!
//!   FWIW, we consider OpenH264's `h264dec` the reference decoder. If you can get it to emit YUV it would be a bug
//!   if we can't. However, any stream / frame it fails on is pretty much a _wontfix_ for us.
//!
//!
//! - **What's the deal with the `source` and `libloading` features?**
//!
//!   See [this issue](https://github.com/ralfbiedert/openh264-rs/issues/43).
//!
//!
//! ## Contributing
//!
//! PRs are very welcome. Feel free to submit PRs and fixes right away. You can open issues if you want to discuss things, but due to time restrictions on my side the project will have to rely on people contributing.
//!
//! Especially needed:
//!
//! - [ ] BT.601 / BT.709 YUV <-> RGB Conversion
//! - [ ] User-pluggable and color conversions
//! - [ ] WASM investigation (either patch, or evidence it can't be fixed)
//! - [ ] Feedback which platforms successfully built on
//! - [x] Faster YUV to RGB conversion (done in [#66](https://github.com/ralfbiedert/openh264-rs/pull/66))
//!
//! Big shout-out to all the [contributors](https://github.com/ralfbiedert/openh264-rs/graphs/contributors) who have filed
//! PRs so far.
//!
//! Special thanks to:
//!
//! - Jannik Schleicher for addressing the long-standing issue of faster YUV-to-RGB conversion, which resulted in a more than 3x speedup.
//!
//!
//! ## Changelog
//!
//! - **v0.6** - Encoder supports dynamic resolution; API cleanup.
//! - **v0.5** - Can now use built-in source, or Cisco's prebuilt library.
//! - **v0.4** - Update build system, remove unused API.
//! - **v0.3** - Change some APIs to better reflect OpenH264 behavior.
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
mod time;
mod utils;

pub mod decoder;
pub mod encoder;
pub mod formats;

pub use error::Error;
pub use time::Timestamp;
pub use utils::{nal_units, NalParser};

pub use openh264_sys2::DynamicAPI as OpenH264API;
