
[![Latest Version]][crates.io]
[![docs]][docs.rs]
![BSD-2]
[![Rust](https://img.shields.io/badge/rust-1.53%2B-blue.svg?maxAge=3600)](https://github.com/ralfbiedert/openh264-rust)
[![Rust](https://github.com/ralfbiedert/openh264-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/ralfbiedert/openh264-rust/actions/workflows/rust.yml)

## OpenH264 Rust API

Idiomatic and low-level bindings for [OpenH264](https://github.com/cisco/openh264), converting<sup>*</sup> between these two in Rust:

![sample_image](https://media.githubusercontent.com/media/ralfbiedert/openh264-rust/master/gfx/title2.jpg)


*High-level wrapped decoder only for now, encoder PRs welcome.

### Example API

Here we convert the last image of a H264 stream to a RGB byte array.

```rust
use openh264::Decoder;

let mut decoder = Decoder::new()?;
let mut rgb_out = [0; 512 * 512 * 3];
let h264_in = include_bytes!("../tests/data/multi_512x512.h264");

// Decode to YUV, then convert and write RGB.
decoder.decode_no_delay(&h264_in[..])?.write_rgb8(&mut rgb_out)?;

```

### Platform Support

Test results on various platforms:

| Platform | Compiled | Unit Tested |
| --- | --- | --- |
| `x86_64-pc-windows-msvc` | ✅ | ✅ |
| `x86_64-unknown-linux-gnu` | ✅ | ✅ |
| `x86_64-apple-darwin` | ✅ | ✅ |
| `aarch64-linux-android` | 🆗<sup>1</sup>  | - |
| `wasm32-unknown-unknown` | ❌<sup>1,2</sup> | - |

✅ works out of the box;
🆗 the usual shenanigans required;
❌ not supported.

<sup>1</sup> via `cargo build --target <platform>`, [needs `CXX` set](https://cheats.rs/#cross-compilation) <br/>
<sup>2</sup> unclear if could ever work, investigation welcome


### Performance

Tested on a i9-9900K, Windows 10, single threaded decoding:

```
test decode_yuv_single_1920x1080     ... bench:   9,243,380 ns/iter (+/- 497,200)
test decode_yuv_single_512x512_cabac ... bench:   1,841,775 ns/iter (+/- 53,211)
test decode_yuv_single_512x512_cavlc ... bench:   2,076,030 ns/iter (+/- 7,287)
test whole_decoder                   ... bench:   2,874,107 ns/iter (+/- 62,643)

test result: ok. 0 passed; 0 failed; 0 ignored; 5 measured; 0 filtered out; finished in 14.26s

Running unittests (target\release\deps\yuv2rgb-5a3aaabbb6bf3e8a.exe)

running 2 tests
test convert_yuv_to_rgb_1920x1080 ... bench:   7,226,290 ns/iter (+/- 110,871)
test convert_yuv_to_rgb_512x512   ... bench:     907,340 ns/iter (+/- 28,296)
```

If you want to improve these numbers you can submit PRs that

- [ ] better enable autovectorization converting YUV to RGB,
- [ ] conditionally enable assembly in `build.rs` for OpenH264.

### Compile Features

- `backtrace` - Enable backtraces on errors (requires nightly)

### FAQ

- **How does `openh264-sys2` differ from `openh264-sys`?**

We directly ship OpenH264 source code and provide simple, hand-crafted compilation via `cc` in `build.rs`. Our`openh264-sys2` crate should compile via `cargo build` out of the box on most platforms, and cross-compile via `cargo build --target ...` as
long as the environment variable `CXX` is properly set.


- **I need to fix an important OpenH264 security hole, how can I update the library?**

Cisco's OpenH264 library is contained in `openh264-sys2/upstream`. Updating is (almost, see below) as simple as [pulling their latest source](https://github.com/cisco/openh264),
copying it into that directory, and manually removing all "resource" files. We probably should have a script to strip that folder automatically ...


- **I heard Rust is super-safe, will this make decoding my videos safe too?**

No. Below a thin Rust layer we rely on a _very complex_ C library, and an equally complex standard. Apart from Rust being a
much nicer language to work with, depending on this  project will give you _no_ additional safety guarantees as far as video
handling is concerned. FYI, this is _not_ making a statement about OpenH264, but about the realities of securing +50k lines
of C against attacks.


- **Feature X is missing or broken, will you fix it?**

Right now I only have time to implement what I need. However, I will gladly accept PRs either extending the APIs, or fixing bugs; see below.


### OpenH264 Patches Applied

Ideally the embedded upstream should be pristine. That said, the following
patches have been applied to fix Valgrind issues and crashes on some platforms:

- `decoder.cpp` - removed `if (pCtx->pDstInfo) pCtx->pDstInfo->iBufferStatus = 0;` which seems to write to previously deallocated memory.

Help with upstreaming them would be appreciated.


### Contributing

PRs are very welcome. Feel free to submit PRs and fixes right away. You can open issues if you want to discuss things, but due to time restrictions on my side the project will have to rely on people contributing.

Especially needed:

- [ ] CI testing
- [ ] Encoder wrapper
- [ ] Enabling of platform specific assembly (without breaking or complicating build)
- [ ] Faster YUV to RGB conversion
- [ ] Have script to automatically update / import OpenH264 source (or submodule?)
- [ ] WASM investigation (either patch, or evidence it can't be fixed)
- [ ] Submit patches upstream
- [ ] Feedback which platforms successfully built on


### License

- OpenH264 core library is [BSD-2](openh264-sys2/upstream/LICENSE), Cisco.
- Wrapper code is [BSD-2](https://opensource.org/licenses/BSD-2-Clause), Ralf Biedert.

[Latest Version]: https://img.shields.io/crates/v/openh264.svg
[crates.io]: https://crates.io/crates/openh264
[BSD-2]: https://img.shields.io/badge/license-BSD2-blue.svg
[docs]: https://docs.rs/openh264/badge.svg
[docs.rs]: https://docs.rs/openh264/
