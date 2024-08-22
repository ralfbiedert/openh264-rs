use std::sync::{Arc, RwLock};

use crate::decoder;

#[derive(PartialEq, Debug)]
pub enum VideoStreamAction {
    CallNext,
    ReadMore,
    ProcessPacket(Vec<u8>),
}

// NalParser parses NAL marks (0, 0, 1) from the byte stream
// It deals with cross-boundary checks when frame is partially
// read.
pub struct NalParser {
    leftover_buffer: Vec<u8>,
    curr_offset: usize,
    last_nal: Option<usize>,
}

impl NalParser {
    pub fn new() -> Self {
        Self {
            leftover_buffer: Vec::new(),
            curr_offset: 0,
            last_nal: None,
        }
    }

    // This is the main function responsible for read more, handling current buffer,
    // returning packet for parsing and buffer truncation (from the start)
    pub fn get_packet(&mut self) -> VideoStreamAction {
        if self.leftover_buffer.is_empty() {
            return VideoStreamAction::ReadMore;
        }

        if let Some(idx) = self.get_nal_mark() {
            if let Some(last_offset) = self.last_nal {
                // Last mark and current mark found, process packet
                let packet = self.leftover_buffer[last_offset..idx].to_vec();
                self.leftover_buffer = self.leftover_buffer[idx..].to_vec();
                self.last_nal = Some(0);
                self.curr_offset = 2;
                return VideoStreamAction::ProcessPacket(packet);
            } else {
                // Try your luck searching for 0, 0, 1
                // In case there is no 0, 0, 1 in the next try, you get ReadMore
                self.curr_offset = idx + 2;
                self.last_nal = Some(idx);
                return VideoStreamAction::CallNext;
            }
        } else {
            // No 0, 0, 1 mark here, read more data
            return VideoStreamAction::ReadMore;
        }
    }

    pub fn send_stream(&mut self, buffer: &mut Vec<u8>) {
        self.leftover_buffer.append(buffer);
    }

    fn get_nal_mark(&self) -> Option<usize> {
        for i in self.curr_offset..self.leftover_buffer.len() - 2 {
            if self.leftover_buffer[i] == 0 && self.leftover_buffer[i + 1] == 0 && self.leftover_buffer[i + 2] == 1 {
                return Some(i);
            }
        }
        return None;
    }
}

#[derive(Debug)]
struct VideoStreamDecoderProps {
    skip_frames: usize,
    frame_no: usize,
    packet_no: usize,
    packet_decode_ok: usize,
}

// Video stream decoder can decode h264 from byte stream received over network
pub struct VideoStreamDecoder {
    decoder: decoder::Decoder,
    props: VideoStreamDecoderProps,
    np: NalParser,
}

impl VideoStreamDecoder {
    pub fn new(skip_frames: usize) -> Self {
        Self {
            props: VideoStreamDecoderProps {
                skip_frames,
                frame_no: 0,
                packet_no: 0,
                packet_decode_ok: 0,
            },
            decoder: decoder::Decoder::new().expect("can't create h264 decoder"),
            np: NalParser::new(),
        }
    }

    pub fn send_stream(&mut self, buffer: &mut Vec<u8>) {
        self.np.send_stream(buffer);
    }

