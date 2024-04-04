//! Handles conversions, e.g., between RGB and YUV.

mod rgb;
mod yuv;

pub use rgb::{AbgrSlice, BgrSlice, BgraSlice, RgbSlice, RgbaSlice};
pub use yuv::YUVBuffer;

/// Allows the [Encoder](crate::encoder::Encoder) to be generic over a YUV source.
pub trait YUVSource {
    /// Size of the image as `(w, h)`.
    #[must_use]
    fn dimensions(&self) -> (i32, i32);

    /// YUV strides as `(y, u, v)`.
    ///
    /// For now you should make sure `u == v`.
    #[must_use]
    fn strides(&self) -> (i32, i32, i32);

    /// Y buffer, should be of size `dimension.1 * strides.0`.
    #[must_use]
    fn y(&self) -> &[u8];

    /// U buffer, should be of size `dimension.1 * strides.1`.
    #[must_use]
    fn u(&self) -> &[u8];

    /// V buffer, should be of size `dimension.1 * strides.2`.
    #[must_use]
    fn v(&self) -> &[u8];

    /// Estimates how many bytes you'll need to store this YUV as RGB.
    #[must_use]
    fn estimate_rgb_size(&self) -> usize {
        let (w, h) = self.dimensions();
        w as usize * h as usize * 3
    }

    /// Estimates how many bytes you'll need to store this YUV as RGBA.
    #[must_use]
    fn estimate_rgba_size(&self) -> usize {
        let (w, h) = self.dimensions();
        w as usize * h as usize * 4
    }
}

/// Source of arbitrarily formatted RGB data
pub trait RGBSource {
    /// Extract the pixel value at the specified location. Pixel values are
    /// expected to be floats in the range `[0, 256)` (`u8` represented as `f32`).
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32);
}
