//! Converts YUV / RGB images to NAL packets.

use crate::error::NativeErrorExt;
use crate::formats::YUVSource;
use crate::{Error, OpenH264API, Timestamp};
use openh264_sys2::{
    videoFormatI420, EVideoFormatType, ISVCEncoder, ISVCEncoderVtbl, SEncParamBase, SEncParamExt, SFrameBSInfo, SLayerBSInfo,
    SSourcePicture, API, ENCODER_OPTION, ENCODER_OPTION_DATAFORMAT, ENCODER_OPTION_SVC_ENCODE_PARAM_EXT,
    ENCODER_OPTION_TRACE_LEVEL, RC_MODES, VIDEO_CODING_LAYER, WELS_LOG_DETAIL, WELS_LOG_QUIET,
};
use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr::{addr_of_mut, null, null_mut};

/// Convenience wrapper with guaranteed function pointers for easy access.
///
/// This struct automatically handles `WelsCreateSVCEncoder` and `WelsDestroySVCEncoder`.
#[rustfmt::skip]
#[allow(non_snake_case)]
pub struct EncoderRawAPI {
    api: OpenH264API,
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
    fn new(api: OpenH264API) -> Result<Self, Error> {
        unsafe {
            let mut encoder_ptr = null::<ISVCEncoderVtbl>() as *mut *const ISVCEncoderVtbl;

            api.WelsCreateSVCEncoder(&mut encoder_ptr as *mut *mut *const ISVCEncoderVtbl).ok()?;

            let e = || {
                Error::msg("VTable missing function.")
            };

            Ok(Self {
                api,
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
            self.api.WelsDestroySVCEncoder(self.encoder_ptr);
        }
    }
}

unsafe impl Send for EncoderRawAPI {}
unsafe impl Sync for EncoderRawAPI {}

/// Specifies the mode used by the encoder to control the rate.
#[derive(Copy, Clone, Debug)]
pub enum RateControlMode {
    /// Quality mode.
    Quality,
    /// Bitrate mode.
    Bitrate,
    /// No bitrate control, only using buffer status, adjust the video quality.
    Bufferbased,
    /// Rate control based timestamp.
    Timestamp,
    /// This is in-building RC MODE, WILL BE DELETED after algorithm tuning!
    BitrateModePostSkip,
    /// Rate control off mode.
    Off,
}

impl Default for RateControlMode {
    fn default() -> Self {
        Self::Quality
    }
}

impl RateControlMode {
    fn to_c(self) -> RC_MODES {
        match self {
            RateControlMode::Quality => openh264_sys2::RC_QUALITY_MODE,
            RateControlMode::Bitrate => openh264_sys2::RC_BITRATE_MODE,
            RateControlMode::Bufferbased => openh264_sys2::RC_BUFFERBASED_MODE,
            RateControlMode::Timestamp => openh264_sys2::RC_TIMESTAMP_MODE,
            RateControlMode::BitrateModePostSkip => openh264_sys2::RC_BITRATE_MODE_POST_SKIP,
            RateControlMode::Off => openh264_sys2::RC_OFF_MODE,
        }
    }
}

/// Configuration for the [`Encoder`].
///
/// Setting missing? Please file a PR!
#[derive(Default, Copy, Clone, Debug)]
pub struct EncoderConfig {
    enable_skip_frame: bool,
    target_bitrate: u32,
    enable_denoise: bool,
    debug: i32,
    data_format: EVideoFormatType,
    max_frame_rate: f32,
    rate_control_mode: RateControlMode,
}

impl EncoderConfig {
    /// Creates a new default encoder config.
    pub fn new() -> Self {
        Self {
            enable_skip_frame: true,
            target_bitrate: 120_000,
            enable_denoise: false,
            debug: 0,
            data_format: videoFormatI420,
            max_frame_rate: 0.0,
            rate_control_mode: Default::default(),
        }
    }

    /// Sets the requested bit rate in bits per second.
    pub fn set_bitrate_bps(mut self, bps: u32) -> Self {
        self.target_bitrate = bps;
        self
    }

    /// Enables detailed console logging inside OpenH264.
    pub fn debug(mut self, value: bool) -> Self {
        self.debug = if value { WELS_LOG_DETAIL } else { WELS_LOG_QUIET };
        self
    }

    /// Set whether frames can be skipped to meet desired rate control target.
    pub fn enable_skip_frame(mut self, value: bool) -> Self {
        self.enable_skip_frame = value;
        self
    }

    /// Sets the requested maximum frame rate in Hz.
    pub fn max_frame_rate(mut self, value: f32) -> Self {
        self.max_frame_rate = value;
        self
    }

    /// Sets the requested rate control mode.
    pub fn rate_control_mode(mut self, value: RateControlMode) -> Self {
        self.rate_control_mode = value;
        self
    }
}

/// An [OpenH264](https://github.com/cisco/openh264) encoder.
pub struct Encoder {
    config: EncoderConfig,
    raw_api: EncoderRawAPI,
    bit_stream_info: SFrameBSInfo,
    previous_dimensions: Option<(i32, i32)>,
}

unsafe impl Send for Encoder {}
unsafe impl Sync for Encoder {}

impl Encoder {
    /// Create an encoder with the provided configuration.
    ///
    /// The width and height will be taken from the [`YUVSource`] when calling [`Encoder::encode()`].
    pub fn with_config(api: OpenH264API, config: EncoderConfig) -> Result<Self, Error> {
        let raw_api = EncoderRawAPI::new(api)?;

        Ok(Self {
            config,
            raw_api,
            bit_stream_info: Default::default(),
            previous_dimensions: None,
        })
    }

    /// Encodes a YUV source and returns the encoded bitstream.
    ///
    /// The returned bitstream consists of one or more NAL units or packets. The first packets contain
    /// initialization information. Subsequent packages then contain, amongst others, keyframes
    /// ("I frames") or delta frames. The interval at which they are produced depends on the encoder settings.
    ///
    /// The resolution of the encoded frame is allowed to change. Each time it changes, the
    /// encoder is re-initialized with the new values.
    ///
    /// # Panics
    ///
    /// Panics if the provided timestamp as milliseconds is out of range of i64.
    pub fn encode<T: YUVSource>(&mut self, yuv_source: &T) -> Result<EncodedBitStream<'_>, Error> {
        self.encode_at(yuv_source, Timestamp::ZERO)
    }

    /// Encodes a YUV source and returns the encoded bitstream.
    ///
    /// The returned bitstream consists of one or more NAL units or packets. The first packets contain
    /// initialization information. Subsequent packages then contain, amongst others, keyframes
    /// ("I frames") or delta frames. The interval at which they are produced depends on the encoder settings.
    ///
    /// The resolution of the encoded frame is allowed to change. Each time it changes, the
    /// encoder is re-initialized with the new values.
    ///
    /// # Panics
    ///
    /// Panics if the provided timestamp as milliseconds is out of range of i64.
    pub fn encode_at<T: YUVSource>(&mut self, yuv_source: &T, timestamp: Timestamp) -> Result<EncodedBitStream<'_>, Error> {
        let new_dimensions = (yuv_source.width(), yuv_source.height());

        if self.previous_dimensions != Some(new_dimensions) {
            self.reinit(new_dimensions.0, new_dimensions.1)?;
            self.previous_dimensions = Some(new_dimensions);
        }

        // Converting *const u8 to *mut u8 should be fine because the encoder _should_
        // only read these arrays (TODO: needs verification).
        let source = SSourcePicture {
            iColorFormat: videoFormatI420,
            iStride: [yuv_source.y_stride(), yuv_source.u_stride(), yuv_source.v_stride(), 0],
            pData: [
                yuv_source.y().as_ptr() as *mut c_uchar,
                yuv_source.u().as_ptr() as *mut c_uchar,
                yuv_source.v().as_ptr() as *mut c_uchar,
                null_mut(),
            ],
            iPicWidth: new_dimensions.0,
            iPicHeight: new_dimensions.1,
            uiTimeStamp: timestamp.as_native(),
        };

        unsafe {
            self.raw_api.encode_frame(&source, &mut self.bit_stream_info).ok()?;

            Ok(EncodedBitStream {
                bit_stream_info: &self.bit_stream_info,
            })
        }
    }

