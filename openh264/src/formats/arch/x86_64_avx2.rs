use super::{Rgb8SourceLayout, Yuv420Planes, wide};

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

// RGB24 needs three byte-shuffles per sixteen pixels. Two 32-pixel row tiles
// amortize that setup, while smaller rows are faster with the scalar fallback.
const AVX2_RGB_TO_YUV_MIN_WIDTH: usize = 64;

#[derive(Copy, Clone)]
enum Rgb8Layout {
    Rgb24,
    Rgba32,
    Bgra32,
}

struct Rgb8ToYuvRowPair {
    source0: *const u8,
    source1: *const u8,
    y0: *mut u8,
    y1: *mut u8,
    u: *mut u8,
    v: *mut u8,
    width: usize,
    pixel_stride: usize,
    rgb_offsets: (usize, usize, usize),
}

/// Writes a supported contiguous RGB source to I420 using AVX2.
///
/// Returns `false` for unsupported layouts or dimensions too small to amortize
/// the RGB deinterleaving work.
#[target_feature(enable = "avx2")]
#[allow(clippy::similar_names)]
pub(super) unsafe fn write_rgb8_to_yuv420_avx2(rgb_data: &[u8], layout: Rgb8SourceLayout, target: &mut Yuv420Planes<'_>) -> bool {
    let (width, height) = layout.dimensions;
    let dimensions_padded = layout.dimensions_padded;
    let pixel_stride = layout.pixel_stride;
    let rgb_offsets = layout.rgb_offsets;
    let pixel_layout = match (pixel_stride, rgb_offsets) {
        (3, (0, 1, 2)) => Rgb8Layout::Rgb24,
        (4, (0, 1, 2)) => Rgb8Layout::Rgba32,
        (4, (2, 1, 0)) => Rgb8Layout::Bgra32,
        _ => return false,
    };

    if width < AVX2_RGB_TO_YUV_MIN_WIDTH
        || width % 2 != 0
        || height % 2 != 0
        || dimensions_padded.0 < width
        || dimensions_padded.1 < height
    {
        return false;
    }

    let Some(source_row_len) = dimensions_padded.0.checked_mul(pixel_stride) else {
        return false;
    };
    let Some(source_len) = source_row_len.checked_mul(height) else {
        return false;
    };
    let Some(y_len) = width.checked_mul(height) else {
        return false;
    };
    let uv_len = y_len / 4;
    if rgb_data.len() < source_len || target.y.len() < y_len || target.u.len() < uv_len || target.v.len() < uv_len {
        return false;
    }

    // SAFETY: The validated lengths above cover every source and destination
    // range. Each vector tile is guarded by a sixteen-pixel loop bound.
    unsafe {
        let source = rgb_data.as_ptr();
        let y_plane = target.y.as_mut_ptr();
        let u_plane = target.u.as_mut_ptr();
        let v_plane = target.v.as_mut_ptr();
        let half_width = width / 2;

        for row in (0..height).step_by(2) {
            let row_pair = Rgb8ToYuvRowPair {
                source0: source.add(row * source_row_len),
                source1: source.add((row + 1) * source_row_len),
                y0: y_plane.add(row * width),
                y1: y_plane.add((row + 1) * width),
                u: u_plane.add((row / 2) * half_width),
                v: v_plane.add((row / 2) * half_width),
                width,
                pixel_stride,
                rgb_offsets,
            };

            let mut x = 0;
            while x + 32 <= width {
                write_rgb8_to_yuv420_avx2_16(
                    row_pair.source0.add(x * pixel_stride),
                    row_pair.source1.add(x * pixel_stride),
                    row_pair.y0.add(x),
                    row_pair.y1.add(x),
                    row_pair.u.add(x / 2),
                    row_pair.v.add(x / 2),
                    pixel_layout,
                );
                write_rgb8_to_yuv420_avx2_16(
                    row_pair.source0.add((x + 16) * pixel_stride),
                    row_pair.source1.add((x + 16) * pixel_stride),
                    row_pair.y0.add(x + 16),
                    row_pair.y1.add(x + 16),
                    row_pair.u.add(x / 2 + 8),
                    row_pair.v.add(x / 2 + 8),
                    pixel_layout,
                );
                x += 32;
            }

            while x + 16 <= width {
                write_rgb8_to_yuv420_avx2_16(
                    row_pair.source0.add(x * pixel_stride),
                    row_pair.source1.add(x * pixel_stride),
                    row_pair.y0.add(x),
                    row_pair.y1.add(x),
                    row_pair.u.add(x / 2),
                    row_pair.v.add(x / 2),
                    pixel_layout,
                );
                x += 16;
            }

            if x != width {
                write_rgb8_to_yuv420_tail(&row_pair, x);
            }
        }
    }

    true
}

