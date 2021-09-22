#![cfg(feature = "encoder")]

#[cfg(feature = "decoder")]
use openh264::decoder::{Decoder, DecoderConfig};

use openh264::encoder::{Encoder, EncoderConfig, FrameType};
use openh264::formats::RBGYUVConverter;
use openh264::Error;
use std::fs::File;
use std::io::Write;

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

    let stream = encoder.encode(&converter)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);

    // Test length reasonable.
    assert!(stream.bit_stream().len() > 1000);
    assert!(stream.bit_stream().len() < 100_000);

    Ok(())
}

#[test]
#[cfg(all(feature = "decoder", feature = "encoder"))]
fn what_goes_around_comes_around() -> Result<(), Error> {
    let src = &include_bytes!("data/single_512x512_cavlc.h264")[..];

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config)?;
    let yuv = decoder.decode_no_delay(src)?;

    let config = EncoderConfig::new(512, 512);
    let mut encoder = Encoder::with_config(config)?;

    let stream = encoder.encode(&yuv)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);

    // Test length reasonable.
    assert!(stream.bit_stream().len() > 1000);
    assert!(stream.bit_stream().len() < 100_000);

    // TODO: This fails right now as the encoded stream does not contain (or make available) the SPS / PPS.

    // let mut f = File::create("debug.h264").unwrap();
    // f.write_all(stream.bit_stream());

    // Test we can re-decode what we have encoded.
    // let config = DecoderConfig::default();
    // let mut decoder = Decoder::with_config(config)?;
    // let yuv = decoder.decode_no_delay(stream.bit_stream())?;

    Ok(())
}
