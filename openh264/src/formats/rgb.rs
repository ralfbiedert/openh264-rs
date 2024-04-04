use crate::formats::RGBSource;

/// Default implementation for a naive RGB array in row-major [RGB RGB ...] order.
/// Provided to maintain backwards compatibility.
impl<const N: usize> RGBSource for [u8; N] {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let base_pos = (x + y * width) * 3;
        (self[base_pos] as f32, self[base_pos + 1] as f32, self[base_pos + 2] as f32)
    }
}

/// Container for a slice of contiguous [R G B R G B ...] data
pub struct RgbSlice<'a>(&'a [u8]);

impl<'a> RGBSource for RgbSlice<'a> {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let base_pos = (x + y * width) * 3;
        (
            self.0[base_pos] as f32,
            self.0[base_pos + 1] as f32,
            self.0[base_pos + 2] as f32,
        )
    }
}

/// Container for a slice of contiguous [B G R B G R ...] data
pub struct BgrSlice<'a>(&'a [u8]);

impl<'a> RGBSource for BgrSlice<'a> {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let base_pos = (x + y * width) * 3;
        (
            self.0[base_pos + 2] as f32,
            self.0[base_pos + 1] as f32,
            self.0[base_pos] as f32,
        )
    }
}

/// Container for a slice of contiguous [RGBA RGBA ...] data
///
/// The platform endianness of the data is irrelevant: R is the highest byte and A is the lowest.
pub struct RgbaSlice<'a>(&'a [u32]);

impl<'a> RGBSource for RgbaSlice<'a> {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let px = self.0[x + y * width];
        (
            ((px >> 24) & 0xFF) as f32,
            ((px >> 16) & 0xFF) as f32,
            ((px >> 8) & 0xFF) as f32,
        )
    }
}

/// Container for a slice of contiguous [ARGB ARGB ...] data
///
/// The platform endianness of the data is irrelevant: A is the highest byte and B is the lowest.
pub struct ArgbSlice<'a>(&'a [u32]);

impl<'a> RGBSource for ArgbSlice<'a> {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let px = self.0[x + y * width];
        (((px >> 16) & 0xFF) as f32, ((px >> 8) & 0xFF) as f32, (px & 0xFF) as f32)
    }
}
/// Container for a slice of contiguous [BGRA BGRA ...] data
///
/// The platform endianness of the data is irrelevant: B is the highest byte and A is the lowest.
pub struct BgraSlice<'a>(&'a [u32]);

impl<'a> RGBSource for BgraSlice<'a> {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let px = self.0[x + y * width];
        (
            ((px >> 8) & 0xFF) as f32,
            ((px >> 16) & 0xFF) as f32,
            ((px >> 24) & 0xFF) as f32,
        )
    }
}

/// Container for a slice of contiguous [ABGR ABGR ...] data
///
/// The platform endianness of the data is irrelevant: A is the highest byte and R is the lowest.
pub struct AbgrSlice<'a>(&'a [u32]);

