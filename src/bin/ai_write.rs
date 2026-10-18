//! Combined write with CC0 audio.

use obs_rust_bitstream::audio_sync::AudioSync;
use obs_rust_bitstream::audio_timeline::AudioTimeline;
use obs_rust_bitstream::effected_write::EffectedWrite;
use obs_rust_bitstream::encoder_controls::EncoderControls;
use obs_rust_bitstream::live_weights::LiveWeights;
use obs_rust_bitstream::frame_pacer::FramePacer;
use obs_rust_bitstream::status_report::StatusReport;
use obs_rust_bitstream::status_history::StatusHistory;
use obs_rust_bitstream::capture_rate::CaptureRate;
use obs_rust_bitstream::capture::Capture;
use obs_rust_bitstream::file_capture::FileCapture;
use obs_rust_bitstream::capture_select::CaptureSource;
use obs_rust_bitstream::timestamp_stamp::TimestampStamp;
use obs_rust_bitstream::report::Report;
use obs_rust_bitstream::rate_compare::RateCompare;
use obs_rust_bitstream::write_timing::WriteTiming;
use obs_rust_bitstream::encode_cancel::EncodeCancel;
use obs_rust_bitstream::encode_progress::EncodeProgress;
use obs_rust_bitstream::scene_effect::SceneEffect;
use obs_rust_bitstream::sound_cc0::SoundCc0;
use obs_rust_bitstream::system_status::SystemStatus;
use obs_rust_bitstream::timed_sequence::TimedSequence;
use obs_rust_bitstream::video_fourcc::VideoFormat;
use obs_rust_bitstream::video_frame::VideoFrame;
use std::fs::File;
use std::io::Write;

