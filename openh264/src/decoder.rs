use crate::error::NativeErrorExt;
use crate::Error;
use openh264_sys2::{
    ISVCDecoder, ISVCDecoderVtbl, SBufferInfo, SDecodingParam, SParserBsInfo, WelsCreateDecoder, WelsDestroyDecoder,
    DECODER_OPTION, DECODING_STATE,
};
use std::marker::PhantomData;
use std::os::raw::{c_int, c_long, c_uchar, c_void};
use std::ptr::{null, null_mut};

/// Convenience wrapper with guaranteed function pointers for easy access.
#[rustfmt::skip]
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct DecoderRaw {
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
#[allow(non_snake_case)]
impl DecoderRaw {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let mut decoder_ptr = null::<ISVCDecoderVtbl>() as *mut *const ISVCDecoderVtbl;

            WelsCreateDecoder(&mut decoder_ptr as *mut *mut *const ISVCDecoderVtbl).ok()?;

            let e = Error::msg("VTable missing function.");

            Ok(DecoderRaw {
                decoder_ptr,
                initialize: (*(*decoder_ptr)).Initialize.ok_or(e)?,
                uninitialize: (*(*decoder_ptr)).Uninitialize.ok_or(e)?,
                decode_frame: (*(*decoder_ptr)).DecodeFrame.ok_or(e)?,
                decode_frame_no_delay: (*(*decoder_ptr)).DecodeFrameNoDelay.ok_or(e)?,
                decode_frame2: (*(*decoder_ptr)).DecodeFrame2.ok_or(e)?,
                flush_frame: (*(*decoder_ptr)).FlushFrame.ok_or(e)?,
                decode_parser: (*(*decoder_ptr)).DecodeParser.ok_or(e)?,
                decode_frame_ex: (*(*decoder_ptr)).DecodeFrameEx.ok_or(e)?,
                set_option: (*(*decoder_ptr)).SetOption.ok_or(e)?,
                get_option: (*(*decoder_ptr)).GetOption.ok_or(e)?,
            })
        }
    }

    unsafe fn initialize(&self, pParam: *const SDecodingParam) -> c_long {
        (self.initialize)(self.decoder_ptr, pParam)
    }

    unsafe fn uninitialize(&self, ) -> c_long {
        (self.uninitialize)(self.decoder_ptr)
    }

    unsafe fn decode_frame(&self, Src: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pStride: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int) -> DECODING_STATE {
        (self.decode_frame)(self.decoder_ptr, Src, iSrcLen, ppDst, pStride, iWidth, iHeight)
    }

    unsafe fn decode_frame_no_delay(&self, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE {
        (self.decode_frame_no_delay)(self.decoder_ptr, pSrc, iSrcLen, ppDst, pDstInfo)
    }

    unsafe fn decode_frame2(&self, pSrc: *const c_uchar, iSrcLen: c_int, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE {
        (self.decode_frame2)(self.decoder_ptr, pSrc, iSrcLen, ppDst, pDstInfo)
    }

    unsafe fn flush_frame(&self, ppDst: *mut *mut c_uchar, pDstInfo: *mut SBufferInfo) -> DECODING_STATE {
        (self.flush_frame)(self.decoder_ptr, ppDst, pDstInfo)
    }

    unsafe fn decode_parser(&self, pSrc: *const c_uchar, iSrcLen: c_int, pDstInfo: *mut SParserBsInfo) -> DECODING_STATE {
        (self.decode_parser)(self.decoder_ptr, pSrc, iSrcLen, pDstInfo)
    }

    unsafe fn decode_frame_ex(&self, pSrc: *const c_uchar, iSrcLen: c_int, pDst: *mut c_uchar, iDstStride: c_int, iDstLen: *mut c_int, iWidth: *mut c_int, iHeight: *mut c_int, iColorFormat: *mut c_int) -> DECODING_STATE {
        (self.decode_frame_ex)(self.decoder_ptr, pSrc, iSrcLen, pDst, iDstStride, iDstLen, iWidth, iHeight, iColorFormat)
    }

    unsafe fn set_option(&self, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long {
        (self.set_option)(self.decoder_ptr, eOptionId, pOption)
    }

    unsafe fn get_option(&self, eOptionId: DECODER_OPTION, pOption: *mut c_void) -> c_long {
        (self.get_option)(self.decoder_ptr, eOptionId, pOption)
    }
}

impl Drop for DecoderRaw {
    fn drop(&mut self) {
        // Safe because when we drop the pointer must have been initialized, and we aren't clone.
        unsafe {
            WelsDestroyDecoder(self.decoder_ptr);
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct DecoderConfig {
    params: SDecodingParam,
}

#[derive(Debug)]
pub struct Decoder {
    raw: DecoderRaw,
}

impl Decoder {
    pub fn with_config(config: &DecoderConfig) -> Result<Self, Error> {
        let raw = DecoderRaw::new()?;

        unsafe { raw.initialize(&config.params).ok()? };

        Ok(Self { raw })
    }

    pub fn xxx_decode(&mut self, packet: &[u8]) -> Result<DecodedYUV, Error> {
        let mut dst = [null_mut(); 3];
        let mut buffer_info = SBufferInfo::default();

        unsafe {
            self.raw
                .decode_frame2(packet.as_ptr(), packet.len() as i32, &mut dst as *mut _, &mut buffer_info)
                .ok()?;

            // Is this correct?
            // https://github.com/cisco/openh264/issues/1415
            self.raw.decode_frame2(null(), 0, &mut dst as *mut _, &mut buffer_info).ok()?;

            dbg!(dst);
        }

        Ok(DecodedYUV { x: Default::default() })
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // Safe because when we drop the pointer must have been initialized.
        unsafe {
            self.raw.uninitialize();
        }
    }
}

pub struct DecodedYUV<'a> {
    x: PhantomData<&'a ()>,
}
