//! Minimal runnable: SyntheticCapture → Pipeline → Runner → dump.
//!
//! cargo run --bin run
//! Prints the run header and a one-line FullStatus.

use obs_rust_bitstream::{
    AudioSource, AudioSourceList, AvStatus, CaptionStatus, FullStatus, KeyframeStatus,
    LiveWeights, ObsEncoderDefaults, OutputInfo, OutputPause, OutputService, Pipeline,
    ProgramStream, Reconnect, RoiStatus, Runner, RunnerDump, ServiceStatus, StreamStart,
    StreamStatus, SyntheticCapture, OUTPUT_AV,
};

fn main() {
    let mut weights = LiveWeights::new();
    let _ = weights.set("fps", 30.0);
    let _ = weights.set("h264.qp", 28.0);

    let pipe = Pipeline::new(64, 64, weights).expect("pipeline");
    let cap = Box::new(SyntheticCapture::new(64, 64));
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
    }
}
