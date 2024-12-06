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
    videoFormatI420, ISVCDecoder, ISVCDecoderVtbl, SBufferInfo, SDecodingParam, SParserBsInfo, SSysMEMBuffer, API, DECODER_OPTION, DECODER_OPTION_ERROR_CON_IDC, DECODER_OPTION_NUM_OF_FRAMES_REMAINING_IN_BUFFER, DECODER_OPTION_NUM_OF_THREADS, DECODER_OPTION_TRACE_LEVEL, DECODING_STATE, WELS_LOG_DETAIL, WELS_LOG_QUIET
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

impl<'a> DecodedYUV<'a> {
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

        for y in 0..dim.1 {
            for x in 0..dim.0 {
                let base_tgt = (y * dim.0 + x) * 3;
                let base_y = y * strides.0 + x;
                let base_u = (y / 2 * strides.1) + (x / 2);
                let base_v = (y / 2 * strides.2) + (x / 2);

                let rgb_pixel = &mut target[base_tgt..base_tgt + 3];

                let y = self.y[base_y] as f32;
                let u = self.u[base_u] as f32;
                let v = self.v[base_v] as f32;

                rgb_pixel[0] = (y + 1.402 * (v - 128.0)) as u8;
                rgb_pixel[1] = (y - 0.344 * (u - 128.0) - 0.714 * (v - 128.0)) as u8;
                rgb_pixel[2] = (y + 1.772 * (u - 128.0)) as u8;
            }
        }
    }

    pub fn write_rgb8_lookup(&self, target: &mut [u8]) {
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

        for y in 0..dim.1 {
            for x in 0..dim.0 {
                let base_tgt = (y * dim.0 + x) * 3;
                let base_y = y * strides.0 + x;
                let base_u = (y / 2 * strides.1) + (x / 2);
                let base_v = (y / 2 * strides.2) + (x / 2);

                let rgb_pixel = &mut target[base_tgt..base_tgt + 3];

                let y = self.y[base_y] as f32;
                let u = self.u[base_u] as usize;
                let v = self.v[base_v] as usize;

                rgb_pixel[0] = (y + crate::lookup::float::RV_LOOKUP[v]) as u8;
                rgb_pixel[1] = (y - crate::lookup::float::GU_LOOKUP[u] - crate::lookup::float::GV_LOOKUP[v]) as u8;
                rgb_pixel[2] = (y + crate::lookup::float::BU_LOOKUP[u]) as u8;
            }
        }
    }

    pub fn write_rgb8_i16_math(&self, target: &mut [u8]) {
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

        const F: u8 = 6; // powers of two
        const FACTOR: f32 = (1 << F) as f32;
        const UV_SUB: i16 = 128;
        const RV_FACT: i16 = (1.402 * FACTOR) as i16;
        const GU_FACT: i16 = (0.344 * FACTOR) as i16;
        const GV_FACT: i16 = (0.714 * FACTOR) as i16;
        const BU_FACT: i16 = (1.772 * FACTOR) as i16;
        
        for y in 0..dim.0 {
            for x in 0..dim.1 {
                let base_tgt = (y * dim.0 + x) * 3;
                let base_y = y * strides.0 + x;
                let base_u = (y / 2 * strides.1) + (x / 2);
                let base_v = (y / 2 * strides.2) + (x / 2);

                let rgb_pixel = &mut target[base_tgt..base_tgt + 3];

                let y2 = (self.y[base_y] as i16) << F;
                let u2 = self.u[base_u] as i16;
                let v2 = self.v[base_v] as i16;

                let rv = RV_FACT * (v2 - UV_SUB);
                let r2 = (y2 + rv) >> F;
                let r2 = r2.clamp(0, 255) as u8;

                let g2u = GU_FACT * (u2 - UV_SUB);
                let g2v = GV_FACT * (v2 - UV_SUB);
                let g2 = (y2 - g2u - g2v) >> F;
                let g2 = g2.clamp(0, 255) as u8;

                let bu = BU_FACT * (u2 - UV_SUB);            
                let b2 = (y2 + bu) >> F;
                let b2 = b2.clamp(0, 255) as u8;
        
                rgb_pixel[0] = r2;
                rgb_pixel[1] = g2;
                rgb_pixel[2] = b2;
            }
        }
    }  

    pub fn write_rgb8_i16_lookup(&self, target: &mut [u8]) {
        let dim = self.dimensions();
        let strides = self.strides();

        const F: u8 = 6; // powers of two

        for y in 0..dim.1 {
            for x in 0..dim.0 {
                let base_tgt = (y * dim.0 + x) * 3;
                let base_y = y * strides.0 + x;
                let base_u = (y / 2 * strides.1) + (x / 2);
                let base_v = (y / 2 * strides.2) + (x / 2);

                let rgb_pixel = &mut target[base_tgt..base_tgt + 3];

                let y = (self.y[base_y] as i16) << F;
                let u = self.u[base_u] as usize;
                let v = self.v[base_v] as usize;

                rgb_pixel[0] = ((y + crate::lookup::int16::RV_LOOKUP[v]) >> F).clamp(0, 255) as u8;
                rgb_pixel[1] = ((y - crate::lookup::int16::GU_LOOKUP[u] - crate::lookup::int16::GV_LOOKUP[v]) >> F).clamp(0, 255) as u8;
                rgb_pixel[2] = ((y + crate::lookup::int16::BU_LOOKUP[u]) >> F).clamp(0, 255) as u8;
            }
        }
    }

    pub fn write_rgb8_x8(&self, target: &mut [u8]) {
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

        for y in 0..dim.1 {
            for x in (0..dim.0).step_by(8) {
                let base_tgt = (y * dim.0 + x) * 3;
                let base_y = y * strides.0 + x;
                let base_u = (y / 2 * strides.1) + (x / 2);
                let base_v = (y / 2 * strides.2) + (x / 2);
                
                let pixels = &mut target[base_tgt..(base_tgt + (3 * 8))];

                let y_pack: [f32; 8] = [
                    self.y[base_y] as f32, self.y[base_y + 1] as f32, self.y[base_y + 2] as f32, self.y[base_y + 3] as f32,
                    self.y[base_y + 4] as f32, self.y[base_y + 5] as f32, self.y[base_y + 6] as f32, self.y[base_y + 7] as f32
                ];
                let u_pack: [f32; 8] = [
                    self.u[base_u] as f32, self.u[base_u] as f32, self.u[base_u + 1] as f32, self.u[base_u + 1] as f32,
                    self.u[base_u + 2] as f32, self.u[base_u + 2] as f32, self.u[base_u + 3] as f32, self.u[base_u + 3] as f32
                ];
                let v_pack: [f32; 8] = [
                    self.v[base_v] as f32, self.v[base_v] as f32, self.v[base_v + 1] as f32, self.v[base_v + 1] as f32,
                    self.v[base_v + 2] as f32, self.v[base_v + 2] as f32, self.v[base_v + 3] as f32, self.v[base_v + 3] as f32
                ];

                for i in 0..8 {
                    let y = y_pack[i];
                    let u = u_pack[i];
                    let v = v_pack[i];
    
                    pixels[(3 * i) + 0] = (y + 1.402 * (v - 128.0)) as u8;
                    pixels[(3 * i) + 1] = (y - 0.344 * (u - 128.0) - 0.714 * (v - 128.0)) as u8;
                    pixels[(3 * i) + 2] = (y + 1.772 * (u - 128.0)) as u8;
                }
            }
        }
    }

    // TODO: Ideally we'd like to move these out into a converter in `formats`.
    /// Writes the image into a byte buffer of size `w*h*4`.
    ///
    /// # Panics
    ///
    /// Panics if the target image dimension don't match the configured format.
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
fn test_write_rgb8_int_math() {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;
    
    let tgt = &mut rgb[0..rgb_len];
    yuv.write_rgb8(tgt);

    let mut tgt2 = vec![0; tgt.len()];
    yuv.write_rgb8_i16_math(&mut tgt2);

    if tgt != tgt2 {
        // allow a difference of max (1 / 255) = ca. 0.4% per pixel
        for (i, (a, b)) in tgt.iter().zip(tgt2).enumerate() {
            if (*a as i32 - b as i32).abs() > 1 {
                panic!("mismatch @ {i}, exp: {a}, got {b}");
            }
        }
    }
}

