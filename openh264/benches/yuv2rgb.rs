#![feature(test)]

extern crate test;

use openh264::decoder::{Decoder, DecoderConfig};
use test::Bencher;

#[bench]
fn convert_yuv_to_rgb_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode_no_delay(&source[..]).unwrap();
    let dim = yuv.dimension_rgb();
    let rgb_len = dim.0 * dim.1 * 3;

    let tgt = &mut rgb[0..rgb_len];

    b.iter(|| {
        yuv.write_rgb8(tgt).unwrap();
    });
}

#[bench]
fn convert_yuv_to_rgba_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 4];
    let yuv = decoder.decode_no_delay(&source[..]).unwrap();
    let dim = yuv.dimension_rgb();
    let rgb_len = dim.0 * dim.1 * 4;

    let tgt = &mut rgb[0..rgb_len];

    b.iter(|| {
        yuv.write_rgba8(tgt).unwrap();
    });
}

#[bench]
fn convert_yuv_to_rgb_512x512(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode_no_delay(&source[..]).unwrap();
    let dim = yuv.dimension_rgb();
    let rgb_len = dim.0 * dim.1 * 3;

    let tgt = &mut rgb[0..rgb_len];

    b.iter(|| {
        yuv.write_rgb8(tgt).unwrap();
    });
}
