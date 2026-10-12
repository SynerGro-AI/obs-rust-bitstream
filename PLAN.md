# Public OBS Rust Rewrite — Foundational Plan

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.
**Work smarter, not harder.**

## Current foundation (high-leverage patterns)
- **bitstream** — Position trait (wrap vs no-wrap is type-level). Differential wired for short reads.
- **dstr** — safe path fixes infinite-loop on empty find + overflow. Differential wired.
- **utf8** — safe truncation that never splits a multibyte character. Differential wired.
- **diff** — reusable differential test helper (now used in modules).
- **crc32** — standard CRC32 with the classic test vector. Differential wired.
- **ai** — simple config scaffold to switch Grok / Claude / Seraphina for automated assistance.
- **array_serializer** — Vec-based output serializer + little-endian helpers. Differential wired.
- **text_lookup** — HashMap store with escape expansion. Differential wired.
- **path_extension** — safe extension extractor (None on null/empty). Differential wired.
- **bmem** — allocation size check + 32-byte alignment contract. Differential wired.
- **util_uint64** — exact (num * mul) / div via 128-bit intermediate. Differential wired.
- **util_uint128** — safe add and mul64_64 using native u128. Differential wired.
- **deque** — safe double-ended queue (circular buffer). Differential wired.
- **darray** — safe dynamic array (Vec wrapper). Differential wired.
- **config** — pure in-memory config store with defaults. Differential wired.
- **lexer** — pure base-token lexer. Differential wired.
- **serializer** — pure serializer trait + memory implementation. Differential wired.
- **profiler** — pure local profiler. Differential wired.
- **task** — pure task queue. Differential wired.
- **file_serializer** — pure file serializer. Differential wired.
- **buffered_serializer** — pure buffered serializer. Differential wired.
- **cf_lexer** — pure CF lexer. Differential wired.
- **config_parser** — pure config string parser. Differential wired.
- **source_profiler** — pure source profiler stub. Differential wired.
- **pipe** — pure in-memory pipe. Differential wired.
## Media-io layer (started)
- **frame_rate** — pure FPS rational helpers. Differential wired.
- **audio_math** — pure mul ↔ dB conversion. Differential wired.
- **video_fourcc** — pure fourcc to format mapping. Differential wired.
- **format_conversion** — pure plane-size and scalar helpers. Differential wired.
- **video_matrices** — pure color matrix coefficients. Differential wired.
- **video_frame** — pure video frame ownership. Differential wired.
- **audio_format** — pure audio format helpers. Differential wired.
- **video_scaler** — pure scale info and size helpers. Differential wired.
- **audio_buffer** — pure owned audio buffer. Differential wired.
- **media_timing** — pure sample and frame duration helpers. Differential wired.
- **audio_mixer** — pure gain and sum helpers. Differential wired.
## Graphics pure layer (started)
- **vec2** — pure 2D vector. Differential wired.
- **vec3** — pure 3D vector. Differential wired.
- **vec4** — pure 4D vector. Differential wired.
- **matrix3** — pure 3x4 matrix. Differential wired.
- **matrix4** — pure 4x4 matrix. Differential wired.
- **quat** — pure quaternion. Differential wired.
- **axisang** — pure axis-angle. Differential wired.
- **plane** — pure plane. Differential wired.
- **bounds** — pure AABB. Differential wired.
- **half** — pure half-float conversion. Differential wired.
## Callback pure layer solid
The high-leverage pure surface of callback is now covered:
signal (decl, disconnect, globals, remove),
proc (decl, remove),
calldata (int/float/bool/str/null-str/ptr, key+type match),
decl (declaration-string parser).
Threading and ref-count wrappers can be added later.
The pure foundation is solid and tested.

