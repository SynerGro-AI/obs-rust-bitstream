//! - performance_capture: pure actor performance capture
//! - character_renderer: pure character renderer
//! - skeleton_connector: pure AI skeleton connector
//! - ai_animator: pure AI-driven animator
//! - physics_sim: pure AI-driven physics simulation
//! - learning_pipeline: pure physics-governed AI learning
//! - environment: pure world physics governor
//! - pilot: pure lesson-driven pilot
//! - flight_session: pure closed-loop flight session
//! - flight_replay: pure recorded flight playback
//! - wind: pure environment wind force
//! - plane_renderer: pure flight-to-skeleton renderer
//! - camera_follow: pure flight camera follow
//! - view_frame: pure flight view frame
//! - light: pure light
//! - light_rig: pure multi-light rig
//! - camera_rig: pure multi-camera rig
//! - camera_cut: pure camera cut
//! - material: pure material
//! - lighting: pure material × light evaluation
//! - material_assign: pure material-to-bone binding
//! - shaded_render: pure lit skeleton render
//! - material_registry: pure named material store
//! - material_factory: pure preset material creator
//! - materials_lab: pure materials science lab
//! - periodic_table: pure periodic table
//! - compound: pure molecule
//! - reaction: pure chemical reaction
//! - audio_channel: pure audio channel
//! - audio_bus: pure audio bus
//! - audio_effect: pure audio effect
//! - effect_chain: pure effect chain
//! - master_output: pure master output
//! - eq: pure parametric EQ
//! - compressor: pure compressor
//! - limiter: pure limiter
//! - gate: pure gate
//! - delay: pure delay
//! - reverb: pure reverb
//! - meter: pure peak and RMS meter
//! - audio_timeline: pure multi-track timeline
//! - video_timeline: pure AI video timeline
//! - layer_stack: pure layer stack
//! - onion_viewer: pure onion-skin viewer
//! - mask: pure mask
//! - editor_ui: pure editor panel
//! - adjustment: pure adjustment layer
//! - layer_group: pure layer group
//! - selection: pure selection
//! - brush: pure brush
//! - text_layer: pure text layer
//! - transform: pure transform
//! - curves: pure curves
//! - filter: pure filter
//! - smart_object: pure smart object
//! - history: pure undo / redo
//! - pixel_canvas: pure pixel canvas
//! - object3d: pure 3D object
//! - smart_skin: pure drag-and-drop skin
//! - ai_editor: pure AI editor access
//! - compositor: pure video compositor
//! - input_device: pure input device
//! - output: pure frame output
//! - audio_output: pure audio output
//! - session: pure combined session
//! - preview: pure session preview
//! - program: pure live program
//! - live_camera: pure live camera
//! - ai_operator: pure AI news crew
//! - topic_background: pure topic background
//! - performance: pure singer performance
//! - show_state: pure show state
//! - cue_sheet: pure cue sheet
//! - lower_third: pure lower third
//! - keyer: pure background keyer
//! - ai_director: pure AI scene director
//! - storyboard: pure storyboard
//! - speech: pure live speech
//! - action: pure detected action
//! - effect: pure scene effect
//! - effect_list: pure effect list
//! - live_feed: pure live feed
//! - scene_switch: pure scene switch
//! - multi_angle: pure multi angle
//! - multi_monitor: pure multi monitor
//! - options_menu: pure options menu
//! - studio: pure studio mode
//! - studio_transition: pure studio transition
//! - multiview: pure multiview
//! - tally: pure tally
//! - program_record: pure program recording
//! - program_stream: pure program stream
//! - virtual_camera: pure virtual camera
//! - output_bus: pure output bus
//! - frame_clock: pure frame clock
//! - replay_buffer: pure replay buffer

