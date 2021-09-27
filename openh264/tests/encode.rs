#![cfg(feature = "encoder")]
#![allow(clippy::bool_assert_comparison)]

use openh264::encoder::{Encoder, EncoderConfig, FrameType};
use openh264::formats::RBGYUVConverter;
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

    let stream = encoder.encode(&converter)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);
    assert_eq!(stream.num_layers(), 2);

    // Test NAL headers available.
    let layer = stream.layer(0).unwrap();
    assert!(!layer.is_video());
    assert_eq!(layer.nal_count(), 2);
    assert_eq!(&layer.nal_unit(0).unwrap()[..5], &[0u8, 0u8, 0u8, 1u8, 0x67u8]);
    assert_eq!(&layer.nal_unit(1).unwrap()[..5], &[0u8, 0u8, 0u8, 1u8, 0x68u8]);

    let layer = stream.layer(1).unwrap();
    assert!(layer.is_video());
    assert_eq!(layer.nal_count(), 1);

    // Test video unit has good header and reasonable length.
    let video_unit = layer.nal_unit(0).unwrap();
    assert_eq!(&video_unit[..5], &[0u8, 0u8, 0u8, 1u8, 0x65u8]);
    assert!(video_unit.len() > 1000);
    assert!(video_unit.len() < 100_000);

    Ok(())
}

#[test]
#[cfg(feature = "decoder")]
fn what_goes_around_comes_around() -> Result<(), Error> {
    use openh264::decoder::{Decoder, DecoderConfig};

    let src = &include_bytes!("data/single_512x512_cavlc.h264")[..];

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(config)?;
    let yuv = decoder.decode(src)?;

    let config = EncoderConfig::new(512, 512);
    let mut encoder = Encoder::with_config(config)?;

    let stream = encoder.encode(&yuv)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);
    assert_eq!(stream.num_layers(), 2);

    // Test NAL headers available
    let layer = stream.layer(0).unwrap();
    assert!(!layer.is_video());
    assert_eq!(layer.nal_count(), 2);
    assert_eq!(&layer.nal_unit(0).unwrap()[..5], &[0u8, 0u8, 0u8, 1u8, 0x67u8]);
    assert_eq!(&layer.nal_unit(1).unwrap()[..5], &[0u8, 0u8, 0u8, 1u8, 0x68u8]);

    let layer = stream.layer(1).unwrap();
    assert!(layer.is_video());
    assert_eq!(layer.nal_count(), 1);

    // Test video unit has good header and reasonable length.
    let video_unit = layer.nal_unit(0).unwrap();
    assert_eq!(&video_unit[..5], &[0u8, 0u8, 0u8, 1u8, 0x65u8]);
    assert!(video_unit.len() > 1000);
    assert!(video_unit.len() < 100_000);

    Ok(())
}
