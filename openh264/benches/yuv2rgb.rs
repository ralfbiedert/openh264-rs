#![feature(test)]

extern crate test;

use openh264::OpenH264API;
use openh264::decoder::{Decoder, DecoderConfig};
use openh264::formats::YUVSource;
use test::Bencher;

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;

    let tgt = &mut rgb[0..rgb_len];

    b.iter(|| {
        yuv.write_rgb8(tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgba_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 4];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 4;

    let tgt = &mut rgb[0..rgb_len];

    b.iter(|| {
        yuv.write_rgba8(tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;

    let tgt = &mut rgb[0..rgb_len];

    b.iter(|| {
        yuv.write_rgb8(tgt);
    });
}
