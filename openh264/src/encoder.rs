//! Converts images to packets.

use crate::error::NativeErrorExt;
use crate::Error;
use openh264_sys2::{
    videoFormatI420, videoFrameTypeSkip, EVideoFormatType, EVideoFrameType, ISVCEncoder, ISVCEncoderVtbl, SEncParamBase, SEncParamExt, SFrameBSInfo, SSourcePicture, WelsCreateSVCEncoder, WelsDestroySVCEncoder, ENCODER_OPTION, ENCODER_OPTION_DATAFORMAT, ENCODER_OPTION_TRACE_LEVEL, VIDEO_CODING_LAYER, WELS_LOG_DETAIL, WELS_LOG_QUIET
};
use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr::{addr_of_mut, null};

/// Convenience wrapper with guaranteed function pointers for easy access.
///
/// This struct automatically handles `WelsCreateSVCEncoder` and `WelsDestroySVCEncoder`.
#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct EncoderRawAPI {
    encoder_ptr: *mut *const ISVCEncoderVtbl,
    initialize: unsafe extern "C" fn(arg1: *mut ISVCEncoder, pParam: *const SEncParamBase) -> c_int,
    initialize_ext: unsafe extern "C" fn(arg1: *mut ISVCEncoder, pParam: *const SEncParamExt) -> c_int,
    get_default_params: unsafe extern "C" fn(arg1: *mut ISVCEncoder, pParam: *mut SEncParamExt) -> c_int,
    uninitialize: unsafe extern "C" fn(arg1: *mut ISVCEncoder) -> c_int,
    encode_frame: unsafe extern "C" fn(arg1: *mut ISVCEncoder, kpSrcPic: *const SSourcePicture, pBsInfo: *mut SFrameBSInfo) -> c_int,
    encode_parameter_sets: unsafe extern "C" fn(arg1: *mut ISVCEncoder, pBsInfo: *mut SFrameBSInfo) -> c_int,
    force_intra_frame: unsafe extern "C" fn(arg1: *mut ISVCEncoder, bIDR: bool) -> c_int,
    set_option: unsafe extern "C" fn(arg1: *mut ISVCEncoder, eOptionId: ENCODER_OPTION, pOption: *mut c_void) -> c_int,
    get_option: unsafe extern "C" fn(arg1: *mut ISVCEncoder, eOptionId: ENCODER_OPTION, pOption: *mut c_void) -> c_int,
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::missing_safety_doc)]
#[allow(non_snake_case)]
#[allow(unused)]
impl EncoderRawAPI {
    fn new() -> Result<Self, Error> {
        unsafe {
            let mut encoder_ptr = null::<ISVCEncoderVtbl>() as *mut *const ISVCEncoderVtbl;

            WelsCreateSVCEncoder(&mut encoder_ptr as *mut *mut *const ISVCEncoderVtbl).ok()?;

            let e = || {
                Error::msg("VTable missing function.")
            };

            Ok(EncoderRawAPI {
                encoder_ptr,
                initialize: (*(*encoder_ptr)).Initialize.ok_or_else(e)?,
                initialize_ext: (*(*encoder_ptr)).InitializeExt.ok_or_else(e)?,
                get_default_params: (*(*encoder_ptr)).GetDefaultParams.ok_or_else(e)?,
                uninitialize: (*(*encoder_ptr)).Uninitialize.ok_or_else(e)?,
                encode_frame: (*(*encoder_ptr)).EncodeFrame.ok_or_else(e)?,
                encode_parameter_sets: (*(*encoder_ptr)).EncodeParameterSets.ok_or_else(e)?,
                force_intra_frame: (*(*encoder_ptr)).ForceIntraFrame.ok_or_else(e)?,
                set_option: (*(*encoder_ptr)).SetOption.ok_or_else(e)?,
                get_option: (*(*encoder_ptr)).GetOption.ok_or_else(e)?,
            })
        }
    }

    // Exposing these will probably do more harm than good.
    unsafe fn uninitialize(&self) -> c_int { (self.uninitialize)(self.encoder_ptr) }
    unsafe fn initialize(&self, pParam: *const SEncParamBase) -> c_int { (self.initialize)(self.encoder_ptr, pParam) }
    unsafe fn initialize_ext(&self, pParam: *const SEncParamExt) -> c_int { (self.initialize_ext)(self.encoder_ptr, pParam) }

