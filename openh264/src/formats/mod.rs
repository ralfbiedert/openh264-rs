//! Handles conversions, e.g., between RGB and YUV.
//!
//! In particular when encoding frames your source data might be a YUV buffer, or one of multiple RGB formats. The structs and
//! traits in here can help you with that format conversion.
//!
//! # Examples
//!
//! Load a _vanilla_ 3x8 bit-per-pixel RGB slice into a [YUVBuffer]:
//!
//! ```rust
//! use openh264::formats::{RgbSliceU8, YUVBuffer};
//!
//! // Assume this is a 2x2 pixel RGB buffer you got from somewhere.
//! let raw_rgb = &[ 10, 10, 10, 20, 20, 20, 30, 30, 30, 40, 40, 40 ];
//! let rgb_source = RgbSliceU8::new(raw_rgb, (2, 2));
//!
//! // Now you have a YUV which you can feed into the encoder.
//! let yuv = YUVBuffer::from_rgb_source(rgb_source);
//! ```
//!

mod rgb;
pub(crate) mod rgb2yuv;
mod yuv;
pub(crate) mod yuv2rgb;

pub use rgb::{
    AbgrSliceU32, AbgrSliceU8, ArgbSliceU32, ArgbSliceU8, BgrSliceU8, BgraSliceU32, BgraSliceU8, RGB8Source, RGBSource, RgbSliceU8, RgbaSliceU32, RgbaSliceU8
};
pub use yuv::{YUVBuffer, YUVSlices, YUVSource};