#[test]
fn test_write_rgb8_lookup() {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;
    
    let tgt = &mut rgb[0..rgb_len];
    yuv.write_rgb8(tgt);

    let mut tgt2 = vec![0; tgt.len()];
    yuv.write_rgb8_lookup(&mut tgt2);


    if tgt != tgt2 {
        // allow a difference of max (1 / 255) = ca. 0.4% per pixel
        for (i, (a, b)) in tgt.iter().zip(tgt2).enumerate() {
            if (*a as i32 - b as i32).abs() > 1 {
                panic!("mismatch @ {i}, exp: {a}, got {b}");
            }
        }
    }
}

#[test]
fn test_write_rgb8_int_lookup() {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;
    
    let tgt = &mut rgb[0..rgb_len];
    yuv.write_rgb8(tgt);

    let mut tgt2 = vec![0; tgt.len()];
    yuv.write_rgb8_i16_lookup(&mut tgt2);

    if tgt != tgt2 {
        // allow a difference of max (1 / 255) = ca. 0.4% per pixel
        for (i, (a, b)) in tgt.iter().zip(tgt2).enumerate() {
            if (*a as i32 - b as i32).abs() > 1 {
                panic!("mismatch @ {i}, exp: {a}, got {b}");
            }
        }
    }
}

#[test]
fn test_write_rgb8_x8() {
    let source = include_bytes!("../tests/data/single_512x512_cabac.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config).unwrap();

    let mut rgb = vec![0; 2000 * 2000 * 3];
    let yuv = decoder.decode(&source[..]).unwrap().unwrap();
    let dim = yuv.dimensions();
    let rgb_len = dim.0 * dim.1 * 3;
    
    let tgt = &mut rgb[0..rgb_len];
    yuv.write_rgb8(tgt);

    let mut tgt2 = vec![0; tgt.len()];
    yuv.write_rgb8_x8(&mut tgt2);
    if tgt != tgt2 {
        // allow a difference of max (1 / 255) = ca. 0.4% per pixel
        for (i, (a, b)) in tgt.iter().zip(tgt2).enumerate() {
            if (*a as i32 - b as i32).abs() > 1 {
                panic!("mismatch @ {i}, exp: {a}, got {b}");
            }
        }
    }
}


impl<'a> YUVSource for DecodedYUV<'a> {
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