pub mod bitstream;
pub mod dstr;
pub mod utf8;
pub mod diff;
pub mod crc32;
pub mod ai;
pub mod array_serializer;
pub mod text_lookup;
pub mod path_extension;
pub mod bmem;
pub mod util_uint64;
pub mod util_uint128;
pub mod deque;
pub mod darray;
pub mod config;
pub mod lexer;
pub mod serializer;
pub mod profiler;
pub mod task;
pub mod file_serializer;
pub mod buffered_serializer;
pub mod cf_lexer;
pub mod config_parser;
pub mod source_profiler;
pub mod pipe;
pub mod frame_rate;
pub mod audio_math;
pub mod video_fourcc;
pub mod format_conversion;
pub mod video_matrices;
pub mod video_frame;
pub mod audio_format;
pub mod video_scaler;
pub mod audio_buffer;
pub mod media_timing;
pub mod audio_mixer;
pub mod vec2;
pub mod vec3;
pub mod vec4;
pub mod matrix3;
pub mod matrix4;
pub mod quat;
pub mod axisang;
pub mod plane;
pub mod bounds;
pub mod half;
pub mod signal;
pub mod proc;
pub mod calldata;
pub mod decl;
pub mod obs_data;
pub mod obs_properties;
pub mod source_context;
pub mod source_registry;
pub mod output_context;
pub mod encoder_context;
pub mod service_context;
pub mod output_registry;
pub mod encoder_registry;
pub mod service_registry;
pub mod scene_context;
pub mod scene_registry;
pub mod scene_item;
pub mod filter_context;
pub mod filter_chain;
pub mod filter_registry;
pub mod core;
pub mod module;
pub mod module_registry;
pub mod display_context;
pub mod display_registry;
pub mod hotkey;
pub mod hotkey_registry;
pub mod scene_collection;
pub mod collection_registry;
pub mod transition;
pub mod transition_registry;
pub mod audio_monitor;
pub mod monitor_registry;
pub mod recording;
pub mod recording_registry;
pub mod streaming;
pub mod streaming_registry;
pub mod replay_buffer;
pub mod replay_registry;
pub mod virtual_cam;
pub mod virtual_cam_registry;
pub mod profile;
pub mod profile_registry;
pub mod canvas;
pub mod canvas_registry;
pub mod render_item;
pub mod render_list;
pub mod camera;
pub mod projector;
pub mod bone;
pub mod animation_timeline;
pub mod physics;
pub mod settings_assist;
pub mod performance_capture;
pub mod character_renderer;
pub mod skeleton_connector;
pub mod ai_animator;
pub mod physics_sim;
pub mod learning_pipeline;
pub mod environment;
pub mod pilot;
pub mod flight_session;
pub mod flight_replay;
pub mod wind;
pub mod plane_renderer;
pub mod camera_follow;
pub mod view_frame;
pub mod light;
pub mod light_rig;
pub mod camera_rig;
pub mod camera_cut;
pub mod material;
pub mod lighting;
pub mod material_assign;
pub mod shaded_render;
pub mod material_registry;
pub mod material_factory;
pub mod materials_lab;
pub mod periodic_table;
pub mod compound;
pub mod reaction;
pub mod audio_channel;
pub mod audio_bus;
pub mod audio_effect;
pub mod effect_chain;
pub mod master_output;
pub mod eq;
pub mod compressor;
pub mod limiter;
pub mod gate;
pub mod delay;
pub mod reverb;
pub mod meter;
pub mod audio_timeline;
pub mod video_timeline;
pub mod layer_stack;
pub mod onion_viewer;
pub mod mask;
pub mod editor_ui;
pub mod adjustment;
pub mod layer_group;
pub mod selection;
pub mod brush;
pub mod text_layer;
pub mod transform;
pub mod curves;
pub mod filter;
pub mod smart_object;
pub mod history;
pub mod pixel_canvas;
pub mod object3d;
pub mod smart_skin;
pub mod ai_editor;
pub mod compositor;
pub mod input_device;
pub mod output;
pub mod audio_output;
pub mod session;
pub mod preview;
pub mod program;
pub mod live_camera;
pub mod ai_operator;
pub mod topic_background;
pub mod performance;
pub mod show_state;
pub mod cue_sheet;
pub mod lower_third;
pub mod keyer;
pub mod ai_director;
pub mod storyboard;
pub mod speech;
pub mod action;
pub mod effect;
pub mod effect_list;
pub mod live_feed;
pub mod scene_switch;
pub mod multi_angle;
pub mod multi_monitor;
pub mod options_menu;
pub mod studio;
pub mod studio_transition;
pub mod multiview;
pub mod tally;
pub mod program_record;
pub mod program_stream;
pub mod virtual_camera;
pub mod output_bus;
pub mod frame_clock;
pub mod instant_replay;
pub mod replay_save;
pub mod export;
pub mod file_writer;
pub mod share;
pub mod notify;
pub mod status;
pub mod keybind;
pub mod help;
pub mod bind_profile;
pub mod log;
pub mod undo;
pub mod redo;
pub mod macro_seq;
pub mod schedule;
pub mod nal_parser;
pub mod output_delay;
pub mod deinterlace;
pub mod missing_files;
pub mod fingerprint;
pub mod file_cache;
pub mod color_filter;
pub mod crop_filter;
pub mod scale_filter;
pub mod noise_filter;
pub mod gain_filter;
pub mod compressor_filter;
pub mod limiter_filter;
pub mod gate_filter;
pub mod eq_filter;
pub mod reverb_filter;
pub mod fade_transition;
pub mod swipe_transition;
pub mod slide_transition;
pub mod cut_transition;
pub mod image_source;
pub mod text_source;
pub mod color_source;
pub mod browser_source;
pub mod media_source;
pub mod window_capture;
pub mod display_capture;

