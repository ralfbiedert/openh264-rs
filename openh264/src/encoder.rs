//! Converts images to packets.

use crate::error::NativeErrorExt;
use crate::Error;
use openh264_sys2::{
    videoFormatI420, EVideoFormatType, ISVCEncoder, ISVCEncoderVtbl, SEncParamBase, SEncParamExt, SFrameBSInfo, SSourcePicture, WelsCreateSVCEncoder, WelsDestroySVCEncoder, ENCODER_OPTION, ENCODER_OPTION_DATAFORMAT, ENCODER_OPTION_TRACE_LEVEL, WELS_LOG_DETAIL, WELS_LOG_QUIET
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
    params: SEncParamExt,
    debug: ENCODER_OPTION,
    data_format: EVideoFormatType,
}

impl EncoderConfig {
    /// Creates a new default encoder config.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            params: SEncParamExt {
                iPicWidth: width as c_int,
                iPicHeight: height as c_int,
                bEnableFrameSkip: true,
                iTargetBitrate: 120_000,
                bEnableDenoise: false,
                ..Default::default()
            },
            debug: 0,
            data_format: videoFormatI420,
        }
    }

    pub fn set_bitrate_bps(mut self, bps: u32) -> Self {
        self.params.iTargetBitrate = bps as c_int;
        self
    }

    pub fn debug(mut self, value: bool) -> Self {
        self.debug = if value { WELS_LOG_DETAIL } else { WELS_LOG_QUIET };
        self
    }
}

/// An [OpenH264](https://github.com/cisco/openh264) encoder.
#[derive(Debug)]
pub struct Encoder {
    raw_api: EncoderRawAPI,
}

impl Encoder {
    /// Create an encoder with the provided configuration.
    pub fn with_config(mut config: EncoderConfig) -> Result<Self, Error> {
        let raw = EncoderRawAPI::new()?;

        #[rustfmt::skip]
        unsafe {
            raw.initialize_ext(&config.params).ok()?;
            raw.set_option(ENCODER_OPTION_TRACE_LEVEL, addr_of_mut!(config.debug).cast()).ok()?;
            raw.set_option(ENCODER_OPTION_DATAFORMAT, addr_of_mut!(config.data_format).cast()).ok()?;
        };

        Ok(Self { raw_api: raw })
    }

    pub fn encode_todo(&mut self, y: &[u8], u: &[u8], v: &[u8], sy: usize, su: usize, sv: usize) -> Result<(), Error> {
        unsafe {
            let mut source = SSourcePicture::default();

            // Converting *const u8 to *mut u8 should be fine because the encoder _should_
            // only read these arrays (TOOD: needs verification).
            source.pData[0] = y.as_ptr() as *mut c_uchar;
            source.pData[1] = u.as_ptr() as *mut c_uchar;
            source.pData[2] = v.as_ptr() as *mut c_uchar;
            source.iStride[0] = su as c_int;
            source.iStride[1] = sy as c_int;
            source.iStride[2] = sv as c_int;

            let mut frame_info = SFrameBSInfo::default();

            self.raw_api.encode_frame(&source, &mut frame_info).ok()?;
        }
        Ok(())
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
