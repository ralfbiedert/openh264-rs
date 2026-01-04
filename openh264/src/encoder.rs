//! Converts YUV / RGB images to NAL packets.

use crate::error::NativeErrorExt;
use crate::formats::YUVSource;
use crate::{Error, OpenH264API, Timestamp};
use openh264_sys2::{
    API, DEBLOCKING_IDC_0, ELevelIdc, ENCODER_OPTION, ENCODER_OPTION_DATAFORMAT, ENCODER_OPTION_SVC_ENCODE_PARAM_EXT,
    ENCODER_OPTION_TRACE_LEVEL, EProfileIdc, EUsageType, EVideoFormatType, ISVCEncoder, ISVCEncoderVtbl, RC_MODES, SEncParamBase,
    SEncParamExt, SFrameBSInfo, SLayerBSInfo, SM_SINGLE_SLICE, SM_SIZELIMITED_SLICE, SSourcePicture, VIDEO_CODING_LAYER,
    WELS_LOG_DETAIL, WELS_LOG_QUIET, videoFormatI420,
};
use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr::{addr_of_mut, from_mut, null, null_mut};

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
#[allow(clippy::must_use_candidate)]
#[allow(non_snake_case, unused, missing_docs)]
impl EncoderRawAPI {
    fn new(api: OpenH264API) -> Result<Self, Error> {
        unsafe {
            let mut encoder_ptr = null::<ISVCEncoderVtbl>() as *mut *const ISVCEncoderVtbl;

            api.WelsCreateSVCEncoder(from_mut(&mut encoder_ptr)).ok()?;

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
    unsafe fn uninitialize(&self) -> c_int { unsafe { (self.uninitialize)(self.encoder_ptr) }}
    unsafe fn initialize(&self, pParam: *const SEncParamBase) -> c_int { unsafe { (self.initialize)(self.encoder_ptr, pParam) }}
    unsafe fn initialize_ext(&self, pParam: *const SEncParamExt) -> c_int { unsafe { (self.initialize_ext)(self.encoder_ptr, pParam) }}

    pub unsafe fn get_default_params(&self, pParam: *mut SEncParamExt) -> c_int { unsafe { (self.get_default_params)(self.encoder_ptr, pParam) }}
    pub unsafe fn encode_frame(&self, kpSrcPic: *const SSourcePicture, pBsInfo: *mut SFrameBSInfo) -> c_int { unsafe { (self.encode_frame)(self.encoder_ptr, kpSrcPic, pBsInfo) }}
    pub unsafe fn encode_parameter_sets(&self, pBsInfo: *mut SFrameBSInfo) -> c_int { unsafe { (self.encode_parameter_sets)(self.encoder_ptr, pBsInfo) }}
    pub unsafe fn force_intra_frame(&self, bIDR: bool) -> c_int { unsafe { (self.force_intra_frame)(self.encoder_ptr, bIDR) }}
    pub unsafe fn set_option(&self, eOptionId: ENCODER_OPTION, pOption: *mut c_void) -> c_int { unsafe { (self.set_option)(self.encoder_ptr, eOptionId, pOption) }}
    pub unsafe fn get_option(&self, eOptionId: ENCODER_OPTION, pOption: *mut c_void) -> c_int { unsafe { (self.get_option)(self.encoder_ptr, eOptionId, pOption) }}
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
#[derive(Copy, Clone, Debug, Default)]
pub enum RateControlMode {
    /// Quality mode.
    #[default]
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

impl RateControlMode {
    const fn to_c(self) -> RC_MODES {
        match self {
            Self::Quality => openh264_sys2::RC_QUALITY_MODE,
            Self::Bitrate => openh264_sys2::RC_BITRATE_MODE,
            Self::Bufferbased => openh264_sys2::RC_BUFFERBASED_MODE,
            Self::Timestamp => openh264_sys2::RC_TIMESTAMP_MODE,
            Self::BitrateModePostSkip => openh264_sys2::RC_BITRATE_MODE_POST_SKIP,
            Self::Off => openh264_sys2::RC_OFF_MODE,
        }
    }
}

/// Sets the behavior for generating SPS/PPS.
#[derive(Copy, Clone, Debug, Default)]
pub enum SpsPpsStrategy {
    /// Use a constant SPS/PPS ID. The ID will not change across encoded video frames.
    ///
    /// This is the default value.
    #[default]
    ConstantId,

    /// Increment the SPS/PPS ID with each IDR frame.
    ///
    /// This allows decoders to detect missing frames.
    IncreasingId,

    /// Use SPS in the existing list if possible.
    SpsListing,

    /// _find doc for this_
    SpsListingAndPpsIncreasing,

    /// _find doc for this_
    SpsPpsListing,
}

impl SpsPpsStrategy {
    const fn to_c(self) -> RC_MODES {
        match self {
            Self::ConstantId => openh264_sys2::CONSTANT_ID,
            Self::IncreasingId => openh264_sys2::INCREASING_ID,
            Self::SpsListing => openh264_sys2::SPS_LISTING,
            Self::SpsListingAndPpsIncreasing => openh264_sys2::SPS_LISTING_AND_PPS_INCREASING,
            Self::SpsPpsListing => openh264_sys2::SPS_PPS_LISTING,
        }
    }
}

/// The intended usage scenario for the encoder.
///
/// Note, this documen
#[derive(Copy, Clone, Debug, Default)]
pub enum UsageType {
    /// Camera video for real-time communication.
    #[default]
    CameraVideoRealTime,
    /// Used for real-time screen sharing.
    ScreenContentRealTime,
    /// Camera video for non-real-time communication.
    CameraVideoNonRealTime,
    /// Used for non-real-time screen recordings.
    ScreenContentNonRealTime,
    /// It's unclear what this does, PRs adding documentation welcome.
    InputContentTypeAll,
}

impl UsageType {
    const fn to_c(self) -> EUsageType {
        match self {
            Self::CameraVideoRealTime => openh264_sys2::CAMERA_VIDEO_REAL_TIME,
            Self::ScreenContentRealTime => openh264_sys2::SCREEN_CONTENT_REAL_TIME,
            Self::CameraVideoNonRealTime => openh264_sys2::CAMERA_VIDEO_NON_REAL_TIME,
            Self::ScreenContentNonRealTime => openh264_sys2::SCREEN_CONTENT_NON_REAL_TIME,
            Self::InputContentTypeAll => openh264_sys2::INPUT_CONTENT_TYPE_ALL,
        }
    }
}

/// Bitrate of the encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BitRate(u32);

impl BitRate {
    /// Creates a new bitrate with the given bits per second.
    #[must_use]
    pub const fn from_bps(bps: u32) -> Self {
        Self(bps)
    }
}

/// Frame rate of the encoder.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct FrameRate(f32);

impl FrameRate {
    /// Creates a new framerate with the given Hertz.
    #[must_use]
    pub const fn from_hz(hz: f32) -> Self {
        Self(hz)
    }
}

/// The H.264 encoding profile
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub enum Profile {
    Baseline,
    Main,
    Extended,
    High,
    High10,
    High422,
    High444,
    CAVLC444,
    ScalableBaseline,
    ScalableHigh,
}

impl Profile {
    const fn to_c(self) -> EProfileIdc {
        match self {
            Self::Baseline => openh264_sys2::PRO_BASELINE,
            Self::Main => openh264_sys2::PRO_MAIN,
            Self::Extended => openh264_sys2::PRO_EXTENDED,
            Self::High => openh264_sys2::PRO_HIGH,
            Self::High10 => openh264_sys2::PRO_HIGH10,
            Self::High422 => openh264_sys2::PRO_HIGH422,
            Self::High444 => openh264_sys2::PRO_HIGH444,
            Self::CAVLC444 => openh264_sys2::PRO_CAVLC444,
            Self::ScalableBaseline => openh264_sys2::PRO_SCALABLE_BASELINE,
            Self::ScalableHigh => openh264_sys2::PRO_SCALABLE_HIGH,
        }
    }
}

/// H.264 encoding levels with their corresponding capabilities.
///
/// | Level   | Max Resolution (Pixels) | Max Frame Rate (fps) | Max Bitrate (Main Profile) | Max Bitrate (High Profile) |
/// |---------|--------------------------|-----------------------|-----------------------------|-----------------------------|
/// | 1.0     | 176x144 (QCIF)          | 15                   | 64 kbps                    | 80 kbps                    |
/// | 1.1     | 176x144 (QCIF)          | 30                   | 192 kbps                   | 240 kbps                   |
/// | 1.2     | 320x240 (QVGA)          | 30                   | 384 kbps                   | 480 kbps                   |
/// | 2.0     | 352x288 (CIF)           | 30                   | 2 Mbps                     | 2.5 Mbps                   |
/// | 3.0     | 720x576 (SD)            | 30                   | 10 Mbps                    | 12.5 Mbps                  |
/// | 3.1     | 1280x720 (HD)           | 30                   | 14 Mbps                    | 17.5 Mbps                  |
/// | 4.0     | 1920x1080 (Full HD)     | 30                   | 20 Mbps                    | 25 Mbps                    |
/// | 4.1     | 1920x1080 (Full HD)     | 60                   | 50 Mbps                    | 62.5 Mbps                  |
/// | 5.0     | 3840x2160 (4K)          | 30                   | 135 Mbps                   | 168.75 Mbps                |
/// | 5.1     | 3840x2160 (4K)          | 60                   | 240 Mbps                   | 300 Mbps                   |
/// | 5.2     | 4096x2160 (4K Cinema)   | 60                   | 480 Mbps                   | 600 Mbps                   |
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs, non_camel_case_types)]
pub enum Level {
    /// Level 1.0: Max resolution 176x144 (QCIF), 15 fps, 64 kbps (Main), 80 kbps (High)
    Level_1_0,
    /// Level 1.B: Specialized low-complexity baseline level.
    Level_1_B,
    /// Level 1.1: Max resolution 176x144 (QCIF), 30 fps, 192 kbps (Main), 240 kbps (High)
    Level_1_1,
    /// Level 1.2: Max resolution 320x240 (QVGA), 30 fps, 384 kbps (Main), 480 kbps (High)
    Level_1_2,
    /// Level 1.3: Reserved in standard, similar to Level 2.0.
    Level_1_3,
    /// Level 2.0: Max resolution 352x288 (CIF), 30 fps, 2 Mbps (Main), 2.5 Mbps (High)
    Level_2_0,
    /// Level 2.1: Max resolution 352x288 (CIF), 30 fps, 4 Mbps (Main), 5 Mbps (High)
    Level_2_1,
    /// Level 2.2: Max resolution 352x288 (CIF), 30 fps, 10 Mbps (Main), 12.5 Mbps (High)
    Level_2_2,
    /// Level 3.0: Max resolution 720x576 (SD), 30 fps, 10 Mbps (Main), 12.5 Mbps (High)
    Level_3_0,
    /// Level 3.1: Max resolution 1280x720 (HD), 30 fps, 14 Mbps (Main), 17.5 Mbps (High)
    Level_3_1,
    /// Level 3.2: Max resolution 1280x720 (HD), 60 fps, 20 Mbps (Main), 25 Mbps (High)
    Level_3_2,
    /// Level 4.0: Max resolution 1920x1080 (Full HD), 30 fps, 20 Mbps (Main), 25 Mbps (High)
    Level_4_0,
    /// Level 4.1: Max resolution 1920x1080 (Full HD), 60 fps, 50 Mbps (Main), 62.5 Mbps (High)
    Level_4_1,
    /// Level 4.2: Max resolution 1920x1080 (Full HD), 120 fps, 100 Mbps (Main), 125 Mbps (High)
    Level_4_2,
    /// Level 5.0: Max resolution 3840x2160 (4K), 30 fps, 135 Mbps (Main), 168.75 Mbps (High)
    Level_5_0,
    /// Level 5.1: Max resolution 3840x2160 (4K), 60 fps, 240 Mbps (Main), 300 Mbps (High)
    Level_5_1,
    /// Level 5.2: Max resolution 4096x2160 (4K Cinema), 60 fps, 480 Mbps (Main), 600 Mbps (High)
    Level_5_2,
}

impl Level {
    const fn to_c(self) -> ELevelIdc {
        match self {
            Self::Level_1_0 => openh264_sys2::LEVEL_1_0,
            Self::Level_1_B => openh264_sys2::LEVEL_1_B,
            Self::Level_1_1 => openh264_sys2::LEVEL_1_1,
            Self::Level_1_2 => openh264_sys2::LEVEL_1_2,
            Self::Level_1_3 => openh264_sys2::LEVEL_1_3,
            Self::Level_2_0 => openh264_sys2::LEVEL_2_0,
            Self::Level_2_1 => openh264_sys2::LEVEL_2_1,
            Self::Level_2_2 => openh264_sys2::LEVEL_2_2,
            Self::Level_3_0 => openh264_sys2::LEVEL_3_0,
            Self::Level_3_1 => openh264_sys2::LEVEL_3_1,
            Self::Level_3_2 => openh264_sys2::LEVEL_3_2,
            Self::Level_4_0 => openh264_sys2::LEVEL_4_0,
            Self::Level_4_1 => openh264_sys2::LEVEL_4_1,
            Self::Level_4_2 => openh264_sys2::LEVEL_4_2,
            Self::Level_5_0 => openh264_sys2::LEVEL_5_0,
            Self::Level_5_1 => openh264_sys2::LEVEL_5_1,
            Self::Level_5_2 => openh264_sys2::LEVEL_5_2,
        }
    }
}

/// Complexity of the encoder (speed vs. quality).
#[derive(Debug, Default, Clone, Copy)]
#[allow(missing_docs)]
pub enum Complexity {
    /// The lowest complexity, the fastest speed.
    Low,
    /// Medium complexity, medium speed, medium quality.
    #[default]
    Medium,
    /// High complexity, lowest speed, high quality.
    High,
}

impl Complexity {
    const fn to_c(self) -> ELevelIdc {
        match self {
            Self::Low => openh264_sys2::LOW_COMPLEXITY,
            Self::Medium => openh264_sys2::MEDIUM_COMPLEXITY,
            Self::High => openh264_sys2::HIGH_COMPLEXITY,
        }
    }
}

/// Quantization parameter range to control the degree of compression.
///
/// This can be used to control the balance between size and video quality.
#[derive(Debug, Clone, Copy)]
pub struct QpRange {
    min: u8,
    max: u8,
}

impl QpRange {
    /// Limit the quantization of the encoder to the given range.
    ///
    /// Valid values for `min` and `max` are between 0 and 51, where 0
    /// represents highest quality and 51 the strongest compression.
    ///
    /// # Panics
    ///
    /// Panics if `max > 51` or if `min > max`.
    #[must_use]
    pub const fn new(min: u8, max: u8) -> Self {
        assert!(max <= 51, "quantization value out of range (0..=51)");
        assert!(min <= max, "quantization min value larger than max");

        Self { min, max }
    }
}

impl Default for QpRange {
    fn default() -> Self {
        Self { min: 0, max: 51 }
    }
}

/// A period in frames after which a new I-Frame is generated.
#[derive(Debug, Clone, Copy, Default)]
pub struct IntraFramePeriod(u32);

impl IntraFramePeriod {
    /// Creates a period in which I-Frames (group of pictures, "GOP size") are generated.
    ///
    /// Using lower values improves error resilience and allows for faster seeking within the video,
    /// but increases the overall required bitrate.
    ///
    /// Setting the value to zero is equal to calling [`IntraFramePeriod::auto()`].
    #[must_use]
    pub const fn from_num_frames(frames: u32) -> Self {
        Self(frames)
    }

    /// Lets the encoder create I-frames as desired(?).
    #[must_use]
    pub const fn auto() -> Self {
        Self(0)
    }
}

// =============================================================================
// VUI (Video Usability Information) Parameters
// =============================================================================
// These parameters are embedded in the H.264 SPS to signal color space
// information to decoders. See ITU-T H.264 Annex E for details.

/// H.264 colour_primaries values (ITU-T H.264 Table E-3).
///
/// Specifies the chromaticity coordinates of the source primaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ColorPrimaries {
    /// ITU-R BT.709-6 / sRGB / IEC 61966-2-1 (HD television, sRGB displays)
    #[default]
    Bt709 = 1,
    /// Unspecified - decoder determines based on context
    Unspecified = 2,
    /// ITU-R BT.470-6 System M (historical NTSC)
    Bt470M = 4,
    /// ITU-R BT.470-6 System B, G / ITU-R BT.601-7 625 (PAL)
    Bt470BG = 5,
    /// SMPTE 170M / ITU-R BT.601-7 525 (NTSC)
    Smpte170M = 6,
    /// SMPTE 240M (historical)
    Smpte240M = 7,
    /// Generic film (C illuminant)
    Film = 8,
    /// ITU-R BT.2020-2 / ITU-R BT.2100-2 (UHD/HDR)
    Bt2020 = 9,
}

impl ColorPrimaries {
    /// Get the raw u8 value for the VUI colour_primaries field.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// H.264 transfer_characteristics values (ITU-T H.264 Table E-4).
///
/// Specifies the opto-electronic transfer characteristic (gamma).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum TransferCharacteristics {
    /// ITU-R BT.709-6 / ITU-R BT.1361 (HD television)
    #[default]
    Bt709 = 1,
    /// Unspecified
    Unspecified = 2,
    /// ITU-R BT.470-6 System M (2.2 gamma)
    Bt470M = 4,
    /// ITU-R BT.470-6 System B, G (2.8 gamma)
    Bt470Bg = 5,
    /// SMPTE 170M / BT.601 (same curve as BT.709)
    Smpte170M = 6,
    /// SMPTE 240M
    Smpte240M = 7,
    /// Linear transfer (gamma 1.0)
    Linear = 8,
    /// IEC 61966-2-1 (sRGB) - recommended for computer graphics
    Srgb = 13,
    /// ITU-R BT.2020 10-bit (same curve as BT.709)
    Bt2020_10 = 14,
    /// ITU-R BT.2020 12-bit (same curve as BT.709)
    Bt2020_12 = 15,
    /// SMPTE ST 2084 (PQ / HDR10)
    Smpte2084 = 16,
    /// ARIB STD-B67 (HLG)
    Hlg = 18,
}

impl TransferCharacteristics {
    /// Get the raw u8 value for the VUI transfer_characteristics field.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// H.264 matrix_coefficients values (ITU-T H.264 Table E-5).
///
/// Specifies the matrix coefficients for deriving luma and chroma from RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum MatrixCoefficients {
    /// Identity (RGB, no matrix transformation)
    Identity = 0,
    /// ITU-R BT.709-6 (Kr=0.2126, Kb=0.0722) - HD television
    #[default]
    Bt709 = 1,
    /// Unspecified
    Unspecified = 2,
    /// FCC 73.682 (historical)
    Fcc = 4,
    /// ITU-R BT.470-6 System B, G (same as BT.601-7 625)
    Bt470Bg = 5,
    /// SMPTE 170M / ITU-R BT.601-7 525 (Kr=0.299, Kb=0.114) - SD television
    Smpte170M = 6,
    /// SMPTE 240M
    Smpte240M = 7,
    /// YCgCo (lossless)
    Ycgco = 8,
    /// ITU-R BT.2020 non-constant luminance
    Bt2020Ncl = 9,
    /// ITU-R BT.2020 constant luminance
    Bt2020Cl = 10,
}

impl MatrixCoefficients {
    /// Get the raw u8 value for the VUI matrix_coefficients field.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// H.264 VUI configuration for signaling color space to decoders.
///
/// This struct groups all VUI color-related fields together for convenience.
/// Use [`VuiConfig::bt709()`] or similar constructors for common presets.
///
/// # Example
///
/// ```
/// use openh264::encoder::{EncoderConfig, VuiConfig};
///
/// let config = EncoderConfig::new()
///     .vui(VuiConfig::bt709().full_range(true));  // HD BT.709 with full range
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[must_use]
pub struct VuiConfig {
    /// Chromaticity coordinates of source primaries
    color_primaries: ColorPrimaries,
    /// Transfer characteristics (gamma)
    transfer_characteristics: TransferCharacteristics,
    /// Matrix coefficients for RGB↔YCbCr conversion
    matrix_coefficients: MatrixCoefficients,
    /// True for full range (0-255), false for limited range (16-235)
    full_range: bool,
}

impl VuiConfig {
    /// Create a new VuiConfig with default values (BT.709 limited range).
    pub const fn new() -> Self {
        Self::bt709()
    }

