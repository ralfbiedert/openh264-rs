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

macro_rules! gen_range {
    ($range:expr, $count:expr) => {{
        let mut rng = rand::thread_rng();
        (0..1).map(move |_| rand::Rng::gen_range(&mut rng, $range)).cycle().take($count).collect()        
    }};
}

#[bench]
#[cfg(feature = "source")]
fn clamping_i32_u8(b: &mut Bencher) {
    use std::hint::black_box;

    let nums: Vec<i32> = gen_range!(-227..480, 512 * 512 * 3);
    assert_eq!(512 * 512 * 3, nums.len());
    
    let mut dump = [0u8; 1];
    b.iter(|| {
        for n in nums.iter() {
            dump[0] = black_box((*n).clamp(0, 255) as u8);
        }
    });
}

#[bench]
#[cfg(feature = "source")]
fn clamping_f32_u8(b: &mut Bencher) {
    use std::hint::black_box;

    let nums: Vec<i32> = gen_range!(-227..480, 512 * 512 * 3);
    assert_eq!(512 * 512 * 3, nums.len());

    let mut dump = [0u8; 1];
    b.iter(|| {
        for n in nums.iter() {
            // clamps implicitly
            dump[0] = black_box((*n) as u8);
        }
    });
}

#[bench]
#[cfg(feature = "source")]
fn clamping_lookup(b: &mut Bencher) {
    use std::hint::black_box;
    
    /*
        lookup table as described in Etienne Dupuis paper, works like hard-sigmoid function

        Unclamped values:
        yuv 0, 0, 0
        yields rbg -180, 135, -227

        yuv 128, 128, 128
        yields rgb 128, 128, 128

        yuv 255, 255, 255
        yields rgb 433, 120, 480    
    */

    // generate a lookup table where:
    // - the first 227 values are mapped to 0
    // - 227 to (227 + 255) linear mapping from 0 to 255
    // - (227 + 255) to end are mapped to 255
    // for the all possible blue values [-227, 480], we have an indexed lookup,
    // mapping all over/underflowing values into the valid u8 range    
    let mut blue_lookup = [0u8; 480 + 227];
    for (i, v) in (0..(480 + 227 as i32)).map(|i| (i - 227).clamp(0, 255) as u8).enumerate() {
        blue_lookup[i] = v;
    }

    // index 227 is the "origin"
    assert_eq!(0, blue_lookup[227]);
    // values before the origin are mapped to 0
    assert_eq!(0, blue_lookup[226]);
    // values after the origin are mapped to positive numbers
    assert_eq!(1, blue_lookup[228]);    
    assert_eq!(255, blue_lookup[228 + 255]);
    assert_eq!(255, blue_lookup[228 + 256]);
    
    let nums: Vec<isize> = gen_range!(-227..480, 512 * 512 * 3);
    assert_eq!(512 * 512 * 3, nums.len());

    let mut dump = [0u8; 1];
    b.iter(|| {
        for i in &nums {
            dump[0] = black_box(
                blue_lookup[(i + 227) as usize]
            );
        }
    });
}