## Source pure layer (started)
- **source_context** — pure source context (name, settings, properties, signal, proc). Differential wired.
- **source_registry** — pure source registry. Differential wired.
- **output_context** — pure output context. Differential wired.
- **encoder_context** — pure encoder context. Differential wired.
- **service_context** — pure service context. Differential wired.
- **output_registry** — pure output registry. Differential wired.
- **encoder_registry** — pure encoder registry. Differential wired.
- **service_registry** — pure service registry. Differential wired.
- **scene_context** — pure scene context. Differential wired.
- **scene_registry** — pure scene registry. Differential wired.
- **scene_item** — pure scene item with transform. Differential wired.
- **filter_context** — pure filter context. Differential wired.
- **filter_chain** — pure ordered filter list, wired into SourceContext. Differential wired.
- **filter_registry** — pure filter registry. Differential wired.
- **core** — pure core root that owns the registries. Differential wired.
- **module** — pure module registration. Differential wired.
- **module_registry** — pure module registry, owned by Core. Differential wired.
- **display_context** — pure display / preview context. Differential wired.
- **display_registry** — pure display registry, owned by Core. Differential wired.
- **hotkey** — pure hotkey. Differential wired.
- **hotkey_registry** — pure hotkey registry, owned by Core. Differential wired.
- **scene_collection** — pure scene collection. Differential wired.
- **collection_registry** — pure collection registry, owned by Core. Differential wired.
- **transition** — pure scene transition. Differential wired.
- **transition_registry** — pure transition registry, owned by Core. Differential wired.
- **audio_monitor** — pure audio monitor. Differential wired.
- **monitor_registry** — pure monitor registry, owned by Core. Differential wired.
- **recording** — pure recording. Differential wired.
- **recording_registry** — pure recording registry, owned by Core. Differential wired.
- **streaming** — pure streaming. Differential wired.
- **streaming_registry** — pure streaming registry, owned by Core. Differential wired.
- **replay_buffer** — pure replay buffer. Differential wired.
- **replay_registry** — pure replay registry, owned by Core. Differential wired.
- **virtual_cam** — pure virtual camera. Differential wired.
- **virtual_cam_registry** — pure virtual camera registry, owned by Core. Differential wired.
- **profile** — pure profile. Differential wired.
- **profile_registry** — pure profile registry, owned by Core. Differential wired.
- **canvas** — pure canvas. Differential wired.
- **canvas_registry** — pure canvas registry, owned by Core. Differential wired.
- **render_item** — pure render item. Differential wired.
- **render_list** — pure render list. Differential wired.
- **camera** — pure camera. Differential wired.
- **projector** — pure 2D-to-3D projector. Differential wired.
- **bone** — pure bone hierarchy. Differential wired.
- **animation_timeline** — pure animation timeline. Differential wired.
- **physics** — pure physics settings with AI assist. Differential wired.
- **settings_assist** — pure AI assist for any settings bag. Differential wired.
- **performance_capture** — pure actor performance capture. Differential wired.
- **character_renderer** — pure character renderer. Differential wired.
- **skeleton_connector** — pure AI skeleton connector. Differential wired.
- **ai_animator** — pure AI-driven animator. Differential wired.
- **physics_sim** — pure AI-driven physics simulation. Differential wired.
- **learning_pipeline** — pure physics-governed AI learning. Differential wired.
- **environment** — pure world physics governor. Differential wired.
- **pilot** — pure lesson-driven pilot. Differential wired.

## Data pure layer (started)
- **obs_data** — pure settings bag with defaults. Differential wired.
- **obs_properties** — pure property list. Differential wired.

## Graphics pure layer solid
The high-leverage pure surface of graphics is now covered:
vec2, vec3, vec4, matrix3, matrix4, quat, axisang, plane, bounds, half.
All are inlined for graphics performance and carry differential tests.
Device/effect/shader code can be thin wrappers later.

## Media-io pure layer solid
The high-leverage pure surface of media-io is now covered:
frame-rate, audio-math, video-fourcc, format-conversion, video-matrices,
video-frame, audio-format, video-scaler, audio-buffer, media-timing,
audio-mixer. FFmpeg-backed resampler/scaler/remux can be thin wrappers
later. The pure foundation is solid and tested.

## Util foundation complete
The high-leverage pure surface of libobs/util is now covered.
Remaining OS-specific pieces (platform-*, threading-*, sse-intrin, uthash)
can be added later as thin wrappers. The pure foundation is solid,
tested, and ready for the next layer (media-io / graphics math) if desired.
- CI that runs cargo test on every push.

## Smart multiplier
One perfect pattern applies to many files.
The differential helper is reused across modules.
AI config lets you swap backends with one variable change for automated help.
Parser + quirks table turns one action into coverage for dozens of modules.

## Next steps (in order)
1. Batch-apply the patterns to the remaining util files (path_extension, serializers, bmem, etc.).
2. Wire a real HTTP client into the AI scaffold (reqwest) for live automated review/generation.
3. Workspace layout so new crates drop in with the same structure.
4. Keep the public repo growing with high-leverage commits.

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve public C ABI
- Fix the bugs in the safe path
- Work smarter: reusable patterns > one-off ports
- AI-assisted contributions welcome (Grok, Claude, or Seraphina)

## How to contribute
Clone, cargo test, open a PR. Point the parser at the next .c file; reuse the patterns, the differential helper, and the AI scaffold.