    /// BT.709 with limited range (default for HD content).
    ///
    /// This is the standard for HD television and most video content.
    pub const fn bt709() -> Self {
        Self {
            color_primaries: ColorPrimaries::Bt709,
            transfer_characteristics: TransferCharacteristics::Bt709,
            matrix_coefficients: MatrixCoefficients::Bt709,
            full_range: false,
        }
    }

    /// BT.709 with full range (for PC/computer graphics content).
    pub const fn bt709_full() -> Self {
        Self {
            color_primaries: ColorPrimaries::Bt709,
            transfer_characteristics: TransferCharacteristics::Bt709,
            matrix_coefficients: MatrixCoefficients::Bt709,
            full_range: true,
        }
    }

    /// BT.601 (SMPTE 170M) with limited range (for SD content).
    pub const fn bt601() -> Self {
        Self {
            color_primaries: ColorPrimaries::Smpte170M,
            transfer_characteristics: TransferCharacteristics::Smpte170M,
            matrix_coefficients: MatrixCoefficients::Smpte170M,
            full_range: false,
        }
    }

    /// sRGB with full range (ideal for desktop/web content).
    ///
    /// Uses BT.709 primaries but with sRGB transfer function.
    pub const fn srgb() -> Self {
        Self {
            color_primaries: ColorPrimaries::Bt709,
            transfer_characteristics: TransferCharacteristics::Srgb,
            matrix_coefficients: MatrixCoefficients::Bt709,
            full_range: true,
        }
    }

