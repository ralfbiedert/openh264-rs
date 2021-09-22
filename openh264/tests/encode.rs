#![cfg(feature = "encoder")]

#[cfg(feature = "decoder")]
use openh264::decoder::{Decoder, DecoderConfig};

use openh264::encoder::{Encoder, EncoderConfig, RBGYUVConverter};
use openh264::Error;

#[test]
fn can_get_encoder() -> Result<(), Error> {
    let config = EncoderConfig::new(300, 200);
    let _encoder = Encoder::with_config(config)?;

    Ok(())
}

#[test]
fn encode() -> Result<(), Error> {
    let src = &include_bytes!("data/lenna_128x128.rgb")[..];

    let config = EncoderConfig::new(128, 128);
    let mut encoder = Encoder::with_config(config)?;
    let mut converter = RBGYUVConverter::new(128, 128);

    converter.convert(src);
    encoder.encode(&converter)?;

    Ok(())
}

// Encode function broken for now.
#[test]
#[cfg(all(feature = "decoder", feature = "encoder"))]
fn what_goes_around_comes_around() -> Result<(), Error> {
    let src = &include_bytes!("data/single_512x512_cavlc.h264")[..];

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config)?;
    let yuv = decoder.decode_no_delay(src)?;

    let config = EncoderConfig::new(512, 512);
    let mut encoder = Encoder::with_config(config)?;

    encoder.encode(&yuv)?;

    Ok(())
}
