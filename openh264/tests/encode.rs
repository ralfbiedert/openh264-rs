#![allow(clippy::bool_assert_comparison)]

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::encoder::{Encoder, EncoderConfig, FrameType};
use openh264::formats::YUVBuffer;
use openh264::{Error, OpenH264API, Timestamp};

#[test]
#[cfg(feature = "source")]
fn can_get_encoder() -> Result<(), Error> {
    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let _encoder = Encoder::with_config(api, config)?;

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn encode() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_config(api, config)?;
    let mut converter = YUVBuffer::new(128, 128);

    converter.read_rgb(src);

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
#[ignore]
#[cfg(feature = "source")]
fn encode_at_timestamp_roundtrips() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_config(api, config)?;
    let mut converter = YUVBuffer::new(128, 128);

    converter.read_rgb(src);

    let timestamp = Timestamp::from_millis(64);
    let encoded = encoder.encode_at(&converter, timestamp)?.to_vec();

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(api, config)?;
    let yuv = decoder
        .decode(encoded.as_slice())?
        .ok_or_else(|| Error::msg("Must have image"))?;

    assert_eq!(yuv.dimension_y().0, 128);
    assert_eq!(yuv.dimension_y().1, 128);
    assert_eq!(yuv.timestamp(), timestamp); // TODO: This fails, the returned timestamp is 0.

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn encoder_sps_pps() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_config(api, config)?;
    let mut converter = YUVBuffer::new(128, 128);

    converter.read_rgb(src);

    let stream = encoder.encode(&converter)?;

    let layer_0 = stream.layer(0).unwrap();
    let raw_sps = layer_0.nal_unit(0).unwrap();
    let raw_pps = layer_0.nal_unit(1).unwrap();

    assert!(!raw_sps.is_empty());
    assert!(!raw_pps.is_empty());

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn what_goes_around_comes_around() -> Result<(), Error> {
    use openh264::decoder::{Decoder, DecoderConfig};

    let src = include_bytes!("data/single_512x512_cavlc.h264");

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_config(api, config)?;
    let yuv = decoder.decode(src)?.ok_or_else(|| Error::msg("Must have image"))?;

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_config(api, config)?;

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

#[test]
#[cfg(feature = "source")]
fn encode_change_resolution() -> Result<(), Error> {
    let src1 = include_bytes!("data/lenna_128x128.rgb");
    let src2 = include_bytes!("data/lenna_512x512.rgb");

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_config(api, config)?;

    let converter1 = {
        let mut buf = YUVBuffer::new(128, 128);
        buf.read_rgb(src1);
        buf
    };

    let stream = encoder.encode(&converter1)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);
    assert_eq!(stream.num_layers(), 2);
    assert_eq!(stream.layer(0).unwrap().nal_count(), 2);

    let converter2 = {
        let mut buf = YUVBuffer::new(512, 512);
        buf.read_rgb(src2);
        buf
    };

    let stream = encoder.encode(&converter2)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);
    assert_eq!(stream.num_layers(), 2);
    assert_eq!(stream.layer(0).unwrap().nal_count(), 2);

    Ok(())
}
