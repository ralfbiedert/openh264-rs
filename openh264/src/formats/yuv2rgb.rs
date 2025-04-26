/// Converts 8 float values into a f32x8 SIMD lane, taking into account block size.
///
/// If you have a (pixel buffer) slice of at least 8 f32 values like so `[012345678...]`, this function
/// will convert the first N <= 8 elements into a packed f32x8 SIMD struct. For example
///
/// - if block size `1` (like for Y values), you will get  `f32x8(012345678)`.
/// - if block size is `2` (for U and V), you will get `f32x8(00112233)`
macro_rules! f32x8_from_slice_with_blocksize {
    ($buf:expr, $block_size:expr) => {{
        wide::f32x8::from([
            (f32::from($buf[0])),
            (f32::from($buf[1 / $block_size])),
            (f32::from($buf[2 / $block_size])),
            (f32::from($buf[3 / $block_size])),
            (f32::from($buf[4 / $block_size])),
            (f32::from($buf[5 / $block_size])),
            (f32::from($buf[6 / $block_size])),
            (f32::from($buf[7 / $block_size])),
        ])
    }};
}

const Y_MUL: f32 = 255.0 / 219.0;
const RV_MUL: f32 = 255.0 / 224.0 * 1.402;
const GV_MUL: f32 = -255.0 / 224.0 * 1.402 * 0.299 / 0.687;
const GU_MUL: f32 = -255.0 / 224.0 * 1.772 * 0.114 / 0.587;
const BU_MUL: f32 = 255.0 / 224.0 * 1.772;

/// Write RGB8 data from YUV420 using scalar (non SIMD) math.
#[allow(dead_code)]
pub fn write_rgb8_scalar(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    for y in 0..dim.1 {
        for x in 0..dim.0 {
            let base_tgt = (y * dim.0 + x) * 3;
            let base_y = y * strides.0 + x;
            let base_u = (y / 2 * strides.1) + (x / 2);
            let base_v = (y / 2 * strides.2) + (x / 2);

            let rgb_pixel = &mut target[base_tgt..base_tgt + 3];

            // Convert limited range YUV to RGB
            // https://en.wikipedia.org/wiki/YCbCr#ITU-R_BT.601_conversion
            let y_mul = Y_MUL * (f32::from(y_plane[base_y]) - 16.0);
            let u = f32::from(u_plane[base_u]) - 128.0;
            let v = f32::from(v_plane[base_v]) - 128.0;

            rgb_pixel[0] = RV_MUL.mul_add(v, y_mul) as u8;
            rgb_pixel[1] = GV_MUL.mul_add(v, GU_MUL.mul_add(u, y_mul)) as u8;
            rgb_pixel[2] = BU_MUL.mul_add(u, y_mul) as u8;
        }
    }
}

/// Write RGB8 data from YUV420 using scalar (non SIMD) math.
#[allow(dead_code)]
pub fn write_rgb8_scalar_par(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    // distribute data across threads
    // the call to `std::thread::available_parallelism()` takes quite long (77 micros for me)
    const NUM_THREADS: usize = 4;

    // split output slices
    let pixels_per_thread = (dim.0 * dim.1 * 3) / NUM_THREADS;
    let target_chunks = target.chunks_mut(pixels_per_thread);

    // input planes
    let rows_per_thread = dim.1 / NUM_THREADS;
    let mut row_indices: Vec<(usize, usize)> = (0..NUM_THREADS)
        .map(|i| (i * rows_per_thread, (i + 1) * rows_per_thread))
        .collect();
    // add more rows to the last thread, if not able to distribute evenly
    // --> mirror behavior from chunks_mut
    row_indices[NUM_THREADS - 1].1 += dim.1 % NUM_THREADS;

    std::thread::scope(|s| {
        for (target, (row_start, row_end)) in target_chunks.zip(row_indices) {
            s.spawn(move || {
                for y in row_start..row_end {
                    for x in 0..dim.0 {
                        let base_tgt = ((y - row_start) * dim.0 + x) * 3;
                        let base_y = y * strides.0 + x;
                        let base_u = (y / 2 * strides.1) + (x / 2);
                        let base_v = (y / 2 * strides.2) + (x / 2);

                        let rgb_pixel = &mut target[base_tgt..base_tgt + 3];

                        let y = f32::from(y_plane[base_y]);
                        let u = f32::from(u_plane[base_u]);
                        let v = f32::from(v_plane[base_v]);

                        rgb_pixel[0] = 1.402f32.mul_add(v - 128.0, y) as u8;
                        rgb_pixel[1] = 0.714f32.mul_add(-(v - 128.0), 0.344f32.mul_add(-(u - 128.0), y)) as u8;
                        rgb_pixel[2] = 1.772f32.mul_add(u - 128.0, y) as u8;
                    }
                }
            });
        }
    });
}

