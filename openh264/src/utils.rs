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
                }
                count_0 = 0;
                n += 1;
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

/// Splits an incrementally arriving bitstream into NAL units.
///
/// This searches for `001` marks in a byte stream, and deals with cross-boundary checks when
/// a frame is partially read.
#[derive(Default)]
pub struct NalParser {
    leftover_buffer: Vec<u8>,
    curr_offset: usize,
    last_nal: Option<usize>,
}

impl NalParser {
    /// Creates a new NAL parser.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Tries to retrieve the next NAL unit, if present.
    ///
    /// After feeding new data you should keep calling this method until it returns `None`.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Vec<u8>> {
        if self.leftover_buffer.is_empty() {
            return None;
        }

        if let Some(idx) = self.get_nal_mark() {
            if let Some(last_offset) = self.last_nal {
                // Last mark and current mark found, process packet
                let packet = self.leftover_buffer[last_offset..idx].to_vec();
                self.leftover_buffer = self.leftover_buffer[idx..].to_vec();
                self.last_nal = Some(0);
                self.curr_offset = 2;
                Some(packet)
            } else {
                // Try your luck searching for 0, 0, 1
                // In case there is no 0, 0, 1 in the next try, you get ReadMore
                self.curr_offset = idx + 2;
                self.last_nal = Some(idx);
                None
            }
        } else {
            // No 0, 0, 1 mark here, read more data
            None
        }
    }

    /// Feeds more data to the processor.
    ///
    /// After calling this method, there may be between 0 to M new NAL units present, which you can query with [`Self::next()`].
    pub fn feed(&mut self, buffer: impl AsRef<[u8]>) {
        self.leftover_buffer.extend_from_slice(buffer.as_ref());
    }

    fn get_nal_mark(&self) -> Option<usize> {
        (self.curr_offset..self.leftover_buffer.len() - 2)
            .find(|&i| self.leftover_buffer[i] == 0 && self.leftover_buffer[i + 1] == 0 && self.leftover_buffer[i + 2] == 1)
    }
}

#[cfg(test)]
mod test {
    use super::{nal_units, NalParser};

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
    fn nal_mark_stream_boundary() {
        let v1 = [1, 2, 3, 0];
        let v2 = [0, 1, 104, 238, 56, 127, 0];
        let v3 = [0, 0, 1, 104, 238, 56, 128, 0];

        let mut np = NalParser::new();

        // nothing read, read some data
        assert_eq!(None, np.next());

        np.feed(v1);
        assert_eq!(None, np.next());

        np.feed(v2);
        assert_eq!(None, np.next());

        np.feed(v3);
        assert_eq!(Some(vec![0, 0, 1, 104, 238, 56, 127, 0]), np.next());
        assert_eq!(None, np.next());
    }

    #[test]
    fn nal_mark_empty() {
        let mut np = NalParser::new();
        assert_eq!(None, np.next());
    }

    #[test]
    fn nal_mark_no_mark() {
        let mut np = NalParser::new();
        np.feed([2, 3]);
        assert_eq!(None, np.next());
    }

    #[test]
    fn nal_mark_single_mark() {
        let mut np = NalParser::new();
        np.feed([0, 0, 1]);
        assert_eq!(None, np.next());
    }

    #[test]
    fn nal_mark_multiple_marks_same_vec() {
        let mut np = NalParser::new();
        np.feed([1, 2, 3, 4, 5, 0, 0, 1, 22, 33, 44, 0, 0, 0, 1, 0, 5, 6, 7, 0, 0, 1, 7, 8, 9]);
        assert_eq!(None, np.next());
        assert_eq!(Some(vec![0, 0, 1, 22, 33, 44, 0]), np.next());
        assert_eq!(Some(vec![0, 0, 1, 0, 5, 6, 7]), np.next());
        assert_eq!(None, np.next());
    }

    #[test]
    fn nal_mark_multiple_marks() {
        let mut np = NalParser::new();

        np.feed([0, 0, 1, 2, 3, 4, 0, 0, 1]);
        assert_eq!(None, np.next());
        assert_eq!(Some(vec![0, 0, 1, 2, 3, 4]), np.next());
        assert_eq!(None, np.next());

        np.feed([2, 2, 2]);
        assert_eq!(None, np.next());

        np.feed([3, 3, 3, 0, 0, 1, 5, 6, 7]);
        assert_eq!(Some(vec![0, 0, 1, 2, 2, 2, 3, 3, 3]), np.next());
        assert_eq!(None, np.next());
    }
}
