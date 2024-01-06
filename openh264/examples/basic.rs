use openh264::decoder::Decoder;
use openh264::{Error, OpenH264API};

#[cfg(feature = "source")]
fn main() -> Result<(), Error> {
    let h264_packets = &include_bytes!("../tests/data/multi_512x512.h264")[..];
    let mut rgb = [0; 512 * 512 * 3];

    let api = OpenH264API::from_source();
    let mut decoder = Decoder::new(api)?;
    let image = decoder.decode(h264_packets)?.ok_or_else(|| Error::msg("Must have image"))?;

    image.write_rgb8(&mut rgb);

    Ok(())
}

#[cfg(not(feature = "source"))]
fn main() {}