    /// BT.2020 with limited range (for UHD/HDR content).
    pub const fn bt2020() -> Self {
        Self {
            color_primaries: ColorPrimaries::Bt2020,
            transfer_characteristics: TransferCharacteristics::Bt2020_10,
            matrix_coefficients: MatrixCoefficients::Bt2020Ncl,
            full_range: false,
        }
    }

    /// Set the color primaries.
    pub const fn color_primaries(mut self, value: ColorPrimaries) -> Self {
        self.color_primaries = value;
        self
    }

    /// Set the transfer characteristics.
    pub const fn transfer_characteristics(mut self, value: TransferCharacteristics) -> Self {
        self.transfer_characteristics = value;
        self
    }

    /// Set the matrix coefficients.
    pub const fn matrix_coefficients(mut self, value: MatrixCoefficients) -> Self {
        self.matrix_coefficients = value;
        self
    }

    /// Set full range mode.
    pub const fn full_range(mut self, value: bool) -> Self {
        self.full_range = value;
        self
    }
}

/// Configuration for the [`Encoder`].
///
/// Setting missing? Please file a PR!
#[derive(Default, Copy, Clone, Debug)]
#[must_use]
#[allow(clippy::struct_excessive_bools)]
pub struct EncoderConfig {
    enable_skip_frame: bool,
    target_bitrate: BitRate,
    enable_denoise: bool,
    debug: i32,
    data_format: EVideoFormatType,
    max_frame_rate: FrameRate,
    rate_control_mode: RateControlMode,
    sps_pps_strategy: SpsPpsStrategy,
    multiple_thread_idc: u16,
    usage_type: UsageType,
    max_slice_len: Option<u32>,
    profile: Option<Profile>,
    level: Option<Level>,
    complexity: Complexity,
    qp: QpRange,
    scene_change_detect: bool,
    adaptive_quantization: bool,
    background_detection: bool,
    long_term_reference: bool,
    intra_frame_period: IntraFramePeriod,
    vui: Option<VuiConfig>,
}

impl EncoderConfig {
    /// Creates a new default encoder config.
    pub const fn new() -> Self {
        Self {
            enable_skip_frame: true,
            target_bitrate: BitRate::from_bps(120_000),
            enable_denoise: false,
            debug: 0,
            data_format: videoFormatI420,
            max_frame_rate: FrameRate::from_hz(0.0),
            rate_control_mode: RateControlMode::Quality,
            sps_pps_strategy: SpsPpsStrategy::ConstantId,
            multiple_thread_idc: 0,
            usage_type: UsageType::CameraVideoRealTime,
            max_slice_len: None,
            profile: None,
            level: None,
            complexity: Complexity::Medium,
            qp: QpRange::new(0, 51),
            scene_change_detect: true,
            adaptive_quantization: true,
            background_detection: true,
            long_term_reference: false,
            intra_frame_period: IntraFramePeriod::from_num_frames(0),
            vui: None,
        }
    }

