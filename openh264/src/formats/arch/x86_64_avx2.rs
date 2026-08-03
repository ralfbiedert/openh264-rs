use super::wide;

const RGB_PIXEL_LEN: usize = 3;
const RGBA_PIXEL_LEN: usize = 4;
const FIXED_Y_COEFFICIENT: i16 = 74;
const FIXED_RV_COEFFICIENT: i16 = 102;
const FIXED_GU_COEFFICIENT: i16 = -25;
const FIXED_GV_COEFFICIENT: i16 = -52;
const FIXED_BU_COEFFICIENT: i16 = 129;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn write_yuv420_to_rgb_avx2_rgb(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    let (width, height) = dim;
    for y in 0..height {
        let y_row = &y_plane[y * strides.0..y * strides.0 + width];
        let u_row = &u_plane[(y / 2) * strides.1..(y / 2) * strides.1 + width / 2];
        let v_row = &v_plane[(y / 2) * strides.2..(y / 2) * strides.2 + width / 2];
        let target = &mut target[y * width * RGB_PIXEL_LEN..(y + 1) * width * RGB_PIXEL_LEN];

        let mut x = 0;
        while x + 16 <= width {
            // SAFETY: The loop condition leaves sixteen luma and eight chroma bytes, plus
            // sixteen RGB output pixels, in the respective slices.
            unsafe {
                write_yuv420_to_rgb_avx2_rgb_16(
                    y_row.as_ptr().add(x),
                    u_row.as_ptr().add(x / 2),
                    v_row.as_ptr().add(x / 2),
                    target.as_mut_ptr().add(x * RGB_PIXEL_LEN),
                );
            }
            x += 16;
        }

        if x != width {
            wide::write_yuv420_to_rgb_wide_row(
                &y_row[x..],
                &u_row[x / 2..],
                &v_row[x / 2..],
                &mut target[x * RGB_PIXEL_LEN..],
                RGB_PIXEL_LEN,
            );
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn write_yuv420_to_rgb_avx2_rgba(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    dim: (usize, usize),
    strides: (usize, usize, usize),
    target: &mut [u8],
) {
    let (width, height) = dim;
    for y in 0..height {
        let y_row = &y_plane[y * strides.0..y * strides.0 + width];
        let u_row = &u_plane[(y / 2) * strides.1..(y / 2) * strides.1 + width / 2];
        let v_row = &v_plane[(y / 2) * strides.2..(y / 2) * strides.2 + width / 2];
        let target = &mut target[y * width * RGBA_PIXEL_LEN..(y + 1) * width * RGBA_PIXEL_LEN];

        let mut x = 0;
        while x + 16 <= width {
            // SAFETY: The loop condition leaves sixteen luma and eight chroma bytes, plus
            // sixteen RGBA output pixels, in the respective slices.
            unsafe {
                write_yuv420_to_rgb_avx2_rgba_16(
                    y_row.as_ptr().add(x),
                    u_row.as_ptr().add(x / 2),
                    v_row.as_ptr().add(x / 2),
                    target.as_mut_ptr().add(x * RGBA_PIXEL_LEN),
                );
            }
            x += 16;
        }

        if x != width {
            wide::write_yuv420_to_rgb_wide_row(
                &y_row[x..],
                &u_row[x / 2..],
                &v_row[x / 2..],
                &mut target[x * RGBA_PIXEL_LEN..],
                RGBA_PIXEL_LEN,
            );
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(clippy::many_single_char_names, clippy::wildcard_imports)]
unsafe fn write_yuv420_to_rgb_avx2_rgba_16(y_ptr: *const u8, u_ptr: *const u8, v_ptr: *const u8, target: *mut u8) {
    use std::arch::x86_64::*;

    // SAFETY: The caller guarantees that all source and destination ranges are valid.
    unsafe {
        let (r, g, b) = yuv420_to_rgb_avx2_16(y_ptr, u_ptr, v_ptr);
        let alpha = _mm_set1_epi8(-1);
        let rg_low = _mm_unpacklo_epi8(r, g);
        let rg_high = _mm_unpackhi_epi8(r, g);
        let ba_low = _mm_unpacklo_epi8(b, alpha);
        let ba_high = _mm_unpackhi_epi8(b, alpha);

        _mm_storeu_si128(target.cast(), _mm_unpacklo_epi16(rg_low, ba_low));
        _mm_storeu_si128(target.add(16).cast(), _mm_unpackhi_epi16(rg_low, ba_low));
        _mm_storeu_si128(target.add(32).cast(), _mm_unpacklo_epi16(rg_high, ba_high));
        _mm_storeu_si128(target.add(48).cast(), _mm_unpackhi_epi16(rg_high, ba_high));
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn write_yuv420_to_rgb_avx2_rgb_16(y_ptr: *const u8, u_ptr: *const u8, v_ptr: *const u8, target: *mut u8) {
    use std::arch::x86_64::*;

    // SAFETY: The caller guarantees that all source and destination ranges are valid.
    unsafe {
        let (r, g, b) = yuv420_to_rgb_avx2_16(y_ptr, u_ptr, v_ptr);
        let (first_low, tail_low) = pack_rgb8_avx2_8(r, g, b);
        let (first_high, tail_high) = pack_rgb8_avx2_8(_mm_srli_si128::<8>(r), _mm_srli_si128::<8>(g), _mm_srli_si128::<8>(b));

        _mm_storeu_si128(target.cast(), first_low);
        _mm_storeu_si128(target.add(16).cast(), _mm_unpacklo_epi64(tail_low, first_high));
        _mm_storeu_si128(
            target.add(32).cast(),
            _mm_unpacklo_epi64(_mm_srli_si128::<8>(first_high), tail_high),
        );
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(clippy::many_single_char_names, clippy::wildcard_imports)]
unsafe fn yuv420_to_rgb_avx2_16(
    y_ptr: *const u8,
    u_ptr: *const u8,
    v_ptr: *const u8,
) -> (
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
) {
    use std::arch::x86_64::*;

    // SAFETY: The caller guarantees that all source ranges are valid.
    unsafe {
        let y = _mm256_sub_epi16(_mm256_cvtepu8_epi16(_mm_loadu_si128(y_ptr.cast())), _mm256_set1_epi16(16));
        let u = _mm256_sub_epi16(
            _mm256_cvtepu8_epi16(_mm_unpacklo_epi8(_mm_loadl_epi64(u_ptr.cast()), _mm_loadl_epi64(u_ptr.cast()))),
            _mm256_set1_epi16(128),
        );
        let v = _mm256_sub_epi16(
            _mm256_cvtepu8_epi16(_mm_unpacklo_epi8(_mm_loadl_epi64(v_ptr.cast()), _mm_loadl_epi64(v_ptr.cast()))),
            _mm256_set1_epi16(128),
        );

        // The coefficients are scaled by 64 so all intermediate values fit in i16.
        // Saturating addition preserves the final u8 clamp for values above 255.
        let y = _mm256_add_epi16(
            _mm256_mullo_epi16(y, _mm256_set1_epi16(FIXED_Y_COEFFICIENT)),
            _mm256_srai_epi16::<1>(y),
        );
        let r_v = _mm256_add_epi16(
            _mm256_mullo_epi16(v, _mm256_set1_epi16(FIXED_RV_COEFFICIENT)),
            _mm256_srai_epi16::<2>(v),
        );
        let r = fixed_bt601_avx2(y, _mm256_setzero_si256(), r_v);
        let g = fixed_bt601_avx2(
            y,
            _mm256_mullo_epi16(u, _mm256_set1_epi16(FIXED_GU_COEFFICIENT)),
            _mm256_mullo_epi16(v, _mm256_set1_epi16(FIXED_GV_COEFFICIENT)),
        );
        let b = fixed_bt601_avx2(
            y,
            _mm256_mullo_epi16(u, _mm256_set1_epi16(FIXED_BU_COEFFICIENT)),
            _mm256_setzero_si256(),
        );
        (pack_avx2_i16_to_u8(r), pack_avx2_i16_to_u8(g), pack_avx2_i16_to_u8(b))
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn pack_rgb8_avx2_8(
    r: std::arch::x86_64::__m128i,
    g: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> (std::arch::x86_64::__m128i, std::arch::x86_64::__m128i) {
    use std::arch::x86_64::*;

    // AVX2 CPUs also support SSSE3, which provides the byte shuffles.
    let rg = _mm_unpacklo_epi8(r, g);
    let first_from_rg = _mm_shuffle_epi8(
        rg,
        _mm_setr_epi8(0, 1, -128, 2, 3, -128, 4, 5, -128, 6, 7, -128, 8, 9, -128, 10),
    );
    let first_from_b = _mm_shuffle_epi8(
        b,
        _mm_setr_epi8(-128, -128, 0, -128, -128, 1, -128, -128, 2, -128, -128, 3, -128, -128, 4, -128),
    );
    let tail_from_rg = _mm_shuffle_epi8(
        rg,
        _mm_setr_epi8(
            11, -128, 12, 13, -128, 14, 15, -128, -128, -128, -128, -128, -128, -128, -128, -128,
        ),
    );
    let tail_from_b = _mm_shuffle_epi8(
        b,
        _mm_setr_epi8(
            -128, 5, -128, -128, 6, -128, -128, 7, -128, -128, -128, -128, -128, -128, -128, -128,
        ),
    );

    (
        _mm_or_si128(first_from_rg, first_from_b),
        _mm_or_si128(tail_from_rg, tail_from_b),
    )
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn fixed_bt601_avx2(
    y: std::arch::x86_64::__m256i,
    u: std::arch::x86_64::__m256i,
    v: std::arch::x86_64::__m256i,
) -> std::arch::x86_64::__m256i {
    use std::arch::x86_64::*;

    let value = _mm256_adds_epi16(_mm256_adds_epi16(y, u), _mm256_adds_epi16(v, _mm256_set1_epi16(32)));
    _mm256_srai_epi16::<6>(value)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
fn pack_avx2_i16_to_u8(value: std::arch::x86_64::__m256i) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;

    let values = _mm256_permute4x64_epi64::<216>(_mm256_packus_epi16(value, value));
    _mm256_castsi256_si128(values)
}