    // This is the main function responsible for decoding images.
    // You have to pass read write lock reference to the *pre-allocated* array where
    // this function update the frames in RGB.
    //
    // This function returns `StreamAction`:
    //  * CallNext - do next call to this function without reading more
    //  * ReadMore - you have to read more data
    //  * ProcessPacket - return what we processed
    pub fn decode_images(&mut self, target_image: &Arc<RwLock<Vec<u8>>>) -> VideoStreamAction {
        let r = self.np.get_packet();
        match r {
            VideoStreamAction::ProcessPacket(img) => {
                self.props.packet_no += 1;
                let skip_frame = self.props.skip_frames != 0 && self.props.frame_no % self.props.skip_frames != 0;

                if let Ok(maybe_yuv) = self.decoder.decode(&img) {
                    self.props.packet_decode_ok += 1;

                    if let Some(yuv) = maybe_yuv {
                        if !skip_frame {
                            let mut g = target_image.write().unwrap();
                            yuv.write_rgb8(&mut g);
                            drop(g);
                        }
                        self.props.frame_no += 1;
                    }
                }
                VideoStreamAction::ProcessPacket(img)
            }

            _ => r,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use crate::stream::VideoStreamDecoder;

    use super::NalParser;

    #[test]
    fn decode_h264_frame() {
        let mut v1 = vec![1, 2, 3, 0];
        let mut v2 = vec![0, 1, 103, 77, 64, 40, 149, 160, 60, 5, 185, 0];
        let mut v3 = vec![0, 0, 1, 104, 238, 56, 128, 0];
        let mut vd = VideoStreamDecoder::new(3);
        let video_frame = Arc::new(RwLock::new(make(960 * 720 * 3)));
        let image_rw_lock = &video_frame;
        assert_eq!(super::VideoStreamAction::ReadMore, vd.decode_images(image_rw_lock));
        vd.np.send_stream(&mut v1);
        assert_eq!(super::VideoStreamAction::ReadMore, vd.decode_images(image_rw_lock));
        vd.np.send_stream(&mut v2);
        assert_eq!(super::VideoStreamAction::CallNext, vd.decode_images(image_rw_lock));
        assert_eq!(super::VideoStreamAction::ReadMore, vd.decode_images(image_rw_lock));
        vd.np.send_stream(&mut v3);
        assert_eq!(
            super::VideoStreamAction::ProcessPacket(vec![0, 0, 1, 103, 77, 64, 40, 149, 160, 60, 5, 185, 0]),
            vd.decode_images(image_rw_lock)
        );
        assert_eq!(1, vd.props.packet_decode_ok);
    }

    #[test]
    fn nal_mark_stream_boundary() {
        //XXX: [0, 0, 1, 103, 77, 64, 40, 149, 160, 60, 5, 185, 0]
        //XXX: [0, 0, 1, 104, 238, 56, 128, 0]
        let mut v1 = vec![1, 2, 3, 0];
        let mut v2 = vec![0, 1, 104, 238, 56, 128, 0];
        let mut v3 = vec![0, 0, 1, 104, 238, 56, 128, 0];

        let mut np = NalParser::new();
        // nothing read, read some data
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        assert_eq!(None, np.last_nal);
        np.send_stream(&mut v1);

        // no sign of 0, 0, 1 mark, read more
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        np.send_stream(&mut v2);

        // First 0, 0, 1 mark found at offset 3
        assert_eq!(super::VideoStreamAction::CallNext, np.get_packet());
        assert_eq!(Some(3), np.last_nal);

        // However no follow-up mark found till the end of current stream, hence, read more
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        np.send_stream(&mut v3);

        // now the packet it complete, process it
        assert_eq!(
            super::VideoStreamAction::ProcessPacket(vec![0, 0, 1, 104, 238, 56, 128, 0]),
            np.get_packet()
        );
        assert_eq!(Some(0), np.last_nal);

        // However no follow-up mark found till the end of current stream, hence, read more
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
    }

    #[test]
    fn nal_mark_empty() {
        let mut np = NalParser::new();
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        assert_eq!(None, np.last_nal);
    }

    #[test]
    fn nal_mark_no_mark() {
        let mut np = NalParser::new();
        np.send_stream(&mut vec![2, 3]);
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        assert_eq!(None, np.last_nal);
    }

    #[test]
    fn nal_mark_single_mark() {
        let mut np = NalParser::new();
        np.send_stream(&mut vec![0, 0, 1]);
        assert_eq!(super::VideoStreamAction::CallNext, np.get_packet());
        assert_eq!(Some(0), np.last_nal);
    }

    #[test]
    fn nal_mark_multiple_marks_same_vec() {
        let mut np = NalParser::new();
        np.send_stream(&mut vec![
            1, 2, 3, 4, 5, 0, 0, 1, 22, 33, 44, 0, 0, 0, 1, 0, 5, 6, 7, 0, 0, 1, 7, 8, 9,
        ]);
        assert_eq!(super::VideoStreamAction::CallNext, np.get_packet());
        assert_eq!(Some(5), np.last_nal);
        assert_eq!(
            super::VideoStreamAction::ProcessPacket(vec![0, 0, 1, 22, 33, 44, 0]),
            np.get_packet()
        );
        assert_eq!(
            super::VideoStreamAction::ProcessPacket(vec![0, 0, 1, 0, 5, 6, 7]),
            np.get_packet()
        );
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
    }

    #[test]
    fn nal_mark_multiple_marks() {
        let mut np = NalParser::new();
        np.send_stream(&mut vec![0, 0, 1, 2, 3, 4, 0, 0, 1]);
        assert_eq!(super::VideoStreamAction::CallNext, np.get_packet());
        assert_eq!(Some(0), np.last_nal);
        assert_eq!(super::VideoStreamAction::ProcessPacket(vec![0, 0, 1, 2, 3, 4]), np.get_packet());
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        assert_eq!(Some(0), np.last_nal);
        np.send_stream(&mut vec![2, 2, 2]);
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        assert_eq!(Some(0), np.last_nal);
        np.send_stream(&mut vec![3, 3, 3, 0, 0, 1, 5, 6, 7]);
        assert_eq!(
            super::VideoStreamAction::ProcessPacket(vec![0, 0, 1, 2, 2, 2, 3, 3, 3]),
            np.get_packet()
        );
        assert_eq!(Some(0), np.last_nal);
        assert_eq!(super::VideoStreamAction::ReadMore, np.get_packet());
        assert_eq!(Some(0), np.last_nal);
    }

    fn make<T>(capacity: usize) -> Vec<T> {
        let mut v = Vec::with_capacity(capacity);
        unsafe {
            v.set_len(capacity);
        }
        v
    }
}
