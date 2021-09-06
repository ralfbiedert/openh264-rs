use openh264::{Decoder, DecoderConfig, Error};

#[test]
fn can_get_decoder() -> Result<(), Error> {
    let config = DecoderConfig::default();
    let _decoder = Decoder::with_config(config)?;

    Ok(())
}

#[test]
#[rustfmt::skip]
fn can_decode_single() -> Result<(), Error> {
    let sources = [
        &include_bytes!("data/single_1920x1080_cabac.h264")[..],
        &include_bytes!("data/single_512x512_cabac.h264")[..],
        &include_bytes!("data/single_512x512_cavlc.h264")[..],
    ];

    for (_, src) in sources.iter().enumerate() {
        let config = unsafe { DecoderConfig::default().debug(true).num_threads(0) };
        let mut decoder = Decoder::with_config(config)?;

        let yuv = decoder.decode_no_delay(src)?;

        let dim = yuv.dimension_rgb();
        let rgb_len = dim.0 * dim.1 * 3;
        let mut rgb = vec![0; rgb_len];

        yuv.write_rgb8(&mut rgb)?;
    }

    Ok(())
}

#[test]
fn can_decode_multi_to_end() -> Result<(), Error> {
    let src = &include_bytes!("data/multi_512x512.h264")[..];

    let config = unsafe { DecoderConfig::default().debug(true).num_threads(1) };
    let mut decoder = Decoder::with_config(config)?;

    decoder.decode_no_delay(src)?;

    Ok(())
}

#[test]
fn can_decode_multi_by_step() -> Result<(), Error> {
    let src = &include_bytes!("data/multi_512x512.h264")[..];

    let packet_lengths = [30, 2736, 2688, 2672, 2912, 3215];

    let config = DecoderConfig::default().debug(true);
    let mut decoder = Decoder::with_config(config)?;

    let mut p = 0;

    for l in packet_lengths {
        decoder.decode_no_delay(&src[p..p + l])?;

        p += l;
    }

    Ok(())
}

#[test]
fn fails_on_truncated() -> Result<(), Error> {
    let src = &include_bytes!("data/multi_512x512_truncated.h264")[..];

    let config = DecoderConfig::default().debug(true);
    let mut decoder = Decoder::with_config(config)?;

    assert!(decoder.decode_no_delay(src).is_err());

    Ok(())
}
