/// Source of arbitrarily formatted RGB data.
///
/// This is the "compatible" trait for RGB sources, but it will
/// be slow, since it only supports single pixel lookup.
pub trait RGBSource {
    /// Returns the underlying image size as an `i32` tuple `(w, h)`.
    #[must_use]
    fn dimensions_i32(&self) -> (i32, i32) {
        let (w, h) = self.dimensions();
        (w as i32, h as i32)
    }

    /// Returns the underlying image size as an `usize` tuple `(w, h)`.
    #[must_use]
    fn dimensions(&self) -> (usize, usize);

    /// Extract the pixel value at the specified location. Pixel values are
    /// expected to be floats in the range `[0, 255]` (`u8` represented as `f32`).
    #[must_use]
    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32);
}

/// Source of RGB8 data for fast pixel access.
///
/// This is the "fast" trait for RGB sources. If you can expose continuous pixels
/// slices with RGB8 data you might (eventually) be rewarded with SIMD conversion.
pub trait RGB8Source: RGBSource {
    /// Returns padded dimensions of the underlying slice.
    ///
    /// For example, the data might have a display format of 100x100, but the
    /// underlying RGB8 array is of size 128x100.
    #[must_use]
    fn dimensions_padded(&self) -> (usize, usize);

    /// Slice of RGB8 data, with given padding.
    #[must_use]
    fn rgb8_data(&self) -> &[u8];
}

/// Container for a slice of contiguous `[R G B R G B ...]` data.<sup>⭐</sup>
///
/// This is the preferred format for reading data, for use with `_rgb8` methods.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct RgbSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[B G R B G R ...]` data.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct BgrSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[R G B A R G B A ...]` data.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct RgbaSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[RGBA RGBA ...]` data.
///
/// The platform endianness of the data is irrelevant: R is the highest byte and A is the lowest.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct RgbaSliceU32<'a> {
    data: &'a [u32],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[B G R A B G R A ...]` data.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct BgraSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[BGRA BGRA ...]` data.
///
/// The platform endianness of the data is irrelevant: B is the highest byte and A is the lowest.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct BgraSliceU32<'a> {
    data: &'a [u32],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[A B G R A B G R ...]` data.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct AbgrSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[ABGR ABGR ...]` data.
///
/// The platform endianness of the data is irrelevant: A is the highest byte and R is the lowest.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct AbgrSliceU32<'a> {
    data: &'a [u32],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[A R G B A R G B ...]` data.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct ArgbSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

/// Container for a slice of contiguous `[ARGB ARGB ...]` data.
///
/// The platform endianness of the data is irrelevant: A is the highest byte and B is the lowest.
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct ArgbSliceU32<'a> {
    data: &'a [u32],
    dimensions: (usize, usize),
}

macro_rules! impl_slice_wrapper_u8 {
    ($t:ty, $stride:expr, $offsets:expr) => {
        impl<'a> $t {
            /// Creates a new instance given the byte slice and dimensions.
            ///
            /// # Panics
            ///
            /// May panic if the given sizes are not multiples of 2, or if the slice length mismatches the given dimensions.
            #[allow(unused)]
            pub fn new(data: &'a [u8], dimensions: (usize, usize)) -> Self {
                assert_eq!(data.len(), dimensions.0 * dimensions.1 * $stride);
                assert_eq!(dimensions.0 % 2, 0, "width needs to be multiple of 2");
                assert_eq!(dimensions.1 % 2, 0, "height needs to be a multiple of 2");

                Self { data, dimensions }
            }
        }

        impl<'a> RGBSource for $t {
            fn dimensions(&self) -> (usize, usize) {
                self.dimensions
            }

            fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
                let base_pos = (x + y * self.dimensions.0) * $stride;
                (
                    self.data[base_pos + $offsets[0]].into(),
                    self.data[base_pos + $offsets[1]].into(),
                    self.data[base_pos + $offsets[2]].into(),
                )
            }
        }
    };
}

macro_rules! impl_slice_wrapper_u32 {
    ($t:ty, $offsets:expr) => {
        impl<'a> $t {
            /// Creates a new instance given the data slice and dimensions.
            ///
            /// # Panics
            ///
            /// May panic if the given sizes are not multiples of 2, or if the slice length mismatches the given dimensions.
            #[allow(unused)]
            pub fn new(data: &'a [u32], dimensions: (usize, usize)) -> Self {
                assert_eq!(data.len(), dimensions.0 * dimensions.1);
                assert_eq!(dimensions.0 % 2, 0, "width needs to be multiple of 2");
                assert_eq!(dimensions.1 % 2, 0, "height needs to be a multiple of 2");

                Self { data, dimensions }
            }
        }

        impl<'a> RGBSource for $t {
            fn dimensions(&self) -> (usize, usize) {
                self.dimensions
            }

            fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
                let px = self.data[x + y * self.dimensions.0];
                (
                    ((px >> $offsets[0]) & 0xFF) as f32,
                    ((px >> $offsets[1]) & 0xFF) as f32,
                    ((px >> $offsets[2]) & 0xFF) as f32,
                )
            }
        }
    };
}

