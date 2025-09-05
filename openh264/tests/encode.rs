#![allow(clippy::bool_assert_comparison)]

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::encoder::{Encoder, EncoderConfig, FrameType};
use openh264::formats::{RgbSliceU8, YUVBuffer, YUVSource};
use openh264::{Error, OpenH264API, Timestamp};
use openh264_sys2::DynamicAPI;

#[test]
#[cfg(feature = "source")]
fn can_get_encoder() -> Result<(), Error> {
    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let _encoder = Encoder::with_api_config(api, config)?;

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn can_get_encoder_default() -> Result<(), Error> {
    let _encoder = Encoder::new()?;

    Ok(())
}

#[test]
#[cfg(feature = "source")]
fn encode() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");
    let rgb_source = RgbSliceU8::new(src, (128, 128));
    let yuv = YUVBuffer::from_rgb_source(rgb_source);

    let mut encoder = Encoder::new()?;
    let stream = encoder.encode(&yuv)?;

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
#[ignore = "Timestamp logic broken atm"]
#[allow(clippy::similar_names)]
#[cfg(feature = "source")]
fn encode_at_timestamp_roundtrips() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");
    let rgb_source = RgbSliceU8::new(src, (128, 128));
    let yuv = YUVBuffer::from_rgb_source(rgb_source);

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_api_config(api, config)?;

    let timestamp = Timestamp::from_millis(64);
    let encoded = encoder.encode_at(&yuv, timestamp)?.to_vec();

    let api = OpenH264API::from_source();
    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;
    let yuv = decoder
        .decode(encoded.as_slice())?
        .ok_or_else(|| Error::msg("Must have image"))?;

    assert_eq!(yuv.dimensions().0, 128);
    assert_eq!(yuv.dimensions().1, 128);
    assert_eq!(yuv.timestamp(), timestamp); // TODO: This fails, the returned timestamp is 0.

    Ok(())
}

#[test]
#[cfg(feature = "source")]
#[allow(clippy::similar_names)]
fn encoder_sps_pps() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");
    let rgb_source = RgbSliceU8::new(src, (128, 128));
    let yuv = YUVBuffer::from_rgb_source(rgb_source);

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();

    let mut encoder = Encoder::with_api_config(api, config)?;
    let stream = encoder.encode(&yuv)?;

    let layer_0 = stream.layer(0).unwrap();
    let raw_sps = layer_0.nal_unit(0).unwrap();
    let raw_pps = layer_0.nal_unit(1).unwrap();

    assert!(!raw_sps.is_empty());
    assert!(!raw_pps.is_empty());

    Ok(())
}

fn can_encode_decoded(api: DynamicAPI) -> Result<(), Error> {
    use openh264::decoder::{Decoder, DecoderConfig};

    let src = include_bytes!("data/single_512x512_cavlc.h264");

    let config = DecoderConfig::default();
    let mut decoder = Decoder::with_api_config(api, config)?;
    let yuv = decoder.decode(src)?.ok_or_else(|| Error::msg("Must have image"))?;

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_api_config(api, config)?;

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
fn can_encode_decoded_via_source() -> Result<(), Error> {
    let api = OpenH264API::from_source();
    can_encode_decoded(api)
}

#[test]
#[cfg(all(target_os = "windows", target_arch = "x86_64", feature = "libloading"))]
fn can_encode_decoded_via_dll() -> Result<(), Error> {
    let dll = format!("../openh264-sys2/tests/reference/{}", openh264_sys2::reference_dll_name());
    let api = OpenH264API::from_blob_path(dll)?;
    can_encode_decoded(api)
}

#[test]
#[cfg(feature = "source")]
fn encode_change_resolution() -> Result<(), Error> {
    let src = include_bytes!("data/lenna_128x128.rgb");
    let rgb_source = RgbSliceU8::new(src, (128, 128));
    let yuv1 = YUVBuffer::from_rgb_source(rgb_source);

    let src = include_bytes!("data/lenna_512x512.rgb");
    let rgb_source = RgbSliceU8::new(src, (512, 512));
    let yuv2 = YUVBuffer::from_rgb_source(rgb_source);

    let api = OpenH264API::from_source();
    let config = EncoderConfig::new();
    let mut encoder = Encoder::with_api_config(api, config)?;

    let stream = encoder.encode(&yuv1)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);
    assert_eq!(stream.num_layers(), 2);
    assert_eq!(stream.layer(0).unwrap().nal_count(), 2);

    let stream = encoder.encode(&yuv2)?;

    assert_eq!(stream.frame_type(), FrameType::IDR);
    assert_eq!(stream.num_layers(), 2);
    assert_eq!(stream.layer(0).unwrap().nal_count(), 2);

    Ok(())
}