    pub unsafe fn get_default_params(&self, pParam: *mut SEncParamExt) -> c_int { (self.get_default_params)(self.encoder_ptr, pParam) }
    pub unsafe fn encode_frame(&self, kpSrcPic: *const SSourcePicture, pBsInfo: *mut SFrameBSInfo) -> c_int { (self.encode_frame)(self.encoder_ptr, kpSrcPic, pBsInfo) }
    pub unsafe fn encode_parameter_sets(&self, pBsInfo: *mut SFrameBSInfo) -> c_int { (self.encode_parameter_sets)(self.encoder_ptr, pBsInfo) }
    pub unsafe fn force_intra_frame(&self, bIDR: bool) -> c_int { (self.force_intra_frame)(self.encoder_ptr, bIDR) }
    pub unsafe fn set_option(&self, eOptionId: ENCODER_OPTION, pOption: *mut c_void) -> c_int { (self.set_option)(self.encoder_ptr, eOptionId, pOption) }
    pub unsafe fn get_option(&self, eOptionId: ENCODER_OPTION, pOption: *mut c_void) -> c_int { (self.get_option)(self.encoder_ptr, eOptionId, pOption) }
}

impl Drop for EncoderRawAPI {
    fn drop(&mut self) {
        // Safe because when we drop the pointer must have been initialized, and we aren't clone.
        unsafe {
            WelsDestroySVCEncoder(self.encoder_ptr);
        }
    }
}

/// Configuration for the [`Encoder`].
///
/// Setting missing? Please file a PR!
#[derive(Default, Copy, Clone)]
pub struct EncoderConfig {
    width: u32,
    height: u32,
    enable_skip_frame: bool,
    target_bitrate: u32,
    enable_denoise: bool,
    debug: i32,
    data_format: EVideoFormatType,
}

impl EncoderConfig {
    /// Creates a new default encoder config.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            enable_skip_frame: true,
            target_bitrate: 120_000,
            enable_denoise: false,
            debug: 0,
            data_format: videoFormatI420,
        }
    }

    pub fn set_bitrate_bps(mut self, bps: u32) -> Self {
        self.target_bitrate = bps;
        self
    }

    pub fn debug(mut self, value: bool) -> Self {
        self.debug = if value { WELS_LOG_DETAIL } else { WELS_LOG_QUIET };
        self
    }
}

/// An [OpenH264](https://github.com/cisco/openh264) encoder.
pub struct Encoder {
    params: SEncParamExt,
    raw_api: EncoderRawAPI,
}

impl Encoder {
    /// Create an encoder with the provided configuration.
    pub fn with_config(mut config: EncoderConfig) -> Result<Self, Error> {
        let raw_api = EncoderRawAPI::new()?;
        let mut params = SEncParamExt::default();

        unsafe {
            raw_api.get_default_params(&mut params).ok()?;
            params.iPicWidth = config.width as c_int;
            params.iPicHeight = config.height as c_int;
            params.bEnableFrameSkip = config.enable_skip_frame;
            params.iTargetBitrate = config.target_bitrate as c_int;
            params.bEnableDenoise = config.enable_denoise;
            raw_api.initialize_ext(&params).ok()?;
            raw_api
                .set_option(ENCODER_OPTION_TRACE_LEVEL, addr_of_mut!(config.debug).cast())
                .ok()?;
            raw_api
                .set_option(ENCODER_OPTION_DATAFORMAT, addr_of_mut!(config.data_format).cast())
                .ok()?;
        };

        Ok(Self { params, raw_api })
    }

