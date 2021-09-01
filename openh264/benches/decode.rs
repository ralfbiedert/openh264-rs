#![feature(test)]

extern crate test;

use openh264::{Decoder, DecoderConfig};
use openh264_sys2::{SBufferInfo, SDecodingParam, ERROR_CON_IDC, VIDEO_BITSTREAM_TYPE};
use std::ptr::null_mut;
use test::Bencher;

#[bench]
fn decode_rgb_single_512x512_cavlc(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(&config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];

    b.iter(|| {
        let yuv = decoder.xxx_decode(&source[..]).unwrap();
        let dim = yuv.dimension_rgb();
        let rgb_len = dim.0 * dim.1 * 3;

        let tgt = &mut rgb[0..rgb_len];
        yuv.write_rgb8(tgt).unwrap();
    });
}

#[bench]
fn decode_rgb_single_512x512_cabac(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(&config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];

    b.iter(|| {
        let yuv = decoder.xxx_decode(&source[..]).unwrap();
        let dim = yuv.dimension_rgb();
        let rgb_len = dim.0 * dim.1 * 3;

        let tgt = &mut rgb[0..rgb_len];
        yuv.write_rgb8(tgt).unwrap();
    });
}

#[bench]
fn decode_rgb_single_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(&config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];

    b.iter(|| {
        let yuv = decoder.xxx_decode(&source[..]).unwrap();
        let dim = yuv.dimension_rgb();
        let rgb_len = dim.0 * dim.1 * 3;

        let tgt = &mut rgb[0..rgb_len];
        yuv.write_rgb8(tgt).unwrap();
    });
}
