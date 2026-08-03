use super::{Rgb8SourceLayout, Yuv420Planes};

/// Writes an RGB-compatible source to I420 using portable scalar operations.
#[allow(clippy::many_single_char_names)]
pub fn write_rgb8_to_yuv420(rgb_data: &[u8], layout: Rgb8SourceLayout, target: &mut Yuv420Planes<'_>) {
    let (width, height) = layout.dimensions;
    let (padded_width, padded_height) = layout.dimensions_padded;
    let (r_offset, g_offset, b_offset) = layout.rgb_offsets;

    assert_eq!(width % 2, 0, "width needs to be a multiple of 2");
    assert_eq!(height % 2, 0, "height needs to be a multiple of 2");
    assert!(padded_width >= width, "padded width needs to be at least the image width");
    assert!(padded_height >= height, "padded height needs to be at least the image height");
    assert!(
        r_offset < layout.pixel_stride && g_offset < layout.pixel_stride && b_offset < layout.pixel_stride,
        "RGB channel offsets need to be within the pixel stride"
    );

    let source_row_len = padded_width
        .checked_mul(layout.pixel_stride)
        .expect("RGB source row length overflowed");
    let source_len = source_row_len.checked_mul(height).expect("RGB source length overflowed");
    let y_len = width.checked_mul(height).expect("Y plane length overflowed");
    let uv_len = y_len / 4;

    assert!(rgb_data.len() >= source_len, "RGB source is too small");
    assert!(target.y.len() >= y_len, "Y plane is too small");
    assert!(target.u.len() >= uv_len, "U plane is too small");
    assert!(target.v.len() >= uv_len, "V plane is too small");

    for row in 0..height {
        let source_row = &rgb_data[row * source_row_len..][..source_row_len];
        let y_row = &mut target.y[row * width..][..width];
        for (pixel, y) in source_row.chunks_exact(layout.pixel_stride).take(width).zip(y_row) {
            *y = rgb_to_y(pixel[r_offset], pixel[g_offset], pixel[b_offset]);
        }
    }

    let half_width = width / 2;
    for row in (0..height).step_by(2) {
        let source0 = &rgb_data[row * source_row_len..][..source_row_len];
        let source1 = &rgb_data[(row + 1) * source_row_len..][..source_row_len];
        let u_row = &mut target.u[(row / 2) * half_width..][..half_width];
        let v_row = &mut target.v[(row / 2) * half_width..][..half_width];

        for x in 0..half_width {
            let first = 2 * x * layout.pixel_stride;
            let second = first + layout.pixel_stride;
            let r = average_2x2(
                source0[first + r_offset],
                source0[second + r_offset],
                source1[first + r_offset],
                source1[second + r_offset],
            );
            let g = average_2x2(
                source0[first + g_offset],
                source0[second + g_offset],
                source1[first + g_offset],
                source1[second + g_offset],
            );
            let b = average_2x2(
                source0[first + b_offset],
                source0[second + b_offset],
                source1[first + b_offset],
                source1[second + b_offset],
            );

            u_row[x] = (((-38 * r + 112 * b - 74 * g) >> 8) + 128) as u8;
            v_row[x] = (((112 * r - 18 * b - 94 * g) >> 8) + 128) as u8;
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn rgb_to_y(r: u8, g: u8, b: u8) -> u8 {
    (((66 * u32::from(r) + 129 * u32::from(g) + 25 * u32::from(b)) >> 8) + 16) as u8
}

fn average_2x2(a: u8, b: u8, c: u8, d: u8) -> i16 {
    (i16::from(a) + i16::from(b) + i16::from(c) + i16::from(d) + 2) / 4
}
