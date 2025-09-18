#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![doc = include_str!("../../README.md")]

mod error;
mod time;
mod utils;

pub mod decoder;
pub mod encoder;
pub mod formats;

pub use error::Error;
pub use time::Timestamp;
pub use utils::{NalParser, nal_units};

pub use openh264_sys2::DynamicAPI as OpenH264API;
