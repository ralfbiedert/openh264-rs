//! Converts NAL packets to YUV images.
//!
//! # Examples
//!
//! Basic [Decoder] use looks as follows. In practice, you might get your `h264`
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
//! let rgb_len = yuv.rgb8_len();
//! let mut rgb_raw = vec![0; rgb_len];
//!
//! yuv.write_rgb8(&mut rgb_raw);
//! # }
//! # Ok(())
//! # }
//! ```

use crate::error::NativeErrorExt;
use crate::formats::yuv2rgb::write_rgb8_f32x8_par;
// use crate::formats::yuv2rgb::{write_rgb8_f32x8, write_rgb8_f32x8_par, write_rgb8_scalar, write_rgb8_scalar_par};
use crate::formats::YUVSource;
use crate::formats::yuv2rgb::{write_rgb8_f32x8, write_rgb8_scalar, write_rgba8_f32x8, write_rgba8_scalar};
use crate::{Error, OpenH264API, Timestamp};
use openh264_sys2::{
    API, DECODER_OPTION, DECODER_OPTION_ERROR_CON_IDC, DECODER_OPTION_NUM_OF_FRAMES_REMAINING_IN_BUFFER,
    DECODER_OPTION_NUM_OF_THREADS, DECODER_OPTION_TRACE_LEVEL, DECODING_STATE, ISVCDecoder, ISVCDecoderVtbl, SBufferInfo,
    SDecodingParam, SParserBsInfo, SSysMEMBuffer, SVideoProperty, TagBufferInfo, WELS_LOG_DETAIL, WELS_LOG_QUIET,
    videoFormatI420,
};
use std::os::raw::{c_int, c_long, c_uchar, c_void};
use std::ptr::{addr_of_mut, from_mut, null, null_mut};

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
#[allow(non_snake_case, unused, missing_docs)]
impl DecoderRawAPI {
    fn new(api: OpenH264API) -> Result<Self, Error> {
        unsafe {
            let mut decoder_ptr = null::<ISVCDecoderVtbl>() as *mut *const ISVCDecoderVtbl;

            api.WelsCreateDecoder(from_mut(&mut decoder_ptr)).ok()?;

            let e = || {
                Error::msg("VTable missing function.")
            };

            Ok(Self {
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
    unsafe fn initialize(&self, pParam: *const SDecodingParam) -> c_long { unsafe { (self.initialize)(self.decoder_ptr, pParam) }}
    unsafe fn uninitialize(&self, ) -> c_long { unsafe { (self.uninitialize)(self.decoder_ptr) }}

    pub unsafe fn decode_frame(&self, Src: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pStride: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int) -> DECODING_STATE { unsafe { (self.decode_frame)(self.decoder_ptr, Src, iSrcLen, ppDst, pStride, iWidth, iHeight) }}
    pub unsafe fn decode_frame_no_delay(&self, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE { unsafe { (self.decode_frame_no_delay)(self.decoder_ptr, pSrc, iSrcLen, ppDst, pDstInfo) }}
    pub unsafe fn decode_frame2(&self, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE { unsafe { (self.decode_frame2)(self.decoder_ptr, pSrc, iSrcLen, ppDst, pDstInfo) }}
    pub unsafe fn flush_frame(&self, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE { unsafe { (self.flush_frame)(self.decoder_ptr, ppDst, pDstInfo) }}
    pub unsafe fn decode_parser(&self, pSrc: *const c_uchar, iSrcLen: c_int, pDstInfo: *mut SParserBsInfo) -> DECODING_STATE { unsafe { (self.decode_parser)(self.decoder_ptr, pSrc, iSrcLen, pDstInfo) }}
    pub unsafe fn decode_frame_ex(&self, pSrc: *const c_uchar, iSrcLen: c_int, pDst: *mut c_uchar, iDstStride: c_int, iDstLen: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int, iColorFormat: *mut c_int) -> DECODING_STATE { unsafe { (self.decode_frame_ex)(self.decoder_ptr, pSrc, iSrcLen, pDst, iDstStride, iDstLen, iWidth, iHeight, iColorFormat) }}
    pub unsafe fn set_option(&self, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long { unsafe {  (self.set_option)(self.decoder_ptr, eOptionId, pOption) }}
    pub unsafe fn get_option(&self, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long { unsafe { (self.get_option)(self.decoder_ptr, eOptionId, pOption) }}
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

/// How the decoder should handle flushing.
///
/// The behavior of flushing is somewhat unclear upstream. If you run into decoder errors,
/// you should probably disable automatic flushing, and manually call [`Decoder::flush_remaining`]
/// after all NAL units have been processed. It might be a good idea to do the latter regardless.
///
/// If you have more info on flushing best practices, we'd greatly appreciate a PR to make our
/// decoding pipeline more robust.
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Flush {
    /// Uses the current currently configured decoder default (which is attempted flushing after each decode).
    #[default]
    Auto,
    /// Flushes after each decode operation.
    Flush,
    /// Do not flush after decode operations.
    NoFlush,
}

impl Flush {
    /// Given some existing flush options and some current frame decode options, returns
    /// whether flushing should happen.
    #[allow(clippy::match_same_arms)]
    #[allow(clippy::needless_pass_by_value)]
    const fn should_flush(self, decoder_options: DecodeOptions) -> bool {
        match (self, decoder_options.flush_after_decode) {
            (Self::Auto, Self::Auto) => true,
            (Self::NoFlush, Self::Auto) => false,
            (Self::Flush, Self::Auto) => true,
            (_, Self::NoFlush) => false,
            (_, Self::Flush) => true,
        }
    }
}

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
    flush_after_decode: Flush,
}

unsafe impl Send for DecoderConfig {}
unsafe impl Sync for DecoderConfig {}

impl DecoderConfig {
    /// Creates a new default encoder config.
    pub const fn new() -> Self {
        Self {
            params: SDecodingParam {
                pFileNameRestructed: null_mut(),
                uiCpuLoad: 0,
                uiTargetDqLayer: 0,
                eEcActiveIdc: 0,
                bParseOnly: false,
                sVideoProperty: SVideoProperty {
                    size: 0,
                    eVideoBsType: 0,
                },
            },
            num_threads: 0,
            debug: WELS_LOG_QUIET,
            error_concealment: 0,
            flush_after_decode: Flush::Flush,
        }
    }

    /// Sets the number of threads; will probably segfault the decoder, see below.<sup>⚠️</sup>
    ///
    /// # Safety
    ///
    /// This setting might work on some platforms but will probably just segfault.
    /// Consider this a _highly_ experimental option we only expose to test if and
    /// where threading actually works. Ultimately you should consult with the upstream
    /// OpenH264 project where and when it is safe to set this.
    ///
    /// See [this issue](https://github.com/ralfbiedert/openh264-rust/issues/10) for details.
    pub const unsafe fn num_threads(mut self, num_threads: u32) -> Self {
        self.num_threads = num_threads as i32;
        self
    }

    /// Enables detailed console logging inside OpenH264.
    pub const fn debug(mut self, value: bool) -> Self {
        self.debug = if value { WELS_LOG_DETAIL } else { WELS_LOG_QUIET };
        self
    }

    /// Sets the default flush behavior after decode operations..
    pub const fn flush_after_decode(mut self, flush_behavior: Flush) -> Self {
        self.flush_after_decode = flush_behavior;
        self
    }
}

/// Configuration for the current decode operation.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct DecodeOptions {
    flush_after_decode: Flush,
}

impl DecodeOptions {
    /// Creates new decoder options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            flush_after_decode: Flush::Auto,
        }
    }

    /// Sets the flush behavior for the upcoming decode operation.
    #[must_use]
    pub const fn flush_after_decode(mut self, value: Flush) -> Self {
        self.flush_after_decode = value;
        self
    }
}

/// An [OpenH264](https://github.com/cisco/openh264) decoder.
pub struct Decoder {
    raw_api: DecoderRawAPI,
    config: DecoderConfig,
}

impl Decoder {
    /// Create a decoder with default settings and the built-in decoder.
    ///
    /// This method is only available when compiling with the `source` feature.
    ///
    /// # Errors
    ///
    /// This should never error, but the underlying OpenH264 decoder has an error indication and
    /// since we don't know their code that well we just can't guarantee it.
    #[cfg(feature = "source")]
    pub fn new() -> Result<Self, Error> {
        let api = OpenH264API::from_source();
        Self::with_api_config(api, DecoderConfig::new())
    }

    /// Create a decoder with the provided [API](OpenH264API) and [configuration](DecoderConfig).
    ///
    /// # Errors
    ///
    /// Might fail if the provided encoder parameters had issues.
    pub fn with_api_config(api: OpenH264API, mut config: DecoderConfig) -> Result<Self, Error> {
        let raw_api = DecoderRawAPI::new(api)?;

        // config.params.sVideoProperty.eVideoBsType = VIDEO_BITSTREAM_AVC;

        #[rustfmt::skip]
        unsafe {
            raw_api.initialize(&raw const config.params).ok()?;
            raw_api.set_option(DECODER_OPTION_TRACE_LEVEL, addr_of_mut!(config.debug).cast()).ok()?;
            raw_api.set_option(DECODER_OPTION_NUM_OF_THREADS, addr_of_mut!(config.num_threads).cast()).ok()?;
            raw_api.set_option(DECODER_OPTION_ERROR_CON_IDC, addr_of_mut!(config.error_concealment).cast()).ok()?;
        };

        Ok(Self { raw_api, config })
    }

    /// Decodes a series of H.264 NAL packets and returns the latest picture.
    ///
    /// This is a convenience wrapper around [`decode_with_options`](Self::decode_with_options) that uses default decoding options.
    ///
    /// # Errors
    ///
    /// The function returns an error if the bitstream was corrupted.
    pub fn decode(&mut self, packet: &[u8]) -> Result<Option<DecodedYUV<'_>>, Error> {
        self.decode_with_options(packet, DecodeOptions::default())
    }

    /// Decodes a series of H.264 NAL packets and returns the latest picture.
    ///
    /// This function can be called with:
    ///
    /// - only the complete SPS / PPS header (usually the first some 30 bytes of a H.264 stream),
    /// - the headers and series of complete frames,
    /// - new frames after previous headers and frames were successfully decoded.
    ///
    /// In each case, it will return `Some(decoded)` image in YUV format if an image was available, or `None`
    /// if more data needs to be provided. If `options` contains [`Flush`](Flush::Flush) (or if this
    /// is set as the decoder default), it will try to flush a frame no image was available.
    ///
    /// In any case, it is probably a good idea to call [`Decoder::flush_remaining`] after you
    /// finished decoding all available NAL units.
    ///
    /// # Errors
    ///
    /// - The function returns an error if the bitstream was corrupted.
    /// - Also, flushing best practices are somewhat hard to come by in OpenH264. You might get errors
    ///   if you flushed when you shouldn't have, although we cannot exactly tell you when that is.
    ///   If you have more information on how to make this more robust, a PR would be greatly welcome.
    pub fn decode_with_options(&mut self, packet: &[u8], options: DecodeOptions) -> Result<Option<DecodedYUV<'_>>, Error> {
        let mut dst = [null_mut::<u8>(); 3];
        let mut buffer_info = SBufferInfo::default();
        let flush = self.config.flush_after_decode.should_flush(options);

        unsafe {
            self.raw_api
                .decode_frame_no_delay(
                    packet.as_ptr(),
                    packet.len() as i32,
                    from_mut(&mut dst).cast(),
                    &raw mut buffer_info,
                )
                .ok()?;
        }

        match (buffer_info.iBufferStatus, flush) {
            // No outstanding images, but asked to flush, and flushable frames available?
            (0, true) if self.num_frames_in_buffer()? > 0 => {
                let (dst, buffer_info) = self.flush_single_frame_raw()?;

                if buffer_info.iBufferStatus == 0 {
                    return Err(Error::msg(
                        "Buffer status invalid, we have outstanding frames but failed to flush them.",
                    ));
                }

                unsafe { Ok(DecodedYUV::from_raw_open264_ptrs(&dst, &buffer_info)) }
            }
            // No outstanding images otherwise? Nothing to do.
            (0, _) => Ok(None),
            // Outstanding images otherwise? Return one.
            _ => unsafe { Ok(DecodedYUV::from_raw_open264_ptrs(&dst, &buffer_info)) },
        }
    }

    /// Flush and return all remaining frames in the buffer.
    ///
    /// This function should be called after decoding all frames of a NAL stream.
    ///
    /// # Errors
    ///
    /// The function returns an error if the bitstream was corrupted.
    pub fn flush_remaining(&'_ mut self) -> Result<Vec<DecodedYUV<'_>>, Error> {
        let mut frames = Vec::new();

        for _ in 0..self.num_frames_in_buffer()? {
            let (dst, buffer_info) = self.flush_single_frame_raw()?;

            if let Some(image) = unsafe { DecodedYUV::from_raw_open264_ptrs(&dst, &buffer_info) } {
                frames.push(image);
            }
        }

        Ok(frames)
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
    pub const unsafe fn raw_api(&mut self) -> &mut DecoderRawAPI {
        &mut self.raw_api
    }

    /// Returns the number of frames currently remaining in the buffer.
    fn num_frames_in_buffer(&mut self) -> Result<usize, Error> {
        let mut num_frames: DECODER_OPTION = 0;
        unsafe {
            self.raw_api()
                .get_option(
                    DECODER_OPTION_NUM_OF_FRAMES_REMAINING_IN_BUFFER,
                    addr_of_mut!(num_frames).cast(),
                )
                .ok()?;
        }

        Ok(num_frames as usize)
    }

    /// Attempts to flush a single frame (i.e., produce a new YUV from previously passed bitstream data), if available.
    fn flush_single_frame_raw(&mut self) -> Result<([*mut u8; 3], TagBufferInfo), Error> {
        let mut dst = [null_mut::<u8>(); 3];
        let mut buffer_info = SBufferInfo::default();

        unsafe {
            self.raw_api()
                .flush_frame(from_mut(&mut dst).cast(), &raw mut buffer_info)
                .ok()?;
            Ok((dst, buffer_info))
        }
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

impl DecodedYUV<'_> {
    /// Attempts to create a decoded YUV wrapper from a set of Open264 pointers.
    ///
    /// This can soft-fail (return `None`) because we might still have gotten `null` pointers from
    /// OpenH264 despite it not having returned an error on decode.
    const unsafe fn from_raw_open264_ptrs(dst: &[*mut u8; 3], buffer_info: &TagBufferInfo) -> Option<Self> {
        unsafe {
            let info = buffer_info.UsrData.sSystemBuffer;
            let timestamp = Timestamp::from_millis(buffer_info.uiInBsTimeStamp); // TODO: Is this the right one?

            // Apparently it is ok for `decode_frame_no_delay` to not return an error _and_ to return null buffers. In this case
            // the user should try to continue decoding.
            if dst[0].is_null() || dst[1].is_null() || dst[2].is_null() {
                None
            } else {
                // https://github.com/cisco/openh264/issues/2379
                let y = std::slice::from_raw_parts(dst[0], (info.iHeight * info.iStride[0]) as usize);
                let u = std::slice::from_raw_parts(dst[1], (info.iHeight * info.iStride[1] / 2) as usize);
                let v = std::slice::from_raw_parts(dst[2], (info.iHeight * info.iStride[1] / 2) as usize);

                Some(Self {
                    info,
                    timestamp,
                    y,
                    u,
                    v,
                })
            }
        }
    }

    /// Returns the unpadded U size.
    ///
    /// This is often smaller (by half) than the image size.
    #[must_use]
    pub const fn dimensions_uv(&self) -> (usize, usize) {
        (self.info.iWidth as usize / 2, self.info.iHeight as usize / 2)
    }

    /// Timestamp of this frame in milliseconds(?) with respect to the video stream.
    #[must_use]
    pub const fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Cut the YUV buffer into vertical sections of equal length.
    #[must_use]
    pub fn split<const N: usize>(&self) -> [(&[u8], &[u8], &[u8]); N] {
        if N == 1 {
            return [(self.y, self.u, self.v); N];
        }

        // Is there a chance to use self.y.len() / N?
        //   - can len(), stride and width mess it up?
        let y_stride = self.info.iStride[0] as usize;
        let y_lines = self.y.len() / y_stride;
        let lines_per_split = y_lines / N;
        let y_chunks: Vec<&[u8]> = self.y.chunks(lines_per_split * y_stride).collect();

        let uv_stride = self.info.iStride[1] as usize;
        let uv_lines = self.u.len() / uv_stride;
        let lines_per_split = uv_lines / N;
        dbg!(uv_lines, N, lines_per_split);
        let u_chunks: Vec<&[u8]> = self.u.chunks(lines_per_split * uv_stride).collect();
        let v_chunks: Vec<&[u8]> = self.v.chunks(lines_per_split * uv_stride).collect();

        let mut parts = [(self.y, self.u, self.v); N];
        for i in 0..N {
            parts[i] = (y_chunks[i], u_chunks[i], v_chunks[i]);
        }

        parts
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

        write_rgb8_f32x8_par(self.y, self.u, self.v, dim, strides, target);
        // // for f32x8 math, image needs to:
        // //   - have a width divisible by 8
        // //   - have at least two rows
        // if dim.0 % 8 == 0 && dim.1 >= 2 {
        //     write_rgb8_f32x8(self.y, self.u, self.v, dim, strides, target);
        // } else {
        //     write_rgb8_scalar(self.y, self.u, self.v, dim, strides, target);
        // }
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
        // for f32x8 math, image needs to:
        //   - have a width divisible by 8
        //   - have at least two rows
        if dim.0 % 8 == 0 && dim.1 >= 2 {
            write_rgba8_f32x8(self.y, self.u, self.v, dim, strides, target);
        } else {
            write_rgba8_scalar(self.y, self.u, self.v, dim, strides, target);
        }
    }
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

#[cfg(test)]
mod test {
    use openh264_sys2::SSysMEMBuffer;

    use crate::Timestamp;

    use super::DecodedYUV;

    /// Create YUV420 plane buffers.
    ///
    /// Usage: `let (y, u, v) = planes!(strides: (132, 132), dim: (128, 128));`
    macro_rules! planes {
        (strides: ($y_stride:literal, $uv_stride:literal), dim: ($width:literal, $height:literal)) => {{
            // iterate over numbers from 0..255 and start from 0 after 255
            let numbers = (0..u32::MAX).map(|i| (i % 256) as u8);

            let y_plane_len = ($y_stride * $height) as usize;
            let y = numbers.clone().take(y_plane_len).collect::<Vec<_>>();

            // u & v planes are half the height of y plane in YUV420
            let uv_plane_len = ($uv_stride * $height / 4) as usize;
            let u = numbers.clone().take(uv_plane_len).collect::<Vec<_>>();
            let v = numbers.clone().take(uv_plane_len).collect::<Vec<_>>();

            (y, u, v)
        }};
    }

    /// Create a mock DecodedYUV without iFormat and Timestamp::ZERO
    ///
    /// Usage: `let buf = decoded_yuv!(strides: (132, 132), dim: (128, 128), &y, &u, &v);`
    macro_rules! decoded_yuv {
        (strides: ($y_stride:literal, $uv_stride:literal), dim: ($width:literal, $height:literal), $y:expr, $u:expr, $v:expr) => {
            DecodedYUV {
                info: SSysMEMBuffer {
                    iWidth: $width,
                    iHeight: $height,
                    // YUV420 see: https://github.com/cisco/openh264/blob/0c9a557a9a6f1d267c4d372221669a8ae69ccda0/codec/api/wels/codec_def.h#L56
                    iFormat: 23,
                    iStride: [$y_stride as i32, $uv_stride as i32],
                },
                timestamp: Timestamp::ZERO,
                y: $y,
                u: $u,
                v: $v,
            }
        };
    }

    #[test]
    fn test_split_01() {
        // smallest possible buffer in YUV420
        let (y, u, v) = planes!(strides: (4, 4), dim: (4, 4));
        let buf = decoded_yuv!(strides: (4, 4), dim: (4, 4), &y, &u, &v);

        let parts: [(&[u8], &[u8], &[u8]); 1] = buf.split();
        assert_eq!(1, parts.len());
        assert_eq!(parts[0], (y.as_slice(), u.as_slice(), v.as_slice()));
    }

    #[test]
    fn test_split_02() {
        let (y, u, v) = planes!(strides: (132, 132), dim: (128, 128));
        let buf = decoded_yuv!(strides: (132, 132), dim: (128, 128), &y, &u, &v);

        let parts: [(&[u8], &[u8], &[u8]); 4] = buf.split();

        let (mut y_plane, mut u_plane, mut v_plane) = (vec![], vec![], vec![]);
        for (y_p, u_p, v_p) in parts {
            y_plane.extend_from_slice(y_p);
            u_plane.extend_from_slice(u_p);
            v_plane.extend_from_slice(v_p);
        }

        assert_eq!(buf.y.len(), y_plane.len());
        assert_eq!(buf.y, y_plane);
        assert_eq!(buf.u.len(), u_plane.len());
        assert_eq!(buf.u, u_plane);
        assert_eq!(buf.v.len(), v_plane.len());
        assert_eq!(buf.v, v_plane);
    }
}