fn main() {
    let bytes = std::fs::read("/tmp/bell1.wav").expect("read");
    let data = SoundCc0::load(&bytes).expect("parse");
    let mut timeline = AudioTimeline::new();
    timeline.add("bell", 0.0).expect("add");
    timeline.load_wav("bell", &data).expect("load");
    let mut timed = TimedSequence::new(30.0).expect("fps");
    for _ in 0..8 {
        let f = VideoFrame::new(VideoFormat::Y800, 16, 16).expect("frame");
        timed.push(f);
    }
    let mut sync = AudioSync::new(timed.clone());
    *sync.audio_mut() = timeline;
    let frames: Vec<_> = (0..timed.len())
        .filter_map(|i| timed.frames().get(i).ok().cloned())
        .collect();
    let mut controls = EncoderControls::new().expect("controls");
    let mut weights = LiveWeights::new();
    controls.set_quality(30.0, &mut weights).expect("quality");
    weights.set("pipe.bright", 20.0).expect("bright");
    let mut effect = SceneEffect::new();
    effect.add("bright");
    let bundle_file = File::create("out.avbn").expect("bundle");
    let status_file = File::create("status.txt").expect("status");
    let timing = WriteTiming::start();
    let mut progress = EncodeProgress::new(frames.len());
    let cancel = EncodeCancel::new();
    let bundle = EffectedWrite::write_effected_bundle_cancellable(
        &frames,
        &effect,
        &controls,
        &mut weights,
        &sync,
        &mut progress,
        &cancel,
        bundle_file,
        status_file,
    )
    .expect("write");
    println!(
        "wrote out.avbn (video={} audio={}) progress={:.0}% time={:.1}ms fps={:.0}",
        bundle.video_len(),
        bundle.audio_len(),
        progress.fraction() * 100.0,
        timing.elapsed_ms(),
        timing.fps(frames.len())
    );
    let mut status = SystemStatus::from_bundle(&bundle);
    status.set_encode_ms(timing.elapsed_ms());
    println!("status: {}", status.summary());
    let status_file = File::create("status.txt").expect("status");
    status.write(status_file).expect("write status");
    println!("wrote status.txt with time");
    let (small, large) = RateCompare::fps(16, 32).expect("compare");
    println!("rate: 16x16={:.0} fps  32x32={:.0} fps", small, large);
    let (a, b, c) = RateCompare::three(8, 16, 32).expect("three");
    println!("three: 8={:.0} 16={:.0} 32={:.0}", a, b, c);
    let rate_file = File::create("rate.txt").expect("rate");
    RateCompare::write_three(8, 16, 32, rate_file).expect("write rate");
    println!("wrote rate.txt");
    let report_file = File::create("report.txt").expect("report");
    Report::write(&status, 8, 16, 32, report_file).expect("report");
    println!("wrote report.txt");
    let report_text = std::fs::read_to_string("report.txt").expect("read report");
    println!("report matches: {}", Report::matches_status(&report_text, &status));
    let mut stamped = frames[0].clone();
    TimestampStamp::stamp(&mut stamped, 0, 0, 60, 255).expect("stamp");
    println!("stamped frame with 00:01:00");
    TimestampStamp::stamp_corner(&mut stamped, 120, 200).expect("corner");
    println!("stamped corner with 00:02:00");
    println!("full: {}", TimestampStamp::format_full(2026, 10, 17, 60));
    println!("now: {}", TimestampStamp::now());
    println!("frame: {}", TimestampStamp::format_frame(60, 3));
    println!("progress: {}", TimestampStamp::format_progress(3, 10));
    println!("lum white: {}", TimestampStamp::lum("white"));
    weights.set("stamp.lum", 128.0).expect("lum");
    println!("stamp.lum set to 128");
    weights.set("stamp.opacity", 0.5).expect("opacity");
    println!("stamp.opacity set to 0.5 (blend={})", TimestampStamp::blend(128, 0.5));
    let mut src = CaptureSource::best();
    let mut captured_frames = Vec::new();
    for _ in 0..4 {
        if let Some(f) = src.frame() {
            captured_frames.push(f);
        }
    }
    println!("captured {} frames from best", captured_frames.len());
    let cap_bundle = File::create("captured.avbn").expect("cap");
    let cap_status = File::create("captured_status.txt").expect("status");
    let _ = EffectedWrite::write_stamped_bundle(
        &captured_frames,
        30.0,
        &controls,
        &mut weights,
        &sync,
        cap_bundle,
        cap_status,
    )
    .expect("encode captured");
    println!("wrote captured.avbn");
    let cap_size = std::fs::metadata("captured.avbn").map(|m| m.len()).unwrap_or(0);
    println!("captured.avbn size: {} bytes", cap_size);
    // Write a tiny raw file and read it back.
    {
        let mut raw = File::create("/tmp/demo.raw").expect("raw");
        raw.write_all(&[42u8; 16]).expect("write");
    }
    let mut file_cap = FileCapture::open("/tmp/demo.raw", 4, 4).expect("file cap");
    let from_file = file_cap.frame().expect("frame");
    println!("file capture: {}x{} first={}", from_file.width, from_file.height, from_file.planes[0][0]);
    let mut sel = CaptureSource::file("/tmp/demo.raw", 4, 4).expect("sel");
    let via_sel = sel.frame().expect("via sel");
    println!("via selector: first={}", via_sel.planes[0][0]);
    // Encode the file frame.
    let mut file_frames = Vec::new();
    if let Some(f) = CaptureSource::file("/tmp/demo.raw", 4, 4).expect("open").frame() {
        file_frames.push(f);
    }
    if !file_frames.is_empty() {
        let fb = File::create("from_file.avbn").expect("fb");
        let fs = File::create("from_file_status.txt").expect("fs");
        let _ = EffectedWrite::write_stamped_bundle(
            &file_frames,
            30.0,
            &controls,
            &mut weights,
            &sync,
            fb,
            fs,
        );
        println!("wrote from_file.avbn");
        let sz = std::fs::metadata("from_file.avbn").map(|m| m.len()).unwrap_or(0);
        println!("from_file.avbn size: {} bytes", sz);
    }
    println!(
        "stamp example: {} {}",
        TimestampStamp::format_frame(0, 0),
        TimestampStamp::format_progress(0, frames.len())
    );
    println!("stamped encode uses frame index");
    let stamped_bundle = File::create("stamped.avbn").expect("stamped");
    let stamped_status = File::create("stamped_status.txt").expect("status");
    let _ = EffectedWrite::write_stamped_bundle(
        &frames,
        30.0,
        &controls,
        &mut weights,
        &sync,
        stamped_bundle,
        stamped_status,
    )
    .expect("stamped write");
    println!("wrote stamped.avbn");
    println!("base time: {}", TimestampStamp::now());
    println!("stamped: {}", TimestampStamp::is_stamped(&stamped, 0, 0, 48));
    status.set_stamped(true);
    status.set_stamp_opacity(0.5);
    status.set_captured_frames(captured_frames.len());
    status.set_source("synthetic");
    status.set_sources_available(CaptureSource::available_kinds().len());
    status.set_timestamp(SystemStatus::now());
    println!("status with stamp: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with available count");
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with source");
    println!("sources: {}", CaptureSource::kinds().join(", "));
    for k in CaptureSource::kinds() {
        println!("  {}: {}", k, CaptureSource::available(k));
    }
    println!("available: {}", CaptureSource::available_kinds().join(", "));
    let mut best = CaptureSource::best();
    let f = best.frame().expect("best");
    println!("best: {}x{}", f.width, f.height);
    println!("source: {}", best.summary());
    let rate = CaptureRate::measure(&mut best, 8);
    println!("capture rate: {:.0} fps", rate);
    let encode_fps = timing.fps(frames.len());
    println!(
        "capture faster than encode: {}",
        CaptureRate::faster_than(rate, encode_fps)
    );
    println!("bottleneck: {}", CaptureRate::bottleneck(rate, encode_fps));
    println!("recommend: {}", CaptureRate::recommend(rate, encode_fps));
    println!("healthy: {}", CaptureRate::healthy(rate, encode_fps, 24.0));
    status.set_healthy(CaptureRate::healthy(rate, encode_fps, 24.0));
    status.set_source_summary(&best.summary());
    status.set_capture_fps(rate);
    status.set_capture_faster(CaptureRate::faster_than(rate, encode_fps));
    status.set_bottleneck(CaptureRate::bottleneck(rate, encode_fps));
    status.set_recommendation(CaptureRate::recommend(rate, encode_fps));
    println!("status with health: {}", status.summary());
    println!("one line: {}", status.one_line());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    let read_back = SystemStatus::read_one_line(File::open("status.txt").expect("open")).unwrap_or_default();
    println!("read back: {}", read_back);
    println!("matches: {}", status.matches_one_line(&read_back));
    let mut history = StatusHistory::new(3);
    history.push(&status.one_line());
    println!("history: {} last={}", history.len(), history.last().unwrap_or(""));
    let hist_file = File::create("history.txt").expect("hist");
    history.write(hist_file).expect("write hist");
    println!("wrote history.txt");
    let loaded = StatusHistory::read(3, File::open("history.txt").expect("open")).expect("load");
    println!("loaded: {} last={}", loaded.len(), loaded.last().unwrap_or(""));
    println!("history matches: {}", history.matches_last(&loaded));
    let header = StatusHistory::read_header(File::open("history.txt").expect("open")).unwrap_or_default();
    println!("header: {}", header);
    println!("header matches: {}", history.matches_header(&header));
    history.append_to("history.txt").expect("append");
    let grown = StatusHistory::read(10, File::open("history.txt").expect("open")).expect("grown");
    println!("history grown: {} lines", grown.len());
    println!("oldest: {}", grown.first().unwrap_or(""));
    println!("lines: {}", grown.lines().len());
    println!("healthy lines: {}", grown.filter("healthy").len());
    println!("latest healthy: {}", grown.latest_healthy().unwrap_or("none"));
    println!("streak: {}", grown.streak());
    println!("best streak: {}", grown.best_streak());
    println!("at record: {}", grown.at_record());
    println!("word: {}", grown.word());
    println!("color: {}", grown.color());
    println!("report: {}", grown.report());
    println!("dashboard: {}", grown.dashboard());
    println!("dashboard at: {}", grown.dashboard_at(SystemStatus::now()));
    std::fs::write("dashboard.txt", grown.dashboard_at(SystemStatus::now())).expect("dash");
    println!("wrote dashboard.txt");
    let dash = StatusHistory::read_dashboard(File::open("dashboard.txt").expect("open")).unwrap_or_default();
    println!("read dashboard: {}", dash);
    println!("timestamp: {}", StatusHistory::timestamp_of(&dash));
    println!("age: {}s", StatusHistory::age(&dash, SystemStatus::now()));
    println!("fresh: {}", StatusHistory::fresh(&dash, SystemStatus::now(), 60));
    let report = StatusReport::from(&grown);
    println!("report: {}", report.summary());
    let rep_file = File::create("report.txt").expect("rep");
    report.write(rep_file).expect("write rep");
    println!("wrote report.txt");
    let loaded = StatusReport::read(File::open("report.txt").expect("open")).unwrap_or_default();
    println!("loaded report: {}", loaded);
    println!("report matches: {}", report.matches(&loaded));
    println!("report json: {}", report.to_json());
    std::fs::write("report.json", report.to_json()).expect("rep json");
    println!("wrote report.json");
    let mut pacer = FramePacer::new(30.0);
    println!("pacer target: {} fps", pacer.target());
    status.set_target_fps(pacer.target());
    println!("status target: {}", status.summary().contains("target=30"));
    println!("hit target: {}", FramePacer::hit(30.0, rate, 0.5));
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with target");
    println!("pacer ready: {}", pacer.ready());
    let mut src = CaptureSource::best();
    let mut unlimited = FramePacer::new(0.0);
    let paced = unlimited.collect(&mut src, 4);
    println!("paced frames: {}", paced.len());
    if !paced.is_empty() {
        let pb = File::create("paced.avbn").expect("pb");
        let ps = File::create("paced_status.txt").expect("ps");
        let _ = EffectedWrite::write_stamped_bundle(
            &paced,
            30.0,
            &controls,
            &mut weights,
            &sync,
            pb,
            ps,
        );
        let sz = std::fs::metadata("paced.avbn").map(|m| m.len()).unwrap_or(0);
        println!("wrote paced.avbn ({} bytes)", sz);
    }
    println!("dashboard matches: {}", grown.matches_dashboard(&dash));
    let mut cleared = history.clone();
    cleared.clear();
    println!("cleared: {}", cleared.is_empty());
    println!("healthy runs: {}", grown.healthy_count());
    println!("health ratio: {:.0}%", grown.health_ratio() * 100.0);
    println!("avg fps: {:.0}", grown.avg_fps());
    let (lo, hi) = grown.fps_range();
    println!("fps range: {:.0} to {:.0}", lo, hi);
    println!("fps spread: {:.0}", grown.fps_spread());
    println!("stable: {}", grown.stable(0.1));
    println!("trend: {}", grown.trend());
    println!("history summary: {}", grown.summary());
    println!("json: {}", grown.to_json());
    println!("json at: {}", grown.to_json_at(SystemStatus::now()));
    std::fs::write("history.json", grown.to_json_at(SystemStatus::now())).expect("json");
    println!("wrote history.json");
    let json = std::fs::read_to_string("history.json").expect("read json");
    let trend = StatusHistory::json_field(&json, "trend").unwrap_or_default();
    println!("json trend: {}", trend);
    println!("json matches: {}", grown.matches_json(&json));
    grown.write_both("history.txt", "history.json").expect("both");
    println!("wrote both files");
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with health");
    status.set_recommendation(CaptureRate::recommend(rate, encode_fps));
    println!("status with recommend: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with recommendation");
    status.set_bottleneck(CaptureRate::bottleneck(rate, encode_fps));
    println!("status with bottleneck: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with bottleneck");
    status.set_capture_faster(CaptureRate::faster_than(rate, encode_fps));
    println!("status with compare: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with comparison");
    status.set_capture_fps(rate);
    println!("status with rate: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with capture rate");
    status.set_source_summary(&best.summary());
    println!("status with summary: {}", status.summary());
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with source summary");
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with capture count");
    let final_status = File::create("status.txt").expect("final");
    status.write(final_status).expect("write final");
    println!("wrote status.txt with stamp flag");
    println!("matches: {}", status.matches_bundle(&bundle));
}