    fn reinit(&mut self, width: i32, height: i32) -> Result<(), Error> {
        let mut params = SEncParamExt::default();

        unsafe { self.raw_api.get_default_params(&mut params).ok()? };

        params.iPicWidth = width as c_int;
        params.iPicHeight = height as c_int;
        params.iRCMode = self.config.rate_control_mode.to_c();
        params.bEnableFrameSkip = self.config.enable_skip_frame;
        params.iTargetBitrate = self.config.target_bitrate as c_int;
        params.bEnableDenoise = self.config.enable_denoise;
        params.fMaxFrameRate = self.config.max_frame_rate;

        if self.previous_dimensions.is_none() {
            // First time we call initialize_ext
            unsafe {
                self.raw_api.initialize_ext(&params).ok()?;
                self.raw_api
                    .set_option(ENCODER_OPTION_TRACE_LEVEL, addr_of_mut!(self.config.debug).cast())
                    .ok()?;
                self.raw_api
                    .set_option(ENCODER_OPTION_DATAFORMAT, addr_of_mut!(self.config.data_format).cast())
                    .ok()?;
            };
        } else {
            // Subsequent times we call SetOption
            unsafe {
                self.raw_api
                    .set_option(ENCODER_OPTION_SVC_ENCODE_PARAM_EXT, addr_of_mut!(params).cast())
                    .ok()?;

                // Start with a new keyframe.
                self.raw_api.force_intra_frame(true);
            }
        }

        Ok(())
    }