/// Write RGB8 data from YUV420 using f32x8 SIMD.
#[allow(clippy::identity_op)]
#[allow(dead_code)]
pub fn write_rgb8_f32x8(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    const RGB_PIXEL_LEN: usize = 3;

    // this assumes we are decoding YUV420
    assert_eq!(y_plane.len(), u_plane.len() * 4);
    assert_eq!(y_plane.len(), v_plane.len() * 4);
    assert_eq!(dim.0 % 8, 0);

    let (width, height) = dim;
    let rgb_bytes_per_row: usize = RGB_PIXEL_LEN * width; // rgb pixel size in bytes

    for y in 0..(height / 2) {
        // load U and V values for two rows of pixels
        let base_u = y * strides.1;
        let u_row = &u_plane[base_u..base_u + strides.1];
        let base_v = y * strides.2;
        let v_row = &v_plane[base_v..base_v + strides.2];

        // load Y values for first row
        let base_y = 2 * y * strides.0;
        let y_row = &y_plane[base_y..base_y + strides.0];

        // calculate first RGB row
        let base_tgt = 2 * y * rgb_bytes_per_row;
        let row_target = &mut target[base_tgt..base_tgt + rgb_bytes_per_row];
        write_rgb8_f32x8_row(y_row, u_row, v_row, width, row_target);

        // load Y values for second row
        let base_y = (2 * y + 1) * strides.0;
        let y_row = &y_plane[base_y..base_y + strides.0];

        // calculate second RGB row
        let base_tgt = (2 * y + 1) * rgb_bytes_per_row;
        let row_target = &mut target[base_tgt..(base_tgt + rgb_bytes_per_row)];
        write_rgb8_f32x8_row(y_row, u_row, v_row, width, row_target);
    }
}

/// Write RGB8 data from YUV420 using f32x8 SIMD.
#[allow(clippy::identity_op)]
pub fn write_rgb8_f32x8_par(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    const RGB_PIXEL_LEN: usize = 3;
    // the call to `std::thread::available_parallelism()` takes quite long (77 micros for me)
    const NUM_THREADS: usize = 4;

    // this assumes we are decoding YUV420
    assert_eq!(y_plane.len(), u_plane.len() * 4);
    assert_eq!(y_plane.len(), v_plane.len() * 4);
    assert_eq!(dim.0 % 8, 0);

    let (width, _height) = dim;
    let rgb_bytes_per_row: usize = RGB_PIXEL_LEN * width; // rgb pixel size in bytes

    // distribute data across threads
    let rows_per_thread = dim.1 / NUM_THREADS;
    let chunk_sz = (dim.0 * dim.1 * RGB_PIXEL_LEN) / NUM_THREADS;
    let target_chunks = target.chunks_mut(chunk_sz).enumerate();

    std::thread::scope(|s| {
        for (i, target) in target_chunks {
            s.spawn(move || {
                let range = 0..(rows_per_thread / 2);
                let offset = i * (rows_per_thread / 2);
                for y in range {
                    // load U and V values for two rows of pixels
                    let base_u = (y + offset) * strides.1;
                    let u_row = &u_plane[base_u..base_u + strides.1];
                    let base_v = (y + offset) * strides.2;
                    let v_row = &v_plane[base_v..base_v + strides.2];

                    // load Y values for first row
                    let base_y = 2 * (y + offset) * strides.0;
                    let y_row = &y_plane[base_y..base_y + strides.0];

                    // calculate first RGB row
                    let base_tgt = 2 * y * rgb_bytes_per_row;
                    let row_target = &mut target[base_tgt..base_tgt + rgb_bytes_per_row];
                    write_rgb8_f32x8_row(y_row, u_row, v_row, width, row_target);

                    // load Y values for second row
                    let base_y = (2 * (y + offset) + 1) * strides.0;
                    let y_row = &y_plane[base_y..base_y + strides.0];

                    // calculate second RGB row
                    let base_tgt = (2 * y + 1) * rgb_bytes_per_row;
                    let row_target = &mut target[base_tgt..(base_tgt + rgb_bytes_per_row)];
                    write_rgb8_f32x8_row(y_row, u_row, v_row, width, row_target);
                }
            });
        }
    });
}