impl<'a> RGBSource for AbgrSlice<'a> {
    fn pixel(&self, x: usize, y: usize, width: usize, _height: usize) -> (f32, f32, f32) {
        let px = self.0[x + y * width];
        ((px & 0xFF) as f32, ((px >> 8) & 0xFF) as f32, ((px >> 16) & 0xFF) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::{AbgrSlice, ArgbSlice, BgrSlice, BgraSlice, RGBSource, RgbSlice, RgbaSlice};

    #[test]
    fn u8_array_3x3() {
        let arr: [u8; 27] = (0..27).collect::<Vec<u8>>().try_into().unwrap();
        assert_eq!(arr.pixel(0, 0, 3, 3), (0f32, 1f32, 2f32));
        assert_eq!(arr.pixel(1, 0, 3, 3), (3f32, 4f32, 5f32));
        assert_eq!(arr.pixel(2, 0, 3, 3), (6f32, 7f32, 8f32));
        assert_eq!(arr.pixel(0, 1, 3, 3), (9f32, 10f32, 11f32));
        assert_eq!(arr.pixel(1, 1, 3, 3), (12f32, 13f32, 14f32));
        assert_eq!(arr.pixel(2, 1, 3, 3), (15f32, 16f32, 17f32));
        assert_eq!(arr.pixel(0, 2, 3, 3), (18f32, 19f32, 20f32));
        assert_eq!(arr.pixel(1, 2, 3, 3), (21f32, 22f32, 23f32));
        assert_eq!(arr.pixel(2, 2, 3, 3), (24f32, 25f32, 26f32));
    }

    #[test]
    fn rgb_slice_3x3() {
        let vec: Vec<u8> = (0..27).collect();
        let slice = RgbSlice(&vec);
        assert_eq!(slice.pixel(0, 0, 3, 3), (0f32, 1f32, 2f32));
        assert_eq!(slice.pixel(1, 0, 3, 3), (3f32, 4f32, 5f32));
        assert_eq!(slice.pixel(2, 0, 3, 3), (6f32, 7f32, 8f32));
        assert_eq!(slice.pixel(0, 1, 3, 3), (9f32, 10f32, 11f32));
        assert_eq!(slice.pixel(1, 1, 3, 3), (12f32, 13f32, 14f32));
        assert_eq!(slice.pixel(2, 1, 3, 3), (15f32, 16f32, 17f32));
        assert_eq!(slice.pixel(0, 2, 3, 3), (18f32, 19f32, 20f32));
        assert_eq!(slice.pixel(1, 2, 3, 3), (21f32, 22f32, 23f32));
        assert_eq!(slice.pixel(2, 2, 3, 3), (24f32, 25f32, 26f32));
    }

    #[test]
    fn bgr_slice_3x3() {
        let vec: Vec<u8> = (0..27).collect();
        let slice = BgrSlice(&vec);
        assert_eq!(slice.pixel(0, 0, 3, 3), (2f32, 1f32, 0f32));
        assert_eq!(slice.pixel(1, 0, 3, 3), (5f32, 4f32, 3f32));
        assert_eq!(slice.pixel(2, 0, 3, 3), (8f32, 7f32, 6f32));
        assert_eq!(slice.pixel(0, 1, 3, 3), (11f32, 10f32, 9f32));
        assert_eq!(slice.pixel(1, 1, 3, 3), (14f32, 13f32, 12f32));
        assert_eq!(slice.pixel(2, 1, 3, 3), (17f32, 16f32, 15f32));
        assert_eq!(slice.pixel(0, 2, 3, 3), (20f32, 19f32, 18f32));
        assert_eq!(slice.pixel(1, 2, 3, 3), (23f32, 22f32, 21f32));
        assert_eq!(slice.pixel(2, 2, 3, 3), (26f32, 25f32, 24f32));
    }

    #[test]
    fn rgba_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = RgbaSlice(&data[1..]);
        assert_eq!(slice.pixel(0, 0, 2, 2), (255f32, 1f32, 0f32));
        assert_eq!(slice.pixel(1, 0, 2, 2), (255f32, 0f32, 2f32));
        assert_eq!(slice.pixel(0, 1, 2, 2), (255f32, 2f32, 0f32));
        assert_eq!(slice.pixel(1, 1, 2, 2), (170f32, 187f32, 204f32));
    }

    #[test]
    fn argb_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = ArgbSlice(&data[1..]);
        assert_eq!(slice.pixel(0, 0, 2, 2), (1f32, 0f32, 2f32));
        assert_eq!(slice.pixel(1, 0, 2, 2), (0f32, 2f32, 1f32));
        assert_eq!(slice.pixel(0, 1, 2, 2), (2f32, 0f32, 1f32));
        assert_eq!(slice.pixel(1, 1, 2, 2), (187f32, 204f32, 221f32));
    }

    #[test]
    fn bgra_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = BgraSlice(&data[1..]);
        assert_eq!(slice.pixel(0, 0, 2, 2), (0f32, 1f32, 255f32));
        assert_eq!(slice.pixel(1, 0, 2, 2), (2f32, 0f32, 255f32));
        assert_eq!(slice.pixel(0, 1, 2, 2), (0f32, 2f32, 255f32));
        assert_eq!(slice.pixel(1, 1, 2, 2), (204f32, 187f32, 170f32));
    }

    #[test]
    fn abgr_slice_2x2() {
        let data: [u32; 5] = [0xFF000102, 0xFF010002, 0xFF000201, 0xFF020001, 0xAABBCCDD];
        let slice = AbgrSlice(&data[1..]);
        assert_eq!(slice.pixel(0, 0, 2, 2), (2f32, 0f32, 1f32));
        assert_eq!(slice.pixel(1, 0, 2, 2), (1f32, 2f32, 0f32));
        assert_eq!(slice.pixel(0, 1, 2, 2), (1f32, 0f32, 2f32));
        assert_eq!(slice.pixel(1, 1, 2, 2), (221f32, 204f32, 187f32));
    }
}
