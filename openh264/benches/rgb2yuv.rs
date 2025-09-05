#![feature(test)]

extern crate test;

use openh264::OpenH264API;
use openh264::decoder::{Decoder, DecoderConfig};
use openh264::formats::{RgbSliceU8, YUVBuffer, YUVSource};
use test::Bencher;

#[bench]
fn convert_rgb_to_yuv_512x512(b: &mut Bencher) {
    let src = include_bytes!("../tests/data/lenna_512x512.rgb");
    let rgb_source = RgbSliceU8::new(src, (512, 512));

    b.iter(|| {
        _ = YUVBuffer::from_rgb_source(rgb_source);
    });
}

#[bench]
fn convert_rgb8_to_yuv_512x512(b: &mut Bencher) {
    let src = include_bytes!("../tests/data/lenna_512x512.rgb");
    let rgb_source = RgbSliceU8::new(src, (512, 512));

    b.iter(|| {
        _ = YUVBuffer::from_rgb8_source(rgb_source);
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
        converter.read_rgb(rgb_source);
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
        converter.read_rgb8(rgb_source);
    });
}
