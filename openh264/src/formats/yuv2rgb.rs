use super::arch;

const RGB_PIXEL_LEN: usize = 3;
const RGBA_PIXEL_LEN: usize = 4;
const Y_MUL: f32 = 255.0 / 219.0;
const RV_MUL: f32 = 255.0 / 224.0 * 1.402;
const GV_MUL: f32 = -255.0 / 224.0 * 1.402 * 0.299 / 0.587;
const GU_MUL: f32 = -255.0 / 224.0 * 1.772 * 0.114 / 0.587;
const BU_MUL: f32 = 255.0 / 224.0 * 1.772;

/// Write RGB8 data from YUV420 using scalar (non-SIMD) math.
pub fn write_rgb8_scalar(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    write_yuv420_float(y_plane, u_plane, v_plane, dim, strides, target, RGB_PIXEL_LEN);
}

/// Write RGBA8 data from YUV420 using scalar (non-SIMD) math.
pub fn write_rgba8_scalar(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    write_yuv420_float(y_plane, u_plane, v_plane, dim, strides, target, RGBA_PIXEL_LEN);
}

/// Write RGB8 data from YUV420 using AVX2 integer SIMD when available, otherwise portable wide SIMD.
pub fn write_rgb8_simd(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    write_yuv420_simd(y_plane, u_plane, v_plane, dim, strides, target, RGB_PIXEL_LEN);
}

/// Write RGBA8 data from YUV420 using AVX2 integer SIMD when available, otherwise portable wide SIMD.
pub fn write_rgba8_simd(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    write_yuv420_simd(y_plane, u_plane, v_plane, dim, strides, target, RGBA_PIXEL_LEN);
}

#[allow(clippy::cast_possible_truncation)]
const fn clamp_to_u8(value: f32) -> u8 {
    value.clamp(0.0, 255.0) as u8
}

fn write_yuv420_float(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
    pixel_len: usize,
) {
    validate_yuv420_target(dim, target, pixel_len);
    let (width, height) = dim;

    for y in 0..height {
        for x in 0..width {
            let y_value = f32::from(y_plane[y * strides.0 + x]) - 16.0;
            let u_value = f32::from(u_plane[(y / 2) * strides.1 + (x / 2)]) - 128.0;
            let v_value = f32::from(v_plane[(y / 2) * strides.2 + (x / 2)]) - 128.0;

            // Limited-range BT.601.
            let y_value = Y_MUL * y_value;
            let r = RV_MUL.mul_add(v_value, y_value);
            let g = GV_MUL.mul_add(v_value, GU_MUL.mul_add(u_value, y_value));
            let b = BU_MUL.mul_add(u_value, y_value);
            let target = &mut target[(y * width + x) * pixel_len..][..pixel_len];
            target[0] = clamp_to_u8(r);
            target[1] = clamp_to_u8(g);
            target[2] = clamp_to_u8(b);
            if pixel_len == RGBA_PIXEL_LEN {
                target[3] = 255;
            }
        }
    }
}

fn validate_yuv420_target(dim: (usize, usize), target: &[u8], pixel_len: usize) {
    let (width, height) = dim;
    assert_eq!(target.len(), width * height * pixel_len);
}

fn write_yuv420_simd(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
    pixel_len: usize,
) {
    validate_yuv420_target(dim, target, pixel_len);
    assert_eq!(dim.0 % 8, 0);

    arch::write_yuv420_to_rgb(y_plane, u_plane, v_plane, dim, strides, target, pixel_len);
}

#[cfg(test)]
mod test {
    use super::{RGB_PIXEL_LEN, RGBA_PIXEL_LEN, write_rgb8_scalar, write_rgb8_simd, write_rgba8_scalar, write_rgba8_simd};
    use crate::OpenH264API;
    use crate::decoder::{Decoder, DecoderConfig};
    use crate::formats::YUVSource;

