mod mp4_bitstream_converter;

use crate::mp4_bitstream_converter::Mp4BitstreamConverter;
use anyhow::{anyhow, Error};
use openh264::decoder::{Decoder, DecoderConfig, Flush};
use std::fs::File;
use std::io::{Cursor, Read, Write};

#[cfg(feature = "source")]
fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    let bin = args.next().unwrap_or_else(|| String::from("mp4"));
    let path = args.next().ok_or_else(|| anyhow!("usage: {bin} <filename> [out]"))?;
    let out = args.next().unwrap_or_else(|| String::from("."));

    let mut file = std::fs::File::open(path)?;
    let mut mp4 = Vec::new();
    file.read_to_end(&mut mp4).unwrap();

    let mut mp4 = mp4::Mp4Reader::read_header(Cursor::new(&mp4), mp4.len() as u64)?;

    let track = mp4
        .tracks()
        .iter()
        .find(|(_, t)| t.media_type().unwrap() == mp4::MediaType::H264)
        .ok_or_else(|| anyhow!("Must exist"))?
        .1;
    let track_id = track.track_id();
    let width = track.width() as usize;
    let height = track.height() as usize;
    let decoder_options = DecoderConfig::new().debug(true).flush_after_decode(Flush::NoFlush);

    // mp4 spits out length-prefixed NAL units, but openh264 expects start codes
    // the mp4 stream also lacks parameter sets, so we need to add them
    // Mp4BitstreamConverter does this for us
    let mut bitstream_converter = Mp4BitstreamConverter::for_mp4_track(track)?;
    let mut decoder = Decoder::with_api_config(openh264::OpenH264API::from_source(), decoder_options).unwrap();

    let mut buffer = Vec::new();
    let mut rgb = vec![0; width * height * 3];

    let mut frame_idx = 0;
    for i in 1..=track.sample_count() {
        let Some(sample) = mp4.read_sample(track_id, i)? else {
            continue;
        };

        // convert the packet from mp4 representation to one that openh264 can decode
        bitstream_converter.convert_packet(&sample.bytes, &mut buffer);
        match decoder.decode(&buffer) {
            Ok(Some(image)) => {
                image.write_rgb8(&mut rgb);
                save_file(&format!("{out}/frame-0{frame_idx:04}.ppm"), &rgb, width, height)?;
                frame_idx += 1;
            }
            Ok(None) => {
                // decoder is not ready to provide an image
                continue;
            }
            Err(err) => {
                println!("error frame {i}: {err}");
            }
        }
    }

    for image in decoder.flush_remaining()? {
        image.write_rgb8(&mut rgb);
        save_file(&format!("{out}/frame-0{frame_idx:04}.ppm"), &rgb, width, height)?;
        frame_idx += 1;
    }

    Ok(())
}

#[cfg(not(feature = "source"))]
fn main() {}

fn save_file(filename: &str, frame: &[u8], width: usize, height: usize) -> std::result::Result<(), std::io::Error> {
    let mut file = File::create(filename)?;
    file.write_all(format!("P6\n{width} {height}\n255\n").as_bytes())?;
    file.write_all(frame)?;
    Ok(())
}