    /// Obtain the raw API for advanced use cases.
    ///
    /// When resorting to this call, please consider filing an issue / PR.
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

/// Bitstream output resulting from an [encode()](Encoder::encode) operation.
pub struct EncodedBitStream<'a> {
    /// Holds the bitstream info just encoded.
    bit_stream_info: &'a SFrameBSInfo,
}

impl<'a> EncodedBitStream<'a> {
    /// Raw bitstream info returned by the encoder.
    pub fn raw_info(&self) -> &'a SFrameBSInfo {
        self.bit_stream_info
    }

    /// Frame type of the encoded packet.
    pub fn frame_type(&self) -> FrameType {
        FrameType::from_c_int(self.bit_stream_info.eFrameType)
    }

    /// Number of layers in the encoded packet.
    pub fn num_layers(&self) -> usize {
        self.bit_stream_info.iLayerNum as usize
    }

    /// Returns ith layer of this bitstream.
    pub fn layer(&self, i: usize) -> Option<Layer<'a>> {
        if i < self.num_layers() {
            Some(Layer {
                layer_info: &self.bit_stream_info.sLayerInfo[i],
            })
        } else {
            None
        }
    }

    /// Writes the current bitstream into the given Vec.
    pub fn write_vec(&self, dst: &mut Vec<u8>) {
        for l in 0..self.num_layers() {
            let layer = self.layer(l).unwrap();

            for n in 0..layer.nal_count() {
                let nal = layer.nal_unit(n).unwrap();

                dst.extend_from_slice(nal)
            }
        }
    }

    /// Writes the current bitstream into the given Writer.
    pub fn write<T: std::io::Write>(&self, writer: &mut T) -> Result<(), Error> {
        for l in 0..self.num_layers() {
            let layer = self.layer(l).unwrap();

            for n in 0..layer.nal_count() {
                let nal = layer.nal_unit(n).unwrap();

                match writer.write(nal) {
                    Ok(num) if num < nal.len() => {
                        return Err(Error::msg(&format!("only wrote {} out of {} bytes", num, nal.len())));
                    }
                    Err(e) => {
                        return Err(Error::msg(&format!("failed to write: {}", e)));
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }

    /// Convenience method returning a Vec containing the encoded bitstream.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut rval = Vec::new();
        self.write_vec(&mut rval);
        rval
    }
}

/// An encoded layer, contains the Network Abstraction Layer inputs.
#[derive(Debug)]
pub struct Layer<'a> {
    /// Native layer info.
    layer_info: &'a SLayerBSInfo,
}

impl<'a> Layer<'a> {
    /// Raw layer info contained in a bitstream.
    pub fn raw_info(&self) -> &'a SLayerBSInfo {
        self.layer_info
    }

    /// NAL count of this layer.
    pub fn nal_count(&self) -> usize {
        self.layer_info.iNalCount as usize
    }

    /// Returns NAL unit data for the ith element.
    pub fn nal_unit(&self, i: usize) -> Option<&[u8]> {
        if i < self.nal_count() {
            let mut offset = 0;

            let slice = unsafe {
                // Fast forward through all NALs we didn't request
                // TODO: We can probably do this math a bit more efficiently, not counting up all the time.
                // pNalLengthInByte is a c_int C array containing the nal unit sizes
                for nal_idx in 0..i {
                    let size = *self.layer_info.pNalLengthInByte.add(nal_idx) as usize;
                    offset += size;
                }

                let size = *self.layer_info.pNalLengthInByte.add(i) as usize;
                std::slice::from_raw_parts(self.layer_info.pBsBuf.add(offset), size)
            };

            Some(slice)
        } else {
            None
        }
    }

    /// If this is a video layer or not.
    pub fn is_video(&self) -> bool {
        self.layer_info.uiLayerType == VIDEO_CODING_LAYER as c_uchar
    }
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
        use openh264_sys2::{videoFrameTypeI, videoFrameTypeIDR, videoFrameTypeIPMixed, videoFrameTypeP, videoFrameTypeSkip};

        #[allow(non_upper_case_globals)]
        match native {
            videoFrameTypeIDR => Self::IDR,
            videoFrameTypeI => Self::I,
            videoFrameTypeP => Self::P,
            videoFrameTypeSkip => Self::Skip,
            videoFrameTypeIPMixed => Self::IPMixed,
            _ => Self::Invalid,
        }
    }
}
