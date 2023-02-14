#![cfg(all(feature = "decoder", feature = "mp4"))]

use openh264::decoder::{Decoder, DecoderConfig};
use openh264::{Error, Mp4BitstreamConverter};
use std::io::Cursor;

#[test]
fn can_decode_mp4_bitstream() -> Result<(), Error> {
    let mp4 = include_bytes!("data/multi_512x512.mp4");

    let mut mp4 = mp4::Mp4Reader::read_header(Cursor::new(mp4), mp4.len() as u64).unwrap();

    let track = mp4
        .tracks()
        .iter()
        .find(|(_, t)| t.media_type().unwrap() == mp4::MediaType::H264)
        .unwrap()
        .1;
    let track_id = track.track_id();

    // mp4 spits out length-prefixed NAL units, but openh264 expects start codes
    // the mp4 stream also lacks parameter sets, so we need to add them
    // Mp4BitstreamConverter does this for us
    let mut bitstream_converter = Mp4BitstreamConverter::for_mp4_track(track);

    let config = DecoderConfig::default().debug(false);
    let mut decoder = Decoder::with_config(config)?;

    let mut buffer = Vec::new();

    for i in 1..track.sample_count() + 1 {
        let sample = mp4.read_sample(track_id, i).unwrap().unwrap();
        // convert the packet from mp4 representation to one that openh264 can decode
        bitstream_converter.convert_packet(&sample.bytes, &mut buffer);

        decoder.decode(&buffer)?;
    }

    Ok(())
}
