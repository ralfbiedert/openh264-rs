pub mod scalar;
pub mod wide;

#[cfg(target_arch = "x86_64")]
pub mod x86_64_avx2;

#[derive(Copy, Clone)]
pub struct Rgb8SourceLayout {
    pub dimensions: (usize, usize),
    pub dimensions_padded: (usize, usize),
    pub pixel_stride: usize,
    pub rgb_offsets: (usize, usize, usize),
}

pub struct Yuv420Planes<'a> {
    pub y: &'a mut [u8],
    pub u: &'a mut [u8],
    pub v: &'a mut [u8],
}

/// Writes a contiguous RGB-compatible source to I420 using the fastest
/// supported implementation.
pub fn write_rgb8_to_yuv420(rgb_data: &[u8], layout: Rgb8SourceLayout, target: &mut Yuv420Planes<'_>) {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        // SAFETY: AVX2 support was checked immediately above. The native
        // implementation validates all slice lengths before using raw pointers.
        if unsafe { x86_64_avx2::write_rgb8_to_yuv420_avx2(rgb_data, layout, target) } {
            return;
        }
    }

    scalar::write_rgb8_to_yuv420(rgb_data, layout, target);
}

pub fn write_yuv420_to_rgb(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
    pixel_len: usize,
) {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("avx2") {
        // SAFETY: AVX2 support was checked immediately above.
        unsafe {
            if pixel_len == 3 {
                x86_64_avx2::write_yuv420_to_rgb_avx2_rgb(y_plane, u_plane, v_plane, dim, strides, target);
            } else {
                x86_64_avx2::write_yuv420_to_rgb_avx2_rgba(y_plane, u_plane, v_plane, dim, strides, target);
            }
        }
        return;
    }

    wide::write_yuv420_to_rgb_wide(y_plane, u_plane, v_plane, dim, strides, target, pixel_len);
}
