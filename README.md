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

All have unit tests. C ABI shims keep original quirks where required.

## Parser

`tools/obs_cpp_to_rust_parser.py` loads `tools/lessons.json` and generates
Rust skeletons that already document the learned fixes. Add new lessons
as you discover more quirks — the next parse gets smarter automatically.

## Status

- [x] Fifty foundation modules with tests
- [x] Util foundation complete (high-leverage pure surface covered)
- [x] Media-io pure layer solid (frame-rate through audio-mixer)
- [x] Graphics pure layer solid (vec2 through half)
- [ ] Callback pure layer started (signal, proc, calldata, decl)
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
