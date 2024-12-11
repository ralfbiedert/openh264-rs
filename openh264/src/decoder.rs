//! Converts NAL packets to YUV images.
//!
//! # Examples
//!
//! Basic [Decoder] use looks as follows. In practice, you might get your `h256`
//! bitstream from reading a file or network source.
//!
//! ```rust
//! use openh264::decoder::Decoder;
//! use openh264::nal_units;
//!
//! # use openh264::{Error, OpenH264API};
//! # fn main() -> Result<(), Error> {
//! let h264_in = include_bytes!("../tests/data/multi_512x512.h264");
//! let mut decoder = Decoder::new()?;
//!
//! for packet in nal_units(h264_in) {
//!     // If everything goes well this should yield a `DecodedYUV`.
//!     // It can also be `Err()` if the bitstream had errors, or
//!     // `Ok(None)` if no pictures were available (yet).
//!     let Ok(Some(yuv)) = decoder.decode(packet) else { continue };
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Once you have your `yuv`, which should be of type [`DecodedYUV`], you can proceed to converting it to RGB:
//!
//! ```rust
//! # use openh264::decoder::Decoder;
//! # use openh264::nal_units;
//! use openh264::formats::YUVSource;
//!
//! # use openh264::{Error, OpenH264API};
//! # fn main() -> Result<(), Error> {
//! #
//! # let h264_in = include_bytes!("../tests/data/multi_512x512.h264");
//! # let mut decoder = Decoder::new()?;
//! #
//! # for packet in nal_units(h264_in) {
//! #    let Ok(Some(yuv)) = decoder.decode(packet) else { continue; };
//! let rgb_len = yuv.estimate_rgb_u8_size();
//! let mut rgb_raw = vec![0; rgb_len];
//!
//! yuv.write_rgb8(&mut rgb_raw);
//! # }
//! # Ok(())
//! # }
//! ```

use crate::error::NativeErrorExt;
use crate::formats::YUVSource;
use crate::{Error, OpenH264API, Timestamp};
use openh264_sys2::{
    videoFormatI420, ISVCDecoder, ISVCDecoderVtbl, SBufferInfo, SDecodingParam, SParserBsInfo, SSysMEMBuffer, API,
    DECODER_OPTION, DECODER_OPTION_ERROR_CON_IDC, DECODER_OPTION_NUM_OF_FRAMES_REMAINING_IN_BUFFER,
    DECODER_OPTION_NUM_OF_THREADS, DECODER_OPTION_TRACE_LEVEL, DECODING_STATE, WELS_LOG_DETAIL, WELS_LOG_QUIET,
};
use std::os::raw::{c_int, c_long, c_uchar, c_void};
use std::ptr::{addr_of_mut, null, null_mut};

