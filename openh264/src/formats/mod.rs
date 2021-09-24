//! Handles conversions, e.g., between RGB and YUV.

mod rgb2yuv;

pub use rgb2yuv::RBGYUVConverter;

/// Allows the [Encoder](crate::encoder::Encoder) to be generic over a YUV source.
pub trait YUVSource {
    fn width(&self) -> i32;
    fn height(&self) -> i32;

    fn y(&self) -> &[u8];
    fn u(&self) -> &[u8];
    fn v(&self) -> &[u8];

    fn y_stride(&self) -> i32;
    fn u_stride(&self) -> i32;
    fn v_stride(&self) -> i32;
}
