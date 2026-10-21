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
| 19 | Go live + Record → status bar "Live 00:00:04", "REC 00:00:04"; stop → Offline / REC off | PASS (v2: simulated output; real ffmpeg output since v3, see below) |
| 20 | Options bar does not overlap zoom/undo at narrow widths (wraps) | PASS after fix |
| 21 | Theme toggle dark ↔ light | PASS |
| 22 | Settings / Account / Shortcuts windows open; Esc closes them | PASS after fix (Esc only closed Shortcuts) |
| 23 | Live Seraphina chat in the GUI (grok-4.7): "Set Mist Overlay opacity to 55% and switch to Composite" → action card with 2 Done lines, Composite active, Paint Mist Overlay at 55 % | PASS |
| 24 | One-click Undo on the action card restores 20 % and Paint mode, strikes the lines, learning +4 | PASS |
| 25 | Grok Imagine panel shows "Configured" with a key; Generate video / Image → layer buttons present | PASS (the GUI generate button was not clicked, to avoid a second charge; the provider was live-tested from `cargo test`) |
| 26 | Wrap counter toggle from the counter segment | NOT VERIFIED IN GUI (the click missed; covered by unit test `stream_commands_and_status`) |

## v3 (2026-10-20, MDT): real output, 3D tools, PSD pixels, Wine

Release build on Xvfb (software GL), screenshots in `/workspace/obs-ui-shots-v3/`.

| # | Check | Result |
|---|-------|--------|
| 27 | 3D viewport renders z-buffered flat shading; selected mesh tinted (`3d_shaded.png`) | PASS |
| 28 | Create Cylinder, set y 0.9 / scale 0.6, select cube, shelf **Difference** → blind hole, inspector "Closed: yes" (`3d_drill_placed.png`, `3d_boolean_difference.png`) | PASS (first try showed "Closed: no"; fixed with T-junction repair) |
| 29 | Ctrl+Z undoes the boolean (cube + cylinder back) (`3d_undo_boolean.png`) | PASS |
| 30 | Key 6 Shaded+wire, key 4 Wireframe (`3d_shaded_wire.png`, `3d_wireframe.png`) | PASS |
| 31 | Edge Loop tool: click a cube edge → loop inserted, 12 verts / 10 faces, closed (`3d_edge_loop.png`) | PASS |
| 32 | Go live from the status bar → mediamtx logs "publishing … 2 tracks (H264, MPEG-4 Audio)"; status "Live 00:00:04 · 31 fps · 421 kb/s · 3 dropped" (`live_status_tmp.png`) | PASS |
| 33 | Record + live at once while the 3D timeline plays; external ffprobe of the RTMP URL: h264 1280×720 + aac; frame grabbed from RTMP shows the 3D viewport (`status_bar_rec_live_zoom.png`, `rtmp_received_frame.png`) | PASS (status-bar labels are small; two clicks missed and one hit the wrong label) |
| 34 | Stream panel "Drop connection (test)" kills ffmpeg → toast "Reconnecting in 2.0s (attempt 1/10)", status bar "Reconnecting in 2s"; mediamtx shows republish 2 s later (`stream_reconnecting.png`) | PASS |
| 35 | Stop both → recording `recordings/Seraphina_2026-10-20_22-28-46Z.mp4`, 143.9 s, 1280×720 h264 + aac; no ffmpeg left running | PASS |
| 36 | Stream key shown masked "•••••••• (12 characters)" in the panel (`stream_mode_output_panel.png`) | PASS |
| 37 | `ui assets/mountain_spire.psd` opens 7 layers + Environment group with real pixels (`psd_real_pixels.png`) | PASS |
| 38 | Windows exe under Wine 10: starts, all five modes render (`wine_startup.png`, `wine_composite.png`, `wine_3d.png`, `wine_animate.png`, `wine_stream.png`) | PASS after fix: widgets used egui's light style because Wine reports a light system theme (theme now pinned) |
| 39 | Windows exe under Wine without ffmpeg: clear "ffmpeg not found" + help text (`wine_stream.png`) | PASS |
| 40 | Windows exe under Wine with Windows ffmpeg.exe: Record + Go live, RTMP verified by ffprobe, 28.9 s MP4 h264+aac (`wine_recording_streaming.png`, `wine_recordings/`) | PASS after fix: `-thread_queue_size` is rejected as an input option by current ffmpeg |
| 41 | Windows test binary under Wine: `output::tests` 5 passed + RTMP test skipped (no mediamtx); MP4 `record_test_5s_windows_wine.mp4` | PASS |

