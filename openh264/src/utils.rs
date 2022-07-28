use crate::Error;
use std::convert::TryInto;
use std::mem::size_of;

// How many `0` we have to observe before a `1` means NAL.
const NAL_MIN_0_COUNT: usize = 2;

/// Given a stream, finds the index of the nth NAL start.
#[inline]
fn nth_nal_index(stream: &[u8], nth: usize) -> Option<usize> {
    let mut count_0 = 0;
    let mut n = 0;

    for (i, byte) in stream.iter().enumerate() {
        match byte {
            0 => count_0 += 1,
            1 if count_0 >= NAL_MIN_0_COUNT => {
                if n == nth {
                    return Some(i - NAL_MIN_0_COUNT);
                } else {
                    count_0 = 0;
                    n += 1;
                }
            }
            _ => count_0 = 0,
        }
    }

    None
}

/// Splits a bitstream into NAL units.
///
/// This function is useful if you happen to have a H.264 bitstream and want to decode it frame by frame: You
/// apply this function to the underlying stream and run your decoder on each returned slice, preferably
/// ignoring isolated decoding errors.
///
/// In detail, given a bitstream like so (`001` being the NAL start prefix code):
///
/// ```text
/// ......001.........001......001.....
/// ```
///
/// This function will return an iterator returning packets:
/// ```text
///      [001.......][001....][001.....]
/// ```
///
/// In other words, any incomplete data at the beginning of the buffer is skipped,
/// NAL units in the middle are split at their boundaries, the last packet is returned
/// as-is.
///
pub fn nal_units(mut stream: &[u8]) -> impl Iterator<Item = &[u8]> {
    std::iter::from_fn(move || {
        let first = nth_nal_index(stream, 0);
        let next = nth_nal_index(stream, 1);

        match (first, next) {
            (Some(f), Some(n)) => {
                let rval = &stream[f..n];
                stream = &stream[n..];
                Some(rval)
            }
            (Some(f), None) => {
                let rval = &stream[f..];
                stream = &stream[f + NAL_MIN_0_COUNT..];
                Some(rval)
            }
            _ => None,
        }
    })
}

/// Converts a bit stream with length but without start codes to one without length but start codes.
///
/// When parsing MP4 files with [mp4](https://crates.io/crates/mp4) you might get a `Mp4Sample` that comes
/// without start codes, but are prefixed with length information instead. For OpenH264 to read them, they
/// must be converted.
///
/// In detail, given a bitstream like so (`001` being the NAL start prefix code, `LLLL` length bytes):
///
/// ```text
/// LLLL..........LLLL............
/// ```
///
/// This function will modify a vector to contain
///
/// ```text
/// [001.........][001...........]
/// ```
///
/// If a slice could not be decoded, e.g., because of a mismatch of slice length and indicated length,
/// the final failing block will be ignored.
pub fn to_bitstream_with_001<T: BitstreamLength>(mut stream: &[u8], out: &mut Vec<u8>) {
    out.clear();

    while let Ok((skip, payload)) = T::read(stream) {
        out.extend_from_slice(&[0, 0, 1]);
        out.extend_from_slice(payload);

        stream = &stream[skip..];
    }
}

/// Utility trait to read a bit stream without start prefix but with an encoded
/// length of the given type.
pub trait BitstreamLength {
    /// First reads the length, then returns as many bytes as indicated.
    ///
    /// Returns the total length of the type and read data (e.g., `4+x` for `u32`)
    /// as well as the indicated data (e.g., `&[1, 2, 3, ..., x]`).
    fn read(data: &[u8]) -> Result<(usize, &[u8]), Error>;
}

macro_rules! impl_bitstream_length {
    ($t:ty) => {
        impl BitstreamLength for $t {
            fn read(data: &[u8]) -> Result<(usize, &[u8]), Error> {
                const SIZE: usize = size_of::<$t>();

                if data.len() < SIZE {
                    return Err(Error::msg("Unable to read length."));
                }

                let len = <$t>::from_be_bytes(
                    (&data[0..SIZE])
                        .try_into()
                        .map_err(|_| Error::msg("Unable to get slice"))?,
                )
                .try_into()
                .expect("Must be able to convert from usize to requested type");

                if len + SIZE > data.len() {
                    return Err(Error::msg("Resulting slice was too short for indicated length."));
                }

                Ok((len + SIZE, &data[SIZE..][..len]))
            }
        }
    };
}

impl_bitstream_length!(u8);
impl_bitstream_length!(u16);
impl_bitstream_length!(u32);

#[cfg(test)]
mod test {
    use super::nal_units;
    use crate::utils::to_bitstream_with_001;

    #[test]
    fn splits_at_nal() {
        let stream = [];
        assert!(nal_units(&stream).next().is_none());

        let stream = [2, 3];
        assert!(nal_units(&stream).next().is_none());

        let stream = [0, 0, 1];
        assert_eq!(nal_units(&stream).next().unwrap(), &[0, 0, 1]);

        let stream = [0, 0, 1, 2];
        assert_eq!(nal_units(&stream).next().unwrap(), &[0, 0, 1, 2]);

        let stream = [0, 0, 1, 2, 0, 0, 1];
        let mut split = nal_units(&stream);
        assert_eq!(split.next().unwrap(), &[0, 0, 1, 2]);
        assert_eq!(split.next().unwrap(), &[0, 0, 1]);
        assert!(split.next().is_none());

        let stream = [0, 0, 0, 0, 0, 1, 2, 0, 0, 1];
        let mut split = nal_units(&stream);
        assert_eq!(split.next().unwrap(), &[0, 0, 1, 2]);
        assert_eq!(split.next().unwrap(), &[0, 0, 1]);
        assert!(split.next().is_none());

        let stream = [0, 0, 0, 0, 0, 1, 2, 0, 0];
        let mut split = nal_units(&stream);
        assert_eq!(split.next().unwrap(), &[0, 0, 1, 2, 0, 0]);
        assert!(split.next().is_none());

        let stream = [0, 0, 0, 0, 0, 1, 2, 0, 0, 1, 2, 3, 0, 0, 1];
        let mut split = nal_units(&stream);
        assert_eq!(split.next().unwrap(), &[0, 0, 1, 2]);
        assert_eq!(split.next().unwrap(), &[0, 0, 1, 2, 3]);
        assert_eq!(split.next().unwrap(), &[0, 0, 1]);
        assert!(split.next().is_none());
    }

    #[test]
    fn bitstream_to_prefixed() {
        let mut vec = Vec::new();
        to_bitstream_with_001::<u32>(&[0, 0, 0, 1, 5, 0, 0, 0, 2, 6, 6], &mut vec);
        assert_eq!(vec.as_slice(), &[0, 0, 1, 5, 0, 0, 1, 6, 6]);

        to_bitstream_with_001::<u32>(&[0, 0, 0, 1, 5, 0, 0, 0, 2, 6, 6, 255, 255], &mut vec);
        assert_eq!(vec.as_slice(), &[0, 0, 1, 5, 0, 0, 1, 6, 6]);
    }
}
