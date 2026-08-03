use openh264::formats::{RGB8Source, RGBSource, RgbSliceU8, YUVBuffer, YUVSource};

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