pub use bitstream::{BitstreamReader, Reader, Position, bitstream_reader};
pub use dstr::Dstr;
pub use utf8::{truncate_utf8, truncate_filename};
pub use crc32::crc32;
pub use ai::{AiAssistant, AiConfig, Backend};
pub use array_serializer::ArrayOutput;
pub use text_lookup::TextLookup;
pub use path_extension::path_extension;
pub use bmem::{is_valid_alloc_size, alignment};
pub use util_uint64::util_mul_div64;
pub use util_uint128::{U128, util_add128, util_mul64_64};
pub use deque::{Deque, ByteDeque};
pub use darray::DArray;
pub use config::Config;
pub use lexer::{Lexer, Token, TokenType};
pub use serializer::{Serializer, MemSerializer, s_w8, s_wl16, s_wl32, s_wl64};
pub use profiler::Profiler;
pub use task::TaskQueue;
pub use file_serializer::FileSerializer;
pub use buffered_serializer::BufferedSerializer;
pub use cf_lexer::{CfLexer, CfToken, CfTokenType};
pub use config_parser::parse_config_string;
pub use source_profiler::SourceProfiler;
pub use pipe::Pipe;
pub use frame_rate::MediaFps;
pub use audio_math::{mul_to_db, db_to_mul};
pub use video_fourcc::{VideoFormat, video_format_from_fourcc};
pub use format_conversion::{y_plane_size, chroma_420_size, chroma_422_size, uyvy_to_yuy2_line};
pub use video_matrices::{ColorSpace, identity_matrix, limited_to_full_scale};
pub use video_frame::VideoFrame;
pub use audio_format::{AudioFormat, SpeakerLayout, audio_size};
pub use video_scaler::{ScaleInfo, lerp_weight};
pub use audio_buffer::AudioBuffer;
pub use media_timing::{sample_duration_ns, frame_duration_ns, frames_to_samples};
pub use audio_mixer::{apply_gain, sum_buffers, peak};
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
pub use matrix3::Matrix3;
pub use matrix4::Matrix4;
pub use quat::Quat;
pub use axisang::AxisAng;
pub use plane::Plane;
pub use bounds::Bounds;
pub use half::{f32_to_f16, f16_to_f32};
pub use signal::SignalHandler;
pub use proc::ProcHandler;
pub use calldata::{Calldata, CallValue, CallType, CallError};
pub use decl::{parse_decl, DeclInfo, DeclParam, DeclError, ParamFlags};
pub use obs_data::{ObsData, DataValue, DataError};
pub use obs_properties::{Properties, Property, PropertyType, PropsError};
pub use source_context::{SourceContext, SourceError};
pub use source_registry::{SourceRegistry, RegistryError};
pub use output_context::{OutputContext, OutputError};
pub use encoder_context::{EncoderContext, EncoderType, EncoderError};
pub use service_context::{ServiceContext, ServiceError};
pub use output_registry::{OutputRegistry, OutputRegistryError};
pub use encoder_registry::{EncoderRegistry, EncoderRegistryError};
pub use service_registry::{ServiceRegistry, ServiceRegistryError};
pub use scene_context::{SceneContext, SceneError};
pub use scene_registry::{SceneRegistry, SceneRegistryError};
pub use scene_item::{SceneItem, Transform, SceneItemError};
pub use filter_context::{FilterContext, FilterError};
pub use filter_chain::{FilterChain, FilterChainError};
pub use filter_registry::{FilterRegistry, FilterRegistryError};
pub use core::{Core, CoreError};
pub use module::{Module, ModuleError};
pub use module_registry::{ModuleRegistry, ModuleRegistryError};
pub use display_context::{DisplayContext, DisplayError};
pub use display_registry::{DisplayRegistry, DisplayRegistryError};
pub use hotkey::{Hotkey, HotkeyError};
pub use hotkey_registry::{HotkeyRegistry, HotkeyRegistryError};
pub use scene_collection::{SceneCollection, CollectionError};
pub use collection_registry::{CollectionRegistry, CollectionRegistryError};
pub use transition::{Transition, TransitionError};
pub use transition_registry::{TransitionRegistry, TransitionRegistryError};
pub use audio_monitor::{AudioMonitor, MonitorError};
pub use monitor_registry::{MonitorRegistry, MonitorRegistryError};
pub use recording::{Recording, RecordingError};
pub use recording_registry::{RecordingRegistry, RecordingRegistryError};
pub use streaming::{Streaming, StreamingError};
pub use streaming_registry::{StreamingRegistry, StreamingRegistryError};
pub use replay_buffer::{ReplayBuffer, ReplayError};
pub use replay_registry::{ReplayRegistry, ReplayRegistryError};
pub use virtual_cam::{VirtualCam, VirtualCamError};
pub use virtual_cam_registry::{VirtualCamRegistry, VirtualCamRegistryError};
pub use profile::{Profile, ProfileError};
pub use profile_registry::{ProfileRegistry, ProfileRegistryError};
pub use canvas::{Canvas, CanvasError};
pub use canvas_registry::{CanvasRegistry, CanvasRegistryError};
pub use render_item::RenderItem;
pub use render_list::RenderList;
pub use camera::Camera;
pub use projector::Projector;
pub use bone::{Bone, BoneError};
pub use animation_timeline::{AnimationTimeline, TimelineError};
pub use physics::PhysicsSettings;
pub use settings_assist::SettingsAssist;
pub use performance_capture::{PerformanceCapture, CaptureError};
pub use character_renderer::CharacterRenderer;
pub use skeleton_connector::{SkeletonConnector, ConnectorError};
pub use ai_animator::AiAnimator;
pub use physics_sim::PhysicsSim;
pub use learning_pipeline::LearningPipeline;
pub use environment::{Environment, EnvironmentError};
pub use pilot::Pilot;
pub use flight_session::FlightSession;
pub use flight_replay::FlightReplay;
pub use wind::Wind;
pub use plane_renderer::PlaneRenderer;
pub use camera_follow::CameraFollow;
pub use view_frame::ViewFrame;
pub use light::{Light, LightError};
pub use light_rig::{LightRig, LightRigError};
pub use camera_rig::{CameraRig, CameraRigError};
pub use camera_cut::{CameraCut, CutError};
pub use material::{Material, MaterialError};
pub use lighting::Lighting;
pub use material_assign::{MaterialAssign, AssignError};
pub use shaded_render::{ShadedRender, ShadedItem};
pub use material_registry::{MaterialRegistry, MaterialRegistryError};
pub use material_factory::MaterialFactory;
pub use materials_lab::{MaterialsLab, Atom, LabError};
pub use periodic_table::{PeriodicTable, Element};
pub use compound::{Compound, CompoundError};
pub use reaction::{Reaction, ReactionError};
pub use audio_channel::{AudioChannel, ChannelError};
pub use audio_bus::{AudioBus, BusError};
pub use audio_effect::{AudioEffect, EffectError};
pub use effect_chain::{EffectChain, ChainError};
pub use master_output::MasterOutput;
pub use eq::{EqBand, EqError};
pub use compressor::Compressor;
pub use limiter::Limiter;
pub use gate::Gate;
pub use delay::Delay;
pub use reverb::Reverb;
pub use meter::Meter;
pub use audio_timeline::{AudioTimeline, Track, TimelineError as AudioTimelineError};
pub use video_timeline::{VideoTimeline, Clip, VideoError};
pub use layer_stack::{LayerStack, Layer, BlendMode, LayerError};
pub use onion_viewer::{OnionViewer, OnionFrame};
pub use mask::{Mask, MaskSet, MaskError};
pub use editor_ui::EditorUi;
pub use adjustment::{Adjustment, AdjustKind, AdjustError};
pub use layer_group::{GroupSet, LayerGroup, GroupError};
pub use selection::{Selection, SelectionError};
pub use brush::{Brush, BrushError};
pub use text_layer::{TextLayer, TextError};
pub use transform::Transform as LayerTransform;
pub use curves::Curves;
pub use filter::{Filter, FilterKind, FilterError as ImageFilterError};
pub use smart_object::{SmartObject, SmartError};
pub use history::History;
pub use pixel_canvas::{PixelCanvas, PixelError};
pub use object3d::{Object3D, ObjectError};
pub use smart_skin::{SmartSkin, SkinError};
pub use ai_editor::AiEditor;
pub use compositor::Compositor;
pub use input_device::InputDevice;
pub use output::{Output, OutputFrame};
pub use audio_output::{AudioOutput, AudioFrame};
pub use session::{Session, SessionFrame};
pub use preview::{Preview, PreviewFrame};
pub use program::Program;
pub use live_camera::{LiveCamera, CameraError};
pub use ai_operator::{AiOperator, CrewState};
pub use topic_background::TopicBackground;
pub use performance::{Performance, PerformanceError};
pub use show_state::{ShowState, ShowMode};
pub use cue_sheet::CueSheet;
pub use lower_third::LowerThird;
pub use keyer::Keyer;
pub use ai_director::{AiDirector, SceneKind};
pub use storyboard::Storyboard;
pub use speech::Speech;
pub use action::Action;
pub use effect::{Effect, FxError};
pub use effect_list::{EffectList, ListError};
pub use live_feed::{LiveFeed, FeedError};
pub use scene_switch::{SceneSwitch, SwitchError};
pub use multi_angle::{MultiAngle, AngleError};
pub use multi_monitor::{MultiMonitor, ScreenError};
pub use options_menu::{OptionsMenu, MenuError};
pub use studio::Studio;
pub use studio_transition::StudioTransition;
pub use multiview::{Multiview, ViewError};
pub use tally::{Tally, TallyError};
pub use program_record::ProgramRecord;
pub use program_stream::{ProgramStream, StreamError};
pub use virtual_camera::VirtualCamera;
pub use output_bus::OutputBus;
pub use frame_clock::FrameClock;
pub use instant_replay::InstantReplay;
pub use replay_save::{ReplaySave, SaveError, SaveFormat};
pub use export::{Export, ExportError};
pub use file_writer::FileWriter;
pub use share::{Share, ShareError};
pub use notify::{Notify, NotifyError};
pub use status::Status;
pub use keybind::{Keybind, KeybindError};
pub use help::Help;
pub use bind_profile::{BindProfile, BindProfileError};
pub use log::Log;
pub use undo::Undo;
pub use redo::Redo;
pub use macro_seq::{Macro, MacroError};
pub use schedule::{Schedule, ScheduleError};
pub use nal_parser::{NalParser, NalUnit, NalError};
pub use output_delay::OutputDelay;
pub use deinterlace::Deinterlace;
pub use missing_files::{MissingFiles, MissingError};
pub use fingerprint::{FingerprintStore, Fingerprint, FingerprintError};
pub use file_cache::{FileCache, CacheError};
pub use color_filter::ColorFilter;
pub use crop_filter::{CropFilter, CropError};
pub use scale_filter::{ScaleFilter, ScaleError};
pub use noise_filter::NoiseFilter;
pub use gain_filter::GainFilter;
pub use compressor_filter::Compressor as CompressorFilter;
pub use limiter_filter::Limiter as LimiterFilter;
pub use gate_filter::Gate as GateFilter;
pub use eq_filter::Eq;
pub use reverb_filter::Reverb as ReverbFilter;
pub use fade_transition::FadeTransition;
pub use swipe_transition::SwipeTransition;
pub use slide_transition::SlideTransition;
pub use cut_transition::CutTransition;
pub use image_source::{ImageSource, ImageError};
pub use text_source::{TextSource, TextError as TextSourceError};
pub use color_source::ColorSource;
pub use browser_source::{BrowserSource, BrowserError};
pub use media_source::{MediaSource, MediaError};
pub use window_capture::{WindowCapture, WindowError};
pub use display_capture::DisplayCapture;