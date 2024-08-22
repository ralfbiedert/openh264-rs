use std::{
    net::UdpSocket,
    sync::{Arc, RwLock},
};

use openh264::stream::{VideoStreamAction, VideoStreamDecoder};

// This example creates an UDP docket on port 11000 where it expects raw h264 stream.
// It decodes the image into `video_frame`, this can be read elsewhere.
fn main() {
    println!("video stream decoding example");
    // video decoder - do not skip frames (we can't really skip frames, we decode into yuv and if we are skipping we don't encode to rgb)
    let mut vd = VideoStreamDecoder::new(0);
    // video RGB frame, dimensions 960x720
    let video_frame = Arc::new(RwLock::new(make(960 * 720 * 3)));
    // udp recv buffer
    let mut buff: [u8; 32768] = [0; 32768];
    // create udp socket where we receive the UDP stream
    let video_conn = udp_sock("0.0.0.0:11000");
    let mut processed_images = 0;
    loop {
        // get new data
        println!("waiting for data");
        let r = video_conn.recv(&mut buff);
        if r.is_err() {
            println!("udp read error: {}", r.unwrap_err());
            continue;
        }
        let nread = r.unwrap();
        println!("read data: {nread}");
        if nread == 0 {
            continue;
        }

        // let video decoder know what we got from the stream
        let mut video_packet = buff[0..nread].to_vec();
        vd.send_stream(&mut video_packet);
        loop {
            let r = vd.decode_images(&video_frame);
            // println!("r={:?}", r);
            if let VideoStreamAction::ProcessPacket(_) = r {
                processed_images += 1;
                println!("decoded frame no. {processed_images}");
            }
            if r == VideoStreamAction::ReadMore {
                break;
            }
        }
    }
}

fn udp_sock(bind_addr: &str) -> UdpSocket {
    let sock = UdpSocket::bind(bind_addr);
    if sock.is_err() {
        let err_str = format!("can't create udp socket for {bind_addr} : {}", sock.err().unwrap());
        fatal(&err_str);
    }
    sock.unwrap()
}

fn fatal(message: &str) -> ! {
    println!("{}", message);
    std::process::exit(-1);
}

fn make<T>(capacity: usize) -> Vec<T> {
    let mut v = Vec::with_capacity(capacity);
    unsafe {
        v.set_len(capacity);
    }
    v
}
