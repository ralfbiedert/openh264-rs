#![feature(test)]

extern crate test;

use openh264::OpenH264API;
use openh264::decoder::{Decoder, DecoderConfig};
use openh264::formats::{BgraSliceU8, RGB8Source, RGBSource, RgbSliceU8, RgbaSliceU8, YUVBuffer, YUVSource};
use test::{Bencher, black_box};

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
struct ScalarRgbSliceU8<'a> {
    data: &'a [u8],
    dimensions: (usize, usize),
}

impl RGBSource for ScalarRgbSliceU8<'_> {
    fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let offset = (y * self.dimensions.0 + x) * 3;
        (
            f32::from(self.data[offset]),
            f32::from(self.data[offset + 1]),
            f32::from(self.data[offset]),
        )
    }
}

impl RGB8Source for ScalarRgbSliceU8<'_> {
    fn dimensions_padded(&self) -> (usize, usize) {
        self.dimensions
    }

    fn rgb8_data(&self) -> &[u8] {
        self.data
    }

    fn rgb_channel_offsets(&self) -> (usize, usize, usize) {
        // This valid R-G-R channel layout deliberately uses the scalar path.
        (0, 1, 0)
    }
}

#[bench]
fn convert_rgb_to_yuv_512x512(b: &mut Bencher) {
    let src = include_bytes!("../tests/data/lenna_512x512.rgb");
    let rgb_source = RgbSliceU8::new(src, (512, 512));
    let mut converter = YUVBuffer::new(512, 512);

    b.iter(|| {
        black_box(&mut converter).read_rgb(black_box(rgb_source));
    });
}