    /// Sets the requested bit rate in bits per second.
    pub const fn bitrate(mut self, bps: BitRate) -> Self {
        self.target_bitrate = bps;
        self
    }

    /// Enables detailed console logging inside OpenH264.
    pub const fn debug(mut self, value: bool) -> Self {
        self.debug = if value { WELS_LOG_DETAIL } else { WELS_LOG_QUIET };
        self
    }

    /// Set whether frames can be skipped to meet desired rate control target.
    pub const fn skip_frames(mut self, value: bool) -> Self {
        self.enable_skip_frame = value;
        self
    }

    /// Sets the requested maximum frame rate in Hz.
    pub const fn max_frame_rate(mut self, value: FrameRate) -> Self {
        self.max_frame_rate = value;
        self
    }

    /// Sets the usage type (e.g, screen vs. camera recording).
    pub const fn usage_type(mut self, value: UsageType) -> Self {
        self.usage_type = value;
        self
    }

    /// Sets the requested rate control mode.
    pub const fn rate_control_mode(mut self, value: RateControlMode) -> Self {
        self.rate_control_mode = value;
        self
    }

    /// Set the SPS/PPS behavior.
    pub const fn sps_pps_strategy(mut self, value: SpsPpsStrategy) -> Self {
        self.sps_pps_strategy = value;
        self
    }