    pub fn encode<T: YUVSource>(&mut self, yuv_source: &T) -> Result<EncodedBitStream, Error> {
        assert_eq!(yuv_source.width(), self.params.iPicWidth);
        assert_eq!(yuv_source.height(), self.params.iPicHeight);

        let mut source = SSourcePicture::default();
        let mut bit_stream_info = SFrameBSInfo::default();

        source.iColorFormat = videoFormatI420;
        source.iPicWidth = self.params.iPicWidth;
        source.iPicHeight = self.params.iPicHeight;
        source.iStride[0] = yuv_source.y_stride();
        source.iStride[1] = yuv_source.u_stride();
        source.iStride[2] = yuv_source.v_stride();

        unsafe {
            // Converting *const u8 to *mut u8 should be fine because the encoder _should_
            // only read these arrays (TOOD: needs verification).
            source.pData[0] = yuv_source.y().as_ptr() as *mut c_uchar;
            source.pData[1] = yuv_source.u().as_ptr() as *mut c_uchar;
            source.pData[2] = yuv_source.v().as_ptr() as *mut c_uchar;

            self.raw_api.encode_frame(&source, &mut bit_stream_info).ok()?;

            // returning the first video layer
            for layer_idx in 0..bit_stream_info.iLayerNum as usize {
                if bit_stream_info.sLayerInfo[layer_idx].uiLayerType != VIDEO_CODING_LAYER as u8 {
                    continue;
                }
                let mut size = 0;
                for nal_idx in 0..bit_stream_info.sLayerInfo[layer_idx].iNalCount as isize {
                    size = size + *bit_stream_info.sLayerInfo[layer_idx].pNalLengthInByte.offset(nal_idx);
                }
                let buffer = std::slice::from_raw_parts(bit_stream_info.sLayerInfo[layer_idx].pBsBuf, size as usize);
                return Ok(EncodedBitStream {
                    bit_stream: buffer,
                    frame_type: bit_stream_info.eFrameType,
                });
            }
        }
        Ok(EncodedBitStream {
            bit_stream: &[],
            frame_type: videoFrameTypeSkip,
        })
    }

    /// Obtain the raw API an initialized encoder object for advanced use cases.
    ///
    /// When resorting to this call, please consider filing an issue / PR to safely wrap your use case.
    ///
    /// # Safety
    ///
    /// You must not set parameters the encoder relies on, we recommend checking the source.
    pub unsafe fn raw_api(&mut self) -> &mut EncoderRawAPI {
        &mut self.raw_api
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // Safe because when we drop the pointer must have been initialized.
        unsafe {
            self.raw_api.uninitialize();
        }
    }
}

/// Encoding Output, currently takes only the first Video Layer
pub struct EncodedBitStream<'a> {
    pub bit_stream: &'a [u8],
    pub frame_type: EVideoFrameType,
}

/// Allows encode to be generic over a YUV Source
pub trait YUVSource {
    fn width(&self) -> i32;
    fn height(&self) -> i32;

    fn y(&self) -> &[u8];
    fn u(&self) -> &[u8];
    fn v(&self) -> &[u8];

    fn y_stride(&self) -> i32;
    fn u_stride(&self) -> i32;
    fn v_stride(&self) -> i32;
}

pub struct RBGYUVConverter {
    yuv: Vec<u8>,
    width: usize,
    height: usize,
}

impl RBGYUVConverter {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            yuv: vec![0u8; (3 * (width * height)) / 2],
            width,
            height,
        }
    }

    pub fn convert(&mut self, rgb: &[u8]) {
        let width = self.width;
        let height = self.height;

        assert_eq!(rgb.len(), width * height * 3);
        assert_eq!(width % 2, 0, "width needs to be multiple of 2");
        assert_eq!(height % 2, 0, "height needs to be a multiple of 2");

        // y is full size, u, v is quarter size
        let pixel = |x: usize, y: usize| -> (f32, f32, f32) {
            // two dim to single dim
            let base_pos = (x + y * width) * 3;
            (rgb[base_pos] as f32, rgb[base_pos + 1] as f32, rgb[base_pos + 2] as f32)
        };
        let write_y = |yuv: &mut [u8], x: usize, y: usize, rgb: (f32, f32, f32)| {
            yuv[x + y * width] = (0.2578125 * rgb.0 + 0.50390625 * rgb.1 + 0.09765625 * rgb.2 + 16.0) as u8;
        };
        let u_base = width * height;
        let half_width = width / 2;
        let write_u = |yuv: &mut [u8], x: usize, y: usize, rgb: (f32, f32, f32)| {
            yuv[u_base + x + y * half_width] = (-0.1484375 * rgb.0 + -0.2890625 * rgb.1 + 0.4375 * rgb.2 + 128.0) as u8;
        };
        let v_base = u_base + u_base / 4;
        let write_v = |yuv: &mut [u8], x: usize, y: usize, rgb: (f32, f32, f32)| {
            yuv[v_base + x + y * half_width] = (0.4375 * rgb.0 + -0.3671875 * rgb.1 + -0.0703125 * rgb.2 + 128.0) as u8
        };
        for i in 0..width / 2 {
            for j in 0..height / 2 {
                let px = i * 2;
                let py = j * 2;
                let pix0x0 = pixel(px, py);
                let pix0x1 = pixel(px, py + 1);
                let pix1x0 = pixel(px + 1, py);
                let pix1x1 = pixel(px + 1, py + 1);
                let avg_pix = (
                    (pix0x0.0 as u32 + pix0x1.0 as u32 + pix1x0.0 as u32 + pix1x1.0 as u32) as f32 / 4.0,
                    (pix0x0.1 as u32 + pix0x1.1 as u32 + pix1x0.1 as u32 + pix1x1.1 as u32) as f32 / 4.0,
                    (pix0x0.2 as u32 + pix0x1.2 as u32 + pix1x0.2 as u32 + pix1x1.2 as u32) as f32 / 4.0,
                );
                write_y(&mut self.yuv[..], px, py, pix0x0);
                write_y(&mut self.yuv[..], px, py + 1, pix0x1);
                write_y(&mut self.yuv[..], px + 1, py, pix1x0);
                write_y(&mut self.yuv[..], px + 1, py + 1, pix1x1);
                write_u(&mut self.yuv[..], i, j, avg_pix);
                write_v(&mut self.yuv[..], i, j, avg_pix);
            }
        }
    }
}

