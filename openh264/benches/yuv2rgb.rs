#![feature(test)]

extern crate test;

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::formats::YUVSource;
use openh264::OpenH264API;
use test::Bencher;

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_1920x1080_scalar(b: &mut Bencher) {
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
        openh264::decoder::DecodedYUV::write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_1920x1080_f32x8(b: &mut Bencher) {
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
        openh264::decoder::DecodedYUV::write_rgb8_f32x8(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);
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
        openh264::decoder::DecodedYUV::write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_f32x8(b: &mut Bencher) {
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
        openh264::decoder::DecodedYUV::write_rgb8_f32x8(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);
    });
}

/// collect a Vec of random numbers within the specified range
macro_rules! gen_range {
    ($range:expr, $count:expr) => {{
        let mut rng = rand::thread_rng();
        (0..1).map(move |_| rand::Rng::gen_range(&mut rng, $range)).cycle().take($count).collect()        
    }};
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_4096x2160_scalar(b: &mut Bencher) {    
    let (im_w, im_h) = (4096, 2160);
    let (y_len, uv_len) = (im_w * im_h, (im_w * im_h) / 4);
    let strides  = (im_w, im_w / 2, im_w / 2);
    
    let y_plane: Vec<u8> = gen_range!(0..=255u8, y_len);
    let u_plane: Vec<u8> = gen_range!(0..=255u8, uv_len);
    let v_plane: Vec<u8> = gen_range!(0..=255u8, uv_len);

    let mut tgt = vec![0; im_w * im_h * 3];
    b.iter(|| {
        openh264::decoder::DecodedYUV::write_rgb8_scalar(&y_plane, &u_plane, &v_plane, (im_w, im_h), strides, &mut tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_4096x2160_f32x8(b: &mut Bencher) {    
    let (im_w, im_h) = (4096, 2160);
    let (y_len, uv_len) = (im_w * im_h, (im_w * im_h) / 4);
    let strides  = (im_w, im_w / 2, im_w / 2);
    
    let y_plane: Vec<u8> = gen_range!(0..=255u8, y_len);
    let u_plane: Vec<u8> = gen_range!(0..=255u8, uv_len);
    let v_plane: Vec<u8> = gen_range!(0..=255u8, uv_len);

    let mut tgt = vec![0; im_w * im_h * 3];
    b.iter(|| {
        openh264::decoder::DecodedYUV::write_rgb8_f32x8(&y_plane, &u_plane, &v_plane, (im_w, im_h), strides, &mut tgt);
    });
}
