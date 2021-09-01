#![feature(test)]

extern crate test;

use openh264::NativeErrorExt;
use openh264_sys2::{
    ISVCDecoderVtbl, SBufferInfo, SDecodingParam, ERROR_CON_IDC, VIDEO_BITSTREAM_TYPE,
};
use std::ptr::{null, null_mut};
use test::{black_box, Bencher};

#[bench]
fn my_algo_a(b: &mut Bencher) {
    unsafe {
        // let mut table = null::<ISVCDecoderVtbl>();
        let mut ptr = null::<ISVCDecoderVtbl>() as *mut *const ISVCDecoderVtbl;
        let ptr2 = &mut ptr as *mut *mut *const ISVCDecoderVtbl;

        openh264_sys2::WelsCreateDecoder(ptr2).ok().unwrap();

        dbg!((*(*ptr)).Initialize);

        let mut decode_param = SDecodingParam::default();
        decode_param.uiTargetDqLayer = u8::MAX;
        decode_param.eEcActiveIdc = ERROR_CON_IDC::ERROR_CON_FRAME_COPY_CROSS_IDR;
        decode_param.sVideoProperty.eVideoBsType = VIDEO_BITSTREAM_TYPE::VIDEO_BITSTREAM_DEFAULT;

        let init = (*(*ptr)).Initialize.unwrap();
        let decode = (*(*ptr)).DecodeFrame2.unwrap();

        init(ptr, &decode_param).ok().unwrap();

        let frame = include_bytes!("../tests/data/test_0.h264");

        b.iter(|| {
            let mut dst = [null_mut(); 3];
            let mut buffer_info = SBufferInfo::default();
            let state = decode(
                ptr,
                frame.as_ptr(),
                frame.len() as i32,
                &mut dst as *mut _,
                &mut buffer_info,
            );

            // https://github.com/cisco/openh264/issues/1415
            let state = decode(ptr, null(), 0, &mut dst as *mut _, &mut buffer_info)
                .ok()
                .unwrap();
        });

        openh264_sys2::WelsDestroyDecoder(ptr);
    }
}
