/// Source of arbitrarily formatted RGB data.
pub trait RGBSource {
    #[must_use]
    fn dimensions_i32(&self) -> (i32, i32) {
        let (w, h) = self.dimensions();
        (w as i32, h as i32)
    }

    #[must_use]
    fn dimensions(&self) -> (usize, usize);

    /// Extract the pixel value at the specified location. Pixel values are
    /// expected to be floats in the range `[0, 256)` (`u8` represented as `f32`).
    #[must_use]
    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32);
}

/// Container for a slice of contiguous `[R G B R G B ...]` data.
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
            #[allow(unused)]
            pub fn new(data: &'a [u8], dimensions: (usize, usize)) -> Self {
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
                    self.data[base_pos + $offsets[0]] as f32,
                    self.data[base_pos + $offsets[1]] as f32,
                    self.data[base_pos + $offsets[2]] as f32,
                )
            }
        }
    };
}

macro_rules! impl_slice_wrapper_u32 {
    ($t:ty, $offsets:expr) => {
        impl<'a> $t {
            /// Creates a new instance given the data slice and dimensions.
            #[allow(unused)]
            pub fn new(data: &'a [u32], dimensions: (usize, usize)) -> Self {
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

#[cfg(test)]
mod tests {
    use super::{AbgrSliceU32, ArgbSliceU32, BgrSliceU8, BgraSliceU32, RGBSource, RgbSliceU8, RgbaSliceU32};

    #[test]
    fn rgb_slice_3x3() {
        let vec: Vec<u8> = (0..27).collect();
        let slice = RgbSliceU8::new(&vec, (3, 3));
        assert_eq!(slice.pixel_f32(0, 0), (0f32, 1f32, 2f32));
        assert_eq!(slice.pixel_f32(1, 0), (3f32, 4f32, 5f32));
        assert_eq!(slice.pixel_f32(2, 0), (6f32, 7f32, 8f32));
        assert_eq!(slice.pixel_f32(0, 1), (9f32, 10f32, 11f32));
        assert_eq!(slice.pixel_f32(1, 1), (12f32, 13f32, 14f32));
        assert_eq!(slice.pixel_f32(2, 1), (15f32, 16f32, 17f32));
        assert_eq!(slice.pixel_f32(0, 2), (18f32, 19f32, 20f32));
        assert_eq!(slice.pixel_f32(1, 2), (21f32, 22f32, 23f32));
        assert_eq!(slice.pixel_f32(2, 2), (24f32, 25f32, 26f32));
    }

    #[test]
    fn bgr_slice_3x3() {
        let vec: Vec<u8> = (0..27).collect();
        let slice = BgrSliceU8::new(&vec, (3, 3));
        assert_eq!(slice.pixel_f32(0, 0), (2f32, 1f32, 0f32));
        assert_eq!(slice.pixel_f32(1, 0), (5f32, 4f32, 3f32));
        assert_eq!(slice.pixel_f32(2, 0), (8f32, 7f32, 6f32));
        assert_eq!(slice.pixel_f32(0, 1), (11f32, 10f32, 9f32));
        assert_eq!(slice.pixel_f32(1, 1), (14f32, 13f32, 12f32));
        assert_eq!(slice.pixel_f32(2, 1), (17f32, 16f32, 15f32));
        assert_eq!(slice.pixel_f32(0, 2), (20f32, 19f32, 18f32));
        assert_eq!(slice.pixel_f32(1, 2), (23f32, 22f32, 21f32));
        assert_eq!(slice.pixel_f32(2, 2), (26f32, 25f32, 24f32));
    }

    #[test]
    fn rgba_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = RgbaSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (255f32, 1f32, 0f32));
        assert_eq!(slice.pixel_f32(1, 0), (255f32, 0f32, 2f32));
        assert_eq!(slice.pixel_f32(0, 1), (255f32, 2f32, 0f32));
        assert_eq!(slice.pixel_f32(1, 1), (170f32, 187f32, 204f32));
    }

    #[test]
    fn argb_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = ArgbSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (1f32, 0f32, 2f32));
        assert_eq!(slice.pixel_f32(1, 0), (0f32, 2f32, 1f32));
        assert_eq!(slice.pixel_f32(0, 1), (2f32, 0f32, 1f32));
        assert_eq!(slice.pixel_f32(1, 1), (187f32, 204f32, 221f32));
    }

    #[test]
    fn bgra_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = BgraSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (0f32, 1f32, 255f32));
        assert_eq!(slice.pixel_f32(1, 0), (2f32, 0f32, 255f32));
        assert_eq!(slice.pixel_f32(0, 1), (0f32, 2f32, 255f32));
        assert_eq!(slice.pixel_f32(1, 1), (204f32, 187f32, 170f32));
    }

    #[test]
    fn abgr_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = AbgrSliceU32::new(&data[1..], (2, 2));
        assert_eq!(slice.pixel_f32(0, 0), (2f32, 0f32, 1f32));
        assert_eq!(slice.pixel_f32(1, 0), (1f32, 2f32, 0f32));
        assert_eq!(slice.pixel_f32(0, 1), (1f32, 0f32, 2f32));
        assert_eq!(slice.pixel_f32(1, 1), (221f32, 204f32, 187f32));
    }
}