#[target_feature(enable = "avx2")]
#[allow(clippy::many_single_char_names, clippy::wildcard_imports)]
unsafe fn write_rgb8_to_yuv420_avx2_16(
    source0: *const u8,
    source1: *const u8,
    y0: *mut u8,
    y1: *mut u8,
    u: *mut u8,
    v: *mut u8,
    layout: Rgb8Layout,
) {
    use std::arch::x86_64::*;

    // SAFETY: The caller guarantees sixteen source pixels in each row and
    // sixteen luma plus eight chroma output bytes.
    unsafe {
        let (r0, g0, b0) = unpack_rgb8_avx2_16(source0, layout);
        let (r1, g1, b1) = unpack_rgb8_avx2_16(source1, layout);

        _mm_storeu_si128(y0.cast(), rgb_to_y_avx2_16(r0, g0, b0));
        _mm_storeu_si128(y1.cast(), rgb_to_y_avx2_16(r1, g1, b1));

        let r = average_2x2_avx2_16(r0, r1);
        let g = average_2x2_avx2_16(g0, g1);
        let b = average_2x2_avx2_16(b0, b1);
        _mm_storel_epi64(u.cast(), rgb_to_u_avx2_8(r, g, b));
        _mm_storel_epi64(v.cast(), rgb_to_v_avx2_8(r, g, b));
    }
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_rgb8_avx2_16(
    source: *const u8,
    layout: Rgb8Layout,
) -> (
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
) {
    match layout {
        // SAFETY: The caller provides sixteen pixels in the selected layout.
        Rgb8Layout::Rgb24 => unsafe { unpack_rgb24_avx2_16(source) },
        Rgb8Layout::Rgba32 => unsafe { unpack_rgba_avx2_16(source) },
        Rgb8Layout::Bgra32 => unsafe { unpack_bgra_avx2_16(source) },
    }
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn unpack_rgb24_avx2_16(
    source: *const u8,
) -> (
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
) {
    use std::arch::x86_64::*;

    // SAFETY: The caller guarantees that the forty-eight source bytes are valid.
    unsafe {
        let first = _mm_loadu_si128(source.cast());
        let second = _mm_loadu_si128(source.add(16).cast());
        let third = _mm_loadu_si128(source.add(32).cast());

        let r_first = _mm_shuffle_epi8(
            first,
            _mm_setr_epi8(0, 3, 6, 9, 12, 15, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128),
        );
        let r_second = _mm_slli_si128::<6>(_mm_shuffle_epi8(
            second,
            _mm_setr_epi8(
                2, 5, 8, 11, 14, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            ),
        ));
        let r_third = _mm_slli_si128::<11>(_mm_shuffle_epi8(
            third,
            _mm_setr_epi8(
                1, 4, 7, 10, 13, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            ),
        ));

        let g_first = _mm_shuffle_epi8(
            first,
            _mm_setr_epi8(
                1, 4, 7, 10, 13, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            ),
        );
        let g_second = _mm_slli_si128::<5>(_mm_shuffle_epi8(
            second,
            _mm_setr_epi8(0, 3, 6, 9, 12, 15, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128),
        ));
        let g_third = _mm_slli_si128::<11>(_mm_shuffle_epi8(
            third,
            _mm_setr_epi8(
                2, 5, 8, 11, 14, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            ),
        ));

        let b_first = _mm_shuffle_epi8(
            first,
            _mm_setr_epi8(
                2, 5, 8, 11, 14, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            ),
        );
        let b_second = _mm_slli_si128::<5>(_mm_shuffle_epi8(
            second,
            _mm_setr_epi8(
                1, 4, 7, 10, 13, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128,
            ),
        ));
        let b_third = _mm_slli_si128::<10>(_mm_shuffle_epi8(
            third,
            _mm_setr_epi8(0, 3, 6, 9, 12, 15, -128, -128, -128, -128, -128, -128, -128, -128, -128, -128),
        ));

        (
            _mm_or_si128(_mm_or_si128(r_first, r_second), r_third),
            _mm_or_si128(_mm_or_si128(g_first, g_second), g_third),
            _mm_or_si128(_mm_or_si128(b_first, b_second), b_third),
        )
    }
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_rgba_avx2_16(
    source: *const u8,
) -> (
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
) {
    // SAFETY: The caller guarantees sixteen RGBA pixels.
    unsafe { unpack_xgba_avx2_16::<0, 1, 2>(source) }
}

#[target_feature(enable = "avx2")]
unsafe fn unpack_bgra_avx2_16(
    source: *const u8,
) -> (
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
) {
    // SAFETY: The caller guarantees sixteen BGRA pixels.
    unsafe { unpack_xgba_avx2_16::<2, 1, 0>(source) }
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn unpack_xgba_avx2_16<const R: i8, const G: i8, const B: i8>(
    source: *const u8,
) -> (
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
    std::arch::x86_64::__m128i,
) {
    use std::arch::x86_64::*;

    // SAFETY: The caller guarantees that the sixty-four source bytes are valid.
    unsafe {
        let first = _mm_loadu_si128(source.cast());
        let second = _mm_loadu_si128(source.add(16).cast());
        let third = _mm_loadu_si128(source.add(32).cast());
        let fourth = _mm_loadu_si128(source.add(48).cast());
        (
            unpack_xgba_component_avx2_16::<R>(first, second, third, fourth),
            unpack_xgba_component_avx2_16::<G>(first, second, third, fourth),
            unpack_xgba_component_avx2_16::<B>(first, second, third, fourth),
        )
    }
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn unpack_xgba_component_avx2_16<const OFFSET: i8>(
    first: std::arch::x86_64::__m128i,
    second: std::arch::x86_64::__m128i,
    third: std::arch::x86_64::__m128i,
    fourth: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;

    let mask = _mm_setr_epi8(
        OFFSET,
        OFFSET + 4,
        OFFSET + 8,
        OFFSET + 12,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
        -128,
    );
    let first = _mm_shuffle_epi8(first, mask);
    let second = _mm_slli_si128::<4>(_mm_shuffle_epi8(second, mask));
    let third = _mm_slli_si128::<8>(_mm_shuffle_epi8(third, mask));
    let fourth = _mm_slli_si128::<12>(_mm_shuffle_epi8(fourth, mask));
    _mm_or_si128(_mm_or_si128(first, second), _mm_or_si128(third, fourth))
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn rgb_to_y_avx2_16(
    r: std::arch::x86_64::__m128i,
    g: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;

    let r = _mm256_cvtepu8_epi16(r);
    let g = _mm256_cvtepu8_epi16(g);
    let b = _mm256_cvtepu8_epi16(b);
    let y = _mm256_add_epi16(
        _mm256_add_epi16(
            _mm256_mullo_epi16(r, _mm256_set1_epi16(66)),
            _mm256_mullo_epi16(g, _mm256_set1_epi16(129)),
        ),
        _mm256_mullo_epi16(b, _mm256_set1_epi16(25)),
    );
    let y = _mm256_add_epi16(_mm256_srli_epi16::<8>(y), _mm256_set1_epi16(16));
    pack_avx2_i16_to_u8(y)
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn average_2x2_avx2_16(row0: std::arch::x86_64::__m128i, row1: std::arch::x86_64::__m128i) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;

    let zero = _mm_setzero_si128();
    let pair_sum = |row0, row1| {
        let low = _mm_add_epi16(_mm_unpacklo_epi8(row0, zero), _mm_unpacklo_epi8(row1, zero));
        let high = _mm_add_epi16(_mm_unpackhi_epi8(row0, zero), _mm_unpackhi_epi8(row1, zero));
        _mm_packs_epi32(_mm_madd_epi16(low, _mm_set1_epi16(1)), _mm_madd_epi16(high, _mm_set1_epi16(1)))
    };
    _mm_srli_epi16::<2>(_mm_add_epi16(pair_sum(row0, row1), _mm_set1_epi16(2)))
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn rgb_to_u_avx2_8(
    r: std::arch::x86_64::__m128i,
    g: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;

    let u = _mm_add_epi16(
        _mm_add_epi16(_mm_mullo_epi16(r, _mm_set1_epi16(-38)), _mm_mullo_epi16(g, _mm_set1_epi16(-74))),
        _mm_mullo_epi16(b, _mm_set1_epi16(112)),
    );
    let u = _mm_add_epi16(_mm_srai_epi16::<8>(u), _mm_set1_epi16(128));
    _mm_packus_epi16(u, _mm_setzero_si128())
}

#[target_feature(enable = "avx2")]
#[allow(clippy::wildcard_imports)]
unsafe fn rgb_to_v_avx2_8(
    r: std::arch::x86_64::__m128i,
    g: std::arch::x86_64::__m128i,
    b: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;

    let v = _mm_add_epi16(
        _mm_add_epi16(_mm_mullo_epi16(r, _mm_set1_epi16(112)), _mm_mullo_epi16(g, _mm_set1_epi16(-94))),
        _mm_mullo_epi16(b, _mm_set1_epi16(-18)),
    );
    let v = _mm_add_epi16(_mm_srai_epi16::<8>(v), _mm_set1_epi16(128));
    _mm_packus_epi16(v, _mm_setzero_si128())
}

#[allow(clippy::many_single_char_names)]
unsafe fn write_rgb8_to_yuv420_tail(row_pair: &Rgb8ToYuvRowPair, start_x: usize) {
    let (r_offset, g_offset, b_offset) = row_pair.rgb_offsets;

    // SAFETY: The caller provides the remaining even number of pixels and
    // matching output ranges.
    unsafe {
        for x in start_x..row_pair.width {
            let pixel_offset = x * row_pair.pixel_stride;
            let r0 = *row_pair.source0.add(pixel_offset + r_offset);
            let g0 = *row_pair.source0.add(pixel_offset + g_offset);
            let b0 = *row_pair.source0.add(pixel_offset + b_offset);
            let r1 = *row_pair.source1.add(pixel_offset + r_offset);
            let g1 = *row_pair.source1.add(pixel_offset + g_offset);
            let b1 = *row_pair.source1.add(pixel_offset + b_offset);
            *row_pair.y0.add(x) = rgb_to_y_scalar(r0, g0, b0);
            *row_pair.y1.add(x) = rgb_to_y_scalar(r1, g1, b1);
        }

        for x in (start_x..row_pair.width).step_by(2) {
            let first = x * row_pair.pixel_stride;
            let second = first + row_pair.pixel_stride;
            let r = average_2x2_scalar(
                *row_pair.source0.add(first + r_offset),
                *row_pair.source0.add(second + r_offset),
                *row_pair.source1.add(first + r_offset),
                *row_pair.source1.add(second + r_offset),
            );
            let g = average_2x2_scalar(
                *row_pair.source0.add(first + g_offset),
                *row_pair.source0.add(second + g_offset),
                *row_pair.source1.add(first + g_offset),
                *row_pair.source1.add(second + g_offset),
            );
            let b = average_2x2_scalar(
                *row_pair.source0.add(first + b_offset),
                *row_pair.source0.add(second + b_offset),
                *row_pair.source1.add(first + b_offset),
                *row_pair.source1.add(second + b_offset),
            );
            *row_pair.u.add(x / 2) = (((-38 * r + 112 * b - 74 * g) >> 8) + 128) as u8;
            *row_pair.v.add(x / 2) = (((112 * r - 18 * b - 94 * g) >> 8) + 128) as u8;
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn rgb_to_y_scalar(r: u8, g: u8, b: u8) -> u8 {
    (((66 * u32::from(r) + 129 * u32::from(g) + 25 * u32::from(b)) >> 8) + 16) as u8
}

fn average_2x2_scalar(a: u8, b: u8, c: u8, d: u8) -> i16 {
    (i16::from(a) + i16::from(b) + i16::from(c) + i16::from(d) + 2) / 4
}