#[bench]
fn convert_rgb8_to_yuv_512x512(b: &mut Bencher) {
    let src = include_bytes!("../tests/data/lenna_512x512.rgb");
    let rgb_source = RgbSliceU8::new(src, (512, 512));
    let mut converter = YUVBuffer::new(512, 512);

    b.iter(|| {
        black_box(&mut converter).read_rgb8(black_box(rgb_source));
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_rgb_to_yuv_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(source.as_slice()).unwrap().unwrap();
    let mut rgb = vec![0u8; yuv.rgb8_len()];
    yuv.write_rgb8(&mut rgb);
    let rgb_source = RgbSliceU8::new(&rgb, (1920, 1080));
    let mut converter = YUVBuffer::new(1920, 1080);

    b.iter(|| {
        black_box(&mut converter).read_rgb(black_box(rgb_source));
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_rgb8_to_yuv_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(source.as_slice()).unwrap().unwrap();
    let mut rgb = vec![0u8; yuv.rgb8_len()];
    yuv.write_rgb8(&mut rgb);
    let rgb_source = RgbSliceU8::new(&rgb, (1920, 1080));
    let mut converter = YUVBuffer::new(1920, 1080);

    b.iter(|| {
        black_box(&mut converter).read_rgb8(black_box(rgb_source));
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_rgb8_scalar_to_yuv_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(source.as_slice()).unwrap().unwrap();
    let mut rgb = vec![0u8; yuv.rgb8_len()];
    yuv.write_rgb8(&mut rgb);
    let mut scalar_rgb = vec![0u8; rgb.len()];
    for (rgb, scalar_rgb) in rgb.chunks_exact(3).zip(scalar_rgb.chunks_exact_mut(3)) {
        scalar_rgb.copy_from_slice(&[rgb[0], rgb[1], rgb[0]]);
    }
    let rgb_source = ScalarRgbSliceU8 {
        data: &scalar_rgb,
        dimensions: (1920, 1080),
    };
    let mut converter = YUVBuffer::new(1920, 1080);

    b.iter(|| {
        black_box(&mut converter).read_rgb8(black_box(rgb_source));
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_rgba8_to_yuv_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(source.as_slice()).unwrap().unwrap();
    let mut rgb = vec![0u8; yuv.rgb8_len()];
    yuv.write_rgb8(&mut rgb);
    let mut rgba = vec![0u8; yuv.rgba8_len()];
    for (rgb, rgba) in rgb.chunks_exact(3).zip(rgba.chunks_exact_mut(4)) {
        rgba[..3].copy_from_slice(rgb);
        rgba[3] = 255;
    }
    let rgba_source = RgbaSliceU8::new(&rgba, (1920, 1080));
    let mut converter = YUVBuffer::new(1920, 1080);

    b.iter(|| {
        black_box(&mut converter).read_rgba8(black_box(rgba_source));
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_bgra8_to_yuv_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(source.as_slice()).unwrap().unwrap();
    let mut rgb = vec![0u8; yuv.rgb8_len()];
    yuv.write_rgb8(&mut rgb);
    let mut bgra = vec![0u8; yuv.rgba8_len()];
    for (rgb, bgra) in rgb.chunks_exact(3).zip(bgra.chunks_exact_mut(4)) {
        bgra[..3].copy_from_slice(&[rgb[2], rgb[1], rgb[0]]);
        bgra[3] = 255;
    }
    let bgra_source = BgraSliceU8::new(&bgra, (1920, 1080));
    let mut converter = YUVBuffer::new(1920, 1080);

    b.iter(|| {
        black_box(&mut converter).read_bgra8(black_box(bgra_source));
    });
}

#[bench]
fn convert_rgb8_to_yuv_1920x1080_padded(b: &mut Bencher) {
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const STRIDE: usize = 1984;

    let src = vec![127_u8; STRIDE * HEIGHT * 3];
    let rgb_source = PaddedRgbSliceU8 {
        data: &src,
        dimensions: (WIDTH, HEIGHT),
        stride: STRIDE,
    };
    let mut converter = YUVBuffer::new(WIDTH, HEIGHT);

    b.iter(|| {
        black_box(&mut converter).read_rgb8(black_box(rgb_source));
    });
}

#[bench]
fn convert_rgb8_to_yuv_3840x2160(b: &mut Bencher) {
    const WIDTH: usize = 3840;
    const HEIGHT: usize = 2160;

    let src = vec![127_u8; WIDTH * HEIGHT * 3];
    let rgb_source = RgbSliceU8::new(&src, (WIDTH, HEIGHT));
    let mut converter = YUVBuffer::new(WIDTH, HEIGHT);

    b.iter(|| {
        black_box(&mut converter).read_rgb8(black_box(rgb_source));
    });
}

#[bench]
fn convert_rgb8_scalar_to_yuv_3840x2160(b: &mut Bencher) {
    const WIDTH: usize = 3840;
    const HEIGHT: usize = 2160;

    let src = vec![127_u8; WIDTH * HEIGHT * 3];
    let rgb_source = ScalarRgbSliceU8 {
        data: &src,
        dimensions: (WIDTH, HEIGHT),
    };
    let mut converter = YUVBuffer::new(WIDTH, HEIGHT);

    b.iter(|| {
        black_box(&mut converter).read_rgb8(black_box(rgb_source));
    });
}

#[bench]
fn convert_rgba8_to_yuv_3840x2160(b: &mut Bencher) {
    const WIDTH: usize = 3840;
    const HEIGHT: usize = 2160;

    let src = vec![127_u8; WIDTH * HEIGHT * 4];
    let rgba_source = RgbaSliceU8::new(&src, (WIDTH, HEIGHT));
    let mut converter = YUVBuffer::new(WIDTH, HEIGHT);

    b.iter(|| {
        black_box(&mut converter).read_rgba8(black_box(rgba_source));
    });
}

#[bench]
fn convert_bgra8_to_yuv_3840x2160(b: &mut Bencher) {
    const WIDTH: usize = 3840;
    const HEIGHT: usize = 2160;

    let src = vec![127_u8; WIDTH * HEIGHT * 4];
    let bgra_source = BgraSliceU8::new(&src, (WIDTH, HEIGHT));
    let mut converter = YUVBuffer::new(WIDTH, HEIGHT);

    b.iter(|| {
        black_box(&mut converter).read_bgra8(black_box(bgra_source));
    });
}