impl YUVSource for RBGYUVConverter {
    fn width(&self) -> i32 {
        self.width as i32
    }

    fn height(&self) -> i32 {
        self.height as i32
    }

    fn y(&self) -> &[u8] {
        &self.yuv[0..self.width * self.height]
    }

    fn u(&self) -> &[u8] {
        let base_u = self.width * self.height;
        &self.yuv[base_u..base_u + base_u / 4]
    }

    fn v(&self) -> &[u8] {
        let base_u = self.width * self.height;
        let base_v = base_u + base_u / 4;
        &self.yuv[base_v..]
    }

    fn y_stride(&self) -> i32 {
        self.width as i32
    }

    fn u_stride(&self) -> i32 {
        (self.width / 2) as i32
    }

    fn v_stride(&self) -> i32 {
        (self.width / 2) as i32
    }
}

#[cfg(test)]
mod tests {
    use crate::encoder::YUVSource;

    use super::RBGYUVConverter;

    #[test]
    fn rgb_to_yuv_conversion_black_2x2() {
        let mut converter = RBGYUVConverter::new(2, 2);
        let rgb = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        converter.convert(&rgb);
        assert_eq!(converter.y(), [16u8, 16u8, 16u8, 16u8]);
        assert_eq!(converter.u(), [128u8]);
        assert_eq!(converter.v(), [128u8]);
        assert_eq!(converter.y_stride(), 2);
        assert_eq!(converter.u_stride(), 1);
        assert_eq!(converter.v_stride(), 1);
    }

    #[test]
    fn rgb_to_yuv_conversion_white_4x2() {
        let mut converter = RBGYUVConverter::new(4, 2);
        let rgb = [
            255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
            255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        ];
        converter.convert(&rgb);
        assert_eq!(converter.y(), [235u8, 235u8, 235u8, 235u8, 235u8, 235u8, 235u8, 235u8]);
        assert_eq!(converter.u(), [128u8, 128u8]);
        assert_eq!(converter.v(), [128u8, 128u8]);
        assert_eq!(converter.y_stride(), 4);
        assert_eq!(converter.u_stride(), 2);
        assert_eq!(converter.v_stride(), 2);
    }

    #[test]
    fn rgb_to_yuv_conversion_red_2x4() {
        let mut converter = RBGYUVConverter::new(4, 2);
        let rgb = [
            255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8, 0u8, 255u8, 0u8,
            0u8, 255u8, 0u8, 0u8,
        ];
        converter.convert(&rgb);
        assert_eq!(converter.y(), [81u8, 81u8, 81u8, 81u8, 81u8, 81u8, 81u8, 81u8]);
        assert_eq!(converter.u(), [90u8, 90u8]);
        assert_eq!(converter.v(), [239u8, 239u8]);
        assert_eq!(converter.y_stride(), 4);
        assert_eq!(converter.u_stride(), 2);
        assert_eq!(converter.v_stride(), 2);
    }
}
