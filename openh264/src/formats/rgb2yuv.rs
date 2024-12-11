use crate::formats::RGBSource;

/// Writes an RGB source into 420 Y, U and V buffers.
pub(crate) fn write_yuv_scalar(
    rgb: impl RGBSource,
    dimensions: (usize, usize),
    y_buf: &mut [u8],
    u_buf: &mut [u8],
    v_buf: &mut [u8],
) {
    // Make sure we only attempt to read sources that match our own size.
    assert_eq!(rgb.dimensions(), dimensions);

    let width = dimensions.0;
    let height = dimensions.1;
    let half_width = width / 2;

    // y is full size, u, v is quarter size
    let mut write_y = |x: usize, y: usize, rgb: (f32, f32, f32)| {
        y_buf[x + y * width] = (0.2578125 * rgb.0 + 0.50390625 * rgb.1 + 0.09765625 * rgb.2 + 16.0) as u8;
    };

    let mut write_u = |x: usize, y: usize, rgb: (f32, f32, f32)| {
        u_buf[x + y * half_width] = (-0.1484375 * rgb.0 + -0.2890625 * rgb.1 + 0.4375 * rgb.2 + 128.0) as u8;
    };

    let mut write_v = |x: usize, y: usize, rgb: (f32, f32, f32)| {
        v_buf[x + y * half_width] = (0.4375 * rgb.0 + -0.3671875 * rgb.1 + -0.0703125 * rgb.2 + 128.0) as u8;
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
