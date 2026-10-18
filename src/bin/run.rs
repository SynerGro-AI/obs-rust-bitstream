//! Minimal runnable: SyntheticCapture → Pipeline → Runner → dump.
//!
//! cargo run --bin run
//! Prints the run header and a one-line FullStatus.

use obs_rust_bitstream::{
    AudioSource, AudioSourceList, AvStatus, CaptionStatus, FullStatus, H264Codec,
    KeyframeStatus, LiveWeights, ObsEncoderDefaults, OutputInfo, OutputPause, OutputService, Pipeline,
    ProgramStream, Reconnect, RoiStatus, Runner, RunnerDump, ServiceStatus, StreamStart,
    StreamStatus, SyntheticCapture, PayloadDump, RtmpOutput, CanvasEffect,
    CaptureSource, OUTPUT_AV,
};

fn main() {
    let mut weights = LiveWeights::new();
    let _ = weights.set("fps", 30.0);
    let _ = weights.set("h264.qp", 28.0);

    let pipe = Pipeline::new(64, 64, weights).expect("pipeline");
    let (src, live) = CaptureSource::best(64, 64);
    println!("capture={} live={}", src.kind(), live);
    let cap: Box<dyn obs_rust_bitstream::Capture> = match src {
        CaptureSource::Synthetic(c) => Box::new(c),
        CaptureSource::File(c) => Box::new(c),
        CaptureSource::Platform(c) => c,
    };
    let mut audio = AudioSourceList::new();
    let _ = audio.add("tone", AudioSource::synthetic(48000, 1));
    let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).expect("output");
    out.start();
    let out_for_stream = out.clone();

    let mut runner = Runner::new(pipe, cap, audio, out);
    let buf = RunnerDump::run_and_write(&mut runner, 8).expect("dump");
    let header = RunnerDump::read(&buf).expect("header");

    let status = FullStatus::from(
        &obs_rust_bitstream::PathStatus {
            held: 0,
            dropped: 0,
            latency_ns: 0,
        },
        &KeyframeStatus {
            required: 0,
            done: 0,
            complete: true,
        },
        &CaptionStatus {
            lines: 0,
            duration: 0.0,
        },
        &RoiStatus {
            count: 0,
            supported: false,
        },
        &Reconnect::new(),
        &OutputPause::new(),
        &AvStatus {
            pixels: 64 * 64,
            rate: 48000,
            channels: 1,
            matches: true,
        },
        &ServiceStatus {
            name: "file".to_string(),
            id: "dump".to_string(),
        },
    );

    println!(
        "frames={} bytes={}",
        header.frames, header.bytes
    );
    println!("{}", status.one_line());

    let mut stream = ProgramStream::new();
    let mut svc = OutputService::new();
    let mut w2 = LiveWeights::new();
    if let Ok(defaults) = StreamStart::start(
        &mut stream,
        &mut svc,
        &out_for_stream,
        &mut w2,
        "file",
        "dump",
    ) {
        let ss = StreamStatus::from(&stream, &ServiceStatus::from(&svc), &defaults);
        println!("{}", ss.one_line());
        println!(
            "obs_bitrate={} obs_crf={} obs_preset={}",
            ObsEncoderDefaults::default().bitrate_kbps,
            ObsEncoderDefaults::default().crf,
            ObsEncoderDefaults::default().preset
        );
        let path = std::path::Path::new("stream_status.txt");
        if std::fs::write(path, ss.one_line()).is_ok() {
            println!("saved={}", path.display());
        }
        let mut codec = H264Codec::new(64, 64, &w2).expect("codec");
        let mut cap2 = SyntheticCapture::new(64, 64);
        if let Ok(pk) = PayloadDump::write(&mut codec, &mut cap2, 8) {
            let pp = std::path::Path::new("stream_packets.bin");
            if std::fs::write(pp, &pk).is_ok() {
                if let Some(h) = PayloadDump::header(&pk) {
                    println!("packets={} packet_bytes={} saved={}", h.count, h.bytes, pp.display());
                    if let Ok(loaded) = std::fs::read(pp) {
                        if let Some(h2) = PayloadDump::header(&loaded) {
                            println!("loaded_packets={} loaded_bytes={}", h2.count, h2.bytes);
                        }
                        if let Ok(st) = RtmpOutput::send("live.twitch.tv", "demo", &loaded) {
                            println!("rtmp {}", st.one_line());
                        }
                    }
                }
            }
        }
        // Optional canvas effect. Default path above is unchanged.
        let mut we = LiveWeights::new();
        let _ = we.set("fps", 30.0);
        let _ = we.set("h264.qp", 28.0);
        let _ = we.set("effect.brightness", 0.2);
        if let Ok(mut pipe) = Pipeline::new(16, 16, we) {
            let mut cap = SyntheticCapture::new(16, 16);
            let mut audio = AudioSourceList::new();
            let _ = audio.add("tone", AudioSource::synthetic(48000, 1));
            let mut out = OutputInfo::new("fx", "Effects", OUTPUT_AV).expect("out");
            out.start();
            let _ = pipe.tick(&mut cap, &mut audio, &mut out, 0);
            let mean = CanvasEffect::mean(pipe.canvas());
            println!("canvas_mean={:.3}", mean);
        }
    }
}
