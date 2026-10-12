# obs-rust-bitstream

**Public foundation for the OBS Studio rewrite in Rust.**

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.  
**Work smarter, not harder.**

Reusable patterns + differential helper + AI scaffold for automated assistance.

## Foundation modules (live)

| Module | What it does | Notes |
|--------|----------------|-------|
| **bitstream** | Position trait (wrap vs no-wrap is type-level) | Differential on short reads |
| **dstr** | Safe dynamic string | Fixes infinite loop + overflow |
| **utf8** | Safe truncation | Never splits a multibyte character |
| **diff** | Reusable differential test helper | Wired into modules |
| **crc32** | Standard CRC32 | Classic test vector |
| **ai** | AI/SI integration scaffold | Simple config for Grok, Claude, or Seraphina |
| **array_serializer** | Vec-based output serializer + LE helpers | Differential on short writes |
| **text_lookup** | HashMap store with escape expansion | Differential on insert/get |
| **path_extension** | Safe extension extractor | None on null/empty |
| **bmem** | Alloc size check + alignment contract | 32-byte alignment |
| **util_uint64** | Exact mul-then-div | 128-bit intermediate, div-zero safe |
| **util_uint128** | Add / mul64_64 | Native u128 |
| **deque** | Double-ended queue | Safe circular buffer |
| **darray** | Dynamic array | Safe Vec wrapper |
| **config** | Config store | Pure HashMap + defaults |
| **lexer** | Base-token lexer | Pure &str slices |
| **serializer** | Serializer trait | Pure memory impl + LE helpers |
| **profiler** | Local profiler | Pure Instant deltas |
| **task** | Task queue | Pure Vec of closures |
| **file_serializer** | File serializer | Pure load/save via std::fs |
| **buffered_serializer** | Buffered serializer | Pure chunked flush |
| **cf_lexer** | CF lexer | Pure &str tokens |
| **config_parser** | Config string parser | Pure ini → Config |
| **source_profiler** | Source profiler stub | Pure local deltas |
| **pipe** | In-memory pipe | Pure write/read |
| **frame_rate** | Frame-rate helpers | Pure rational FPS |
| **audio_math** | Audio math | Pure mul ↔ dB |
| **video_fourcc** | Fourcc mapping | Pure u32 → format |
| **format_conversion** | Format helpers | Pure plane sizes + scalar |
| **video_matrices** | Color matrices | Pure Kb/Kr + identity |
| **video_frame** | Video frame | Pure owned planes |
| **audio_format** | Audio format | Pure size / planar helpers |
| **video_scaler** | Video scaler | Pure scale size + lerp |
| **audio_buffer** | Audio buffer | Pure owned planes |
| **media_timing** | Media timing | Pure ns durations |
| **audio_mixer** | Audio mixer | Pure gain + sum |
| **vec2** | 2D vector | Pure add/dot/length |
| **vec3** | 3D vector | Pure add/dot/cross |
| **vec4** | 4D vector | Pure add/dot |
| **matrix3** | 3x4 matrix | Pure identity + mul |
| **matrix4** | 4x4 matrix | Pure identity + mul |
| **quat** | Quaternion | Pure mul + normalize |
| **axisang** | Axis-angle | Pure axis + angle |
| **plane** | Plane | Pure normal + dist |
| **bounds** | AABB | Pure min/max + center |
| **half** | Half-float | Pure f32 <-> f16 |
| **signal** | Signal handler | Pure connect + emit |
| **proc** | Procedure handler | Pure add + call |
| **calldata** | Call data | Pure named params |
| **decl** | Decl parser | Pure signal/proc strings |
| **obs_data** | Settings bag | Pure key/value + defaults |
| **obs_properties** | Properties | Pure bool/int/float/text |
| **source_context** | Source context | Pure name + settings + handlers |
| **source_registry** | Source registry | Pure name-to-source map |
| **output_context** | Output context | Pure name + settings + active |
| **encoder_context** | Encoder context | Pure name + codec + type |
| **service_context** | Service context | Pure name + id + settings |
| **output_registry** | Output registry | Pure name-to-output map |
| **encoder_registry** | Encoder registry | Pure name-to-encoder map |
| **service_registry** | Service registry | Pure name-to-service map |
| **scene_context** | Scene context | Pure ordered item list |
| **scene_registry** | Scene registry | Pure name-to-scene map |
| **scene_item** | Scene item | Pure transform + visibility |
| **filter_context** | Filter context | Pure name + id + settings |
| **filter_chain** | Filter chain | Pure ordered filter list |
| **filter_registry** | Filter registry | Pure name-to-filter map |
| **core** | Core root | Pure owner of all registries |
| **module** | Module | Pure type registration |
| **module_registry** | Module registry | Pure name-to-module map |
| **display_context** | Display | Pure preview / program window |
| **display_registry** | Display registry | Pure name-to-display map |
| **hotkey** | Hotkey | Pure name + pressed flag |
| **hotkey_registry** | Hotkey registry | Pure name-to-hotkey map |
| **scene_collection** | Scene collection | Pure named set of scenes |
| **collection_registry** | Collection registry | Pure name-to-collection map |
| **transition** | Transition | Pure scene switch |
| **transition_registry** | Transition registry | Pure name-to-transition map |
| **audio_monitor** | Audio monitor | Pure volume + mute |
| **monitor_registry** | Monitor registry | Pure name-to-monitor map |
| **recording** | Recording | Pure start/stop + path |
| **recording_registry** | Recording registry | Pure name-to-recording map |
| **streaming** | Streaming | Pure start/stop + service |
| **streaming_registry** | Streaming registry | Pure name-to-stream map |
| **replay_buffer** | Replay buffer | Pure duration + save |
| **replay_registry** | Replay registry | Pure name-to-buffer map |
| **virtual_cam** | Virtual camera | Pure start/stop |
| **virtual_cam_registry** | Virtual camera registry | Pure name-to-cam map |
| **profile** | Profile | Pure name + settings |
| **profile_registry** | Profile registry | Pure name-to-profile map |
| **canvas** | Canvas | Pure render surface |
| **canvas_registry** | Canvas registry | Pure name-to-canvas map |
| **render_item** | Render item | Pure model matrix from transform |
| **render_list** | Render list | Pure draw list from a scene |
| **camera** | Camera | Pure Maya-style view matrix |
| **projector** | Projector | Pure 2D-to-3D lift |
| **bone** | Bone | Pure skeleton hierarchy |
| **animation_timeline** | Animation timeline | Pure keyframe playback |
| **physics** | Physics settings | Pure gravity/friction/bounce/mass + AI assist |
| **settings_assist** | Settings assist | Pure AI assist for any ObsData bag |
| **performance_capture** | Performance capture | Pure actor recording into timeline |
| **character_renderer** | Character renderer | Pure skeleton walk + AI look |