/// Write a single RGB8 row from YUV420 row data using f32x8 SIMD.
#[allow(clippy::inline_always)]
#[allow(clippy::similar_names)]
#[inline(always)]
fn write_rgb8_f32x8_row(y_row: &[u8], u_row: &[u8], v_row: &[u8], width: usize, target: &mut [u8]) {
    const STEP: usize = 8;
    const UV_STEP: usize = STEP / 2;
    const TGT_STEP: usize = STEP * 3;

    assert_eq!(y_row.len(), u_row.len() * 2);
    assert_eq!(y_row.len(), v_row.len() * 2);

    let y_mul = wide::f32x8::splat(Y_MUL);
    let rv_mul = wide::f32x8::splat(RV_MUL);
    let gu_mul = wide::f32x8::splat(GU_MUL);
    let gv_mul = wide::f32x8::splat(GV_MUL);
    let bu_mul = wide::f32x8::splat(BU_MUL);

    let upper_bound = wide::f32x8::splat(255.0);
    let lower_bound = wide::f32x8::splat(0.0);

    assert_eq!(y_row.len() % STEP, 0);

    assert_eq!(u_row.len() % UV_STEP, 0);
    assert_eq!(v_row.len() % UV_STEP, 0);

    assert_eq!(target.len() % TGT_STEP, 0);

    let mut base_y = 0;
    let mut base_uv = 0;
    let mut base_tgt = 0;

    for _ in (0..width).step_by(STEP) {
        let pixels = &mut target[base_tgt..(base_tgt + TGT_STEP)];

        let y_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(y_row[base_y..], 1) - 16.0;
        let y_mul: wide::f32x8 = y_pack * y_mul;
        let u_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(u_row[base_uv..], 2) - 128.0;
        let v_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(v_row[base_uv..], 2) - 128.0;

        let r_pack = v_pack.mul_add(rv_mul, y_mul);
        let g_pack = v_pack.mul_add(gv_mul, u_pack.mul_add(gu_mul, y_mul));
        let b_pack = u_pack.mul_add(bu_mul, y_mul);

        let (r_pack, g_pack, b_pack) = (
            r_pack.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int(),
            g_pack.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int(),
            b_pack.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int(),
        );

        let (r_pack, g_pack, b_pack) = (r_pack.as_array_ref(), g_pack.as_array_ref(), b_pack.as_array_ref());

        for i in 0..STEP {
            pixels[3 * i] = r_pack[i] as u8;
            pixels[(3 * i) + 1] = g_pack[i] as u8;
            pixels[(3 * i) + 2] = b_pack[i] as u8;
        }

        base_y += STEP;
        base_uv += UV_STEP;
        base_tgt += TGT_STEP;
    }
}

/// Write RGBA8 data from YUV420 using scalar (non SIMD) math.
pub fn write_rgba8_scalar(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    for y in 0..dim.1 {
        for x in 0..dim.0 {
            let base_tgt = (y * dim.0 + x) * 4;
            let base_y = y * strides.0 + x;
            let base_u = (y / 2 * strides.1) + (x / 2);
            let base_v = (y / 2 * strides.2) + (x / 2);

            let rgb_pixel = &mut target[base_tgt..base_tgt + 4];

            // Convert limited range YUV to RGB
            // https://en.wikipedia.org/wiki/YCbCr#ITU-R_BT.601_conversion
            let y_mul = Y_MUL * (f32::from(y_plane[base_y]) - 16.0);
            let u = f32::from(u_plane[base_u]) - 128.0;
            let v = f32::from(v_plane[base_v]) - 128.0;

            rgb_pixel[0] = RV_MUL.mul_add(v, y_mul) as u8;
            rgb_pixel[1] = GV_MUL.mul_add(v, GU_MUL.mul_add(u, y_mul)) as u8;
            rgb_pixel[2] = BU_MUL.mul_add(u, y_mul) as u8;
            rgb_pixel[3] = 255;
        }
    }
}