Wine is a proxy for Windows, not real Windows. The CI `windows-latest` job
builds and runs the UI tests on real Windows, but this branch has not been pushed, so that job hasn't run.

## v4 (2026-10-20, MDT): Materials Lab and Physics Arena

This was a debug build of branch `materials-lab` on Xvfb (1920×1080,
software GL), driven by xdotool. Screenshots are in `/workspace/obs-ui-shots-v4/`.

| # | Check | Result |
|---|-------|--------|
| 42 | Ctrl+6 or the shelf **Materials** button opens the lab: 18-column table, f-block rows, category colours, hover highlight (`01_periodic_table.png`) | PASS |
| 43 | Click U → the detail card shows mass, configuration, oxidation states, density, melting/boiling, half-life 4.468 × 10⁹ y (`02_element_detail_uranium.png`) | PASS (long values overflowed the card; they now wrap below the key) |
| 44 | Real clicks: double-click H, H, O → **Build** → matched to Water, H2O 18.015 g/mol, "charge balanced", "nuclei stable", PBR (IOR 1.333, transmission 0.95) and sim env shown (`03_composition_builder_h2o.png`) | PASS (the inspector grew a few px every frame; fixed the segmented-control width) |
| 45 | **Assign to 'cube1' in 3D** → the 3D inspector Material card reads Water | PASS |
| 46 | `--run docs/scripts/materials_scene.jsonl`: copper, gold, glass, iron, graphite, aluminium in 3D (`04_3d_scene_materials.png`) | PASS after two fixes: glass was opaque (alpha blending added), and metals looked flat grey-khaki with only a point light (sky environment added) |
| 47 | Arena, Moon, 300 steps: cube1 y 3.10 → 1.83 m, t 1.25 s, KE 29 087 J. Check: ½ × 14 187 kg × (1.62 × 1.25)² = 29 088 J (`05_arena_moon_midair.png`) | PASS after fix: cube1 has Y keyframes and the timeline snapped it back every frame. Physics now owns transforms until Reset; regression test added |
| 48 | Arena melt, Earth, 10 s: log "8.78 s cube2 (Water) solid→liquid at 273.1 K", ice slumped into a puddle, aluminium still solid (`06_arena_ice_melted.png`) | PASS |
| 49 | Windows exe (x86_64-pc-windows-gnu release) under Wine: `--run scripts\\materials_scene.jsonl` logs all 17 lines, the scene renders with materials and transparent glass; Ctrl+6 opens the lab with the Gold card and a 6-material library (`07_wine_3d_materials.png`, `08_wine_materials_lab.png`) | PASS (Wine is a proxy for Windows) |

Capture notes: Xvfb sometimes showed frames that were several frames old,
and a leftover window from an earlier session sat on the same display.
Shots were taken until two consecutive captures matched, and the field
values were read from each image rather than assumed.

## v5 (2026-10-20, MDT): Phase 2 lab (energy, reactions, circuits, computer, relativity)

Screenshots: `/workspace/obs-ui-shots-v5/`. The scripts are in `docs/scripts/lab_*.jsonl`.

