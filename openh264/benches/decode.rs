#![feature(test)]

extern crate test;

use openh264::decoder::Decoder;
use openh264::formats::YUVSource;
use test::{black_box, Bencher};

#[bench]
#[cfg(feature = "source")]
fn decode_yuv_single_512x512_cavlc(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");
    let mut decoder = Decoder::new().unwrap();

    b.iter(|| {
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();

        black_box(dim);
    });
}

#[bench]
#[cfg(feature = "source")]
fn decode_yuv_single_512x512_cabac(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");
    let mut decoder = Decoder::new().unwrap();

    b.iter(|| {
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();

        black_box(dim);
    });
}

#[bench]
#[cfg(feature = "source")]
fn decode_yuv_single_1920x1080(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_1920x1080_cabac.h264");
    let mut decoder = Decoder::new().unwrap();

    b.iter(|| {
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();

        black_box(dim);
    });
}

#[bench]
#[cfg(feature = "source")]
fn decode_yuv_multi_512x512(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/multi_512x512.h264");
    let mut decoder = Decoder::new().unwrap();

    b.iter(|| {
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();

        black_box(dim);
    });
}

#[bench]
#[cfg(feature = "source")]
fn whole_decoder(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");

    b.iter(|| {
        let mut decoder = Decoder::new().unwrap();
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();

        black_box(dim);
    });
}
