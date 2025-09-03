use crate::formats::RGBSource;
use crate::formats::rgb::RGB8Source;

/// Writes an RGB source into 420 Y, U and V buffers.
#[allow(clippy::needless_pass_by_value)]
pub fn write_yuv_by_pixel(rgb: impl RGBSource, dimensions: (usize, usize), y_buf: &mut [u8], u_buf: &mut [u8], v_buf: &mut [u8]) {
    // Make sure we only attempt to read sources that match our own size.
    assert_eq!(rgb.dimensions(), dimensions);

    let width = dimensions.0;
    let height = dimensions.1;
    let half_width = width / 2;

    // y is full size, u, v is quarter size
    let mut write_y = |x: usize, y: usize, rgb: (f32, f32, f32)| {
        y_buf[x + y * width] = (0.09765625f32.mul_add(rgb.2, 0.2578125f32.mul_add(rgb.0, 0.50390625 * rgb.1)) + 16.0) as u8;
    };

    let mut write_u = |x: usize, y: usize, rgb: (f32, f32, f32)| {
        u_buf[x + y * half_width] = (0.4375f32.mul_add(rgb.2, (-0.1484375f32).mul_add(rgb.0, -0.2890625 * rgb.1)) + 128.0) as u8;
    };

    let mut write_v = |x: usize, y: usize, rgb: (f32, f32, f32)| {
        v_buf[x + y * half_width] = ((-0.0703125f32).mul_add(rgb.2, 0.4375f32.mul_add(rgb.0, -0.3671875 * rgb.1)) + 128.0) as u8;
    };

    for i in 0..width / 2 {
        for j in 0..height / 2 {
            let px = i * 2;
            let py = j * 2;
            let pix0x0 = rgb.pixel_f32(px, py);
            let pix0x1 = rgb.pixel_f32(px, py + 1);
            let pix1x0 = rgb.pixel_f32(px + 1, py);
            let pix1x1 = rgb.pixel_f32(px + 1, py + 1);
            let avg_pix = (
                (pix0x0.0 as u32 + pix0x1.0 as u32 + pix1x0.0 as u32 + pix1x1.0 as u32) as f32 / 4.0,
                (pix0x0.1 as u32 + pix0x1.1 as u32 + pix1x0.1 as u32 + pix1x1.1 as u32) as f32 / 4.0,
                (pix0x0.2 as u32 + pix0x1.2 as u32 + pix1x0.2 as u32 + pix1x1.2 as u32) as f32 / 4.0,
            );

            write_y(px, py, pix0x0);
            write_y(px, py + 1, pix0x1);
            write_y(px + 1, py, pix1x0);
            write_y(px + 1, py + 1, pix1x1);
            write_u(i, j, avg_pix);
            write_v(i, j, avg_pix);
        }
    }
}

/// Writes an RGB8 source into 420 Y, U and V buffers.
///
/// TODO: We want a faster SIMD version of this.
#[allow(clippy::needless_pass_by_value)]
pub fn write_yuv_scalar(rgb: impl RGB8Source, dimensions: (usize, usize), y_buf: &mut [u8], u_buf: &mut [u8], v_buf: &mut [u8]) {
    // Make sure we only attempt to read sources that match our own size.
    assert_eq!(rgb.dimensions(), dimensions);

    let dimensions_padded = rgb.dimensions_padded();
    let width = dimensions.0;

    let half_width = width / 2;
    let rgb8_data = rgb.rgb8_data();

    for (pix, y) in rgb8_data.chunks_exact(3).zip(y_buf.iter_mut()) {
        *y = (((66 * u32::from(pix[0]) + 129 * u32::from(pix[1]) + 25 * u32::from(pix[2])) >> 8) + 16) as u8;
    }

    let r1 = rgb8_data.chunks_exact(dimensions_padded.0 * 3).step_by(2);
    let r2 = rgb8_data.chunks_exact(dimensions_padded.0 * 3).skip(1).step_by(2);

    let u_rows = u_buf.chunks_exact_mut(half_width);
    let v_rows = v_buf.chunks_exact_mut(half_width);
    for (((r1, r2), u), v) in r1.zip(r2).zip(u_rows).zip(v_rows) {
        for (((pix0, pix1), u), v) in r1.chunks_exact(2 * 3).zip(r2.chunks_exact(2 * 3)).zip(u).zip(v) {
            let r = (i16::from(pix0[0]) + i16::from(pix0[3]) + i16::from(pix1[0]) + i16::from(pix1[3]) + 2) / 4;
            let g = (i16::from(pix0[1]) + i16::from(pix0[4]) + i16::from(pix1[1]) + i16::from(pix1[4]) + 2) / 4;
            let b = (i16::from(pix0[2]) + i16::from(pix0[5]) + i16::from(pix1[2]) + i16::from(pix1[5]) + 2) / 4;

            *u = (((-38 * r + 112 * b - 74 * g) >> 8) + 128) as u8;
            *v = (((112 * r - 18 * b - 94 * g) >> 8) + 128) as u8;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::OpenH264API;
    use crate::decoder::{Decoder, DecoderConfig};
    use crate::formats::rgb2yuv::{write_yuv_by_pixel, write_yuv_scalar};
    use crate::formats::{RgbSliceU8, YUVSource};
    use std::iter::zip;

    #[test]
    fn write_yuv_by_pixel_matches_scalar() {
        let source = include_bytes!("../../tests/data/single_512x512_cavlc.h264");

        let api = OpenH264API::from_source();
        let config = DecoderConfig::default();
        let mut decoder = Decoder::with_api_config(api, config).unwrap();

        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();
        let mut rgb = vec![0; dim.0 * dim.1 * 3];

        yuv.write_rgb8(&mut rgb);

        let rgb_slice = RgbSliceU8::new(&rgb, dim);

        let mut y_by_pixel = vec![0_u8; dim.0 * dim.1];
        let mut u_by_pixel = vec![0_u8; dim.0 * dim.1 / 2];
        let mut v_by_pixel = vec![0_u8; dim.0 * dim.1 / 2];

        let mut y_scalar = vec![0_u8; dim.0 * dim.1];
        let mut u_scalar = vec![0_u8; dim.0 * dim.1 / 2];
        let mut v_scalar = vec![0_u8; dim.0 * dim.1 / 2];

        write_yuv_by_pixel(rgb_slice, dim, &mut y_by_pixel, &mut u_by_pixel, &mut v_by_pixel);
        write_yuv_scalar(rgb_slice, dim, &mut y_scalar, &mut u_scalar, &mut v_scalar);

        let almost_equal = |a: &[u8], b: &[u8]| zip(a, b).map(|(x, y)| u8::abs_diff(*x, *y)).all(|x| x <= 1);

        assert!(almost_equal(&y_by_pixel, &y_scalar));
        assert!(almost_equal(&u_by_pixel, &u_scalar));
        assert!(almost_equal(&v_by_pixel, &v_scalar));
    }
}