/// Write RGB8 data from YUV420 using f32x8 SIMD.
#[allow(clippy::identity_op)]
pub fn write_rgba8_f32x8(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    const RGBA_PIXEL_LEN: usize = 4;

    // this assumes we are decoding YUV420
    assert_eq!(y_plane.len(), u_plane.len() * 4);
    assert_eq!(y_plane.len(), v_plane.len() * 4);
    assert_eq!(dim.0 % 8, 0);

    let (width, height) = dim;
    let rgba_bytes_per_row: usize = RGBA_PIXEL_LEN * width; // rgba pixel size in bytes

    for y in 0..(height / 2) {
        // load U and V values for two rows of pixels
        let base_u = y * strides.1;
        let u_row = &u_plane[base_u..base_u + strides.1];
        let base_v = y * strides.2;
        let v_row = &v_plane[base_v..base_v + strides.2];

        // load Y values for first row
        let base_y = 2 * y * strides.0;
        let y_row = &y_plane[base_y..base_y + strides.0];

        // calculate first RGB row
        let base_tgt = 2 * y * rgba_bytes_per_row;
        let row_target = &mut target[base_tgt..base_tgt + rgba_bytes_per_row];
        write_rgba8_f32x8_row(y_row, u_row, v_row, width, row_target);

        // load Y values for second row
        let base_y = (2 * y + 1) * strides.0;
        let y_row = &y_plane[base_y..base_y + strides.0];

        // calculate second RGB row
        let base_tgt = (2 * y + 1) * rgba_bytes_per_row;
        let row_target = &mut target[base_tgt..(base_tgt + rgba_bytes_per_row)];
        write_rgba8_f32x8_row(y_row, u_row, v_row, width, row_target);
    }
}

/// Write a single RGB8 row from YUV420 row data using f32x8 SIMD.
#[allow(clippy::inline_always)]
#[allow(clippy::similar_names)]
#[inline(always)]
fn write_rgba8_f32x8_row(y_row: &[u8], u_row: &[u8], v_row: &[u8], width: usize, target: &mut [u8]) {
    const STEP: usize = 8;
    const UV_STEP: usize = STEP / 2;
    const TGT_STEP: usize = STEP * 4;

    assert_eq!(y_row.len(), u_row.len() * 2);
    assert_eq!(y_row.len(), v_row.len() * 2);

    let y_mul = wide::f32x8::splat(Y_MUL);
    let rv_mul = wide::f32x8::splat(RV_MUL);
    let gu_mul = wide::f32x8::splat(GU_MUL);
    let gv_mul = wide::f32x8::splat(GV_MUL);
    let bu_mul = wide::f32x8::splat(BU_MUL);

    let upper_bound = wide::f32x8::splat(255.0);
    let lower_bound = wide::f32x8::splat(0.0);

    assert_eq!(y_row.len() % STEP, 0);

    assert_eq!(u_row.len() % UV_STEP, 0);
    assert_eq!(v_row.len() % UV_STEP, 0);

    assert_eq!(target.len() % TGT_STEP, 0);

    let mut base_y = 0;
    let mut base_uv = 0;
    let mut base_tgt = 0;

    for _ in (0..width).step_by(STEP) {
        let pixels = &mut target[base_tgt..(base_tgt + TGT_STEP)];

        let y_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(y_row[base_y..], 1) - 16.0;
        let y_mul: wide::f32x8 = y_pack * y_mul;
        let u_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(u_row[base_uv..], 2) - 128.0;
        let v_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(v_row[base_uv..], 2) - 128.0;

        let r_pack = v_pack.mul_add(rv_mul, y_mul);
        let g_pack = v_pack.mul_add(gv_mul, u_pack.mul_add(gu_mul, y_mul));
        let b_pack = u_pack.mul_add(bu_mul, y_mul);

        let (r_pack, g_pack, b_pack) = (
            r_pack.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int(),
            g_pack.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int(),
            b_pack.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int(),
        );

        let (r_pack, g_pack, b_pack) = (r_pack.as_array_ref(), g_pack.as_array_ref(), b_pack.as_array_ref());

        for i in 0..STEP {
            pixels[3 * i] = r_pack[i] as u8;
            pixels[(3 * i) + 1] = g_pack[i] as u8;
            pixels[(3 * i) + 2] = b_pack[i] as u8;
            pixels[(3 * i) + 3] = 255;
        }

        base_y += STEP;
        base_uv += UV_STEP;
        base_tgt += TGT_STEP;
    }
}
#[cfg(test)]
mod test {
    use crate::decoder::{Decoder, DecoderConfig};
    use crate::formats::yuv2rgb::{write_rgb8_f32x8, write_rgb8_f32x8_par, write_rgb8_scalar, write_rgb8_scalar_par};
    use crate::formats::YUVSource;
    use crate::OpenH264API;
    use crate::decoder::{Decoder, DecoderConfig};
    use crate::formats::YUVSource;
    use crate::formats::yuv2rgb::{write_rgb8_f32x8, write_rgb8_scalar, write_rgba8_scalar};

