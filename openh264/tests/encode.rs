use openh264::decoder::{Decoder, DecoderConfig};
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::Error;

#[test]
fn can_get_encoder() -> Result<(), Error> {
    let config = EncoderConfig::new(300, 200);
    let _encoder = Encoder::with_config(config)?;

    Ok(())
}

// Encode function broken for now.
#[ignore]
#[test]
fn what_goes_around_comes_around() -> Result<(), Error> {
    let src = &include_bytes!("data/single_512x512_cavlc.h264")[..];

    let config = DecoderConfig::default().debug(true);
    let mut decoder = Decoder::with_config(config)?;
    let yuv = decoder.decode_no_delay(src)?;
    let strides = yuv.strides_yuv();

    let config = EncoderConfig::new(512, 512);
    let mut encoder = Encoder::with_config(config)?;

    encoder.encode_todo(
        yuv.v_with_stride(),
        yuv.u_with_stride(),
        yuv.v_with_stride(),
        strides.0,
        strides.1,
        strides.2,
    )?;

    Ok(())
}
