# Seraphina Studio v2: function test (xdotool on Xvfb)

Run on 2026-10-20 (MDT) against the release build of branch `ui-polish`.
The window was 1600×940 on Xvfb with software GL. Each step was driven with
xdotool and checked against a screenshot; the step screenshots live outside
the repo, in `/workspace/obs-ui-shots-v2/ft/`.

| # | Check | Result |
|---|-------|--------|
| 1 | Rectangular marquee draws marching ants (all 4 edges); brush is clipped to the selection | PASS (the outline was too thin to show at 59 % zoom, so it was made 2 px thick) |
| 2 | Tool choice updates "Online learning: N" in the status bar; cursor readout updates | PASS |
| 3 | Eraser, lasso, gradient | PASS (gradient was a flat fill before strokes were anchored to the press origin) |
| 4 | Ctrl+Z undo | PASS |
| 5 | Dodge (O), Burn (Shift+O), Clone stamp (Alt-click source), Healing on Mountain Spire | PASS (some build-up where the pointer pauses) |
| 6 | Pen points + Enter strokes the path; Type tool creates an "Aa" text layer | PASS |
| 7 | Quick select (W) picks the sky region | PASS (was picking 6–43 px before the edge-aware rewrite) |
| 8 | Crop + Enter → 939×598, toast, status bar updated; crop undo | PASS |
| 9 | Move, eyedropper, zoom, hand | PASS |
| 10 | Layer opacity field (Mist Overlay → 20 %), eye toggle changes canvas | PASS |
| 11 | Blend dropdown lists 8 modes; Multiply applies | PASS |
| 12 | Mask density / feather sliders apply live | PASS |
| 13 | Header mode tabs switch Paint / Composite / 3D / Animate / Stream | PASS |
| 14 | 3D shelf: Extrude ×2, Bevel, Move drag (x/z change), Rotate drag (90°) | PASS |
| 15 | 3D timeline scrub animates keyed Y (0.50 → 0.78 at 2.62 s) | PASS |
| 16 | Composite scrub to 4.00 s interpolates Gold Grade opacity (45 %) | PASS |
| 17 | Animate: draw on frame 1, frame 2 shows onion skin | PASS |
| 18 | Stream: Step, ×8, Run (4105 reads, 32840 bits) | PASS |
| 19 | Go live + Record → status bar "Live 00:00:04", "REC 00:00:04"; stop → Offline / REC off | PASS (simulated output) |
| 20 | Options bar does not overlap zoom/undo at narrow widths (wraps) | PASS after fix |
| 21 | Theme toggle dark ↔ light | PASS |
| 22 | Settings / Account / Shortcuts windows open; Esc closes them | PASS after fix (Esc only closed Shortcuts) |
| 23 | Live Seraphina chat in the GUI (grok-4.7): "Set Mist Overlay opacity to 55% and switch to Composite" → action card with 2 Done lines, Composite active, Paint Mist Overlay at 55 % | PASS |
| 24 | One-click Undo on the action card restores 20 % and Paint mode, strikes the lines, learning +4 | PASS |
| 25 | Grok Imagine panel shows "Configured" with a key; Generate video / Image → layer buttons present | PASS (the GUI generate button was not clicked, to avoid a second charge; the provider was live-tested from `cargo test`) |
| 26 | Wrap counter toggle from the counter segment | NOT VERIFIED IN GUI (the click missed; covered by unit test `stream_commands_and_status`) |
