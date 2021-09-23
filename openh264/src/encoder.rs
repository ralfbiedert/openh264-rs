//! Converts YUV / RGB images to NAL packets.

use crate::error::NativeErrorExt;
use crate::formats::YUVSource;
use crate::Error;
use openh264_sys2::{
    videoFormatI420, EVideoFormatType, ISVCEncoder, ISVCEncoderVtbl, SEncParamBase, SEncParamExt, SFrameBSInfo, SSourcePicture, WelsCreateSVCEncoder, WelsDestroySVCEncoder, ENCODER_OPTION, ENCODER_OPTION_DATAFORMAT, ENCODER_OPTION_TRACE_LEVEL, VIDEO_CODING_LAYER, WELS_LOG_DETAIL, WELS_LOG_QUIET
};
use smallvec::SmallVec;
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

    /// Sets the requested bit rate in bits per second.
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

    /// Encodes a YUV source and returns the encoded bitstream.
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

            let num_layers = bit_stream_info.iLayerNum as usize;
            let layers: SmallVec<[Layer; 4]> = bit_stream_info.sLayerInfo[0..num_layers]
                .iter()
                .map(|layer| {
                    let mut offset = 0;
                    let mut nal_units = SmallVec::<[&[u8]; 4]>::new();
                    for nal_idx in 0..layer.iNalCount {
                        // pNalLengthInByte is a c_int C array containing the nal unit sizes
                        let size = *layer.pNalLengthInByte.offset(nal_idx as isize) as usize;
                        let nal_unit = std::slice::from_raw_parts(layer.pBsBuf.offset(offset as isize), size);
                        nal_units.push(nal_unit);
                        offset += size;
                    }
                    Layer {
                        nal_units,
                        is_video: layer.uiLayerType == VIDEO_CODING_LAYER as u8,
                    }
                })
                .collect();

            Ok(EncodedBitStream {
                layers,
                frame_type: FrameType::from_c_int(bit_stream_info.eFrameType),
            })
        }
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

/// A Encoded Layer, contains the Network Abstraction Layer inputs
pub struct Layer<'a> {
    /// Network Abstraction Layer Units for a given layer
    pub nal_units: SmallVec<[&'a [u8]; 4]>,
    /// Set to true if the layer contains video data, false otherwise
    pub is_video: bool,
}

/// Encoding output, currently takes only the first video layer.
pub struct EncodedBitStream<'a> {
    /// Obtains the bitstream as a byte slice.
    pub layers: SmallVec<[Layer<'a>; 4]>,
    /// What this bitstream encodes.
    pub frame_type: FrameType,
}

/// Frame type returned by the encoder.
///
/// The variant documentation was directly taken from OpenH264 project.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum FrameType {
    /// Encoder not ready or parameters are invalidate.
    Invalid,
    /// IDR frame in H.264
    IDR,
    /// I frame type
    I,
    /// P frame type
    P,
    /// Skip the frame based encoder kernel"
    Skip,
    /// A frame where I and P slices are mixing, not supported yet.
    IPMixed,
}

impl FrameType {
    fn from_c_int(native: std::os::raw::c_int) -> Self {
        use openh264_sys2::{
            videoFrameTypeI, videoFrameTypeIDR, videoFrameTypeIPMixed, videoFrameTypeInvalid, videoFrameTypeP, videoFrameTypeSkip
        };

        #[allow(non_upper_case_globals)]
        match native {
            videoFrameTypeInvalid => Self::Invalid,
            videoFrameTypeIDR => Self::IDR,
            videoFrameTypeI => Self::I,
            videoFrameTypeP => Self::P,
            videoFrameTypeSkip => Self::Skip,
            videoFrameTypeIPMixed => Self::IPMixed,
            _ => Self::Invalid,
        }
    }
}
