#![feature(test)]

extern crate test;

use openh264::decoder::Decoder;
use openh264::encoder::Encoder;
use test::{black_box, Bencher};

#[bench]
#[cfg(feature = "source")]
fn encode_512x512_from_yuv(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let mut decoder = Decoder::new().unwrap();
    let yuv = decoder.decode(source).unwrap().unwrap();

    b.iter(|| {
        let mut encoder = Encoder::new().unwrap();

        let stream = encoder.encode(&yuv).unwrap();

        black_box(stream);
    });
}

#[bench]
#[cfg(feature = "source")]
fn encode_1920x1080_from_yuv(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");

    let mut decoder = Decoder::new().unwrap();
    let yuv = decoder.decode(source).unwrap().unwrap();

    b.iter(|| {
        let mut encoder = Encoder::new().unwrap();
        let stream = encoder.encode(&yuv).unwrap();

        black_box(stream);
    });
}