    /// Set the maximum slice length
    pub const fn max_slice_len(mut self, max_slice_len: u32) -> Self {
        self.max_slice_len = Some(max_slice_len);
        self
    }

    /// Set the encoding profile
    pub const fn profile(mut self, profile: Profile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Set the encoding profile level
    pub const fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    /// Set the complexity
    pub const fn complexity(mut self, complexity: Complexity) -> Self {
        self.complexity = complexity;
        self
    }

    /// Set the balance between compression and size
    pub const fn qp(mut self, value: QpRange) -> Self {
        self.qp = value;
        self
    }

    /// Set scene change detect (on by default)
    pub const fn scene_change_detect(mut self, value: bool) -> Self {
        self.scene_change_detect = value;
        self
    }

    /// Set adaptive quantization control (on by default)
    pub const fn adaptive_quantization(mut self, value: bool) -> Self {
        self.adaptive_quantization = value;
        self
    }

    /// Set background detection (on by default)
    pub const fn background_detection(mut self, value: bool) -> Self {
        self.background_detection = value;
        self
    }

    /// Set use of long term reference (off by default)
    pub const fn long_term_reference(mut self, value: bool) -> Self {
        self.long_term_reference = value;
        self
    }

    /// Set the interval of intra frames (0 by default, disabling periodic intra frames)
    pub const fn intra_frame_period(mut self, value: IntraFramePeriod) -> Self {
        self.intra_frame_period = value;
        self
    }

    /// Sets the number of internal encoder threads.
    ///
    /// * 0 - auto mode
    /// * 1 - single threaded operation
    /// * &gt;1 - fixed number of threads
    ///
    /// Defaults to 0 (auto mode).
    pub const fn num_threads(mut self, threads: u16) -> Self {
        self.multiple_thread_idc = threads;
        self
    }

    /// Sets the VUI (Video Usability Information) parameters.
    ///
    /// VUI parameters are written into the H.264 SPS NAL unit and tell decoders
    /// how to interpret the color space of the video data. This is essential for
    /// correct color reproduction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openh264::encoder::{EncoderConfig, VuiConfig};
    ///
    /// let config = EncoderConfig::new()
    ///     .vui(VuiConfig::bt709());  // HD content with BT.709 color space
    /// ```
    ///
    /// See [`VuiConfig`] for common presets like `bt709()`, `srgb()`, and `bt601()`.
    pub const fn vui(mut self, config: VuiConfig) -> Self {
        self.vui = Some(config);
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
    /// Create an encoder with default settings.
    ///
    /// The width and height will be taken from the [`YUVSource`] when calling [`Encoder::encode()`].
    ///
    /// This method is only available when compiling with the `source` feature.
    ///
    /// # Errors
    ///
    /// This should never error, but the underlying OpenH264 encoder has an error indication and
    /// since we don't know their code that well we just can't guarantee it.
    #[cfg(feature = "source")]
    pub fn new() -> Result<Self, Error> {
        let api = OpenH264API::from_source();
        let config = EncoderConfig::new();
        let raw_api = EncoderRawAPI::new(api)?;

        Ok(Self {
            config,
            raw_api,
            bit_stream_info: SFrameBSInfo::default(),
            previous_dimensions: None,
        })
    }
    /// Create an encoder with the provided [API](OpenH264API) and [configuration](EncoderConfig).
    ///
    /// The width and height will be taken from the [`YUVSource`] when calling [`Encoder::encode()`].
    ///
    /// # Errors
    ///
    /// Might fail if the provided encoder parameters had issues.
    pub fn with_api_config(api: OpenH264API, config: EncoderConfig) -> Result<Self, Error> {
        let raw_api = EncoderRawAPI::new(api)?;

        Ok(Self {
            config,
            raw_api,
            bit_stream_info: SFrameBSInfo::default(),
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
    /// # Errors
    ///
    /// This might error for various reasons, many of which aren't clearly documented in OpenH264.
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
    ///
    /// # Errors
    ///
    /// This might error for various reasons, many of which aren't clearly documented in OpenH264.
    pub fn encode_at<T: YUVSource>(&mut self, yuv_source: &T, timestamp: Timestamp) -> Result<EncodedBitStream<'_>, Error> {
        let new_dimensions = yuv_source.dimensions_i32();

        if self.previous_dimensions != Some(new_dimensions) {
            self.reinit(new_dimensions.0, new_dimensions.1)?;
            self.previous_dimensions = Some(new_dimensions);
        }

        let strides = yuv_source.strides_i32();

        // Converting *const u8 to *mut u8 should be fine because the encoder _should_
        // only read these arrays (TODO: needs verification).
        let source = SSourcePicture {
            iColorFormat: videoFormatI420,
            iStride: [strides.0, strides.1, strides.2, 0],
            pData: [
                yuv_source.y().as_ptr().cast_mut(),
                yuv_source.u().as_ptr().cast_mut(),
                yuv_source.v().as_ptr().cast_mut(),
                null_mut(),
            ],
            iPicWidth: new_dimensions.0,
            iPicHeight: new_dimensions.1,
            uiTimeStamp: timestamp.as_native(),
            bPsnrY: false,
            bPsnrU: false,
            bPsnrV: false,
        };

        unsafe {
            self.raw_api
                .encode_frame(&raw const source, &raw mut self.bit_stream_info)
                .ok()?;
        }

        Ok(EncodedBitStream {
            bit_stream_info: &self.bit_stream_info,
        })
    }

    #[rustfmt::skip]
    fn reinit(&mut self, width: i32, height: i32) -> Result<(), Error> {
        // https://github.com/cisco/openh264/blob/master/README.md
        // > Encoder errors when resolution exceeds 3840x2160 or 2160x3840
        //
        // Some more detail here:
        // https://github.com/cisco/openh264/issues/3553
        // > Currently the encoder/decoder could only support up to level 5.2,
        let greater_dim = std::cmp::max(width, height);
        let smaller_dim = std::cmp::min(width, height);

        if greater_dim > 3840 || smaller_dim > 2160 {
            return Err(Error::msg("Encoder max resolution 3840x2160 horizontal or 2160x3840 vertical"));
        }

        let mut params = SEncParamExt::default();

        unsafe { self.raw_api.get_default_params(&raw mut params).ok()? };

        params.iPicWidth = width as c_int; // If we do .into() instead, could this fail to compile on some platforms?
        params.iPicHeight = height as c_int; // If we do .into() instead, could this fail to compile on some platforms?
        params.iRCMode = self.config.rate_control_mode.to_c();
        params.bEnableFrameSkip = self.config.enable_skip_frame;
        params.iTargetBitrate = self.config.target_bitrate.0.try_into()?;
        params.bEnableDenoise = self.config.enable_denoise;
        params.fMaxFrameRate = self.config.max_frame_rate.0;
        params.eSpsPpsIdStrategy = self.config.sps_pps_strategy.to_c();
        params.iMultipleThreadIdc = self.config.multiple_thread_idc;
        params.iUsageType = self.config.usage_type.to_c();

        params.bEnableSceneChangeDetect = self.config.scene_change_detect;
        params.bEnableAdaptiveQuant = self.config.adaptive_quantization;
        params.bEnableBackgroundDetection = self.config.background_detection;
        params.bEnableLongTermReference = self.config.long_term_reference;
        params.iComplexityMode = self.config.complexity.to_c();
        params.uiIntraPeriod = self.config.intra_frame_period.0;
        params.iLoopFilterDisableIdc = DEBLOCKING_IDC_0;
        params.iMinQp = self.config.qp.min.into();
        params.iMaxQp = self.config.qp.max.into();

        if let Some(profile) = self.config.profile {
            params.sSpatialLayers[0].uiProfileIdc = profile.to_c();
        }

        if let Some(level) = self.config.level {
            params.sSpatialLayers[0].uiLevelIdc = level.to_c();
        }

        // Apply VUI (Video Usability Information) parameters for color space signaling
        if let Some(ref vui) = self.config.vui {
            params.sSpatialLayers[0].bVideoSignalTypePresent = true;
            params.sSpatialLayers[0].bColorDescriptionPresent = true;
            params.sSpatialLayers[0].bFullRange = vui.full_range;
            params.sSpatialLayers[0].uiColorPrimaries = vui.color_primaries.as_u8();
            params.sSpatialLayers[0].uiTransferCharacteristics = vui.transfer_characteristics.as_u8();
            params.sSpatialLayers[0].uiColorMatrix = vui.matrix_coefficients.as_u8();
        }

        params.iSpatialLayerNum = 1;
        params.iTemporalLayerNum = 1;
        params.iLtrMarkPeriod = 30;
        params.sSpatialLayers[0].iMaxSpatialBitrate = self.config.target_bitrate.0.try_into()?;
        params.sSpatialLayers[0].iSpatialBitrate = self.config.target_bitrate.0.try_into()?;
        params.sSpatialLayers[0].fFrameRate = self.config.max_frame_rate.0;
        params.sSpatialLayers[0].iVideoWidth = width;
        params.sSpatialLayers[0].iVideoHeight = height;

        if let Some(max_slice_len) = self.config.max_slice_len {
            // Limit the slice length by setting both MaxNalSize and uiSliceSizeConstraint
            params.uiMaxNalSize = max_slice_len;

            params.sSpatialLayers[0].sSliceArgument.uiSliceMode = SM_SIZELIMITED_SLICE;
            params.sSpatialLayers[0].sSliceArgument.uiSliceSizeConstraint = max_slice_len;
        } else {
            // No size limit, explicitly use defaults
            params.sSpatialLayers[0].sSliceArgument.uiSliceMode = SM_SINGLE_SLICE;
            params.sSpatialLayers[0].sSliceArgument.uiSliceNum = 1;
        }

        unsafe {
            if self.previous_dimensions.is_none() {
                // First time we call initialize_ext
                self.raw_api.initialize_ext(&raw const params).ok()?;
                self.raw_api.set_option(ENCODER_OPTION_TRACE_LEVEL, addr_of_mut!(self.config.debug).cast()).ok()?;
                self.raw_api.set_option(ENCODER_OPTION_DATAFORMAT, addr_of_mut!(self.config.data_format).cast()).ok()?;
            } else {
                // Subsequent times we call SetOption
                self.raw_api.set_option(ENCODER_OPTION_SVC_ENCODE_PARAM_EXT, addr_of_mut!(params).cast()).ok()?;

                // Start with a new keyframe after dimensions changed.
                self.force_intra_frame();
            }
        }

        Ok(())
    }

    /// Forces the encoder to emit an intra frame (I-frame, "keyframe") for the next encoded frame.
    pub fn force_intra_frame(&mut self) {
        // SAFETY: This should be safe, simply as there is no indication why it shouldn't be. We are
        // initialized at this point, and forcing an IDR should be straightforward.
        unsafe {
            self.raw_api.force_intra_frame(true);
        }
    }

    /// Obtain the raw API for advanced use cases.
    ///
    /// When resorting to this call, please consider filing an issue / PR.
    ///
    /// # Safety
    ///
    /// You must not set parameters the encoder relies on, we recommend checking the source.
    pub const unsafe fn raw_api(&mut self) -> &mut EncoderRawAPI {
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

/// Bitstream output resulting from an [`encode()`](Encoder::encode) operation.
pub struct EncodedBitStream<'a> {
    /// Holds the bitstream info just encoded.
    bit_stream_info: &'a SFrameBSInfo,
}

impl<'a> EncodedBitStream<'a> {
    /// Raw bitstream info returned by the encoder.
    #[must_use]
    pub const fn raw_info(&self) -> &'a SFrameBSInfo {
        self.bit_stream_info
    }

    /// Frame type of the encoded packet.
    #[must_use]
    pub const fn frame_type(&self) -> FrameType {
        FrameType::from_c_int(self.bit_stream_info.eFrameType)
    }

    /// Number of layers in the encoded packet.
    #[must_use]
    pub const fn num_layers(&self) -> usize {
        self.bit_stream_info.iLayerNum as usize
    }

    /// Returns ith layer of this bitstream.
    #[must_use]
    pub const fn layer(&self, i: usize) -> Option<Layer<'a>> {
        if i < self.num_layers() {
            Some(Layer {
                layer_info: &self.bit_stream_info.sLayerInfo[i],
            })
        } else {
            None
        }
    }

    /// Writes the current bitstream into the given Vec.
    #[allow(clippy::missing_panics_doc)]
    pub fn write_vec(&self, dst: &mut Vec<u8>) {
        for l in 0..self.num_layers() {
            let layer = self.layer(l).unwrap();

            for n in 0..layer.nal_count() {
                let nal = layer.nal_unit(n).unwrap();

                dst.extend_from_slice(nal);
            }
        }
    }

    /// Writes the current bitstream into the given Writer.
    ///
    /// # Errors
    ///
    /// Can error when bytes could not be written.
    #[allow(clippy::missing_panics_doc)]
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
                        return Err(Error::msg(&format!("failed to write: {e}")));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Convenience method returning a Vec containing the encoded bitstream.
    #[must_use]
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
    #[must_use]
    pub const fn raw_info(&self) -> &'a SLayerBSInfo {
        self.layer_info
    }

    /// NAL count of this layer.
    #[must_use]
    pub const fn nal_count(&self) -> usize {
        self.layer_info.iNalCount as usize
    }

    /// Returns NAL unit data for the ith element.
    #[must_use]
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
    #[must_use]
    pub const fn is_video(&self) -> bool {
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
    const fn from_c_int(native: std::os::raw::c_int) -> Self {
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
