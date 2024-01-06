use openh264::decoder::Decoder;
use openh264::encoder::{Encoder, EncoderConfig};
use openh264::{Error, OpenH264API};

fn is_send_sync(_: impl Send + Sync + 'static) {}

#[test]
#[cfg(feature = "source")]
fn decoder_encoder_are_send_sync() -> Result<(), Error> {
    is_send_sync(Decoder::new(OpenH264API::from_source()));
    is_send_sync(Encoder::with_config(OpenH264API::from_source(), EncoderConfig::default()));

    Ok(())
}
