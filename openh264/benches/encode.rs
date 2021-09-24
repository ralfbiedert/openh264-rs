#![feature(test)]

extern crate test;

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::encoder::{Encoder, EncoderConfig};
use test::{black_box, Bencher};

#[bench]
fn encode_512x512_from_yuv(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config).unwrap();
    let yuv = decoder.decode(source).unwrap();

    b.iter(|| {
        let config = EncoderConfig::new(512, 512);
        let mut encoder = Encoder::with_config(config).unwrap();

        let stream = encoder.encode(&yuv).unwrap();

        black_box(stream);
    });
}

#[bench]
fn encode_1920x1080_from_yuv(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config).unwrap();
    let yuv = decoder.decode(source).unwrap();

    b.iter(|| {
        let config = EncoderConfig::new(1920, 1080);
        let mut encoder = Encoder::with_config(config).unwrap();

        let stream = encoder.encode(&yuv).unwrap();

        black_box(stream);
    });
}
