//! Encoder choice demo.

use obs_rust_bitstream::encoder_select::EncoderSelect;
use obs_rust_bitstream::h264_codec::H264Codec;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;

fn main() {
    let choice = EncoderSelect::choose();
    println!("selected: {}", choice.as_str());
    let mut frame = VideoFrame::new(VideoFormat::Y800, 16, 16).expect("frame");
    frame.planes[0].fill(40);
    let weights = LiveWeights::new();
    let mut enc = H264Codec::new(16, 16, &weights).expect("enc");
    println!("codec path: {}", enc.choice().as_str());
    let bytes = enc.encode(&frame).expect("encode");
    println!("encoded {} bytes", bytes.len());
}