/// Convenience wrapper with guaranteed function pointers for easy access.
///
/// This struct automatically handles `WelsCreateDecoder` and `WelsDestroyDecoder`.
#[rustfmt::skip]
#[allow(non_snake_case)]
pub struct DecoderRawAPI {
    api: OpenH264API,
    decoder_ptr: *mut *const ISVCDecoderVtbl,
    initialize: unsafe extern "C" fn(arg1: *mut ISVCDecoder, pParam: *const SDecodingParam) -> c_long,
    uninitialize: unsafe extern "C" fn(arg1: *mut ISVCDecoder) -> c_long,
    decode_frame: unsafe extern "C" fn(arg1: *mut ISVCDecoder, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pStride: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int) -> DECODING_STATE,
    decode_frame_no_delay: unsafe extern "C" fn(arg1: *mut ISVCDecoder, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE,
    decode_frame2: unsafe extern "C" fn(arg1: *mut ISVCDecoder, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE,
    flush_frame:  unsafe extern "C" fn(arg1: *mut ISVCDecoder, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE,
    decode_parser: unsafe extern "C" fn(arg1: *mut ISVCDecoder, pSrc: *const c_uchar, iSrcLen: c_int, pDstInfo: *mut SParserBsInfo) -> DECODING_STATE,
    decode_frame_ex: unsafe extern "C" fn(arg1: *mut ISVCDecoder, pSrc: *const c_uchar, iSrcLen: c_int, pDst: *mut c_uchar, iDstStride: c_int, iDstLen: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int, iColorFormat: *mut c_int) -> DECODING_STATE,
    set_option: unsafe extern "C" fn(arg1: *mut ISVCDecoder, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long,
    get_option: unsafe extern "C" fn(arg1: *mut ISVCDecoder, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long,
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::missing_safety_doc)]
#[allow(non_snake_case)]
#[allow(unused)]
impl DecoderRawAPI {
    fn new(api: OpenH264API) -> Result<Self, Error> {
        unsafe {
            let mut decoder_ptr = null::<ISVCDecoderVtbl>() as *mut *const ISVCDecoderVtbl;

            api.WelsCreateDecoder(&mut decoder_ptr as *mut *mut *const ISVCDecoderVtbl).ok()?;

            let e = || {
                Error::msg("VTable missing function.")
            };

            Ok(DecoderRawAPI {
                api,
                decoder_ptr,
                initialize: (*(*decoder_ptr)).Initialize.ok_or_else(e)?,
                uninitialize: (*(*decoder_ptr)).Uninitialize.ok_or_else(e)?,
                decode_frame: (*(*decoder_ptr)).DecodeFrame.ok_or_else(e)?,
                decode_frame_no_delay: (*(*decoder_ptr)).DecodeFrameNoDelay.ok_or_else(e)?,
                decode_frame2: (*(*decoder_ptr)).DecodeFrame2.ok_or_else(e)?,
                flush_frame: (*(*decoder_ptr)).FlushFrame.ok_or_else(e)?,
                decode_parser: (*(*decoder_ptr)).DecodeParser.ok_or_else(e)?,
                decode_frame_ex: (*(*decoder_ptr)).DecodeFrameEx.ok_or_else(e)?,
                set_option: (*(*decoder_ptr)).SetOption.ok_or_else(e)?,
                get_option: (*(*decoder_ptr)).GetOption.ok_or_else(e)?,
            })
        }
    }

    // Exposing these will probably do more harm than good.
    unsafe fn initialize(&self, pParam: *const SDecodingParam) -> c_long { (self.initialize)(self.decoder_ptr, pParam) }
    unsafe fn uninitialize(&self, ) -> c_long { (self.uninitialize)(self.decoder_ptr) }

    pub unsafe fn decode_frame(&self, Src: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pStride: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int) -> DECODING_STATE { (self.decode_frame)(self.decoder_ptr, Src, iSrcLen, ppDst, pStride, iWidth, iHeight) }
    pub unsafe fn decode_frame_no_delay(&self, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE { (self.decode_frame_no_delay)(self.decoder_ptr, pSrc, iSrcLen, ppDst, pDstInfo) }
    pub unsafe fn decode_frame2(&self, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE { (self.decode_frame2)(self.decoder_ptr, pSrc, iSrcLen, ppDst, pDstInfo) }
    pub unsafe fn flush_frame(&self, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE { (self.flush_frame)(self.decoder_ptr, ppDst, pDstInfo) }
    pub unsafe fn decode_parser(&self, pSrc: *const c_uchar, iSrcLen: c_int, pDstInfo: *mut SParserBsInfo) -> DECODING_STATE { (self.decode_parser)(self.decoder_ptr, pSrc, iSrcLen, pDstInfo) }
    pub unsafe fn decode_frame_ex(&self, pSrc: *const c_uchar, iSrcLen: c_int, pDst: *mut c_uchar, iDstStride: c_int, iDstLen: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int, iColorFormat: *mut c_int) -> DECODING_STATE { (self.decode_frame_ex)(self.decoder_ptr, pSrc, iSrcLen, pDst, iDstStride, iDstLen, iWidth, iHeight, iColorFormat) }
    pub unsafe fn set_option(&self, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long {  (self.set_option)(self.decoder_ptr, eOptionId, pOption) }
    pub unsafe fn get_option(&self, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long { (self.get_option)(self.decoder_ptr, eOptionId, pOption) }
}

impl Drop for DecoderRawAPI {
    fn drop(&mut self) {
        // Safe because when we drop the pointer must have been initialized, and we aren't clone.
        unsafe {
            self.api.WelsDestroyDecoder(self.decoder_ptr);
        }
    }
}

unsafe impl Send for DecoderRawAPI {}
unsafe impl Sync for DecoderRawAPI {}

/// Configuration for the [`Decoder`].
///
/// Setting missing? Please file a PR!
#[derive(Default, Copy, Clone, Debug)]
#[must_use]
pub struct DecoderConfig {
    params: SDecodingParam,
    num_threads: DECODER_OPTION,
    debug: DECODER_OPTION,
    error_concealment: DECODER_OPTION,
}

impl DecoderConfig {
    /// Creates a new default encoder config.
    pub fn new() -> Self {
        Self {
            params: Default::default(),
            num_threads: 0,
            debug: WELS_LOG_QUIET,
            error_concealment: 0,
        }
    }

    /// Sets the number of threads; this will probably segfault, see below.<sup>⚠️</sup>
    ///
    /// # Safety
    ///
    /// This setting might work on some platforms but will probably just segfault.
    /// Consider this a _highly_ experimental option we only expose to test if and
    /// where threading actually works. Ultimately you should consult with the upstream
    /// OpenH264 project where and when it is safe to set this.
    ///
    /// See [this issue](https://github.com/ralfbiedert/openh264-rust/issues/10) for details.
    pub unsafe fn num_threads(mut self, num_threads: u32) -> Self {
        self.num_threads = num_threads as i32;
        self
    }

    /// Enables detailed console logging inside OpenH264.
    pub fn debug(mut self, value: bool) -> Self {
        self.debug = if value { WELS_LOG_DETAIL } else { WELS_LOG_QUIET };
        self
    }
}

/// An [OpenH264](https://github.com/cisco/openh264) decoder.
pub struct Decoder {
    raw_api: DecoderRawAPI,
}

impl Decoder {
    /// Create a decoder with default settings and the built-in decoder.
    ///
    /// This method is only available when compiling with the `source` feature.
    #[cfg(feature = "source")]
    pub fn new() -> Result<Self, Error> {
        let api = OpenH264API::from_source();
        Self::with_api_config(api, DecoderConfig::new())
    }

    /// Create a decoder with the provided [API](OpenH264API) and [configuration](DecoderConfig).
    pub fn with_api_config(api: OpenH264API, mut config: DecoderConfig) -> Result<Self, Error> {
        let raw = DecoderRawAPI::new(api)?;

        // config.params.sVideoProperty.eVideoBsType = VIDEO_BITSTREAM_AVC;

        #[rustfmt::skip]
        unsafe {
            raw.initialize(&config.params).ok()?;
            raw.set_option(DECODER_OPTION_TRACE_LEVEL, addr_of_mut!(config.debug).cast()).ok()?;
            raw.set_option(DECODER_OPTION_NUM_OF_THREADS, addr_of_mut!(config.num_threads).cast()).ok()?;
            raw.set_option(DECODER_OPTION_ERROR_CON_IDC, addr_of_mut!(config.error_concealment).cast()).ok()?;
        };

        Ok(Self { raw_api: raw })
    }

    /// Decodes a series of H.264 NAL packets and returns the latest picture.
    ///
    /// This function can be called with:
    ///
    /// - only a complete SPS / PPS header (usually the first some 30 bytes of a H.264 stream)
    /// - the headers and series of complete frames
    /// - new frames after previous headers and frames were successfully decoded.
    ///
    /// In each case, it will return `Some(decoded)` image in YUV format if an image was available, or `None`
    /// if more data needs to be provided.
    ///
    /// # Errors
    ///
    /// The function returns an error if the bitstream was corrupted.
    pub fn decode(&mut self, packet: &[u8]) -> Result<Option<DecodedYUV<'_>>, Error> {
        let mut dst = [null_mut::<u8>(); 3];
        let mut buffer_info = SBufferInfo::default();

        unsafe {
            self.raw_api
                .decode_frame_no_delay(packet.as_ptr(), packet.len() as i32, &mut dst as *mut _, &mut buffer_info)
                .ok()?;

            // Buffer status == 0 means frame data is not ready.
            if buffer_info.iBufferStatus == 0 {
                let mut num_frames: DECODER_OPTION = 0;
                self.raw_api()
                    .get_option(
                        DECODER_OPTION_NUM_OF_FRAMES_REMAINING_IN_BUFFER,
                        addr_of_mut!(num_frames).cast(),
                    )
                    .ok()?;

                // If we have outstanding frames flush them, if then still no frame data ready we have an error.
                if num_frames > 0 {
                    self.raw_api().flush_frame(&mut dst as *mut _, &mut buffer_info).ok()?;

                    if buffer_info.iBufferStatus == 0 {
                        return Err(Error::msg(
                            "Buffer status invalid, we have outstanding frames but failed to flush them.",
                        ));
                    }
                }
            }

            let info = buffer_info.UsrData.sSystemBuffer;
            let timestamp = Timestamp::from_millis(buffer_info.uiInBsTimeStamp); // TODO: Is this the right one?

            // Apparently it is ok for `decode_frame_no_delay` to not return an error _and_ to return null buffers. In this case
            // the user should try to continue decoding.
            if dst[0].is_null() || dst[1].is_null() || dst[2].is_null() {
                return Ok(None);
            }

            // https://github.com/cisco/openh264/issues/2379
            let y = std::slice::from_raw_parts(dst[0], (info.iHeight * info.iStride[0]) as usize);
            let u = std::slice::from_raw_parts(dst[1], (info.iHeight * info.iStride[1] / 2) as usize);
            let v = std::slice::from_raw_parts(dst[2], (info.iHeight * info.iStride[1] / 2) as usize);

            Ok(Some(DecodedYUV {
                info,
                timestamp,
                y,
                u,
                v,
            }))
        }
    }

    /// Obtain the raw API for advanced use cases.
    ///
    /// When resorting to this call, please consider filing an issue / PR.
    ///
    /// # Safety
    ///
    /// You must not set parameters the decoder relies on, we recommend checking the source.
    ///
    /// # Example
    ///
    /// ```
    /// use openh264::decoder::{DecoderConfig, Decoder};
    ///
    /// # use openh264::{Error, OpenH264API};
    /// #
    /// # fn try_main() -> Result<(), Error> {
    /// let api = OpenH264API::from_source();
    /// let config = DecoderConfig::default();
    /// let mut decoder = Decoder::with_api_config(api, config)?;
    ///
    /// unsafe {
    ///     _ = decoder.raw_api();
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn raw_api(&mut self) -> &mut DecoderRawAPI {
        &mut self.raw_api
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // Safe because when we drop the pointer must have been initialized.
        unsafe {
            self.raw_api.uninitialize();
        }
    }
}

/// Frame returned by the [`Decoder`] and provides safe data access.
#[derive(Debug)]
pub struct DecodedYUV<'a> {
    info: SSysMEMBuffer,
    timestamp: Timestamp,

    y: &'a [u8],
    u: &'a [u8],
    v: &'a [u8],
}

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
            ($buf[0] as f32),
            ($buf[1 / $block_size] as f32),
            ($buf[2 / $block_size] as f32),
            ($buf[3 / $block_size] as f32),
            ($buf[4 / $block_size] as f32),
            ($buf[5 / $block_size] as f32),
            ($buf[6 / $block_size] as f32),
            ($buf[7 / $block_size] as f32),
        ])
    }};
}

impl DecodedYUV<'_> {
    /// Returns the unpadded U size.
    ///
    /// This is often smaller (by half) than the image size.
    pub fn dimensions_uv(&self) -> (usize, usize) {
        (self.info.iWidth as usize / 2, self.info.iHeight as usize / 2)
    }

    /// Timestamp of this frame in milliseconds(?) with respect to the video stream.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    // TODO: Ideally we'd like to move these out into a converter in `formats`.
    /// Writes the image into a byte buffer of size `w*h*3`.
    ///
    /// # Panics
    ///
    /// Panics if the target image dimension don't match the configured format.
    #[allow(clippy::unnecessary_cast)]
    pub fn write_rgb8(&self, target: &mut [u8]) {
        let dim = self.dimensions();
        let strides = self.strides();
        let wanted = dim.0 * dim.1 * 3;

        // This needs some love, and better architecture.
        assert_eq!(self.info.iFormat, videoFormatI420 as i32);
        assert_eq!(
            target.len(),
            wanted,
            "Target RGB8 array does not match image dimensions. Wanted: {} * {} * 3 = {}, got {}",
            dim.0,
            dim.1,
            wanted,
            target.len()
        );

        // for f32x8 math, image needs to:
        //   - have a width divisible by 8
        //   - have at least two rows
        if dim.0 % 8 == 0 && dim.1 >= 2 {
            Self::write_rgb8_f32x8(self.y, self.u, self.v, dim, strides, target);
        } else {
            Self::write_rgb8_scalar(self.y, self.u, self.v, dim, strides, target);
        }
    }

    pub(crate) fn write_rgb8_scalar(
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

                let y = y_plane[base_y] as f32;
                let u = u_plane[base_u] as f32;
                let v = v_plane[base_v] as f32;

                rgb_pixel[0] = (y + 1.402 * (v - 128.0)) as u8;
                rgb_pixel[1] = (y - 0.344 * (u - 128.0) - 0.714 * (v - 128.0)) as u8;
                rgb_pixel[2] = (y + 1.772 * (u - 128.0)) as u8;
            }
        }
    }

    #[allow(clippy::identity_op)]
    pub(crate) fn write_rgb8_f32x8(
        y_plane: &[u8],
        u_plane: &[u8],
        v_plane: &[u8],
        dim: (usize, usize),
        strides: (usize, usize, usize),
        target: &mut [u8],
    ) {
        // this assumes we are decoding YUV420
        assert_eq!(y_plane.len(), u_plane.len() * 4);
        assert_eq!(y_plane.len(), v_plane.len() * 4);
        assert!(dim.0 % 8 == 0);

        let (width, height) = dim;
        /// rgb pixel size in bytes
        const RGB_PIXEL_LEN: usize = 3;
        let rgb_bytes_per_row: usize = RGB_PIXEL_LEN * width;

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
            Self::write_rgb8_f32x8_row(y_row, u_row, v_row, width, row_target);

            // load Y values for second row
            let base_y = (2 * y + 1) * strides.0;
            let y_row = &y_plane[base_y..base_y + strides.0];

            // calculate second RGB row
            let base_tgt = (2 * y + 1) * rgb_bytes_per_row;
            let row_target = &mut target[base_tgt..(base_tgt + rgb_bytes_per_row)];
            Self::write_rgb8_f32x8_row(y_row, u_row, v_row, width, row_target);
        }
    }

    #[inline(always)]
    fn write_rgb8_f32x8_row(y_row: &[u8], u_row: &[u8], v_row: &[u8], width: usize, target: &mut [u8]) {
        assert_eq!(y_row.len(), u_row.len() * 2);
        assert_eq!(y_row.len(), v_row.len() * 2);

        let rv_mul = wide::f32x8::splat(1.402);
        let gu_mul = wide::f32x8::splat(-0.344);
        let gv_mul = wide::f32x8::splat(-0.714);
        let bu_mul = wide::f32x8::splat(1.772);

        let upper_bound = wide::f32x8::splat(255.0);
        let lower_bound = wide::f32x8::splat(0.0);

        const STEP: usize = 8;
        assert!(y_row.len() % STEP == 0);

        const UV_STEP: usize = STEP / 2;
        assert!(u_row.len() % UV_STEP == 0);
        assert!(v_row.len() % UV_STEP == 0);

        const TGT_STEP: usize = STEP * 3;
        assert!(target.len() % TGT_STEP == 0);

        let mut base_y = 0;
        let mut base_uv = 0;
        let mut base_tgt = 0;

        for _ in (0..width).step_by(STEP) {
            let pixels = &mut target[base_tgt..(base_tgt + TGT_STEP)];

            let y_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(y_row[base_y..], 1);
            let u_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(u_row[base_uv..], 2) - 128.0;
            let v_pack: wide::f32x8 = f32x8_from_slice_with_blocksize!(v_row[base_uv..], 2) - 128.0;

            let r_pack = v_pack.mul_add(rv_mul, y_pack);
            let g_pack = v_pack.mul_add(gv_mul, u_pack.mul_add(gu_mul, y_pack));
            let b_pack = u_pack.mul_add(bu_mul, y_pack);

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

    // TODO: Ideally we'd like to move these out into a converter in `formats`.
    /// Writes the image into a byte buffer of size `w*h*4`.
    ///
    /// # Panics
    ///
    /// Panics if the target image dimension don't match the configured format.
    #[allow(clippy::unnecessary_cast)]
    pub fn write_rgba8(&self, target: &mut [u8]) {
        let dim = self.dimensions();
        let strides = self.strides();
        let wanted = dim.0 * dim.1 * 4;

        // This needs some love, and better architecture.
        assert_eq!(self.info.iFormat, videoFormatI420 as i32);
        assert_eq!(
            target.len(),
            wanted,
            "Target RGBA8 array does not match image dimensions. Wanted: {} * {} * 4 = {}, got {}",
            dim.0,
            dim.1,
            wanted,
            target.len()
        );

        for y in 0..dim.1 {
            for x in 0..dim.0 {
                let base_tgt = (y * dim.0 + x) * 4;
                let base_y = y * strides.0 + x;
                let base_u = (y / 2 * strides.1) + (x / 2);
                let base_v = (y / 2 * strides.2) + (x / 2);

                let rgb_pixel = &mut target[base_tgt..base_tgt + 4];

                let y = self.y[base_y] as f32;
                let u = self.u[base_u] as f32;
                let v = self.v[base_v] as f32;

                rgb_pixel[0] = (y + 1.402 * (v - 128.0)) as u8;
                rgb_pixel[1] = (y - 0.344 * (u - 128.0) - 0.714 * (v - 128.0)) as u8;
                rgb_pixel[2] = (y + 1.772 * (u - 128.0)) as u8;
                rgb_pixel[3] = 255;
            }
        }
    }
}

#[test]
fn convert_yuv_to_rgb_512x512() {
    let source = include_bytes!("../tests/data/single_512x512_cavlc.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;

    let tgt = &mut rgb[0..rgb_len];

    DecodedYUV::write_rgb8_scalar(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), tgt);

    let mut tgt2 = vec![0; tgt.len()];
    DecodedYUV::write_rgb8_f32x8(yuv.y(), yuv.u(), yuv.v(), yuv.dimensions(), yuv.strides(), &mut tgt2);

    assert_eq!(tgt, tgt2);
}

impl YUVSource for DecodedYUV<'_> {
    fn dimensions_i32(&self) -> (i32, i32) {
        (self.info.iWidth, self.info.iHeight)
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.info.iWidth as usize, self.info.iHeight as usize)
    }

    fn strides(&self) -> (usize, usize, usize) {
        // iStride is an array of size 2, so indices are really (0, 1, 1)
        (
            self.info.iStride[0] as usize,
            self.info.iStride[1] as usize,
            self.info.iStride[1] as usize,
        )
    }

    fn strides_i32(&self) -> (i32, i32, i32) {
        // iStride is an array of size 2, so indices are really (0, 1, 1)
        (self.info.iStride[0], self.info.iStride[1], self.info.iStride[1])
    }

    fn y(&self) -> &[u8] {
        self.y
    }

    fn u(&self) -> &[u8] {
        self.u
    }

    fn v(&self) -> &[u8] {
        self.v
    }
}