    fn assert_rgb_within_one(reference: &[u8], actual: &[u8], pixel_len: usize) {
        assert_eq!(reference.len(), actual.len());
        for (index, (&reference, &actual)) in reference.iter().zip(actual).enumerate() {
            if index % pixel_len == 3 {
                assert_eq!(reference, actual);
            } else {
                assert!(
                    (i16::from(reference) - i16::from(actual)).abs() <= 1,
                    "channel at byte {index} differed: {reference} vs {actual}"
                );
            }
        }
    }

    #[test]
    fn write_rgb8_scalar_range() {
        let mut tgt = vec![0; 3];
        write_rgb8_scalar(&[235], &[128], &[128], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [255, 255, 255]);

        write_rgb8_scalar(&[16], &[128], &[128], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [0, 0, 0]);

        write_rgb8_scalar(&[235], &[240], &[240], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [255, 120, 255]);

        write_rgb8_scalar(&[235], &[0], &[240], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [255, 214, 0]);

        write_rgb8_scalar(&[235], &[240], &[0], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [50, 255, 255]);
    }

    #[test]
    fn write_rgb8_simd_matches_scalar() {
        let source = include_bytes!("../../tests/data/single_512x512_cavlc.h264");
        let mut decoder = Decoder::with_api_config(OpenH264API::from_source(), DecoderConfig::default()).unwrap();
        let yuv = decoder.decode(source).unwrap().unwrap();
        let mut reference = vec![0; yuv.rgb8_len()];
        let mut actual = vec![0; reference.len()];

        write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut reference);
        write_rgb8_simd(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut actual);

        assert_rgb_within_one(&reference, &actual, RGB_PIXEL_LEN);
    }

    #[test]
    fn write_rgba8_simd_matches_scalar() {
        let source = include_bytes!("../../tests/data/single_512x512_cavlc.h264");
        let mut decoder = Decoder::with_api_config(OpenH264API::from_source(), DecoderConfig::default()).unwrap();
        let yuv = decoder.decode(source).unwrap().unwrap();
        let mut reference = vec![0; yuv.rgba8_len()];
        let mut actual = vec![0; reference.len()];

        write_rgba8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut reference);
        write_rgba8_simd(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut actual);

        assert_rgb_within_one(&reference, &actual, RGBA_PIXEL_LEN);
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn simd_handles_padded_yuv420_strides() {
        let dim = (24, 4);
        let strides = (27, 14, 15);
        let y = (0..strides.0 * dim.1).map(|i| (i * 29) as u8).collect::<Vec<_>>();
        let u = (0..strides.1 * (dim.1 / 2)).map(|i| (i * 47) as u8).collect::<Vec<_>>();
        let v = (0..strides.2 * (dim.1 / 2)).map(|i| (i * 73) as u8).collect::<Vec<_>>();
        let mut rgb_reference = vec![0; dim.0 * dim.1 * RGB_PIXEL_LEN];
        let mut rgb_actual = vec![0; rgb_reference.len()];
        let mut rgba_reference = vec![0; dim.0 * dim.1 * RGBA_PIXEL_LEN];
        let mut rgba_actual = vec![0; rgba_reference.len()];

        write_rgb8_scalar(&y, &u, &v, dim, strides, &mut rgb_reference);
        write_rgb8_simd(&y, &u, &v, dim, strides, &mut rgb_actual);
        write_rgba8_scalar(&y, &u, &v, dim, strides, &mut rgba_reference);
        write_rgba8_simd(&y, &u, &v, dim, strides, &mut rgba_actual);

        assert_rgb_within_one(&rgb_reference, &rgb_actual, RGB_PIXEL_LEN);
        assert_rgb_within_one(&rgba_reference, &rgba_actual, RGBA_PIXEL_LEN);
    }

    #[test]
    fn simd_handles_neutral_extremes() {
        let y = [16, 16, 16, 16, 235, 235, 235, 235];
        let u = [128; 4];
        let v = [128; 4];
        let mut target = [0; 24];
        write_rgb8_simd(&y, &u, &v, (8, 1), (8, 4, 4), &mut target);
        assert_eq!(
            target,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
            ]
        );
    }
}
