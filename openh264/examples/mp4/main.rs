mod mp4_bitstream_converter;

use crate::mp4_bitstream_converter::Mp4BitstreamConverter;
use anyhow::{anyhow, Error};
use openh264::decoder::Decoder;
use openh264::OpenH264API;
use std::io::Cursor;

fn main() -> Result<(), Error> {
    let mp4 = include_bytes!("../../tests/data/multi_512x512.mp4");
    let mut mp4 = mp4::Mp4Reader::read_header(Cursor::new(mp4), mp4.len() as u64)?;

    let track = mp4
        .tracks()
        .iter()
        .find(|(_, t)| t.media_type().unwrap() == mp4::MediaType::H264)
        .ok_or_else(|| anyhow!("Must exist"))?
        .1;
    let track_id = track.track_id();

    // mp4 spits out length-prefixed NAL units, but openh264 expects start codes
    // the mp4 stream also lacks parameter sets, so we need to add them
    // Mp4BitstreamConverter does this for us
    let api = OpenH264API::from_source();
    let mut bitstream_converter = Mp4BitstreamConverter::for_mp4_track(track)?;
    let mut decoder = Decoder::new(api)?;

    let mut buffer = Vec::new();
    let mut rgb = [0; 512 * 512 * 3];

    for i in 1..track.sample_count() + 1 {
        let Some(sample) = mp4.read_sample(track_id, i)? else {
            continue;
        };

        // convert the packet from mp4 representation to one that openh264 can decode
        bitstream_converter.convert_packet(&sample.bytes, &mut buffer);

        if let Some(image) = decoder.decode(&buffer)? {
            image.write_rgb8(&mut rgb);
        }
    }

    Ok(())
}
