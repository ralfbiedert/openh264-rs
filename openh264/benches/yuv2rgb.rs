#![feature(test)]

extern crate test;

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::formats::YUVSource;
use openh264::OpenH264API;
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


#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_lookup(b: &mut Bencher) {
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
        yuv.write_rgb8_lookup(tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_int_lookup(b: &mut Bencher) {
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
        yuv.write_rgb8_int_lookup(tgt);
    });
}


#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_int_math(b: &mut Bencher) {
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
        yuv.write_rgb8_int_math(tgt);
    });
}

#[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_x8(b: &mut Bencher) {
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
        yuv.write_rgb8_x8(tgt);
    });
}

// #[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_copy_planes(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");
    
    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();

    b.iter(|| {
        yuv.copy_planes();
    });
}

// #[bench]
#[cfg(feature = "source")]
fn convert_yuv_to_rgb_512x512_copy_planes_x8(b: &mut Bencher) {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");
    
    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();

    b.iter(|| {
        yuv.copy_planes_x8();
    });
}

#[bench]
#[cfg(feature = "source")]
fn clamping_i32_u8(b: &mut Bencher) {
    use std::hint::black_box;

    let nums: Vec<i32> = (0..511).cycle().map(|i| i - 256).take(512 * 512 * 3).collect();
    assert_eq!(512 * 512 * 3, nums.len());
    
    b.iter(|| {
        for n in nums.iter() {
            black_box((*n).clamp(0, 255) as u8);
        }
    });
}

#[bench]
#[cfg(feature = "source")]
fn clamping_f32_u8(b: &mut Bencher) {
    use std::hint::black_box;

    let nums: Vec<f32> = (0..511).cycle().map(|i| i as f32 - 256.).take(512 * 512 * 3).collect();
    assert_eq!(512 * 512 * 3, nums.len());

    b.iter(|| {
        for n in nums.iter() {
            // clamps implicitly
            black_box((*n) as u8);
        }
    });
}

#[bench]
#[cfg(feature = "source")]
fn clamping_lookup(b: &mut Bencher) {
    
    /*
        Unclamped values:
        yuv 0, 0, 0
        yields rbg -180, 135, -227

        yuv 128, 128, 128
        yields rgb 128, 128, 128

        yuv 255, 255, 255
        yields rgb 433, 120, 480    
    */

    let mut blue_lookup = [0u8; 480 + 227];
    for (i, v) in (0..(480 + 227 as i32)).map(|i| (i - 227).clamp(0, 255) as u8).enumerate() {
        blue_lookup[i] = v;
    }

    let p = unsafe { blue_lookup.as_ptr().offset(227) };
    assert_eq!(0, unsafe { *p });
    assert_eq!(0, unsafe { *p.offset(-1) });
    assert_eq!(1, unsafe { *p.offset(1) });
    assert_eq!(255, unsafe { *p.offset(255) });
    assert_eq!(255, unsafe { *p.offset(256) });
    
    let mut dump = [0u8; 512];
    let indices: Vec<isize> = (0..(480 + 227 as i32)).cycle().map(|i| (i - 227) as isize).take(512 * 512 * 3).collect();
    assert_eq!(512 * 512 * 3, indices.len());
    b.iter(|| {
        for i in &indices {
            dump[0] = std::hint::black_box(unsafe { index(p, *i) });
        }
    });
}

#[inline]
unsafe fn index(p: *const u8, i: isize) -> u8 {
    *(p.offset(i))
}