impl_slice_wrapper_u8!(RgbSliceU8<'a>, 3, [0, 1, 2]);
impl_slice_wrapper_u8!(RgbaSliceU8<'a>, 4, [0, 1, 2]);
impl_slice_wrapper_u8!(BgrSliceU8<'a>, 3, [2, 1, 0]);
impl_slice_wrapper_u8!(BgraSliceU8<'a>, 4, [2, 1, 0]);
impl_slice_wrapper_u8!(ArgbSliceU8<'a>, 4, [1, 2, 3]);
impl_slice_wrapper_u8!(AbgrSliceU8<'a>, 4, [3, 2, 1]);

impl_slice_wrapper_u32!(RgbaSliceU32<'a>, [24, 16, 8]);
impl_slice_wrapper_u32!(BgraSliceU32<'a>, [8, 16, 24]);
impl_slice_wrapper_u32!(AbgrSliceU32<'a>, [0, 8, 16]);
impl_slice_wrapper_u32!(ArgbSliceU32<'a>, [16, 8, 0]);

impl RGB8Source for RgbSliceU8<'_> {
    fn dimensions_padded(&self) -> (usize, usize) {
        self.dimensions()
    }

    fn rgb8_data(&self) -> &[u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::{AbgrSliceU32, ArgbSliceU32, BgrSliceU8, BgraSliceU32, RGBSource, RgbSliceU8, RgbaSliceU32};

    #[test]
    fn rgb_slice_4x4() {
        let vec: Vec<u8> = (0..4 * 4 * 3).collect();
        let slice = RgbSliceU8::new(&vec, (4, 4));
        assert_eq!(slice.pixel_f32(0, 0), (0., 1., 2.));
        assert_eq!(slice.pixel_f32(1, 0), (3., 4., 5.));
        assert_eq!(slice.pixel_f32(2, 0), (6., 7., 8.));
        assert_eq!(slice.pixel_f32(0, 1), (12., 13., 14.));
        assert_eq!(slice.pixel_f32(1, 1), (15., 16., 17.));
        assert_eq!(slice.pixel_f32(2, 1), (18., 19., 20.));
        assert_eq!(slice.pixel_f32(0, 2), (24., 25., 26.));
        assert_eq!(slice.pixel_f32(1, 2), (27., 28., 29.));
        assert_eq!(slice.pixel_f32(2, 2), (30., 31., 32.));
    }

    #[test]
    fn bgr_slice_4x4() {
        let vec: Vec<u8> = (0..4 * 4 * 3).collect();
        let slice = BgrSliceU8::new(&vec, (4, 4));
        assert_eq!(slice.pixel_f32(0, 0), (2., 1., 0.));
        assert_eq!(slice.pixel_f32(1, 0), (5., 4., 3.));
        assert_eq!(slice.pixel_f32(2, 0), (8., 7., 6.));
        assert_eq!(slice.pixel_f32(0, 1), (14.0, 13.0, 12.0));
        assert_eq!(slice.pixel_f32(1, 1), (17.0, 16.0, 15.0));
        assert_eq!(slice.pixel_f32(2, 1), (20.0, 19.0, 18.0));
        assert_eq!(slice.pixel_f32(0, 2), (26.0, 25.0, 24.0));
        assert_eq!(slice.pixel_f32(1, 2), (29.0, 28.0, 27.0));
        assert_eq!(slice.pixel_f32(2, 2), (32.0, 31.0, 30.0));
    }

    #[test]
    fn rgba_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = RgbaSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (255., 1., 0.));
        assert_eq!(slice.pixel_f32(1, 0), (255., 0., 2.));
        assert_eq!(slice.pixel_f32(0, 1), (255., 2., 0.));
        assert_eq!(slice.pixel_f32(1, 1), (170., 187., 204.));
    }

    #[test]
    fn argb_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = ArgbSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (1., 0., 2.));
        assert_eq!(slice.pixel_f32(1, 0), (0., 2., 1.));
        assert_eq!(slice.pixel_f32(0, 1), (2., 0., 1.));
        assert_eq!(slice.pixel_f32(1, 1), (187., 204., 221.));
    }

    #[test]
    fn bgra_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = BgraSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (0., 1., 255.));
        assert_eq!(slice.pixel_f32(1, 0), (2., 0., 255.));
        assert_eq!(slice.pixel_f32(0, 1), (0., 2., 255.));
        assert_eq!(slice.pixel_f32(1, 1), (204., 187., 170.));
    }

    #[test]
    fn abgr_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = AbgrSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (2., 0., 1.));
        assert_eq!(slice.pixel_f32(1, 0), (1., 2., 0.));
        assert_eq!(slice.pixel_f32(0, 1), (1., 0., 2.));
        assert_eq!(slice.pixel_f32(1, 1), (221., 204., 187.));
    }
}