    #[test]
    fn write_rgb8_scalar_range() {
        let mut tgt = vec![0; 3];
        write_rgb8_scalar(&[235], &[128], &[128], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [255, 255, 255]);

        write_rgb8_scalar(&[16], &[128], &[128], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [0, 0, 0]);

        write_rgb8_scalar(&[235], &[240], &[240], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [255, 133, 255]);

        write_rgb8_scalar(&[235], &[0], &[240], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [255, 227, 0]);

        write_rgb8_scalar(&[235], &[240], &[0], (1, 1), (1, 1, 1), &mut tgt);
        assert_eq!(tgt, [50, 255, 255]);
    }

    #[test]
    fn write_rgb8_f32x8_matches_scalar() {
        let source = include_bytes!("../../tests/data/single_512x512_cavlc.h264");

        let api = OpenH264API::from_source();
        let config = DecoderConfig::default();
        let mut decoder = Decoder::with_api_config(api, config).unwrap();

        let mut rgb = vec![0; 2000 * 2000 * 3];
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();
        let rgb_len = dim.0 * dim.1 * 3;

        let tgt = &mut rgb[0..rgb_len];

        write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);

        let mut tgt2 = vec![0; tgt.len()];
        write_rgb8_f32x8(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut tgt2);

        assert_eq!(tgt, tgt2);
    }

    #[test]
    fn write_rgb8_par_matches_scalar() {
        let source = include_bytes!("../../tests/data/single_512x512_cavlc.h264");

        let api = OpenH264API::from_source();
        let config = DecoderConfig::default();
        let mut decoder = Decoder::with_api_config(api, config).unwrap();

        let mut rgb = vec![0; 2000 * 2000 * 3];
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();
        let rgb_len = dim.0 * dim.1 * 3;

        let tgt = &mut rgb[0..rgb_len];

        write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);

        let mut tgt2 = vec![0; tgt.len()];
        write_rgb8_scalar_par(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut tgt2);

        assert_eq!(tgt, tgt2);
    }

    #[test]
    fn write_rgb8_f32x8_par_matches_scalar() {
        let source = include_bytes!("../../tests/data/single_512x512_cavlc.h264");

        let api = OpenH264API::from_source();
        let config = DecoderConfig::default();
        let mut decoder = Decoder::with_api_config(api, config).unwrap();

        let mut rgb = vec![0; 2000 * 2000 * 3];
        let yuv = decoder.decode(&source[..]).unwrap().unwrap();
        let dim = yuv.dimensions();
        let rgb_len = dim.0 * dim.1 * 3;

        let tgt = &mut rgb[0..rgb_len];

        write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);

        let mut tgt2 = vec![0; tgt.len()];
        write_rgb8_f32x8_par(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut tgt2);

        assert_eq!(tgt, tgt2);
    }
}
