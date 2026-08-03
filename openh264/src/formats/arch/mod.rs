pub mod wide;

#[cfg(target_arch = "x86_64")]
pub mod x86_64_avx2;

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
