#![feature(test)]

extern crate test;

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::OpenH264API;
use test::{black_box, Bencher};

#[bench]
#[cfg(feature = "source")]
fn encode_512x512_from_yuv(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(api, config).unwrap();
    let yuv = decoder.decode(source).unwrap().unwrap();

    b.iter(|| {
        let api = OpenH264API::from_source();
        let config = EncoderConfig::new();
        let mut encoder = Encoder::with_config(api, config).unwrap();

        let stream = encoder.encode(&yuv).unwrap();

        black_box(stream);
    });
}

#[bench]
#[cfg(feature = "source")]
fn encode_1920x1080_from_yuv(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(api, config).unwrap();
    let yuv = decoder.decode(source).unwrap().unwrap();

    b.iter(|| {
        let api = OpenH264API::from_source();
        let config = EncoderConfig::new();
        let mut encoder = Encoder::with_config(api, config).unwrap();

        let stream = encoder.encode(&yuv).unwrap();

        black_box(stream);
    });
}
