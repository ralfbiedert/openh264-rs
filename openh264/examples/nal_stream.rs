use openh264::{
    decoder,
    stream::{NalParser, VideoStreamAction},
};

fn main() {
    println!("NAL parser direct usage example");
    // create NAL parser
    let mut np = NalParser::new();

    // let's read some data, but no NAL found
    let mut buffer = vec![1, 2, 3, 0, 0];
    np.send_stream(&mut buffer);
    let r = np.get_packet();
    println!("  -> no NAL mark, read more: **{:?}**", r);

    // read more data, and together this will create the first NAL mark
    let mut buffer = vec![1, 103, 77, 64, 40, 149, 160, 60, 5, 185, 0];
    np.send_stream(&mut buffer);
    let r = np.get_packet();
    println!("  -> found first NAL mark, continue to catch the next one: **{:?}**", r);

    // without reading the buffer we try to search in the current one
    let r = np.get_packet();
    println!(
        "  -> unfortunately, no NAL mark withing this buffer found, read more: **{:?}**",
        r
    );

    // so we read more and finally
    let mut buffer = vec![0, 0, 1, 104, 238, 56, 128, 0];
    np.send_stream(&mut buffer);
    let r = np.get_packet();
    println!("  -> finnaly we can process the packet: {:?}", r);

    // let's do the processing:
    if let VideoStreamAction::ProcessPacket(image_data) = r {
        let mut decoder = decoder::Decoder::new().expect("can't create h264 decoder");
        if let Ok(maybe_yuv) = decoder.decode(&image_data) {
            println!(
                "  -> packet decoding ok - but I believe there's no yuv inside this one: is_some? {}",
                maybe_yuv.is_some()
            );
        }
    }
}
