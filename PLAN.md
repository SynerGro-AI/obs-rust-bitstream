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
- **flight_session** — pure closed-loop flight session. Differential wired.
- **flight_replay** — pure recorded flight playback. Differential wired.
- **wind** — pure environment wind force. Differential wired.
- **plane_renderer** — pure flight-to-skeleton renderer. Differential wired.
- **camera_follow** — pure flight camera follow. Differential wired.
- **view_frame** — pure flight view frame. Differential wired.
- **light** — pure light. Differential wired.
- **light_rig** — pure multi-light rig. Differential wired.
- **camera_rig** — pure multi-camera rig. Differential wired.
- **camera_cut** — pure camera cut. Differential wired.
- **material** — pure material. Differential wired.
- **lighting** — pure material × light evaluation. Differential wired.
- **material_assign** — pure material-to-bone binding. Differential wired.
- **shaded_render** — pure lit skeleton render. Differential wired.
- **material_registry** — pure named material store. Differential wired.
- **material_factory** — pure preset material creator. Differential wired.
- **materials_lab** — pure materials science lab. Differential wired.
- **periodic_table** — pure periodic table. Differential wired.
- **compound** — pure molecule. Differential wired.
- **reaction** — pure chemical reaction. Differential wired.
- **audio_channel** — pure audio channel. Differential wired.
- **audio_bus** — pure audio bus. Differential wired.
- **audio_effect** — pure audio effect. Differential wired.
- **effect_chain** — pure effect chain. Differential wired.
- **master_output** — pure master output. Differential wired.
- **eq** — pure parametric EQ. Differential wired.
- **compressor** — pure compressor. Differential wired.
- **limiter** — pure limiter. Differential wired.
- **gate** — pure gate. Differential wired.
- **delay** — pure delay. Differential wired.
- **reverb** — pure reverb. Differential wired.
- **meter** — pure peak and RMS meter. Differential wired.
- **audio_timeline** — pure multi-track timeline. Differential wired.
- **video_timeline** — pure AI video timeline. Differential wired.
- **layer_stack** — pure layer stack. Differential wired.
- **onion_viewer** — pure onion-skin viewer. Differential wired.
- **mask** — pure mask. Differential wired.
- **editor_ui** — pure editor panel. Differential wired.
- **adjustment** — pure adjustment layer. Differential wired.
- **layer_group** — pure layer group. Differential wired.
- **selection** — pure selection. Differential wired.
- **brush** — pure brush. Differential wired.
- **text_layer** — pure text layer. Differential wired.
- **transform** — pure transform. Differential wired.
- **curves** — pure curves. Differential wired.
- **filter** — pure filter. Differential wired.
- **smart_object** — pure smart object. Differential wired.
- **history** — pure undo / redo. Differential wired.
- **pixel_canvas** — pure pixel canvas. Differential wired.
- **object3d** — pure 3D object. Differential wired.
- **smart_skin** — pure drag-and-drop skin. Differential wired.
- **ai_editor** — pure AI editor access. Differential wired.
- **compositor** — pure video compositor. Differential wired.
- **input_device** — pure input device. Differential wired.
- **output** — pure frame output. Differential wired.
- **audio_output** — pure audio output. Differential wired.
- **session** — pure combined session. Differential wired.
- **preview** — pure session preview. Differential wired.
- **program** — pure live program. Differential wired.
- **live_camera** — pure live camera. Differential wired.
- **ai_operator** — pure AI news crew. Differential wired.
- **topic_background** — pure topic background. Differential wired.
- **performance** — pure singer performance. Differential wired.
- **show_state** — pure show state. Differential wired.
- **cue_sheet** — pure cue sheet. Differential wired.
- **lower_third** — pure lower third. Differential wired.
- **keyer** — pure background keyer. Differential wired.
- **ai_director** — pure AI scene director. Differential wired.
- **storyboard** — pure storyboard. Differential wired.
- **speech** — pure live speech. Differential wired.
- **action** — pure detected action. Differential wired.
- **effect** — pure scene effect. Differential wired.
- **effect_list** — pure effect list. Differential wired.
- **live_feed** — pure live feed. Differential wired.
- **scene_switch** — pure scene switch. Differential wired.
- **multi_angle** — pure multi angle. Differential wired.
- **multi_monitor** — pure multi monitor. Differential wired.
- **options_menu** — pure options menu. Differential wired.
- **studio** — pure studio mode. Differential wired.
- **studio_transition** — pure studio transition. Differential wired.
- **multiview** — pure multiview. Differential wired.
- **tally** — pure tally. Differential wired.
- **program_record** — pure program recording. Differential wired.
- **program_stream** — pure program stream. Differential wired.
- **virtual_camera** — pure virtual camera. Differential wired.
- **output_bus** — pure output bus. Differential wired.
- **frame_clock** — pure frame clock. Differential wired.
- **instant_replay** — pure instant replay. Differential wired.
- **replay_save** — pure replay save. Differential wired.
- **export** — pure export. Differential wired.
- **file_writer** — pure file writer. Differential wired.
- **share** — pure share. Differential wired.
- **notify** — pure notification. Differential wired.
- **status** — pure status. Differential wired.
- **keybind** — pure keybind. Differential wired.
- **help** — pure help. Differential wired.
- **bind_profile** — pure bind profile. Differential wired.
- **log** — pure log. Differential wired.
- **undo** — pure undo. Differential wired.
- **redo** — pure redo. Differential wired.
- **macro_seq** — pure macro. Differential wired.
- **schedule** — pure schedule. Differential wired.
- **nal_parser** — pure NAL parser. Differential wired.
- **output_delay** — pure output delay. Differential wired.
- **deinterlace** — pure deinterlace. Differential wired.
- **missing_files** — pure missing files. Differential wired.
- **fingerprint** — pure fingerprint. Differential wired.
- **file_cache** — pure file cache. Differential wired.
- **color_filter** — pure color filter. Differential wired.
- **crop_filter** — pure crop filter. Differential wired.
- **scale_filter** — pure scale filter. Differential wired.
- **noise_filter** — pure noise filter. Differential wired.
- **gain_filter** — pure gain filter. Differential wired.
- **compressor_filter** — pure compressor. Differential wired.
- **limiter_filter** — pure limiter. Differential wired.
- **gate_filter** — pure gate. Differential wired.
- **eq_filter** — pure EQ. Differential wired.
- **reverb_filter** — pure reverb. Differential wired.
- **fade_transition** — pure fade transition. Differential wired.
- **swipe_transition** — pure swipe transition. Differential wired.
- **slide_transition** — pure slide transition. Differential wired.
- **cut_transition** — pure cut transition. Differential wired.
- **image_source** — pure image source. Differential wired.
- **text_source** — pure text source. Differential wired.
- **color_source** — pure color source. Differential wired.
- **browser_source** — pure browser source. Differential wired.
- **media_source** — pure media source. Differential wired.
- **window_capture** — pure window capture. Differential wired.
- **display_capture** — pure display capture. Differential wired.
- **game_capture** — pure game capture. Differential wired.
- **audio_input** — pure audio input. Differential wired.
- **audio_output_source** — pure audio output. Differential wired.
- **group_source** — pure group source. Differential wired.
- **organization** — pure organization. Differential wired.
- **invite** — pure invite. Differential wired.
- **replay_buffer** — pure replay buffer. Differential wired.

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
