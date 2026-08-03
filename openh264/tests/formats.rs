use openh264::formats::{
    BGRA8Source, BgraSliceU8, RGB8Source, RGBA8Source, RGBSource, RgbSliceU8, RgbaSliceU8, YUVBuffer, YUVSource,
};

#[derive(Copy, Clone)]
struct PaddedRgbSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
    stride: usize,
}

impl RGBSource for PaddedRgbSliceU8<'_> {
    fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let offset = (y * self.stride + x) * 3;
        (
            f32::from(self.data[offset]),
            f32::from(self.data[offset + 1]),
            f32::from(self.data[offset + 2]),
        )
    }
}

impl RGB8Source for PaddedRgbSliceU8<'_> {
    fn dimensions_padded(&self) -> (usize, usize) {
        (self.stride, self.dimensions.1)
    }

    fn rgb8_data(&self) -> &[u8] {
        self.data
    }
}

#[derive(Copy, Clone)]
struct PaddedRgbaSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
    stride: usize,
}

impl RGBSource for PaddedRgbaSliceU8<'_> {
    fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let offset = (y * self.stride + x) * 4;
        (
            f32::from(self.data[offset]),
            f32::from(self.data[offset + 1]),
            f32::from(self.data[offset + 2]),
        )
    }
}

impl RGB8Source for PaddedRgbaSliceU8<'_> {
    fn dimensions_padded(&self) -> (usize, usize) {
        (self.stride, self.dimensions.1)
    }

    fn rgb8_data(&self) -> &[u8] {
        self.data
    }

    fn pixel_stride(&self) -> usize {
        4
    }

    fn rgb_channel_offsets(&self) -> (usize, usize, usize) {
        (0, 1, 2)
    }
}

impl RGBA8Source for PaddedRgbaSliceU8<'_> {}

#[derive(Copy, Clone)]
struct PaddedBgraSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
    stride: usize,
}

impl RGBSource for PaddedBgraSliceU8<'_> {
    fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let offset = (y * self.stride + x) * 4;
        (
            f32::from(self.data[offset + 2]),
            f32::from(self.data[offset + 1]),
            f32::from(self.data[offset]),
        )
    }
}

impl RGB8Source for PaddedBgraSliceU8<'_> {
    fn dimensions_padded(&self) -> (usize, usize) {
        (self.stride, self.dimensions.1)
    }

    fn rgb8_data(&self) -> &[u8] {
        self.data
    }

    fn pixel_stride(&self) -> usize {
        4
    }

    fn rgb_channel_offsets(&self) -> (usize, usize, usize) {
        (2, 1, 0)
    }
}

impl BGRA8Source for PaddedBgraSliceU8<'_> {}

#[test]
fn rgb8_conversion_handles_padded_rows() {
    let dimensions = (4, 2);
    let stride = 6;
    let padded_data: Vec<u8> = (0..stride * dimensions.1 * 3).map(|value| value as u8).collect();
    let packed_data = [
        &padded_data[0..dimensions.0 * 3],
        &padded_data[stride * 3..(stride + dimensions.0) * 3],
    ]
    .concat();
    let padded = PaddedRgbSliceU8 {
        data: &padded_data,
        dimensions,
        stride,
    };
    let packed = RgbSliceU8::new(&packed_data, dimensions);

    let mut padded_yuv = YUVBuffer::new(dimensions.0, dimensions.1);
    padded_yuv.read_rgb8(padded);
    let packed_yuv = YUVBuffer::from_rgb8_source(packed);

    assert_eq!(padded_yuv.y(), packed_yuv.y());
    assert_eq!(padded_yuv.u(), packed_yuv.u());
    assert_eq!(padded_yuv.v(), packed_yuv.v());
}

#[test]
fn rgba_and_bgra_conversion_have_exact_i420_values() {
    let rgba = RgbaSliceU8::new(&[255, 0, 0, 7, 255, 0, 0, 9, 255, 0, 0, 11, 255, 0, 0, 13], (2, 2));
    let bgra = BgraSliceU8::new(&[0, 0, 255, 17, 0, 0, 255, 19, 0, 0, 255, 23, 0, 0, 255, 29], (2, 2));

    for actual in [YUVBuffer::from_rgba8_source(rgba), YUVBuffer::from_bgra8_source(bgra)] {
        assert_eq!(actual.y(), [81, 81, 81, 81]);
        assert_eq!(actual.u(), [90]);
        assert_eq!(actual.v(), [239]);
    }
}

#[test]
fn rgba_and_bgra_conversions_match_rgb8_for_packed_and_padded_rows() {
    const DIMENSIONS: (usize, usize) = (74, 4);
    const STRIDE: usize = 80;
    let mut packed_rgb = vec![0; DIMENSIONS.0 * DIMENSIONS.1 * 3];
    let mut packed_rgba = vec![0; DIMENSIONS.0 * DIMENSIONS.1 * 4];
    let mut packed_bgra = vec![0; DIMENSIONS.0 * DIMENSIONS.1 * 4];
    let mut padded_rgba = vec![255; STRIDE * DIMENSIONS.1 * 4];
    let mut padded_bgra = vec![255; STRIDE * DIMENSIONS.1 * 4];

    for y in 0..DIMENSIONS.1 {
        for x in 0..DIMENSIONS.0 {
            let r = ((x * 37 + y * 19) % 256) as u8;
            let g = ((x * 11 + y * 53) % 256) as u8;
            let b = ((x * 71 + y * 7) % 256) as u8;
            let rgb_offset = (y * DIMENSIONS.0 + x) * 3;
            let packed_offset = (y * DIMENSIONS.0 + x) * 4;
            let padded_offset = (y * STRIDE + x) * 4;
            packed_rgb[rgb_offset..rgb_offset + 3].copy_from_slice(&[r, g, b]);
            packed_rgba[packed_offset..packed_offset + 4].copy_from_slice(&[r, g, b, 17]);
            packed_bgra[packed_offset..packed_offset + 4].copy_from_slice(&[b, g, r, 19]);
            padded_rgba[padded_offset..padded_offset + 4].copy_from_slice(&[r, g, b, 23]);
            padded_bgra[padded_offset..padded_offset + 4].copy_from_slice(&[b, g, r, 29]);
        }
    }

    let reference = YUVBuffer::from_rgb8_source(RgbSliceU8::new(&packed_rgb, DIMENSIONS));
    let rgba = YUVBuffer::from_rgba8_source(RgbaSliceU8::new(&packed_rgba, DIMENSIONS));
    let bgra = YUVBuffer::from_bgra8_source(BgraSliceU8::new(&packed_bgra, DIMENSIONS));
    let padded_rgba = PaddedRgbaSliceU8 {
        data: &padded_rgba,
        dimensions: DIMENSIONS,
        stride: STRIDE,
    };
    let padded_bgra = PaddedBgraSliceU8 {
        data: &padded_bgra,
        dimensions: DIMENSIONS,
        stride: STRIDE,
    };
    let mut read_rgba = YUVBuffer::new(DIMENSIONS.0, DIMENSIONS.1);
    let mut read_bgra = YUVBuffer::new(DIMENSIONS.0, DIMENSIONS.1);
    read_rgba.read_rgba8(padded_rgba);
    read_bgra.read_bgra8(padded_bgra);

    for actual in [&rgba, &bgra, &read_rgba, &read_bgra] {
        assert_eq!(actual.y(), reference.y());
        assert_eq!(actual.u(), reference.u());
        assert_eq!(actual.v(), reference.v());
    }
}
