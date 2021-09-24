use openh264::decoder::Decoder;
use openh264::Error;

fn main() -> Result<(), Error> {
    let h264_packets = &include_bytes!("../tests/data/multi_512x512.h264")[..];
    let mut rgb = [0; 512 * 512 * 3];

    let mut decoder = Decoder::new()?;
    let image = decoder.decode(h264_packets)?;

    image.write_rgb8(&mut rgb)?;

    Ok(())
}
