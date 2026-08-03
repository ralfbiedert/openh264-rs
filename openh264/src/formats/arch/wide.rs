const RGBA_PIXEL_LEN: usize = 4;
const Y_MUL: f32 = 255.0 / 219.0;
const RV_MUL: f32 = 255.0 / 224.0 * 1.402;
const GV_MUL: f32 = -255.0 / 224.0 * 1.402 * 0.299 / 0.587;
const GU_MUL: f32 = -255.0 / 224.0 * 1.772 * 0.114 / 0.587;
const BU_MUL: f32 = 255.0 / 224.0 * 1.772;

pub fn write_yuv420_to_rgb_wide(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
    pixel_len: usize,
) {
    let (width, height) = dim;
    for y in 0..height {
        let y_row = &y_plane[y * strides.0..y * strides.0 + width];
        let u_row = &u_plane[(y / 2) * strides.1..(y / 2) * strides.1 + width / 2];
        let v_row = &v_plane[(y / 2) * strides.2..(y / 2) * strides.2 + width / 2];
        let target = &mut target[y * width * pixel_len..(y + 1) * width * pixel_len];
        write_yuv420_to_rgb_wide_row(y_row, u_row, v_row, target, pixel_len);
    }
}

#[allow(clippy::similar_names, clippy::inline_always, clippy::many_single_char_names)]
#[inline(always)]
pub fn write_yuv420_to_rgb_wide_row(y_row: &[u8], u_row: &[u8], v_row: &[u8], target: &mut [u8], pixel_len: usize) {
    let y_mul = wide::f32x8::splat(Y_MUL);
    let rv_mul = wide::f32x8::splat(RV_MUL);
    let gu_mul = wide::f32x8::splat(GU_MUL);
    let gv_mul = wide::f32x8::splat(GV_MUL);
    let bu_mul = wide::f32x8::splat(BU_MUL);
    let upper_bound = wide::f32x8::splat(255.0);
    let lower_bound = wide::f32x8::splat(0.0);

    for (((y, u), v), output) in y_row
        .chunks_exact(8)
        .zip(u_row.chunks_exact(4))
        .zip(v_row.chunks_exact(4))
        .zip(target.chunks_exact_mut(8 * pixel_len))
    {
        let y: &[u8; 8] = y.try_into().unwrap();
        let u: &[u8; 4] = u.try_into().unwrap();
        let v: &[u8; 4] = v.try_into().unwrap();
        let (y, u, v) = pack_into_yuv420_f32x8(y, u, v);
        let y = y * y_mul;
        let r = v.mul_add(rv_mul, y);
        let g = v.mul_add(gv_mul, u.mul_add(gu_mul, y));
        let b = u.mul_add(bu_mul, y);
        let r = r.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int();
        let g = g.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int();
        let b = b.fast_min(upper_bound).fast_max(lower_bound).fast_trunc_int();
        let (r, g, b) = (r.as_array(), g.as_array(), b.as_array());

        for i in 0..8 {
            let offset = i * pixel_len;
            output[offset] = r[i] as u8;
            output[offset + 1] = g[i] as u8;
            output[offset + 2] = b[i] as u8;
            if pixel_len == RGBA_PIXEL_LEN {
                output[offset + 3] = 255;
            }
        }
    }
}

#[inline(always)]
#[allow(clippy::inline_always)]
fn pack_into_yuv420_f32x8(y: &[u8; 8], u: &[u8; 4], v: &[u8; 4]) -> (wide::f32x8, wide::f32x8, wide::f32x8) {
    let [y0, y1, y2, y3, y4, y5, y6, y7] = *y;
    let y = wide::f32x8::from([
        f32::from(y0),
        f32::from(y1),
        f32::from(y2),
        f32::from(y3),
        f32::from(y4),
        f32::from(y5),
        f32::from(y6),
        f32::from(y7),
    ]) - 16.0;
    let [u0, u1, u2, u3] = *u;
    let u = wide::f32x8::from([
        f32::from(u0),
        f32::from(u0),
        f32::from(u1),
        f32::from(u1),
        f32::from(u2),
        f32::from(u2),
        f32::from(u3),
        f32::from(u3),
    ]) - 128.0;
    let [v0, v1, v2, v3] = *v;
    let v = wide::f32x8::from([
        f32::from(v0),
        f32::from(v0),
        f32::from(v1),
        f32::from(v1),
        f32::from(v2),
        f32::from(v2),
        f32::from(v3),
        f32::from(v3),
    ]) - 128.0;
    (y, u, v)
}
