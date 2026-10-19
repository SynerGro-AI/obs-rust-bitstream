//! Minimal runnable: SyntheticCapture → Pipeline → Runner → dump.
//!
//! cargo run --bin run
//! Prints the run header and a one-line FullStatus.

use obs_rust_bitstream::{
    AudioSource, AudioSourceList, AvStatus, CaptionStatus, FullStatus, H264Codec,
    KeyframeStatus, LiveWeights, ObsEncoderDefaults, OutputInfo, OutputPause, OutputService, Pipeline,
    ProgramStream, Reconnect, RoiStatus, Runner, RunnerDump, ServiceStatus, StreamStart,
    StreamStatus, SyntheticCapture, PayloadDump, RtmpOutput, CanvasEffect,
    CaptureSource, RtmpReconnectStatus, RunnerAudio, AudioBackendStatus, RtmpSocket,
    RtmpOpenStatus, ShellRound, LayerComposite, EditRound, EditComposite, GroupComposite, BlendComposite, ShellStatus, ShellFrame, ShellRunner, ShellRtmp, ShellPublish, ShellSocket, ShellSent, ShellLoop, ShellAction, ShellLoopFrame, ShellLoopSocket, ShellSummary, ShellEvent, ShellInput, ShellFilter, ShellSession, DisplayFrame, YouTubeTarget, StreamTarget, Platform, History, Workspace, OUTPUT_AV,
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
    let (audio, audio_live) = RunnerAudio::open(48000, 1);
    let abs = AudioBackendStatus::from(audio_live);
    println!("{}", abs.one_line());
    let ap = std::path::Path::new("audio_backend.txt");
    if std::fs::write(ap, abs.one_line()).is_ok() {
        println!("saved={}", ap.display());
    }
    let mut out = OutputInfo::new("rec", "Recording", OUTPUT_AV).expect("output");
    out.start();
    let out_for_stream = out.clone();

    let mut runner = Runner::new(pipe, cap, audio, out);
    let timed = runner
        .run_for(std::time::Duration::from_millis(10))
        .expect("timed");
    println!("timed_ticks={}", timed);
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
                            let rc = Reconnect::new();
                            let rs = RtmpReconnectStatus::from(&st, &rc);
                            println!("rtmp_retry {}", rs.one_line());
                            let rp = std::path::Path::new("rtmp_retry.txt");
                            if std::fs::write(rp, rs.one_line()).is_ok() {
                                println!("saved={}", rp.display());
                            }
                            // Real socket send. Local listener so the path is proven.
                            let listener = std::net::TcpListener::bind("127.0.0.1:0").ok();
                            if let Some(listener) = listener {
                                if let Ok(addr) = listener.local_addr() {
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    std::thread::spawn(move || {
                                        let _ = tx.send(());
                                        let _ = listener.accept();
                                        let _ = listener.accept();
                                        let _ = listener.accept();
                                    });
                                    let _ = rx.recv_timeout(std::time::Duration::from_millis(200));
                                    std::thread::sleep(std::time::Duration::from_millis(50));
                                    let mut rc = Reconnect::new();
                                    match RtmpSocket::send("127.0.0.1", addr.port(), &loaded, &mut rc) {
                                        Ok(sent) => {
                                            let open = RtmpOpenStatus::from(sent.bytes);
                                            println!("{}", open.one_line());
                                            let op = std::path::Path::new("rtmp_open.txt");
                                            if std::fs::write(op, open.one_line()).is_ok() {
                                                println!("saved={}", op.display());
                                            }
                                        }
                                        Err(_) => println!("socket attempts={}", rc.attempts()),
                                    }
                                    // Shell payload through the same socket path.
                                    if let Some(shell_payload) = {
                                        use obs_rust_bitstream::photo_layers::LayerStack as PhotoLayers;
                                        use obs_rust_bitstream::video_fourcc::VideoFormat;
                                        use obs_rust_bitstream::video_frame::VideoFrame;
                                        let mut s = PhotoLayers::new();
                                        if let Some(mut bg) = VideoFrame::new(VideoFormat::Y800, 16, 16) {
                                            if let Some(p) = bg.planes.get_mut(0) {
                                                for px in p.iter_mut() { *px = 100; }
                                            }
                                            let mut top = bg.clone();
                                            if let Some(p) = top.planes.get_mut(0) {
                                                for px in p.iter_mut() { *px = 200; }
                                            }
                                            let _ = s.add("Background", bg);
                                            let _ = s.add("Layer 1", top);
                                            ShellSocket::payload(&s)
                                        } else {
                                            None
                                        }
                                    } {
                                        let mut rc2 = Reconnect::new();
                                        match RtmpSocket::send("127.0.0.1", addr.port(), &shell_payload, &mut rc2) {
                                            Ok(sent) => println!("shell_socket_sent={}", sent.bytes),
                                            Err(_) => println!("shell_socket_sent=0"),
                                        }
                                    }
                                    // Loop payload on its own listener.
                                    let loop_actions = [
                                        ShellAction::Tool("Brush"),
                                        ShellAction::Size(30),
                                        ShellAction::NewLayer,
                                    ];
                                    if let Some(loop_payload) = ShellLoopSocket::payload(&loop_actions) {
                                        if let Some(loop_listener) = std::net::TcpListener::bind("127.0.0.1:0").ok() {
                                            if let Ok(loop_addr) = loop_listener.local_addr() {
                                                let (ltx, lrx) = std::sync::mpsc::channel();
                                                std::thread::spawn(move || {
                                                    let _ = ltx.send(());
                                                    let _ = loop_listener.accept();
                                                });
                                                let _ = lrx.recv_timeout(std::time::Duration::from_millis(1000));
                                                std::thread::sleep(std::time::Duration::from_millis(200));
                                                let mut rc3 = Reconnect::new();
                                                match RtmpSocket::send("127.0.0.1", loop_addr.port(), &loop_payload, &mut rc3) {
                                                    Ok(sent) => {
                                                        println!("shell_loop_sent={}", sent.bytes);
                                                        let _ = std::fs::write("shell_loop_sent.txt", format!("{}", sent.bytes));
                                                        println!("shell_loop_sent_file=shell_loop_sent.txt");
                                                    }
                                                    Err(_) => println!("shell_loop_sent=0"),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
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
            println!("--- shell ---");
            print!("{}", ShellRound::run());
            use obs_rust_bitstream::edit_target::EditTarget;
            use obs_rust_bitstream::layer_select::LayerSelect;
            use obs_rust_bitstream::ui_view::UiView;
            use obs_rust_bitstream::workspace::Workspace;
            let w = Workspace::new(16, 16);
            let s = obs_rust_bitstream::layer_stack::LayerStack::new();
            println!("{}", UiView::render_with_target(&w, &s, &LayerSelect::new(), &EditTarget::Layer("Layer 1".into())));
            println!("{}", UiView::render_with_target(&w, &s, &LayerSelect::new(), &EditTarget::Group("Group 1".into())));
            // Composite of the showing layers.
            use obs_rust_bitstream::video_fourcc::VideoFormat;
            use obs_rust_bitstream::video_frame::VideoFrame;
            use obs_rust_bitstream::photo_layers::LayerStack as PhotoLayers;
            let mut photos = PhotoLayers::new();
            if let Some(mut bg) = VideoFrame::new(VideoFormat::Y800, 4, 4) {
                if let Some(p) = bg.planes.get_mut(0) {
                    for px in p.iter_mut() { *px = 100; }
                }
                let mut top = bg.clone();
                if let Some(p) = top.planes.get_mut(0) {
                    for px in p.iter_mut() { *px = 200; }
                }
                let _ = photos.add("Background", bg);
                let _ = photos.add("Layer 1", top);
                println!("composite: {}", LayerComposite::line(&photos));
                photos.hide("Layer 1").ok();
                println!("hidden:    {}", LayerComposite::line(&photos));
            }
            let (layer, group) = EditRound::run();
            println!("edit: layer={} group={}", layer, group);
            let (full, half) = EditComposite::run();
            println!("composite_edit: full={:.1} half={:.1}", full, half);
            let (single, group) = GroupComposite::run();
            println!("group_edit: single={} group={}", single, group);
            let (normal, multiply) = BlendComposite::run();
            println!("blend: normal={:.1} multiply={:.1}", normal, multiply);
            println!("shell: {}", ShellStatus::line());
            let _ = std::fs::write("shell.txt", ShellStatus::line());
            println!("shell_file=shell.txt");
            let (full, half) = ShellFrame::run();
            println!("shell_frame: full={} half={}", full, half);
            let (ft, ht) = ShellRunner::compare();
            println!("shell_runner: full={} half={}", ft, ht);
            let _ = std::fs::write("shell_runner.txt", format!("full={} half={}", ft, ht));
            println!("shell_runner_file=shell_runner.txt");
            let (pf, ph) = ShellRtmp::run();
            println!("shell_rtmp: full={} half={}", pf, ph);
            let _ = std::fs::write("shell_rtmp.txt", format!("full={} half={}", pf, ph));
            println!("shell_rtmp_file=shell_rtmp.txt");
            let (full_line, half_line) = ShellPublish::run();
            println!("shell_publish: full={} half={}", full_line, half_line);
            let _ = std::fs::write("shell_publish.txt", format!("full={} half={}", full_line, half_line));
            println!("shell_publish_file=shell_publish.txt");
            let (sf, sh) = ShellSocket::run();
            println!("shell_socket: full={} half={}", sf, sh);
            let _ = std::fs::write("shell_socket.txt", format!("full={} half={}", sf, sh));
            println!("shell_socket_file=shell_socket.txt");
            let (sent_full, sent_half) = ShellSent::run();
            println!("shell_sent: full={} half={}", sent_full, sent_half);
            let _ = std::fs::write("shell_sent.txt", format!("full={} half={}", sent_full, sent_half));
            println!("shell_sent_file=shell_sent.txt");
            let actions = [
                ShellAction::Tool("Brush"),
                ShellAction::Size(30),
                ShellAction::NewLayer,
                ShellAction::Opacity(0.5),
            ];
            println!("shell_loop: {}", ShellLoop::run(&actions));
            let _ = std::fs::write("shell_loop.txt", ShellLoop::run(&actions));
            println!("shell_loop_file=shell_loop.txt");
            let encoded = ShellLoopFrame::encode(&actions);
            println!("shell_loop_frame: {}", encoded);
            let _ = std::fs::write("shell_loop_frame.txt", format!("{}", encoded));
            println!("shell_loop_frame_file=shell_loop_frame.txt");
            if let Some(payload) = ShellLoopSocket::payload(&actions) {
                println!("shell_loop_socket: {} valid={}", payload.len(), ShellLoopSocket::valid(&payload));
                let _ = std::fs::write("shell_loop_socket.txt", format!("{} valid={}", payload.len(), ShellLoopSocket::valid(&payload)));
                println!("shell_loop_socket_file=shell_loop_socket.txt");
            }
            let summary = ShellSummary::line(&actions);
            println!("shell_summary: {}", summary);
            let _ = std::fs::write("shell_summary.txt", &summary);
            println!("shell_summary_file=shell_summary.txt");
            if let Ok(sent_str) = std::fs::read_to_string("shell_loop_sent.txt") {
                if let Ok(sent) = sent_str.trim().parse::<usize>() {
                    let with_sent = ShellSummary::line_with_sent(&actions, sent);
                    println!("shell_summary_sent: {}", with_sent);
                }
            }
            let mut ew = Workspace::new(32, 32);
            let mut es = obs_rust_bitstream::layer_stack::LayerStack::new();
            let mut eh = History::new();
            let er = ShellEvent::handle(&mut ew, &mut es, &mut eh, ShellInput::Key("B"));
            println!("shell_event: {}", er);
            let (plain, bright) = ShellFilter::run();
            println!("shell_filter: plain={} bright={}", plain, bright);
            let _ = std::fs::write("shell_filter.txt", format!("plain={} bright={}", plain, bright));
            println!("shell_filter_file=shell_filter.txt");
            let mut sw = Workspace::new(32, 32);
            sw.click("Brush", obs_rust_bitstream::active_tool::ToolSource::Photoshop);
            sw.options_mut().set("size", 30.0);
            let state = ShellSession::save(&sw, 1, "Layer 1");
            let line = ShellSession::to_line(&state);
            let _ = std::fs::write("shell_session.txt", &line);
            println!("shell_session: {}", line);
            let (m0, m1) = DisplayFrame::run();
            println!("display_frame: m0={} m1={}", m0, m1);
            if let Some(yt) = YouTubeTarget::new("demo-key") {
                println!("youtube: {}", yt.line());
                let _ = std::fs::write("youtube.txt", yt.endpoint());
                println!("youtube_file=youtube.txt");
            }
            for p in [Platform::YouTube, Platform::Twitch, Platform::Facebook, Platform::X, Platform::Discord, Platform::Cloudflare] {
                if let Some(t) = StreamTarget::new(p, "demo") {
                    println!("platform: {}", t.line());
                }
            }
            if let Some(owned) = StreamTarget::owned("stream.example.com", "my-key") {
                println!("owned: {}", owned.endpoint());
            }
            if let Ok(saved) = std::fs::read_to_string("shell_session.txt") {
                if let Some(loaded) = ShellSession::from_line(&saved) {
                    let mut lw = Workspace::new(32, 32);
                    let layers = ShellSession::load(&mut lw, &loaded);
                    println!("shell_session_loaded: tool={} size={:.0} layers={}", lw.active(), lw.options().get("size").unwrap_or(0.0), layers);
                }
            }
        }
    }
}