All have unit tests. C ABI shims keep original quirks where required.

## Parser

`tools/obs_cpp_to_rust_parser.py` loads `tools/lessons.json` and generates
Rust skeletons that already document the learned fixes. Add new lessons
as you discover more quirks — the next parse gets smarter automatically.

## Status

- [x] One hundred one foundation modules with tests
- [x] Util foundation complete (high-leverage pure surface covered)
- [x] Media-io pure layer solid (frame-rate through audio-mixer)
- [x] Graphics pure layer solid (vec2 through half)
- [x] Callback pure layer solid (signal, proc, calldata, decl)
- [ ] Data pure layer started (obs_data, obs_properties)
- [x] Source pure layer: context + registry complete for source, output, encoder, service
- [x] Scene pure layer: context, registry, items, collection + registry
- [x] Filter pure layer: context + chain + registry complete
- [x] Core root owns all registries including modules, displays, hotkeys, collections, transitions, monitors, recordings, streams, replays, virtual cams, profiles, and canvases
- [x] Module registration + registry complete
- [x] Display pure layer: context + registry complete
- [x] Hotkey pure layer: hotkey + registry complete
- [x] Transition pure layer: transition + registry complete
- [x] Audio monitor pure layer: monitor + registry complete
- [x] Recording pure layer: recording + registry complete
- [x] Streaming pure layer: streaming + registry complete
- [x] Replay buffer pure layer: buffer + registry complete
- [x] Virtual camera pure layer: cam + registry complete
- [x] Profile pure layer: profile + registry complete
- [x] Canvas pure layer: canvas + registry complete
- [ ] Render pure layer started (render_item, render_list, camera, projector, bone, animation_timeline, performance_capture, character_renderer)
- [x] Physics + AI settings assist wired
- [x] Differential helper wired into modules
- [x] AI integration scaffold (swap backends with one variable)
- [x] Parser tools wired
- [x] CI (cargo test on push/PR)
- [x] Public under SynerGro-AI
- [ ] Batch application to remaining util files
- [ ] Live HTTP client in AI scaffold
- [ ] Workspace layout

## Layout

```
src/
  lib.rs
  bitstream.rs
  dstr.rs
  utf8.rs
  diff.rs
  crc32.rs
  ai.rs
  array_serializer.rs
  text_lookup.rs
  path_extension.rs
  bmem.rs
tools/
  obs_cpp_to_rust_parser.py
.github/workflows/ci.yml
PLAN.md
```

## Usage

```bash
cargo test
```

## AI integration

Simple variable adjustment:

```rust
use obs_rust_bitstream::ai::{AiAssistant, AiConfig};

let ai = AiAssistant::new(AiConfig::default());          // Grok
// let ai = AiAssistant::new(AiConfig::claude());       // Claude
// let ai = AiAssistant::new(AiConfig::seraphina());    // Seraphina
let review = ai.review_skeleton("/* generated rust */");
```

## Foundational plan

1. Foundation patterns first — done
2. Parser wired — done
3. High-leverage modules + differential helper — done
4. AI scaffold for automated help — done
5. Batch-apply to the rest
6. Keep pushing — public, free, open

AI-assisted contributions and PRs welcome.

See also the upstream [obs-rust/obs-studio](https://github.com/obs-rust/obs-studio) effort.

## License

GPL-2.0-or-later (matching OBS Studio).