| # | Check | Result |
|---|-------|--------|
| 50 | Arena, Earth, 5 s: copper cube dropped, silicon cube with a 20 kW heater. Energy ledger: heaters in +100.00 kJ, impacts → heat +4.36 kJ, ΔThermal +104.31 kJ, residual −0.000 J (`01_energy.png`) | PASS |
| 51 | `chem_react` thermite from 1500 K: balanced to Fe2O3 + 2 Al → Al2O3 + 2 Fe, ΔH° −851.5 kJ/mol, K 4.6e26. Reactor ends at 3137 K, the same as the adiabatic flame temperature; the plateau is Fe boiling at 3134 K (`02_thermite.png`) | PASS after two fixes. (1) The reactor hung: a negative reaction order made the steps Zeno-like, and each step re-bisected at equilibrium. (2) The reactor ignored latent heat and reached 8302 K; latent heats are now spread over ±5 K so enthalpy is conserved exactly |
| 52 | `circuit_run` fuse: 20 mm × ⌀0.1 mm Cu on 1 V. R = ρL/A = 43.6 mΩ; the wire heats, R rises to 223 mΩ, and it melts open at 1364 K after 0.090 s (Cu melts at 1358 K) (`03_circuit.png`) | PASS after a fix: the step size was cut from 14 ms (31 K overshoot) to 1 ms |
| 53 | `cpu_run` fibonacci at 27 °C, 1 GHz: output [1,1,2,3,5,8,13], correct; carry chain 309 ps of an 876 ps budget; all four levels green (`04_cpu.png`) | PASS |
| 54 | `cpu_heat_sweep` at 2 GHz: fails at 97 °C because the carry chain needs 363 ps and the clock leaves 355 ps. The CPU then prints [1,1,2,3,5,0,5,5,10,15]: a wrong carry corrupts the sequence, and it misses its overflow stop (`05_overheat.png`) | PASS |
| 55 | Mercury with GR ×100 000 (a visual aid): the rosette precesses, 10 126″/orbit simulated vs 10 342″ from 6πGM/c²a(1−e²). Measure at real c: 42.98″/century (analytic 42.98). Muon β 0.995: γ 10.0125 (`06_mercury.png`) | PASS after fixes: the perihelion angle is now unwrapped, the Mercury clock uses the real c, and fractional steps are carried between frames |
| 56 | `cpu_run die_object=cube1` with the arena playing: 10 cm Si die, 15 kW heater. The die temperature follows the body, and the CPU re-runs every 5 K and fails on timing (`07_die.png`) | PASS (`arena_play` switches to 3D, so the script switches back with `lab_tab`) |
| 57 | Every tab's controls go through the bus. Bad input gives an error message, not a panic: unknown species, unknown wire element, β ≥ 1, unknown preset, an assembly error, a die body that isn't in the arena (unit tests in `src/bin/ui/lab.rs`) | PASS |
| 58 | Windows exe under Wine runs `docs/scripts/lab_04_cpu.jsonl` (`08_wine_lab.png`) | see the report |

### Exact vs approximate (v5)

Exact, up to floating point:
- Equation balancing.
- Thermodynamic identities: ΔG = ΔH − TΔS and K = exp(−ΔG/RT).
- Enthalpy conservation in the reactor.
- The MNA solution of the linear networks; DC results are exact.
- Special-relativity formulas.
- Leapfrog's conservation properties.
- The energy-ledger bookkeeping: the residual sits at about 1e-5 relative, set by f32 positions.

Data, as tabulated: formation enthalpies, entropies, Cp at 298 K, transition enthalpies (CODATA/JANAF/CRC), resistivities, GM values, CODATA constants.

Approximate:
- **Reactions.**
  - Rates are global empirical fits; some are generic estimates, which the UI flags.
  - There is no dissociation, so flames run about 100 K too hot in air, and far too hot in pure O2.
  - Gases are ideal, and Cp uses the RRHO model.
  - Latent heats are spread over ±5 K.
- **Circuits.**
  - Backward Euler damps a little (RLC peak shown next to the exact value).
  - Device parameters are generic 180 nm-class values, not a foundry PDK.
  - The MOSFET has no velocity saturation or short-channel effects.
  - Each wire is a single thermal mass with a constant h.
- **Computer.**
  - Gate delays come from transistor simulation into a fixed 20 fF load, with no wire RC.
  - Control and registers are behavioural, with a setup check; only the ALU runs at gate level.
  - "Intrinsic at 850 K" is the first-principles limit. Real chips fail earlier from leakage, electromigration and packaging, none of which is modelled.
- **Relativity.**
  - 1PN is applied only as the test-particle term around the central body, which is right for Mercury but approximate for binaries.
  - Exaggeration beyond about ×1e4 leaves the regime of the first-order formula.
  - Orbits start from mean elements, not an ephemeris.
