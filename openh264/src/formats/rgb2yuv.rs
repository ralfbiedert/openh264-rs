use crate::formats::rgb::RGB8Source;
use crate::formats::{RGBSource, arch};

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

/// Writes a contiguous RGB8-compatible source into 420 Y, U, and V buffers.
///
/// Uses the fastest supported conversion implementation.
#[allow(clippy::needless_pass_by_value)]
pub fn write_yuv(rgb: impl RGB8Source, dimensions: (usize, usize), y_buf: &mut [u8], u_buf: &mut [u8], v_buf: &mut [u8]) {
    // Make sure we only attempt to read sources that match our own size.
    assert_eq!(rgb.dimensions(), dimensions);

    let dimensions_padded = rgb.dimensions_padded();
    let rgb8_data = rgb.rgb8_data();
    let layout = arch::Rgb8SourceLayout {
        dimensions,
        dimensions_padded,
        pixel_stride: rgb.pixel_stride(),
        rgb_offsets: rgb.rgb_channel_offsets(),
    };
    let mut target = arch::Yuv420Planes {
        y: y_buf,
        u: u_buf,
        v: v_buf,
    };

    arch::write_rgb8_to_yuv420(rgb8_data, layout, &mut target);
}

#[cfg(test)]
mod test {
    use crate::OpenH264API;
    use crate::decoder::{Decoder, DecoderConfig};
    use crate::formats::arch::{self, Rgb8SourceLayout, Yuv420Planes};
    use crate::formats::rgb2yuv::{write_yuv, write_yuv_by_pixel};
    use crate::formats::{RGB8Source, RGBSource, RgbSliceU8, YUVSource};
    use std::iter::zip;

    #[derive(Copy, Clone)]
    struct PaddedRgbSliceU8<'a> {
        data: &'a [u8],
        dimensions: (usize, usize),
        stride: usize,
    }

    impl RGBSource for PaddedRgbSliceU8<'_> {
        fn dimensions(&self) -> (usize, usize) {
            self.dimensions
        }

        fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
            let offset = (y * self.stride + x) * 3;
            (
                f32::from(self.data[offset]),
                f32::from(self.data[offset + 1]),
                f32::from(self.data[offset + 2]),
            )
        }
    }

    impl RGB8Source for PaddedRgbSliceU8<'_> {
        fn dimensions_padded(&self) -> (usize, usize) {
            (self.stride, self.dimensions.1)
        }

        fn rgb8_data(&self) -> &[u8] {
            self.data
        }
    }

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
        let layout = Rgb8SourceLayout {
            dimensions: dim,
            dimensions_padded: rgb_slice.dimensions_padded(),
            pixel_stride: rgb_slice.pixel_stride(),
            rgb_offsets: rgb_slice.rgb_channel_offsets(),
        };
        let mut target = Yuv420Planes {
            y: &mut y_scalar,
            u: &mut u_scalar,
            v: &mut v_scalar,
        };
        arch::scalar::write_rgb8_to_yuv420(rgb_slice.rgb8_data(), layout, &mut target);

        let almost_equal = |a: &[u8], b: &[u8]| zip(a, b).map(|(x, y)| u8::abs_diff(*x, *y)).all(|x| x <= 1);

        assert!(almost_equal(&y_by_pixel, &y_scalar));
        assert!(almost_equal(&u_by_pixel, &u_scalar));
        assert!(almost_equal(&v_by_pixel, &v_scalar));
    }

    #[test]
    fn accelerated_rgb24_matches_scalar_for_packed_and_padded_rows() {
        const DIMENSIONS: (usize, usize) = (74, 4);
        const STRIDE: usize = 80;
        let padded_data = (0..STRIDE * DIMENSIONS.1 * 3)
            .map(|index| (index.wrapping_mul(47) % 251) as u8)
            .collect::<Vec<_>>();
        let packed_data = (0..DIMENSIONS.1)
            .flat_map(|row| &padded_data[row * STRIDE * 3..row * STRIDE * 3 + DIMENSIONS.0 * 3])
            .copied()
            .collect::<Vec<_>>();
        let packed = RgbSliceU8::new(&packed_data, DIMENSIONS);
        let padded = PaddedRgbSliceU8 {
            data: &padded_data,
            dimensions: DIMENSIONS,
            stride: STRIDE,
        };

        let mut y_reference = vec![0; DIMENSIONS.0 * DIMENSIONS.1];
        let mut u_reference = vec![0; DIMENSIONS.0 * DIMENSIONS.1 / 4];
        let mut v_reference = vec![0; DIMENSIONS.0 * DIMENSIONS.1 / 4];
        let layout = Rgb8SourceLayout {
            dimensions: DIMENSIONS,
            dimensions_padded: packed.dimensions_padded(),
            pixel_stride: packed.pixel_stride(),
            rgb_offsets: packed.rgb_channel_offsets(),
        };
        let mut target = Yuv420Planes {
            y: &mut y_reference,
            u: &mut u_reference,
            v: &mut v_reference,
        };
        arch::scalar::write_rgb8_to_yuv420(packed.rgb8_data(), layout, &mut target);

        let mut y_packed = vec![0; y_reference.len()];
        let mut u_packed = vec![0; u_reference.len()];
        let mut v_packed = vec![0; v_reference.len()];
        write_yuv(packed, DIMENSIONS, &mut y_packed, &mut u_packed, &mut v_packed);

        let mut y_padded = vec![0; y_reference.len()];
        let mut u_padded = vec![0; u_reference.len()];
        let mut v_padded = vec![0; v_reference.len()];
        write_yuv(padded, DIMENSIONS, &mut y_padded, &mut u_padded, &mut v_padded);

        assert_eq!(y_packed, y_reference);
        assert_eq!(u_packed, u_reference);
        assert_eq!(v_packed, v_reference);
        assert_eq!(y_padded, y_reference);
        assert_eq!(u_padded, u_reference);
        assert_eq!(v_padded, v_reference);
    }
}
