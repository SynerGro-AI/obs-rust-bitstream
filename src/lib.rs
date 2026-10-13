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