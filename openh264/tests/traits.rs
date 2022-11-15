use openh264::decoder::Decoder;
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::Error;

fn is_send_sync(_: impl Send + Sync + 'static) {}

#[test]
fn decoder_encoder_are_send_sync() -> Result<(), Error> {
    is_send_sync(Decoder::new());
    is_send_sync(Encoder::with_config(EncoderConfig::default()));

    Ok(())
}
