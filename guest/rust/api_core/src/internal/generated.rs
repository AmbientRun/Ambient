#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused)]
pub use raw::ambient_core;
#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant
)]
mod raw {
    pub mod ambient_core {
        pub mod animation {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_ANIMATION_PLAYER: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::is_animation_player")
                });
                #[doc = "**Is animation player**: This entity is treated as an animation player. Attach an animation node as a child for it to play.\n\n*Attributes*: Debuggable, Networked"]
                pub fn is_animation_player() -> Component<()> {
                    *IS_ANIMATION_PLAYER
                }
                static ANIMATION_ERRORS: Lazy<Component<Vec<String>>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::animation_errors")
                });
                #[doc = "**Animation errors**: A list of errors that were produced trying to play the animation.\n\n*Attributes*: Debuggable"]
                pub fn animation_errors() -> Component<Vec<String>> {
                    *ANIMATION_ERRORS
                }
                static APPLY_ANIMATION_PLAYER: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::apply_animation_player")
                });
                #[doc = "**Apply animation player**: Apply the designated animation player to this entity and its sub-tree.\n\n*Attributes*: Debuggable, Networked"]
                pub fn apply_animation_player() -> Component<EntityId> {
                    *APPLY_ANIMATION_PLAYER
                }
                static PLAY_CLIP_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::play_clip_from_url")
                });
                #[doc = "**Play clip from URL**: Make this entity a 'play animation clip' node. The value is the URL to the clip we'd like to play.\n\n*Attributes*: Debuggable, Networked"]
                pub fn play_clip_from_url() -> Component<String> {
                    *PLAY_CLIP_FROM_URL
                }
                static LOOPING: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::looping"));
                #[doc = "**Looping**: When this is true, the animation clip will repeat infinitely.\n\n*Attributes*: Debuggable, Networked"]
                pub fn looping() -> Component<bool> {
                    *LOOPING
                }
                static SPEED: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::speed"));
                #[doc = "**Speed**: Animation playback speed. Default is 1, higher values speeds up the animation.\n\n*Attributes*: Debuggable, Networked"]
                pub fn speed() -> Component<f32> {
                    *SPEED
                }
                static START_TIME: Lazy<Component<Duration>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::start_time"));
                #[doc = "**Start time**: Start time of an animation node.\n\n*Attributes*: Debuggable, Networked"]
                pub fn start_time() -> Component<Duration> {
                    *START_TIME
                }
                static FREEZE_AT_PERCENTAGE: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::freeze_at_percentage")
                });
                #[doc = "**Freeze at percentage**: Sample the input animation at a certain percentage of the animation track length.\n\n*Attributes*: Debuggable, Networked"]
                pub fn freeze_at_percentage() -> Component<f32> {
                    *FREEZE_AT_PERCENTAGE
                }
                static FREEZE_AT_TIME: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::freeze_at_time")
                });
                #[doc = "**Freeze at time**: Sample the input animation at a certain time (in seconds).\n\n*Attributes*: Debuggable, Networked"]
                pub fn freeze_at_time() -> Component<f32> {
                    *FREEZE_AT_TIME
                }
                static CLIP_DURATION: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::clip_duration")
                });
                #[doc = "**Clip duration**: The clip duration is loaded from the clip, and then applied to the entity.\n\n*Attributes*: Debuggable"]
                pub fn clip_duration() -> Component<f32> {
                    *CLIP_DURATION
                }
                static CLIP_LOADED: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::clip_loaded"));
                #[doc = "**Clip loaded**: The clip has been loaded.\n\n*Attributes*: Debuggable"]
                pub fn clip_loaded() -> Component<()> {
                    *CLIP_LOADED
                }
                static BLEND: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::blend"));
                #[doc = "**Blend**: Blend two animations together. The values is the blend weight. Use `children` to set the animations. Blend 0 means we only sample from the first animation, 1 means only the second one, and values in between blend between them.\n\n*Attributes*: Debuggable, Networked"]
                pub fn blend() -> Component<f32> {
                    *BLEND
                }
                static MASK_BIND_IDS: Lazy<Component<Vec<String>>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::mask_bind_ids")
                });
                #[doc = "**Mask bind ids**: List of bind ids that will be masked.\n\n*Attributes*: Debuggable, Networked"]
                pub fn mask_bind_ids() -> Component<Vec<String>> {
                    *MASK_BIND_IDS
                }
                static MASK_WEIGHTS: Lazy<Component<Vec<f32>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::mask_weights"));
                #[doc = "**Mask weights**: Weights for each bind id in `mask_bind_ids`.\n\n*Attributes*: Debuggable, Networked"]
                pub fn mask_weights() -> Component<Vec<f32>> {
                    *MASK_WEIGHTS
                }
                static RETARGET_MODEL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::retarget_model_from_url")
                });
                #[doc = "**Retarget Model from URL**: Retarget the animation using the model at the given URL.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn retarget_model_from_url() -> Component<String> {
                    *RETARGET_MODEL_FROM_URL
                }
                static RETARGET_ANIMATION_SCALED: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::retarget_animation_scaled")
                });
                #[doc = "**Retarget animation scaled**: Retarget animation scaled. True means normalize hip.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn retarget_animation_scaled() -> Component<bool> {
                    *RETARGET_ANIMATION_SCALED
                }
                static APPLY_BASE_POSE: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::animation::apply_base_pose")
                });
                #[doc = "**Apply base pose**: Apply the base pose to this clip.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn apply_base_pose() -> Component<()> {
                    *APPLY_BASE_POSE
                }
                static BIND_ID: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::bind_id"));
                #[doc = "**Bind id**: Animation bind ID.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn bind_id() -> Component<String> {
                    *BIND_ID
                }
                static BIND_IDS: Lazy<Component<Vec<String>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::animation::bind_ids"));
                #[doc = "**Bind ids**: Animation bind IDs.\n\n*Attributes*: Debuggable, Store"]
                pub fn bind_ids() -> Component<Vec<String>> {
                    *BIND_IDS
                }
            }
        }
        pub mod app {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static CURSOR_POSITION: Lazy<Component<Vec2>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::cursor_position"));
                #[doc = "**Cursor position**: Absolute mouse cursor position in screen-space. This is the *logical* position. Multiply by the `window_scale_factor` to get the physical position.\n\n*Attributes*: MaybeResource, Debuggable, Networked"]
                pub fn cursor_position() -> Component<Vec2> {
                    *CURSOR_POSITION
                }
                static DELTA_TIME: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::delta_time"));
                #[doc = "**Delta time**: How long the previous tick took in seconds.\n\n*Attributes*: Debuggable, Resource"]
                pub fn delta_time() -> Component<f32> {
                    *DELTA_TIME
                }
                static EPOCH_TIME: Lazy<Component<Duration>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::epoch_time"));
                #[doc = "**Epoch time**: Time since epoch (Jan 1, 1970). Non_monotonic.\n\n*Attributes*: Debuggable, Resource"]
                pub fn epoch_time() -> Component<Duration> {
                    *EPOCH_TIME
                }
                static GAME_TIME: Lazy<Component<Duration>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::game_time"));
                #[doc = "**Game time**: Time since the game was started. Monotonic.\n\n*Attributes*: Debuggable, Resource"]
                pub fn game_time() -> Component<Duration> {
                    *GAME_TIME
                }
                static ELEMENT: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::element"));
                #[doc = "**Element**: The identifier of the `Element` that controls this entity.\n\nThis is automatically generated by `ElementTree`.\n\n*Attributes*: Debuggable, Networked"]
                pub fn element() -> Component<String> {
                    *ELEMENT
                }
                static ELEMENT_UNMANAGED_CHILDREN: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::app::element_unmanaged_children")
                });
                #[doc = "**Element unmanaged children**: If this is set, the user is expected to manage the children of the `Element` themselves.\n\n*Attributes*: Debuggable, Networked"]
                pub fn element_unmanaged_children() -> Component<()> {
                    *ELEMENT_UNMANAGED_CHILDREN
                }
                static MAIN_SCENE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::main_scene"));
                #[doc = "**Main scene**: If attached, this entity belongs to the main scene.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn main_scene() -> Component<()> {
                    *MAIN_SCENE
                }
                static MAP_SEED: Lazy<Component<u64>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::map_seed"));
                #[doc = "**Map seed**: A random number seed for this map.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn map_seed() -> Component<u64> {
                    *MAP_SEED
                }
                static NAME: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::name"));
                #[doc = "**Name**: A human-friendly name for this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn name() -> Component<String> {
                    *NAME
                }
                static DESCRIPTION: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::description"));
                #[doc = "**Description**: A human-friendly description for this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn description() -> Component<String> {
                    *DESCRIPTION
                }
                static MAIN_PACKAGE_NAME: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::main_package_name"));
                #[doc = "**Main Package Name**: The name of the main package being run.\n\nDefaults to \"Ambient\".\n\n*Attributes*: Debuggable, Resource"]
                pub fn main_package_name() -> Component<String> {
                    *MAIN_PACKAGE_NAME
                }
                static SELECTABLE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::selectable"));
                #[doc = "**Selectable**: If attached, this object can be selected in the editor.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn selectable() -> Component<()> {
                    *SELECTABLE
                }
                static SNAP_TO_GROUND: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::snap_to_ground"));
                #[doc = "**Snap to ground**: This object should automatically be moved with the terrain if the terrain is changed.\n\nThe value is the offset from the terrain.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn snap_to_ground() -> Component<f32> {
                    *SNAP_TO_GROUND
                }
                static TAGS: Lazy<Component<Vec<String>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::tags"));
                #[doc = "**Tags**: Tags for categorizing this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn tags() -> Component<Vec<String>> {
                    *TAGS
                }
                static UI_SCENE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::ui_scene"));
                #[doc = "**UI scene**: If attached, this entity belongs to the UI scene.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn ui_scene() -> Component<()> {
                    *UI_SCENE
                }
                static WINDOW_LOGICAL_SIZE: Lazy<Component<UVec2>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::app::window_logical_size")
                });
                #[doc = "**Window logical size**: The logical size is the physical size divided by the scale factor.\n\n*Attributes*: MaybeResource, Debuggable, Networked"]
                pub fn window_logical_size() -> Component<UVec2> {
                    *WINDOW_LOGICAL_SIZE
                }
                static WINDOW_PHYSICAL_SIZE: Lazy<Component<UVec2>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::app::window_physical_size")
                });
                #[doc = "**Window physical size**: The physical size is the actual number of pixels on the screen.\n\n*Attributes*: MaybeResource, Debuggable, Networked"]
                pub fn window_physical_size() -> Component<UVec2> {
                    *WINDOW_PHYSICAL_SIZE
                }
                static WINDOW_SCALE_FACTOR: Lazy<Component<f64>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::app::window_scale_factor")
                });
                #[doc = "**Window scale factor**: The DPI/pixel scale factor of the window.\n\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays.\n\n*Attributes*: MaybeResource, Debuggable, Networked"]
                pub fn window_scale_factor() -> Component<f64> {
                    *WINDOW_SCALE_FACTOR
                }
                static REF_COUNT: Lazy<Component<u32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::app::ref_count"));
                #[doc = "**Reference count**: Ref-counted enity. If this entity doesn't have a `parent` component, and the ref count reaches 0, it will be removed together with all its children recursively.\n\n*Attributes*: MaybeResource, Debuggable, Networked"]
                pub fn ref_count() -> Component<u32> {
                    *REF_COUNT
                }
            }
        }
        pub mod audio {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_AUDIO_PLAYER: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::is_audio_player"));
                #[doc = "**Is audio player**: The entity is an audio player.\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn is_audio_player() -> Component<()> {
                    *IS_AUDIO_PLAYER
                }
                static IS_SPATIAL_AUDIO_PLAYER: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::audio::is_spatial_audio_player")
                });
                #[doc = "**Is spatial audio player**: The entity is a spatial audio player.\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn is_spatial_audio_player() -> Component<()> {
                    *IS_SPATIAL_AUDIO_PLAYER
                }
                static SPATIAL_AUDIO_EMITTER: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::audio::spatial_audio_emitter")
                });
                #[doc = "**Spatial audio emitter**: The entity is a spatial audio emitter.\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn spatial_audio_emitter() -> Component<EntityId> {
                    *SPATIAL_AUDIO_EMITTER
                }
                static SPATIAL_AUDIO_LISTENER: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::audio::spatial_audio_listener")
                });
                #[doc = "**Spatial audio listener**: The entity is a spatial audio listener.\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn spatial_audio_listener() -> Component<EntityId> {
                    *SPATIAL_AUDIO_LISTENER
                }
                static LOOPING: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::looping"));
                #[doc = "**Looping**: Whether or not the audio should loop.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn looping() -> Component<bool> {
                    *LOOPING
                }
                static ONEPOLE_LPF: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::onepole_lpf"));
                #[doc = "**One pole low pass filter**: With this component, the audio will be filtered with a one pole low pass filter.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn onepole_lpf() -> Component<f32> {
                    *ONEPOLE_LPF
                }
                static PLAYING_SOUND: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::playing_sound"));
                #[doc = "**Playing sound**: The entity with this comp is a playing sound.\n\nWe can attach other components to it to control the sound parameters.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn playing_sound() -> Component<()> {
                    *PLAYING_SOUND
                }
                static AMPLITUDE: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::amplitude"));
                #[doc = "**Amplitude**: The amplitude of the audio.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn amplitude() -> Component<f32> {
                    *AMPLITUDE
                }
                static PANNING: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::panning"));
                #[doc = "**Panning**: The panning of the audio.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn panning() -> Component<f32> {
                    *PANNING
                }
                static LPF: Lazy<Component<Vec2>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::lpf"));
                #[doc = "**Low_pass filter**: Low pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn lpf() -> Component<Vec2> {
                    *LPF
                }
                static HPF: Lazy<Component<Vec2>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::hpf"));
                #[doc = "**High_pass filter**: High pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn hpf() -> Component<Vec2> {
                    *HPF
                }
                static AUDIO_URL: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::audio_url"));
                #[doc = "**Audio URL**: The URL of the assets.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn audio_url() -> Component<String> {
                    *AUDIO_URL
                }
                static PLAY_NOW: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::play_now"));
                #[doc = "**Trigger at this frame**: The system will watch for this component and PLAY the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn play_now() -> Component<()> {
                    *PLAY_NOW
                }
                static STOP_NOW: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::audio::stop_now"));
                #[doc = "**Stop at this frame**: The system will watch for this component and STOP the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: MaybeResource, Debuggable"]
                pub fn stop_now() -> Component<()> {
                    *STOP_NOW
                }
            }
        }
        pub mod camera {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static ACTIVE_CAMERA: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::active_camera"));
                #[doc = "**Active camera**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\n\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn active_camera() -> Component<f32> {
                    *ACTIVE_CAMERA
                }
                static ASPECT_RATIO: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::aspect_ratio"));
                #[doc = "**Aspect ratio**: The aspect ratio of this camera.\n\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn aspect_ratio() -> Component<f32> {
                    *ASPECT_RATIO
                }
                static ASPECT_RATIO_FROM_WINDOW: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::aspect_ratio_from_window")
                });
                #[doc = "**Aspect ratio from window**: If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn aspect_ratio_from_window() -> Component<EntityId> {
                    *ASPECT_RATIO_FROM_WINDOW
                }
                static FAR: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::far"));
                #[doc = "**Far plane**: The far plane of this camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn far() -> Component<f32> {
                    *FAR
                }
                static FOG: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::fog"));
                #[doc = "**Fog**: If attached, this camera will see/render fog.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn fog() -> Component<()> {
                    *FOG
                }
                static FOVY: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::fovy"));
                #[doc = "**Field of View Y**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn fovy() -> Component<f32> {
                    *FOVY
                }
                static NEAR: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::near"));
                #[doc = "**Near plane**: The near plane of this camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn near() -> Component<f32> {
                    *NEAR
                }
                static ORTHOGRAPHIC: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::orthographic"));
                #[doc = "**Orthographic projection**: If attached, this camera will use a standard orthographic projection matrix.\n\nEnsure that the `orthographic_` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn orthographic() -> Component<()> {
                    *ORTHOGRAPHIC
                }
                static ORTHOGRAPHIC_BOTTOM: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::orthographic_bottom")
                });
                #[doc = "**Orthographic bottom**: The bottom bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn orthographic_bottom() -> Component<f32> {
                    *ORTHOGRAPHIC_BOTTOM
                }
                static ORTHOGRAPHIC_FROM_WINDOW: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::orthographic_from_window")
                });
                #[doc = "**Orthographic from window**: The bounds of this orthographic camera will be updated to match the window automatically. Should point to an entity with a `window_logical_size` component.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn orthographic_from_window() -> Component<EntityId> {
                    *ORTHOGRAPHIC_FROM_WINDOW
                }
                static ORTHOGRAPHIC_LEFT: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::orthographic_left")
                });
                #[doc = "**Orthographic left**: The left bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn orthographic_left() -> Component<f32> {
                    *ORTHOGRAPHIC_LEFT
                }
                static ORTHOGRAPHIC_RIGHT: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::orthographic_right")
                });
                #[doc = "**Orthographic right**: The right bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn orthographic_right() -> Component<f32> {
                    *ORTHOGRAPHIC_RIGHT
                }
                static ORTHOGRAPHIC_TOP: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::orthographic_top")
                });
                #[doc = "**Orthographic top**: The top bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn orthographic_top() -> Component<f32> {
                    *ORTHOGRAPHIC_TOP
                }
                static PERSPECTIVE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::perspective"));
                #[doc = "**Perspective projection**: If attached, this camera will use a standard perspective projection matrix.\n\nEnsure that `near` and `far` are set.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn perspective() -> Component<()> {
                    *PERSPECTIVE
                }
                static PERSPECTIVE_INFINITE_REVERSE: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::camera::perspective_infinite_reverse")
                });
                #[doc = "**Perspective-infinite-reverse projection**: If attached, this camera will use a perspective-infinite-reverse projection matrix.\n\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn perspective_infinite_reverse() -> Component<()> {
                    *PERSPECTIVE_INFINITE_REVERSE
                }
                static PROJECTION: Lazy<Component<Mat4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::projection"));
                #[doc = "**Projection**: The projection matrix of this camera.\n\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn projection() -> Component<Mat4> {
                    *PROJECTION
                }
                static PROJECTION_VIEW: Lazy<Component<Mat4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::projection_view"));
                #[doc = "**Projection-view**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn projection_view() -> Component<Mat4> {
                    *PROJECTION_VIEW
                }
                static SHADOWS_FAR: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::camera::shadows_far"));
                #[doc = "**Shadows far plane**: The far plane for the shadow camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn shadows_far() -> Component<f32> {
                    *SHADOWS_FAR
                }
            }
            #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
            #[doc = r""]
            #[doc = r" They do not have any runtime representation outside of the components that compose them."]
            pub mod concepts {
                use crate::prelude::*;
                #[doc = "**Camera**: Base components for a camera. You will need other components to make a fully-functioning camera.\n\n**Extends**: `ambient_core::transform::Transformable`"]
                #[derive(Clone, Debug)]
                pub struct Camera {
                    #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                    pub local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::near`\n\n**Suggested value**: `0.1f32`\n\n**Component description**: The near plane of this camera, measured in meters.\n\n"]
                    pub near: f32,
                    #[doc = "**Component**: `ambient_core::camera::projection`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n"]
                    pub projection: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::projection_view`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n"]
                    pub projection_view: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::active_camera`\n\n**Suggested value**: `0f32`\n\n**Component description**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n"]
                    pub active_camera: f32,
                    #[doc = "**Component**: `ambient_core::transform::inv_local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Converts a world position to a local position.\nThis is automatically updated.\n\n"]
                    pub inv_local_to_world: Mat4,
                    #[doc = r" Optional components."]
                    pub optional: CameraOptional,
                }
                #[doc = "Optional part of [Camera]."]
                #[derive(Clone, Debug, Default)]
                pub struct CameraOptional {
                    #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                    pub translation: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                    pub rotation: Option<Quat>,
                    #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                    pub scale: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::app::main_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the main scene.\n\n"]
                    pub main_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::app::ui_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the UI scene.\n\n"]
                    pub ui_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::player::user_id`\n\n**Description**: If set, this camera will only be used for the specified user.\n\n**Component description**: An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n"]
                    pub user_id: Option<String>,
                }
                impl crate::ecs::Concept for Camera {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::transform::components::local_to_world(),
                                self.local_to_world,
                            )
                            .with(crate::ambient_core::camera::components::near(), self.near)
                            .with(
                                crate::ambient_core::camera::components::projection(),
                                self.projection,
                            )
                            .with(
                                crate::ambient_core::camera::components::projection_view(),
                                self.projection_view,
                            )
                            .with(
                                crate::ambient_core::camera::components::active_camera(),
                                self.active_camera,
                            )
                            .with(
                                crate::ambient_core::transform::components::inv_local_to_world(),
                                self.inv_local_to_world,
                            );
                        if let Some(translation) = self.optional.translation {
                            entity.set(
                                crate::ambient_core::transform::components::translation(),
                                translation,
                            );
                        }
                        if let Some(rotation) = self.optional.rotation {
                            entity.set(
                                crate::ambient_core::transform::components::rotation(),
                                rotation,
                            );
                        }
                        if let Some(scale) = self.optional.scale {
                            entity.set(crate::ambient_core::transform::components::scale(), scale);
                        }
                        if let Some(main_scene) = self.optional.main_scene {
                            entity.set(
                                crate::ambient_core::app::components::main_scene(),
                                main_scene,
                            );
                        }
                        if let Some(ui_scene) = self.optional.ui_scene {
                            entity.set(crate::ambient_core::app::components::ui_scene(), ui_scene);
                        }
                        if let Some(user_id) = self.optional.user_id {
                            entity.set(crate::ambient_core::player::components::user_id(), user_id);
                        }
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some(Self {
                            local_to_world: entity::get_component(
                                id,
                                crate::ambient_core::transform::components::local_to_world(),
                            )?,
                            near: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::near(),
                            )?,
                            projection: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::projection(),
                            )?,
                            projection_view: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::projection_view(),
                            )?,
                            active_camera: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::active_camera(),
                            )?,
                            inv_local_to_world: entity::get_component(
                                id,
                                crate::ambient_core::transform::components::inv_local_to_world(),
                            )?,
                            optional: CameraOptional {
                                translation: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::translation(),
                                ),
                                rotation: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::rotation(),
                                ),
                                scale: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::scale(),
                                ),
                                main_scene: entity::get_component(
                                    id,
                                    crate::ambient_core::app::components::main_scene(),
                                ),
                                ui_scene: entity::get_component(
                                    id,
                                    crate::ambient_core::app::components::ui_scene(),
                                ),
                                user_id: entity::get_component(
                                    id,
                                    crate::ambient_core::player::components::user_id(),
                                ),
                            },
                        })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some(Self {
                            local_to_world: entity
                                .get(crate::ambient_core::transform::components::local_to_world())?,
                            near: entity.get(crate::ambient_core::camera::components::near())?,
                            projection: entity
                                .get(crate::ambient_core::camera::components::projection())?,
                            projection_view: entity
                                .get(crate::ambient_core::camera::components::projection_view())?,
                            active_camera: entity
                                .get(crate::ambient_core::camera::components::active_camera())?,
                            inv_local_to_world: entity.get(
                                crate::ambient_core::transform::components::inv_local_to_world(),
                            )?,
                            optional: CameraOptional {
                                translation: entity
                                    .get(crate::ambient_core::transform::components::translation()),
                                rotation: entity
                                    .get(crate::ambient_core::transform::components::rotation()),
                                scale: entity
                                    .get(crate::ambient_core::transform::components::scale()),
                                main_scene: entity
                                    .get(crate::ambient_core::app::components::main_scene()),
                                ui_scene: entity
                                    .get(crate::ambient_core::app::components::ui_scene()),
                                user_id: entity
                                    .get(crate::ambient_core::player::components::user_id()),
                            },
                        })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::transform::components::local_to_world(),
                                &crate::ambient_core::camera::components::near(),
                                &crate::ambient_core::camera::components::projection(),
                                &crate::ambient_core::camera::components::projection_view(),
                                &crate::ambient_core::camera::components::active_camera(),
                                &crate::ambient_core::transform::components::inv_local_to_world(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::transform::components::local_to_world(),
                            &crate::ambient_core::camera::components::near(),
                            &crate::ambient_core::camera::components::projection(),
                            &crate::ambient_core::camera::components::projection_view(),
                            &crate::ambient_core::camera::components::active_camera(),
                            &crate::ambient_core::transform::components::inv_local_to_world(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for Camera {
                    fn suggested() -> Self {
                        Self {
                            local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            near: 0.1f32,
                            projection: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            projection_view: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            active_camera: 0f32,
                            inv_local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            optional: Default::default(),
                        }
                    }
                }
                #[doc = "**Perspective Common Camera**: Base components for a perspective camera. Consider `perspective_camera` or `perspective_infinite_reverse_camera`.\n\n**Extends**: `ambient_core::camera::Camera`"]
                #[derive(Clone, Debug)]
                pub struct PerspectiveCommonCamera {
                    #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                    pub local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::near`\n\n**Suggested value**: `0.1f32`\n\n**Component description**: The near plane of this camera, measured in meters.\n\n"]
                    pub near: f32,
                    #[doc = "**Component**: `ambient_core::camera::projection`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n"]
                    pub projection: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::projection_view`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n"]
                    pub projection_view: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::active_camera`\n\n**Suggested value**: `0f32`\n\n**Component description**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n"]
                    pub active_camera: f32,
                    #[doc = "**Component**: `ambient_core::transform::inv_local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Converts a world position to a local position.\nThis is automatically updated.\n\n"]
                    pub inv_local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::fovy`\n\n**Suggested value**: `1f32`\n\n**Component description**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n"]
                    pub fovy: f32,
                    #[doc = "**Component**: `ambient_core::camera::aspect_ratio`\n\n**Suggested value**: `1f32`\n\n**Component description**: The aspect ratio of this camera.\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window.\n\n"]
                    pub aspect_ratio: f32,
                    #[doc = r" Optional components."]
                    pub optional: PerspectiveCommonCameraOptional,
                }
                #[doc = "Optional part of [PerspectiveCommonCamera]."]
                #[derive(Clone, Debug, Default)]
                pub struct PerspectiveCommonCameraOptional {
                    #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                    pub translation: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                    pub rotation: Option<Quat>,
                    #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                    pub scale: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::app::main_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the main scene.\n\n"]
                    pub main_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::app::ui_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the UI scene.\n\n"]
                    pub ui_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::player::user_id`\n\n**Description**: If set, this camera will only be used for the specified user.\n\n**Component description**: An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n"]
                    pub user_id: Option<String>,
                    #[doc = "**Component**: `ambient_core::camera::aspect_ratio_from_window`\n\n**Component description**: If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component.\n\n"]
                    pub aspect_ratio_from_window: Option<EntityId>,
                }
                impl crate::ecs::Concept for PerspectiveCommonCamera {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::transform::components::local_to_world(),
                                self.local_to_world,
                            )
                            .with(crate::ambient_core::camera::components::near(), self.near)
                            .with(
                                crate::ambient_core::camera::components::projection(),
                                self.projection,
                            )
                            .with(
                                crate::ambient_core::camera::components::projection_view(),
                                self.projection_view,
                            )
                            .with(
                                crate::ambient_core::camera::components::active_camera(),
                                self.active_camera,
                            )
                            .with(
                                crate::ambient_core::transform::components::inv_local_to_world(),
                                self.inv_local_to_world,
                            )
                            .with(crate::ambient_core::camera::components::fovy(), self.fovy)
                            .with(
                                crate::ambient_core::camera::components::aspect_ratio(),
                                self.aspect_ratio,
                            );
                        if let Some(translation) = self.optional.translation {
                            entity.set(
                                crate::ambient_core::transform::components::translation(),
                                translation,
                            );
                        }
                        if let Some(rotation) = self.optional.rotation {
                            entity.set(
                                crate::ambient_core::transform::components::rotation(),
                                rotation,
                            );
                        }
                        if let Some(scale) = self.optional.scale {
                            entity.set(crate::ambient_core::transform::components::scale(), scale);
                        }
                        if let Some(main_scene) = self.optional.main_scene {
                            entity.set(
                                crate::ambient_core::app::components::main_scene(),
                                main_scene,
                            );
                        }
                        if let Some(ui_scene) = self.optional.ui_scene {
                            entity.set(crate::ambient_core::app::components::ui_scene(), ui_scene);
                        }
                        if let Some(user_id) = self.optional.user_id {
                            entity.set(crate::ambient_core::player::components::user_id(), user_id);
                        }
                        if let Some(aspect_ratio_from_window) =
                            self.optional.aspect_ratio_from_window
                        {
                            entity.set(
                                crate::ambient_core::camera::components::aspect_ratio_from_window(),
                                aspect_ratio_from_window,
                            );
                        }
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some (Self { local_to_world : entity :: get_component (id , crate :: ambient_core :: transform :: components :: local_to_world ()) ? , near : entity :: get_component (id , crate :: ambient_core :: camera :: components :: near ()) ? , projection : entity :: get_component (id , crate :: ambient_core :: camera :: components :: projection ()) ? , projection_view : entity :: get_component (id , crate :: ambient_core :: camera :: components :: projection_view ()) ? , active_camera : entity :: get_component (id , crate :: ambient_core :: camera :: components :: active_camera ()) ? , inv_local_to_world : entity :: get_component (id , crate :: ambient_core :: transform :: components :: inv_local_to_world ()) ? , fovy : entity :: get_component (id , crate :: ambient_core :: camera :: components :: fovy ()) ? , aspect_ratio : entity :: get_component (id , crate :: ambient_core :: camera :: components :: aspect_ratio ()) ? , optional : PerspectiveCommonCameraOptional { translation : entity :: get_component (id , crate :: ambient_core :: transform :: components :: translation ()) , rotation : entity :: get_component (id , crate :: ambient_core :: transform :: components :: rotation ()) , scale : entity :: get_component (id , crate :: ambient_core :: transform :: components :: scale ()) , main_scene : entity :: get_component (id , crate :: ambient_core :: app :: components :: main_scene ()) , ui_scene : entity :: get_component (id , crate :: ambient_core :: app :: components :: ui_scene ()) , user_id : entity :: get_component (id , crate :: ambient_core :: player :: components :: user_id ()) , aspect_ratio_from_window : entity :: get_component (id , crate :: ambient_core :: camera :: components :: aspect_ratio_from_window ()) , } })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some (Self { local_to_world : entity . get (crate :: ambient_core :: transform :: components :: local_to_world ()) ? , near : entity . get (crate :: ambient_core :: camera :: components :: near ()) ? , projection : entity . get (crate :: ambient_core :: camera :: components :: projection ()) ? , projection_view : entity . get (crate :: ambient_core :: camera :: components :: projection_view ()) ? , active_camera : entity . get (crate :: ambient_core :: camera :: components :: active_camera ()) ? , inv_local_to_world : entity . get (crate :: ambient_core :: transform :: components :: inv_local_to_world ()) ? , fovy : entity . get (crate :: ambient_core :: camera :: components :: fovy ()) ? , aspect_ratio : entity . get (crate :: ambient_core :: camera :: components :: aspect_ratio ()) ? , optional : PerspectiveCommonCameraOptional { translation : entity . get (crate :: ambient_core :: transform :: components :: translation ()) , rotation : entity . get (crate :: ambient_core :: transform :: components :: rotation ()) , scale : entity . get (crate :: ambient_core :: transform :: components :: scale ()) , main_scene : entity . get (crate :: ambient_core :: app :: components :: main_scene ()) , ui_scene : entity . get (crate :: ambient_core :: app :: components :: ui_scene ()) , user_id : entity . get (crate :: ambient_core :: player :: components :: user_id ()) , aspect_ratio_from_window : entity . get (crate :: ambient_core :: camera :: components :: aspect_ratio_from_window ()) , } })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::transform::components::local_to_world(),
                                &crate::ambient_core::camera::components::near(),
                                &crate::ambient_core::camera::components::projection(),
                                &crate::ambient_core::camera::components::projection_view(),
                                &crate::ambient_core::camera::components::active_camera(),
                                &crate::ambient_core::transform::components::inv_local_to_world(),
                                &crate::ambient_core::camera::components::fovy(),
                                &crate::ambient_core::camera::components::aspect_ratio(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::transform::components::local_to_world(),
                            &crate::ambient_core::camera::components::near(),
                            &crate::ambient_core::camera::components::projection(),
                            &crate::ambient_core::camera::components::projection_view(),
                            &crate::ambient_core::camera::components::active_camera(),
                            &crate::ambient_core::transform::components::inv_local_to_world(),
                            &crate::ambient_core::camera::components::fovy(),
                            &crate::ambient_core::camera::components::aspect_ratio(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for PerspectiveCommonCamera {
                    fn suggested() -> Self {
                        Self {
                            local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            near: 0.1f32,
                            projection: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            projection_view: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            active_camera: 0f32,
                            inv_local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            fovy: 1f32,
                            aspect_ratio: 1f32,
                            optional: Default::default(),
                        }
                    }
                }
                #[doc = "**Perspective Camera**: A perspective camera.\n\n**Extends**: `ambient_core::camera::PerspectiveCommonCamera`"]
                #[derive(Clone, Debug)]
                pub struct PerspectiveCamera {
                    #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                    pub local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::near`\n\n**Suggested value**: `0.1f32`\n\n**Component description**: The near plane of this camera, measured in meters.\n\n"]
                    pub near: f32,
                    #[doc = "**Component**: `ambient_core::camera::projection`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n"]
                    pub projection: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::projection_view`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n"]
                    pub projection_view: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::active_camera`\n\n**Suggested value**: `0f32`\n\n**Component description**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n"]
                    pub active_camera: f32,
                    #[doc = "**Component**: `ambient_core::transform::inv_local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Converts a world position to a local position.\nThis is automatically updated.\n\n"]
                    pub inv_local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::fovy`\n\n**Suggested value**: `1f32`\n\n**Component description**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n"]
                    pub fovy: f32,
                    #[doc = "**Component**: `ambient_core::camera::aspect_ratio`\n\n**Suggested value**: `1f32`\n\n**Component description**: The aspect ratio of this camera.\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window.\n\n"]
                    pub aspect_ratio: f32,
                    #[doc = "**Component**: `ambient_core::camera::perspective`\n\n**Suggested value**: `()`\n\n**Component description**: If attached, this camera will use a standard perspective projection matrix.\nEnsure that `near` and `far` are set.\n\n"]
                    pub perspective: (),
                    #[doc = "**Component**: `ambient_core::camera::far`\n\n**Suggested value**: `1000f32`\n\n**Component description**: The far plane of this camera, measured in meters.\n\n"]
                    pub far: f32,
                    #[doc = r" Optional components."]
                    pub optional: PerspectiveCameraOptional,
                }
                #[doc = "Optional part of [PerspectiveCamera]."]
                #[derive(Clone, Debug, Default)]
                pub struct PerspectiveCameraOptional {
                    #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                    pub translation: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                    pub rotation: Option<Quat>,
                    #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                    pub scale: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::app::main_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the main scene.\n\n"]
                    pub main_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::app::ui_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the UI scene.\n\n"]
                    pub ui_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::player::user_id`\n\n**Description**: If set, this camera will only be used for the specified user.\n\n**Component description**: An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n"]
                    pub user_id: Option<String>,
                    #[doc = "**Component**: `ambient_core::camera::aspect_ratio_from_window`\n\n**Component description**: If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component.\n\n"]
                    pub aspect_ratio_from_window: Option<EntityId>,
                }
                impl crate::ecs::Concept for PerspectiveCamera {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::transform::components::local_to_world(),
                                self.local_to_world,
                            )
                            .with(crate::ambient_core::camera::components::near(), self.near)
                            .with(
                                crate::ambient_core::camera::components::projection(),
                                self.projection,
                            )
                            .with(
                                crate::ambient_core::camera::components::projection_view(),
                                self.projection_view,
                            )
                            .with(
                                crate::ambient_core::camera::components::active_camera(),
                                self.active_camera,
                            )
                            .with(
                                crate::ambient_core::transform::components::inv_local_to_world(),
                                self.inv_local_to_world,
                            )
                            .with(crate::ambient_core::camera::components::fovy(), self.fovy)
                            .with(
                                crate::ambient_core::camera::components::aspect_ratio(),
                                self.aspect_ratio,
                            )
                            .with(
                                crate::ambient_core::camera::components::perspective(),
                                self.perspective,
                            )
                            .with(crate::ambient_core::camera::components::far(), self.far);
                        if let Some(translation) = self.optional.translation {
                            entity.set(
                                crate::ambient_core::transform::components::translation(),
                                translation,
                            );
                        }
                        if let Some(rotation) = self.optional.rotation {
                            entity.set(
                                crate::ambient_core::transform::components::rotation(),
                                rotation,
                            );
                        }
                        if let Some(scale) = self.optional.scale {
                            entity.set(crate::ambient_core::transform::components::scale(), scale);
                        }
                        if let Some(main_scene) = self.optional.main_scene {
                            entity.set(
                                crate::ambient_core::app::components::main_scene(),
                                main_scene,
                            );
                        }
                        if let Some(ui_scene) = self.optional.ui_scene {
                            entity.set(crate::ambient_core::app::components::ui_scene(), ui_scene);
                        }
                        if let Some(user_id) = self.optional.user_id {
                            entity.set(crate::ambient_core::player::components::user_id(), user_id);
                        }
                        if let Some(aspect_ratio_from_window) =
                            self.optional.aspect_ratio_from_window
                        {
                            entity.set(
                                crate::ambient_core::camera::components::aspect_ratio_from_window(),
                                aspect_ratio_from_window,
                            );
                        }
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some (Self { local_to_world : entity :: get_component (id , crate :: ambient_core :: transform :: components :: local_to_world ()) ? , near : entity :: get_component (id , crate :: ambient_core :: camera :: components :: near ()) ? , projection : entity :: get_component (id , crate :: ambient_core :: camera :: components :: projection ()) ? , projection_view : entity :: get_component (id , crate :: ambient_core :: camera :: components :: projection_view ()) ? , active_camera : entity :: get_component (id , crate :: ambient_core :: camera :: components :: active_camera ()) ? , inv_local_to_world : entity :: get_component (id , crate :: ambient_core :: transform :: components :: inv_local_to_world ()) ? , fovy : entity :: get_component (id , crate :: ambient_core :: camera :: components :: fovy ()) ? , aspect_ratio : entity :: get_component (id , crate :: ambient_core :: camera :: components :: aspect_ratio ()) ? , perspective : entity :: get_component (id , crate :: ambient_core :: camera :: components :: perspective ()) ? , far : entity :: get_component (id , crate :: ambient_core :: camera :: components :: far ()) ? , optional : PerspectiveCameraOptional { translation : entity :: get_component (id , crate :: ambient_core :: transform :: components :: translation ()) , rotation : entity :: get_component (id , crate :: ambient_core :: transform :: components :: rotation ()) , scale : entity :: get_component (id , crate :: ambient_core :: transform :: components :: scale ()) , main_scene : entity :: get_component (id , crate :: ambient_core :: app :: components :: main_scene ()) , ui_scene : entity :: get_component (id , crate :: ambient_core :: app :: components :: ui_scene ()) , user_id : entity :: get_component (id , crate :: ambient_core :: player :: components :: user_id ()) , aspect_ratio_from_window : entity :: get_component (id , crate :: ambient_core :: camera :: components :: aspect_ratio_from_window ()) , } })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some (Self { local_to_world : entity . get (crate :: ambient_core :: transform :: components :: local_to_world ()) ? , near : entity . get (crate :: ambient_core :: camera :: components :: near ()) ? , projection : entity . get (crate :: ambient_core :: camera :: components :: projection ()) ? , projection_view : entity . get (crate :: ambient_core :: camera :: components :: projection_view ()) ? , active_camera : entity . get (crate :: ambient_core :: camera :: components :: active_camera ()) ? , inv_local_to_world : entity . get (crate :: ambient_core :: transform :: components :: inv_local_to_world ()) ? , fovy : entity . get (crate :: ambient_core :: camera :: components :: fovy ()) ? , aspect_ratio : entity . get (crate :: ambient_core :: camera :: components :: aspect_ratio ()) ? , perspective : entity . get (crate :: ambient_core :: camera :: components :: perspective ()) ? , far : entity . get (crate :: ambient_core :: camera :: components :: far ()) ? , optional : PerspectiveCameraOptional { translation : entity . get (crate :: ambient_core :: transform :: components :: translation ()) , rotation : entity . get (crate :: ambient_core :: transform :: components :: rotation ()) , scale : entity . get (crate :: ambient_core :: transform :: components :: scale ()) , main_scene : entity . get (crate :: ambient_core :: app :: components :: main_scene ()) , ui_scene : entity . get (crate :: ambient_core :: app :: components :: ui_scene ()) , user_id : entity . get (crate :: ambient_core :: player :: components :: user_id ()) , aspect_ratio_from_window : entity . get (crate :: ambient_core :: camera :: components :: aspect_ratio_from_window ()) , } })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::transform::components::local_to_world(),
                                &crate::ambient_core::camera::components::near(),
                                &crate::ambient_core::camera::components::projection(),
                                &crate::ambient_core::camera::components::projection_view(),
                                &crate::ambient_core::camera::components::active_camera(),
                                &crate::ambient_core::transform::components::inv_local_to_world(),
                                &crate::ambient_core::camera::components::fovy(),
                                &crate::ambient_core::camera::components::aspect_ratio(),
                                &crate::ambient_core::camera::components::perspective(),
                                &crate::ambient_core::camera::components::far(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::transform::components::local_to_world(),
                            &crate::ambient_core::camera::components::near(),
                            &crate::ambient_core::camera::components::projection(),
                            &crate::ambient_core::camera::components::projection_view(),
                            &crate::ambient_core::camera::components::active_camera(),
                            &crate::ambient_core::transform::components::inv_local_to_world(),
                            &crate::ambient_core::camera::components::fovy(),
                            &crate::ambient_core::camera::components::aspect_ratio(),
                            &crate::ambient_core::camera::components::perspective(),
                            &crate::ambient_core::camera::components::far(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for PerspectiveCamera {
                    fn suggested() -> Self {
                        Self {
                            local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            near: 0.1f32,
                            projection: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            projection_view: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            active_camera: 0f32,
                            inv_local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            fovy: 1f32,
                            aspect_ratio: 1f32,
                            perspective: (),
                            far: 1000f32,
                            optional: Default::default(),
                        }
                    }
                }
                #[doc = "**Perspective-Infinite-Reverse Camera**: A perspective-infinite-reverse camera. This is recommended for most use-cases.\n\n**Extends**: `ambient_core::camera::PerspectiveCommonCamera`"]
                #[derive(Clone, Debug)]
                pub struct PerspectiveInfiniteReverseCamera {
                    #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                    pub local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::near`\n\n**Suggested value**: `0.1f32`\n\n**Component description**: The near plane of this camera, measured in meters.\n\n"]
                    pub near: f32,
                    #[doc = "**Component**: `ambient_core::camera::projection`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n"]
                    pub projection: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::projection_view`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n"]
                    pub projection_view: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::active_camera`\n\n**Suggested value**: `0f32`\n\n**Component description**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n"]
                    pub active_camera: f32,
                    #[doc = "**Component**: `ambient_core::transform::inv_local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Converts a world position to a local position.\nThis is automatically updated.\n\n"]
                    pub inv_local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::fovy`\n\n**Suggested value**: `1f32`\n\n**Component description**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n"]
                    pub fovy: f32,
                    #[doc = "**Component**: `ambient_core::camera::aspect_ratio`\n\n**Suggested value**: `1f32`\n\n**Component description**: The aspect ratio of this camera.\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window.\n\n"]
                    pub aspect_ratio: f32,
                    #[doc = "**Component**: `ambient_core::camera::perspective_infinite_reverse`\n\n**Suggested value**: `()`\n\n**Component description**: If attached, this camera will use a perspective-infinite-reverse projection matrix.\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set.\n\n"]
                    pub perspective_infinite_reverse: (),
                    #[doc = r" Optional components."]
                    pub optional: PerspectiveInfiniteReverseCameraOptional,
                }
                #[doc = "Optional part of [PerspectiveInfiniteReverseCamera]."]
                #[derive(Clone, Debug, Default)]
                pub struct PerspectiveInfiniteReverseCameraOptional {
                    #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                    pub translation: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                    pub rotation: Option<Quat>,
                    #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                    pub scale: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::app::main_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the main scene.\n\n"]
                    pub main_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::app::ui_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the UI scene.\n\n"]
                    pub ui_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::player::user_id`\n\n**Description**: If set, this camera will only be used for the specified user.\n\n**Component description**: An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n"]
                    pub user_id: Option<String>,
                    #[doc = "**Component**: `ambient_core::camera::aspect_ratio_from_window`\n\n**Component description**: If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component.\n\n"]
                    pub aspect_ratio_from_window: Option<EntityId>,
                }
                impl crate::ecs::Concept for PerspectiveInfiniteReverseCamera {
                    fn make(self) -> Entity {
                        let mut entity = Entity :: new () . with (crate :: ambient_core :: transform :: components :: local_to_world () , self . local_to_world) . with (crate :: ambient_core :: camera :: components :: near () , self . near) . with (crate :: ambient_core :: camera :: components :: projection () , self . projection) . with (crate :: ambient_core :: camera :: components :: projection_view () , self . projection_view) . with (crate :: ambient_core :: camera :: components :: active_camera () , self . active_camera) . with (crate :: ambient_core :: transform :: components :: inv_local_to_world () , self . inv_local_to_world) . with (crate :: ambient_core :: camera :: components :: fovy () , self . fovy) . with (crate :: ambient_core :: camera :: components :: aspect_ratio () , self . aspect_ratio) . with (crate :: ambient_core :: camera :: components :: perspective_infinite_reverse () , self . perspective_infinite_reverse) ;
                        if let Some(translation) = self.optional.translation {
                            entity.set(
                                crate::ambient_core::transform::components::translation(),
                                translation,
                            );
                        }
                        if let Some(rotation) = self.optional.rotation {
                            entity.set(
                                crate::ambient_core::transform::components::rotation(),
                                rotation,
                            );
                        }
                        if let Some(scale) = self.optional.scale {
                            entity.set(crate::ambient_core::transform::components::scale(), scale);
                        }
                        if let Some(main_scene) = self.optional.main_scene {
                            entity.set(
                                crate::ambient_core::app::components::main_scene(),
                                main_scene,
                            );
                        }
                        if let Some(ui_scene) = self.optional.ui_scene {
                            entity.set(crate::ambient_core::app::components::ui_scene(), ui_scene);
                        }
                        if let Some(user_id) = self.optional.user_id {
                            entity.set(crate::ambient_core::player::components::user_id(), user_id);
                        }
                        if let Some(aspect_ratio_from_window) =
                            self.optional.aspect_ratio_from_window
                        {
                            entity.set(
                                crate::ambient_core::camera::components::aspect_ratio_from_window(),
                                aspect_ratio_from_window,
                            );
                        }
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some (Self { local_to_world : entity :: get_component (id , crate :: ambient_core :: transform :: components :: local_to_world ()) ? , near : entity :: get_component (id , crate :: ambient_core :: camera :: components :: near ()) ? , projection : entity :: get_component (id , crate :: ambient_core :: camera :: components :: projection ()) ? , projection_view : entity :: get_component (id , crate :: ambient_core :: camera :: components :: projection_view ()) ? , active_camera : entity :: get_component (id , crate :: ambient_core :: camera :: components :: active_camera ()) ? , inv_local_to_world : entity :: get_component (id , crate :: ambient_core :: transform :: components :: inv_local_to_world ()) ? , fovy : entity :: get_component (id , crate :: ambient_core :: camera :: components :: fovy ()) ? , aspect_ratio : entity :: get_component (id , crate :: ambient_core :: camera :: components :: aspect_ratio ()) ? , perspective_infinite_reverse : entity :: get_component (id , crate :: ambient_core :: camera :: components :: perspective_infinite_reverse ()) ? , optional : PerspectiveInfiniteReverseCameraOptional { translation : entity :: get_component (id , crate :: ambient_core :: transform :: components :: translation ()) , rotation : entity :: get_component (id , crate :: ambient_core :: transform :: components :: rotation ()) , scale : entity :: get_component (id , crate :: ambient_core :: transform :: components :: scale ()) , main_scene : entity :: get_component (id , crate :: ambient_core :: app :: components :: main_scene ()) , ui_scene : entity :: get_component (id , crate :: ambient_core :: app :: components :: ui_scene ()) , user_id : entity :: get_component (id , crate :: ambient_core :: player :: components :: user_id ()) , aspect_ratio_from_window : entity :: get_component (id , crate :: ambient_core :: camera :: components :: aspect_ratio_from_window ()) , } })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some (Self { local_to_world : entity . get (crate :: ambient_core :: transform :: components :: local_to_world ()) ? , near : entity . get (crate :: ambient_core :: camera :: components :: near ()) ? , projection : entity . get (crate :: ambient_core :: camera :: components :: projection ()) ? , projection_view : entity . get (crate :: ambient_core :: camera :: components :: projection_view ()) ? , active_camera : entity . get (crate :: ambient_core :: camera :: components :: active_camera ()) ? , inv_local_to_world : entity . get (crate :: ambient_core :: transform :: components :: inv_local_to_world ()) ? , fovy : entity . get (crate :: ambient_core :: camera :: components :: fovy ()) ? , aspect_ratio : entity . get (crate :: ambient_core :: camera :: components :: aspect_ratio ()) ? , perspective_infinite_reverse : entity . get (crate :: ambient_core :: camera :: components :: perspective_infinite_reverse ()) ? , optional : PerspectiveInfiniteReverseCameraOptional { translation : entity . get (crate :: ambient_core :: transform :: components :: translation ()) , rotation : entity . get (crate :: ambient_core :: transform :: components :: rotation ()) , scale : entity . get (crate :: ambient_core :: transform :: components :: scale ()) , main_scene : entity . get (crate :: ambient_core :: app :: components :: main_scene ()) , ui_scene : entity . get (crate :: ambient_core :: app :: components :: ui_scene ()) , user_id : entity . get (crate :: ambient_core :: player :: components :: user_id ()) , aspect_ratio_from_window : entity . get (crate :: ambient_core :: camera :: components :: aspect_ratio_from_window ()) , } })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity :: has_components (id , & [& crate :: ambient_core :: transform :: components :: local_to_world () , & crate :: ambient_core :: camera :: components :: near () , & crate :: ambient_core :: camera :: components :: projection () , & crate :: ambient_core :: camera :: components :: projection_view () , & crate :: ambient_core :: camera :: components :: active_camera () , & crate :: ambient_core :: transform :: components :: inv_local_to_world () , & crate :: ambient_core :: camera :: components :: fovy () , & crate :: ambient_core :: camera :: components :: aspect_ratio () , & crate :: ambient_core :: camera :: components :: perspective_infinite_reverse ()])
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::transform::components::local_to_world(),
                            &crate::ambient_core::camera::components::near(),
                            &crate::ambient_core::camera::components::projection(),
                            &crate::ambient_core::camera::components::projection_view(),
                            &crate::ambient_core::camera::components::active_camera(),
                            &crate::ambient_core::transform::components::inv_local_to_world(),
                            &crate::ambient_core::camera::components::fovy(),
                            &crate::ambient_core::camera::components::aspect_ratio(),
                            &crate::ambient_core::camera::components::perspective_infinite_reverse(
                            ),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for PerspectiveInfiniteReverseCamera {
                    fn suggested() -> Self {
                        Self {
                            local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            near: 0.1f32,
                            projection: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            projection_view: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            active_camera: 0f32,
                            inv_local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            fovy: 1f32,
                            aspect_ratio: 1f32,
                            perspective_infinite_reverse: (),
                            optional: Default::default(),
                        }
                    }
                }
                #[doc = "**Orthographic Camera**: An orthographic camera.\n\n**Extends**: `ambient_core::camera::Camera`"]
                #[derive(Clone, Debug)]
                pub struct OrthographicCamera {
                    #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                    pub local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::near`\n\n**Suggested value**: `-1f32`\n\n**Component description**: The near plane of this camera, measured in meters.\n\n"]
                    pub near: f32,
                    #[doc = "**Component**: `ambient_core::camera::projection`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n"]
                    pub projection: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::projection_view`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n"]
                    pub projection_view: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::active_camera`\n\n**Suggested value**: `0f32`\n\n**Component description**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n"]
                    pub active_camera: f32,
                    #[doc = "**Component**: `ambient_core::transform::inv_local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Converts a world position to a local position.\nThis is automatically updated.\n\n"]
                    pub inv_local_to_world: Mat4,
                    #[doc = "**Component**: `ambient_core::camera::orthographic`\n\n**Suggested value**: `()`\n\n**Component description**: If attached, this camera will use a standard orthographic projection matrix.\nEnsure that the `orthographic_` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`.\n\n"]
                    pub orthographic: (),
                    #[doc = "**Component**: `ambient_core::camera::orthographic_left`\n\n**Suggested value**: `-1f32`\n\n**Component description**: The left bound for this `orthographic` camera.\n\n"]
                    pub orthographic_left: f32,
                    #[doc = "**Component**: `ambient_core::camera::orthographic_right`\n\n**Suggested value**: `1f32`\n\n**Component description**: The right bound for this `orthographic` camera.\n\n"]
                    pub orthographic_right: f32,
                    #[doc = "**Component**: `ambient_core::camera::orthographic_top`\n\n**Suggested value**: `1f32`\n\n**Component description**: The top bound for this `orthographic` camera.\n\n"]
                    pub orthographic_top: f32,
                    #[doc = "**Component**: `ambient_core::camera::orthographic_bottom`\n\n**Suggested value**: `-1f32`\n\n**Component description**: The bottom bound for this `orthographic` camera.\n\n"]
                    pub orthographic_bottom: f32,
                    #[doc = "**Component**: `ambient_core::camera::far`\n\n**Suggested value**: `1f32`\n\n**Component description**: The far plane of this camera, measured in meters.\n\n"]
                    pub far: f32,
                    #[doc = r" Optional components."]
                    pub optional: OrthographicCameraOptional,
                }
                #[doc = "Optional part of [OrthographicCamera]."]
                #[derive(Clone, Debug, Default)]
                pub struct OrthographicCameraOptional {
                    #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                    pub translation: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                    pub rotation: Option<Quat>,
                    #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                    pub scale: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::app::main_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the main scene.\n\n"]
                    pub main_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::app::ui_scene`\n\n**Description**: Either the main or UI scene must be specified for this camera to be used.\n\n**Component description**: If attached, this entity belongs to the UI scene.\n\n"]
                    pub ui_scene: Option<()>,
                    #[doc = "**Component**: `ambient_core::player::user_id`\n\n**Description**: If set, this camera will only be used for the specified user.\n\n**Component description**: An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n"]
                    pub user_id: Option<String>,
                }
                impl crate::ecs::Concept for OrthographicCamera {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::transform::components::local_to_world(),
                                self.local_to_world,
                            )
                            .with(crate::ambient_core::camera::components::near(), self.near)
                            .with(
                                crate::ambient_core::camera::components::projection(),
                                self.projection,
                            )
                            .with(
                                crate::ambient_core::camera::components::projection_view(),
                                self.projection_view,
                            )
                            .with(
                                crate::ambient_core::camera::components::active_camera(),
                                self.active_camera,
                            )
                            .with(
                                crate::ambient_core::transform::components::inv_local_to_world(),
                                self.inv_local_to_world,
                            )
                            .with(
                                crate::ambient_core::camera::components::orthographic(),
                                self.orthographic,
                            )
                            .with(
                                crate::ambient_core::camera::components::orthographic_left(),
                                self.orthographic_left,
                            )
                            .with(
                                crate::ambient_core::camera::components::orthographic_right(),
                                self.orthographic_right,
                            )
                            .with(
                                crate::ambient_core::camera::components::orthographic_top(),
                                self.orthographic_top,
                            )
                            .with(
                                crate::ambient_core::camera::components::orthographic_bottom(),
                                self.orthographic_bottom,
                            )
                            .with(crate::ambient_core::camera::components::far(), self.far);
                        if let Some(translation) = self.optional.translation {
                            entity.set(
                                crate::ambient_core::transform::components::translation(),
                                translation,
                            );
                        }
                        if let Some(rotation) = self.optional.rotation {
                            entity.set(
                                crate::ambient_core::transform::components::rotation(),
                                rotation,
                            );
                        }
                        if let Some(scale) = self.optional.scale {
                            entity.set(crate::ambient_core::transform::components::scale(), scale);
                        }
                        if let Some(main_scene) = self.optional.main_scene {
                            entity.set(
                                crate::ambient_core::app::components::main_scene(),
                                main_scene,
                            );
                        }
                        if let Some(ui_scene) = self.optional.ui_scene {
                            entity.set(crate::ambient_core::app::components::ui_scene(), ui_scene);
                        }
                        if let Some(user_id) = self.optional.user_id {
                            entity.set(crate::ambient_core::player::components::user_id(), user_id);
                        }
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some(Self {
                            local_to_world: entity::get_component(
                                id,
                                crate::ambient_core::transform::components::local_to_world(),
                            )?,
                            near: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::near(),
                            )?,
                            projection: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::projection(),
                            )?,
                            projection_view: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::projection_view(),
                            )?,
                            active_camera: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::active_camera(),
                            )?,
                            inv_local_to_world: entity::get_component(
                                id,
                                crate::ambient_core::transform::components::inv_local_to_world(),
                            )?,
                            orthographic: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::orthographic(),
                            )?,
                            orthographic_left: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::orthographic_left(),
                            )?,
                            orthographic_right: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::orthographic_right(),
                            )?,
                            orthographic_top: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::orthographic_top(),
                            )?,
                            orthographic_bottom: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::orthographic_bottom(),
                            )?,
                            far: entity::get_component(
                                id,
                                crate::ambient_core::camera::components::far(),
                            )?,
                            optional: OrthographicCameraOptional {
                                translation: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::translation(),
                                ),
                                rotation: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::rotation(),
                                ),
                                scale: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::scale(),
                                ),
                                main_scene: entity::get_component(
                                    id,
                                    crate::ambient_core::app::components::main_scene(),
                                ),
                                ui_scene: entity::get_component(
                                    id,
                                    crate::ambient_core::app::components::ui_scene(),
                                ),
                                user_id: entity::get_component(
                                    id,
                                    crate::ambient_core::player::components::user_id(),
                                ),
                            },
                        })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some(Self {
                            local_to_world: entity
                                .get(crate::ambient_core::transform::components::local_to_world())?,
                            near: entity.get(crate::ambient_core::camera::components::near())?,
                            projection: entity
                                .get(crate::ambient_core::camera::components::projection())?,
                            projection_view: entity
                                .get(crate::ambient_core::camera::components::projection_view())?,
                            active_camera: entity
                                .get(crate::ambient_core::camera::components::active_camera())?,
                            inv_local_to_world: entity.get(
                                crate::ambient_core::transform::components::inv_local_to_world(),
                            )?,
                            orthographic: entity
                                .get(crate::ambient_core::camera::components::orthographic())?,
                            orthographic_left: entity
                                .get(crate::ambient_core::camera::components::orthographic_left())?,
                            orthographic_right: entity.get(
                                crate::ambient_core::camera::components::orthographic_right(),
                            )?,
                            orthographic_top: entity
                                .get(crate::ambient_core::camera::components::orthographic_top())?,
                            orthographic_bottom: entity.get(
                                crate::ambient_core::camera::components::orthographic_bottom(),
                            )?,
                            far: entity.get(crate::ambient_core::camera::components::far())?,
                            optional: OrthographicCameraOptional {
                                translation: entity
                                    .get(crate::ambient_core::transform::components::translation()),
                                rotation: entity
                                    .get(crate::ambient_core::transform::components::rotation()),
                                scale: entity
                                    .get(crate::ambient_core::transform::components::scale()),
                                main_scene: entity
                                    .get(crate::ambient_core::app::components::main_scene()),
                                ui_scene: entity
                                    .get(crate::ambient_core::app::components::ui_scene()),
                                user_id: entity
                                    .get(crate::ambient_core::player::components::user_id()),
                            },
                        })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::transform::components::local_to_world(),
                                &crate::ambient_core::camera::components::near(),
                                &crate::ambient_core::camera::components::projection(),
                                &crate::ambient_core::camera::components::projection_view(),
                                &crate::ambient_core::camera::components::active_camera(),
                                &crate::ambient_core::transform::components::inv_local_to_world(),
                                &crate::ambient_core::camera::components::orthographic(),
                                &crate::ambient_core::camera::components::orthographic_left(),
                                &crate::ambient_core::camera::components::orthographic_right(),
                                &crate::ambient_core::camera::components::orthographic_top(),
                                &crate::ambient_core::camera::components::orthographic_bottom(),
                                &crate::ambient_core::camera::components::far(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::transform::components::local_to_world(),
                            &crate::ambient_core::camera::components::near(),
                            &crate::ambient_core::camera::components::projection(),
                            &crate::ambient_core::camera::components::projection_view(),
                            &crate::ambient_core::camera::components::active_camera(),
                            &crate::ambient_core::transform::components::inv_local_to_world(),
                            &crate::ambient_core::camera::components::orthographic(),
                            &crate::ambient_core::camera::components::orthographic_left(),
                            &crate::ambient_core::camera::components::orthographic_right(),
                            &crate::ambient_core::camera::components::orthographic_top(),
                            &crate::ambient_core::camera::components::orthographic_bottom(),
                            &crate::ambient_core::camera::components::far(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for OrthographicCamera {
                    fn suggested() -> Self {
                        Self {
                            local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            near: -1f32,
                            projection: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            projection_view: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            active_camera: 0f32,
                            inv_local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            orthographic: (),
                            orthographic_left: -1f32,
                            orthographic_right: 1f32,
                            orthographic_top: 1f32,
                            orthographic_bottom: -1f32,
                            far: 1f32,
                            optional: Default::default(),
                        }
                    }
                }
            }
        }
        pub mod ecs {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static DONT_DESPAWN_ON_UNLOAD: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::ecs::dont_despawn_on_unload")
                });
                #[doc = "**Don't automatically despawn on module unload**: Indicates that this entity shouldn't be despawned when the module that spawned it unloads.\n\n*Attributes*: Debuggable, Store"]
                pub fn dont_despawn_on_unload() -> Component<()> {
                    *DONT_DESPAWN_ON_UNLOAD
                }
                static DONT_STORE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::ecs::dont_store"));
                #[doc = "**Don't store**: Indicates that this entity shouldn't be stored on disk.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn dont_store() -> Component<()> {
                    *DONT_STORE
                }
                static ID: Lazy<Component<EntityId>> =
                    Lazy::new(|| __internal_get_component("ambient_core::ecs::id"));
                #[doc = "**ID**: The ID of the entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn id() -> Component<EntityId> {
                    *ID
                }
            }
        }
        pub mod hierarchy {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static PARENT: Lazy<Component<EntityId>> =
                    Lazy::new(|| __internal_get_component("ambient_core::hierarchy::parent"));
                #[doc = "**Parent**: The parent of this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn parent() -> Component<EntityId> {
                    *PARENT
                }
                static CHILDREN: Lazy<Component<Vec<EntityId>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::hierarchy::children"));
                #[doc = "**Children**: The children of this entity.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"]
                pub fn children() -> Component<Vec<EntityId>> {
                    *CHILDREN
                }
            }
        }
        pub mod input {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static MOUSE_OVER_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::input::mouse_over_entity")
                });
                #[doc = "**Mouse over entity**: The entity the mouse is currently over.\n\n*Attributes*: Debuggable, Resource"]
                pub fn mouse_over_entity() -> Component<EntityId> {
                    *MOUSE_OVER_ENTITY
                }
                static MOUSE_OVER_DISTANCE: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::input::mouse_over_distance")
                });
                #[doc = "**Mouse over distance**: This distance to the entity that the mouse is currently over.\n\n*Attributes*: Debuggable, Resource"]
                pub fn mouse_over_distance() -> Component<f32> {
                    *MOUSE_OVER_DISTANCE
                }
                static IS_MOUSE_OVER: Lazy<Component<u32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::input::is_mouse_over"));
                #[doc = "**Mouse over**: The number of mouse cursors that are currently over this entity.\n\n*Attributes*: Debuggable"]
                pub fn is_mouse_over() -> Component<u32> {
                    *IS_MOUSE_OVER
                }
                static MOUSE_PICKABLE_MAX: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::input::mouse_pickable_max")
                });
                #[doc = "**Mouse pickable max**: This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn mouse_pickable_max() -> Component<Vec3> {
                    *MOUSE_PICKABLE_MAX
                }
                static MOUSE_PICKABLE_MIN: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::input::mouse_pickable_min")
                });
                #[doc = "**Mouse pickable min**: This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn mouse_pickable_min() -> Component<Vec3> {
                    *MOUSE_PICKABLE_MIN
                }
            }
            #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
            #[doc = r" and with other modules."]
            pub mod messages {
                use crate::{
                    message::{
                        Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                    },
                    prelude::*,
                };
                #[derive(Clone, Debug)]
                #[doc = "**MouseOverChanged**: Mouse over has been updated"]
                pub struct MouseOverChanged {
                    pub from_external: bool,
                    pub mouse_over: EntityId,
                    pub distance: f32,
                }
                impl MouseOverChanged {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(
                        from_external: impl Into<bool>,
                        mouse_over: impl Into<EntityId>,
                        distance: impl Into<f32>,
                    ) -> Self {
                        Self {
                            from_external: from_external.into(),
                            mouse_over: mouse_over.into(),
                            distance: distance.into(),
                        }
                    }
                }
                impl Message for MouseOverChanged {
                    fn id() -> &'static str {
                        "MouseOverChanged"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.from_external.serialize_message_part(&mut output)?;
                        self.mouse_over.serialize_message_part(&mut output)?;
                        self.distance.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            from_external: bool::deserialize_message_part(&mut input)?,
                            mouse_over: EntityId::deserialize_message_part(&mut input)?,
                            distance: f32::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl ModuleMessage for MouseOverChanged {}
            }
        }
        pub mod layout {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static ALIGN_HORIZONTAL: Lazy<
                    Component<crate::ambient_core::layout::types::Align>,
                > = Lazy::new(|| {
                    __internal_get_component("ambient_core::layout::align_horizontal")
                });
                #[doc = "**Align horizontal**: Layout alignment: horizontal.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn align_horizontal() -> Component<crate::ambient_core::layout::types::Align> {
                    *ALIGN_HORIZONTAL
                }
                static ALIGN_VERTICAL: Lazy<Component<crate::ambient_core::layout::types::Align>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::align_vertical"));
                #[doc = "**Align vertical**: Layout alignment: vertical.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn align_vertical() -> Component<crate::ambient_core::layout::types::Align> {
                    *ALIGN_VERTICAL
                }
                static DOCKING: Lazy<Component<crate::ambient_core::layout::types::Docking>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::docking"));
                #[doc = "**Docking**: Layout docking.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn docking() -> Component<crate::ambient_core::layout::types::Docking> {
                    *DOCKING
                }
                static FIT_HORIZONTAL: Lazy<Component<crate::ambient_core::layout::types::Fit>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::fit_horizontal"));
                #[doc = "**Fit horizontal**: Layout fit: horizontal.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn fit_horizontal() -> Component<crate::ambient_core::layout::types::Fit> {
                    *FIT_HORIZONTAL
                }
                static FIT_VERTICAL: Lazy<Component<crate::ambient_core::layout::types::Fit>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::fit_vertical"));
                #[doc = "**Fit vertical**: Layout fit: vertical.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn fit_vertical() -> Component<crate::ambient_core::layout::types::Fit> {
                    *FIT_VERTICAL
                }
                static LAYOUT: Lazy<Component<crate::ambient_core::layout::types::Layout>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::layout"));
                #[doc = "**Layout**: Layout.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn layout() -> Component<crate::ambient_core::layout::types::Layout> {
                    *LAYOUT
                }
                static ORIENTATION: Lazy<
                    Component<crate::ambient_core::layout::types::Orientation>,
                > = Lazy::new(|| __internal_get_component("ambient_core::layout::orientation"));
                #[doc = "**Orientation**: Layout orientation.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn orientation() -> Component<crate::ambient_core::layout::types::Orientation> {
                    *ORIENTATION
                }
                static IS_BOOK_FILE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::is_book_file"));
                #[doc = "**Is book file**: This is a file in a `layout_bookcase`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn is_book_file() -> Component<()> {
                    *IS_BOOK_FILE
                }
                static MARGIN: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::margin"));
                #[doc = "**Margin**: Layout margin: [top, right, bottom, left].\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn margin() -> Component<Vec4> {
                    *MARGIN
                }
                static PADDING: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::padding"));
                #[doc = "**Padding**: Layout padding: [top, right, bottom, left].\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn padding() -> Component<Vec4> {
                    *PADDING
                }
                static MESH_TO_LOCAL_FROM_SIZE: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::layout::mesh_to_local_from_size")
                });
                #[doc = "**Mesh to local from size**: Update the `mesh_to_local` based on the width and height of this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn mesh_to_local_from_size() -> Component<()> {
                    *MESH_TO_LOCAL_FROM_SIZE
                }
                static MIN_HEIGHT: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::min_height"));
                #[doc = "**Minimum height**: The minimum height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn min_height() -> Component<f32> {
                    *MIN_HEIGHT
                }
                static MIN_WIDTH: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::min_width"));
                #[doc = "**Minimum width**: The minimum width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn min_width() -> Component<f32> {
                    *MIN_WIDTH
                }
                static MAX_HEIGHT: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::max_height"));
                #[doc = "**Maximum height**: The maximum height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn max_height() -> Component<f32> {
                    *MAX_HEIGHT
                }
                static MAX_WIDTH: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::max_width"));
                #[doc = "**Maximum width**: The maximum width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn max_width() -> Component<f32> {
                    *MAX_WIDTH
                }
                static IS_SCREEN: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::is_screen"));
                #[doc = "**Is screen**: This entity will be treated as a screen. Used by the Screen ui component.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn is_screen() -> Component<()> {
                    *IS_SCREEN
                }
                static SPACE_BETWEEN_ITEMS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::layout::space_between_items")
                });
                #[doc = "**Space between items**: Space between items in a layout.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn space_between_items() -> Component<f32> {
                    *SPACE_BETWEEN_ITEMS
                }
                static WIDTH: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::width"));
                #[doc = "**Width**: The width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn width() -> Component<f32> {
                    *WIDTH
                }
                static HEIGHT: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::height"));
                #[doc = "**Height**: The height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn height() -> Component<f32> {
                    *HEIGHT
                }
                static GPU_UI_SIZE: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::layout::gpu_ui_size"));
                #[doc = "**GPU UI size**: Upload the width and height of this UI element to the GPU.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn gpu_ui_size() -> Component<Vec4> {
                    *GPU_UI_SIZE
                }
            }
            #[doc = r" Auto-generated type definitions."]
            pub mod types {
                use crate::{global::serde, message::*};
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Align**: Layout alignment."]
                pub enum Align {
                    #[default]
                    #[doc = "Begin"]
                    Begin,
                    #[doc = "Center"]
                    Center,
                    #[doc = "End"]
                    End,
                }
                impl crate::ecs::EnumComponent for Align {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Begin => Align::Begin as u32,
                            Self::Center => Align::Center as u32,
                            Self::End => Align::End as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Align::Begin as u32 {
                            return Some(Self::Begin);
                        }
                        if value == Align::Center as u32 {
                            return Some(Self::Center);
                        }
                        if value == Align::End as u32 {
                            return Some(Self::End);
                        }
                        None
                    }
                }
                impl crate::ecs::SupportedValue for Align {
                    fn from_result(result: crate::ecs::WitComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_result(result).and_then(Self::from_u32)
                    }
                    fn into_result(self) -> crate::ecs::WitComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_result()
                    }
                    fn from_value(value: crate::ecs::ComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_value(value).and_then(Self::from_u32)
                    }
                    fn into_value(self) -> crate::ecs::ComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_value()
                    }
                }
                impl MessageSerde for Align {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Fit**: Layout fit."]
                pub enum Fit {
                    #[default]
                    #[doc = "None"]
                    None,
                    #[doc = "Parent"]
                    Parent,
                    #[doc = "Children"]
                    Children,
                }
                impl crate::ecs::EnumComponent for Fit {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::None => Fit::None as u32,
                            Self::Parent => Fit::Parent as u32,
                            Self::Children => Fit::Children as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Fit::None as u32 {
                            return Some(Self::None);
                        }
                        if value == Fit::Parent as u32 {
                            return Some(Self::Parent);
                        }
                        if value == Fit::Children as u32 {
                            return Some(Self::Children);
                        }
                        None
                    }
                }
                impl crate::ecs::SupportedValue for Fit {
                    fn from_result(result: crate::ecs::WitComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_result(result).and_then(Self::from_u32)
                    }
                    fn into_result(self) -> crate::ecs::WitComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_result()
                    }
                    fn from_value(value: crate::ecs::ComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_value(value).and_then(Self::from_u32)
                    }
                    fn into_value(self) -> crate::ecs::ComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_value()
                    }
                }
                impl MessageSerde for Fit {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Orientation**: Layout orientation."]
                pub enum Orientation {
                    #[default]
                    #[doc = "Horizontal"]
                    Horizontal,
                    #[doc = "Vertical"]
                    Vertical,
                }
                impl crate::ecs::EnumComponent for Orientation {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Horizontal => Orientation::Horizontal as u32,
                            Self::Vertical => Orientation::Vertical as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Orientation::Horizontal as u32 {
                            return Some(Self::Horizontal);
                        }
                        if value == Orientation::Vertical as u32 {
                            return Some(Self::Vertical);
                        }
                        None
                    }
                }
                impl crate::ecs::SupportedValue for Orientation {
                    fn from_result(result: crate::ecs::WitComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_result(result).and_then(Self::from_u32)
                    }
                    fn into_result(self) -> crate::ecs::WitComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_result()
                    }
                    fn from_value(value: crate::ecs::ComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_value(value).and_then(Self::from_u32)
                    }
                    fn into_value(self) -> crate::ecs::ComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_value()
                    }
                }
                impl MessageSerde for Orientation {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Docking**: The edge to dock to."]
                pub enum Docking {
                    #[default]
                    #[doc = "Left"]
                    Left,
                    #[doc = "Right"]
                    Right,
                    #[doc = "Top"]
                    Top,
                    #[doc = "Bottom"]
                    Bottom,
                    #[doc = "Fill"]
                    Fill,
                }
                impl crate::ecs::EnumComponent for Docking {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Left => Docking::Left as u32,
                            Self::Right => Docking::Right as u32,
                            Self::Top => Docking::Top as u32,
                            Self::Bottom => Docking::Bottom as u32,
                            Self::Fill => Docking::Fill as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Docking::Left as u32 {
                            return Some(Self::Left);
                        }
                        if value == Docking::Right as u32 {
                            return Some(Self::Right);
                        }
                        if value == Docking::Top as u32 {
                            return Some(Self::Top);
                        }
                        if value == Docking::Bottom as u32 {
                            return Some(Self::Bottom);
                        }
                        if value == Docking::Fill as u32 {
                            return Some(Self::Fill);
                        }
                        None
                    }
                }
                impl crate::ecs::SupportedValue for Docking {
                    fn from_result(result: crate::ecs::WitComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_result(result).and_then(Self::from_u32)
                    }
                    fn into_result(self) -> crate::ecs::WitComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_result()
                    }
                    fn from_value(value: crate::ecs::ComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_value(value).and_then(Self::from_u32)
                    }
                    fn into_value(self) -> crate::ecs::ComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_value()
                    }
                }
                impl MessageSerde for Docking {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**Layout**: The type of the layout to use."]
                pub enum Layout {
                    #[default]
                    #[doc = "Bottom-up flow layout."]
                    Flow,
                    #[doc = "Top-down dock layout."]
                    Dock,
                    #[doc = "Min-max bookcase layout."]
                    Bookcase,
                    #[doc = "Width to children."]
                    WidthToChildren,
                }
                impl crate::ecs::EnumComponent for Layout {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Flow => Layout::Flow as u32,
                            Self::Dock => Layout::Dock as u32,
                            Self::Bookcase => Layout::Bookcase as u32,
                            Self::WidthToChildren => Layout::WidthToChildren as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == Layout::Flow as u32 {
                            return Some(Self::Flow);
                        }
                        if value == Layout::Dock as u32 {
                            return Some(Self::Dock);
                        }
                        if value == Layout::Bookcase as u32 {
                            return Some(Self::Bookcase);
                        }
                        if value == Layout::WidthToChildren as u32 {
                            return Some(Self::WidthToChildren);
                        }
                        None
                    }
                }
                impl crate::ecs::SupportedValue for Layout {
                    fn from_result(result: crate::ecs::WitComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_result(result).and_then(Self::from_u32)
                    }
                    fn into_result(self) -> crate::ecs::WitComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_result()
                    }
                    fn from_value(value: crate::ecs::ComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_value(value).and_then(Self::from_u32)
                    }
                    fn into_value(self) -> crate::ecs::ComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_value()
                    }
                }
                impl MessageSerde for Layout {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
            }
        }
        pub mod model {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static MODEL_ANIMATABLE: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::model::model_animatable"));
                #[doc = "**Model animatable**: Controls whether this model can be animated.\n\n*Attributes*: MaybeResource, Debuggable, Networked, Store"]
                pub fn model_animatable() -> Component<bool> {
                    *MODEL_ANIMATABLE
                }
                static MODEL_FROM_URL: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::model::model_from_url"));
                #[doc = "**Model from URL**: Load a model from the given URL or relative path.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn model_from_url() -> Component<String> {
                    *MODEL_FROM_URL
                }
                static MODEL_LOADED: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::model::model_loaded"));
                #[doc = "**Model loaded**: If attached, this entity has a model attached to it.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn model_loaded() -> Component<()> {
                    *MODEL_LOADED
                }
            }
        }
        pub mod network {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_REMOTE_ENTITY: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::network::is_remote_entity")
                });
                #[doc = "**Is remote entity**: If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server).\n\n*Attributes*: Debuggable, Networked"]
                pub fn is_remote_entity() -> Component<()> {
                    *IS_REMOTE_ENTITY
                }
                static IS_PERSISTENT_RESOURCES: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::network::is_persistent_resources")
                });
                #[doc = "**Is persistent resources**: If attached, this entity contains global resources that are persisted to disk and synchronized to clients.\n\n*Attributes*: Debuggable, Networked"]
                pub fn is_persistent_resources() -> Component<()> {
                    *IS_PERSISTENT_RESOURCES
                }
                static IS_SYNCED_RESOURCES: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::network::is_synced_resources")
                });
                #[doc = "**Is synchronized resources**: If attached, this entity contains global resources that are synchronized to clients, but not persisted.\n\n*Attributes*: Debuggable, Networked"]
                pub fn is_synced_resources() -> Component<()> {
                    *IS_SYNCED_RESOURCES
                }
                static NO_SYNC: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::network::no_sync"));
                #[doc = "**No sync**: If attached, this entity will not be synchronized to clients.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn no_sync() -> Component<()> {
                    *NO_SYNC
                }
            }
        }
        pub mod package {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_PACKAGE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::is_package"));
                #[doc = "**Is Package**: Whether or not this entity is a package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn is_package() -> Component<()> {
                    *IS_PACKAGE
                }
                static ENABLED: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::enabled"));
                #[doc = "**Enabled**: Whether or not this package is enabled.\n\n*Attributes*: Debuggable, Networked"]
                pub fn enabled() -> Component<bool> {
                    *ENABLED
                }
                static ID: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::id"));
                #[doc = "**ID**: The ID of the package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn id() -> Component<String> {
                    *ID
                }
                static NAME: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::name"));
                #[doc = "**Name**: The name of the package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn name() -> Component<String> {
                    *NAME
                }
                static VERSION: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::version"));
                #[doc = "**Version**: The version of the package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn version() -> Component<String> {
                    *VERSION
                }
                static AUTHORS: Lazy<Component<Vec<String>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::authors"));
                #[doc = "**Authors**: The authors of the package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn authors() -> Component<Vec<String>> {
                    *AUTHORS
                }
                static DESCRIPTION: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::description"));
                #[doc = "**Description**: The description of the package. If not attached, the package does not have a description.\n\n*Attributes*: Debuggable, Networked"]
                pub fn description() -> Component<String> {
                    *DESCRIPTION
                }
                static REPOSITORY: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::repository"));
                #[doc = "**Repository**: The repository of the package. If not attached, the package does not have a repository.\n\n*Attributes*: Debuggable, Networked"]
                pub fn repository() -> Component<String> {
                    *REPOSITORY
                }
                static ASSET_URL: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::asset_url"));
                #[doc = "**Asset URL**: The asset URL (i.e. where the built assets are) of the package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn asset_url() -> Component<String> {
                    *ASSET_URL
                }
                static CLIENT_MODULES: Lazy<Component<Vec<EntityId>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::client_modules"));
                #[doc = "**Client Modules**: The clientside WASM modules spawned by this package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn client_modules() -> Component<Vec<EntityId>> {
                    *CLIENT_MODULES
                }
                static SERVER_MODULES: Lazy<Component<Vec<EntityId>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::package::server_modules"));
                #[doc = "**Server Modules**: The serverside WASM modules spawned by this package.\n\n*Attributes*: Debuggable, Networked"]
                pub fn server_modules() -> Component<Vec<EntityId>> {
                    *SERVER_MODULES
                }
            }
        }
        pub mod physics {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static ANGULAR_VELOCITY: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::angular_velocity")
                });
                #[doc = "**Angular velocity**: Angular velocity (radians/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's angular velocity in the physics scene.\n\n\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\n\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn angular_velocity() -> Component<Vec3> {
                    *ANGULAR_VELOCITY
                }
                static CUBE_COLLIDER: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::cube_collider"));
                #[doc = "**Cube collider**: If attached, this entity will have a cube physics collider.\n\n`x, y, z` is the size of the cube.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn cube_collider() -> Component<Vec3> {
                    *CUBE_COLLIDER
                }
                static CHARACTER_CONTROLLER_HEIGHT: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::character_controller_height")
                });
                #[doc = "**Character controller height**: The height of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn character_controller_height() -> Component<f32> {
                    *CHARACTER_CONTROLLER_HEIGHT
                }
                static CHARACTER_CONTROLLER_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::character_controller_radius")
                });
                #[doc = "**Character controller radius**: The radius of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn character_controller_radius() -> Component<f32> {
                    *CHARACTER_CONTROLLER_RADIUS
                }
                static COLLIDER_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::collider_from_url")
                });
                #[doc = "**Collider from URL**: This entity will load its physics collider from the URL.\n\nThe value is the URL to load from.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn collider_from_url() -> Component<String> {
                    *COLLIDER_FROM_URL
                }
                static COLLIDER_LOADED: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::collider_loaded")
                });
                #[doc = "**Collider loaded**: This component is automatically attached to an entity once the collider has been loaded (through e.g. `collider_from_url`).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn collider_loaded() -> Component<()> {
                    *COLLIDER_LOADED
                }
                static COLLIDER_LOADS: Lazy<Component<Vec<EntityId>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::collider_loads"));
                #[doc = "**Collider loads**: Contains all colliders that were loaded in this physics tick.\n\n*Attributes*: Debuggable, Networked, Resource, Store"]
                pub fn collider_loads() -> Component<Vec<EntityId>> {
                    *COLLIDER_LOADS
                }
                static CONTACT_OFFSET: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::contact_offset"));
                #[doc = "**Contact offset**: Contact offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's contact offset for each attached shape in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn contact_offset() -> Component<f32> {
                    *CONTACT_OFFSET
                }
                static DENSITY: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::density"));
                #[doc = "**Density**: The density of this entity.\n\nThis is used to update the `mass` when the entity is rescaled.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 1.0"]
                pub fn density() -> Component<f32> {
                    *DENSITY
                }
                static DYNAMIC: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::dynamic"));
                #[doc = "**Dynamic**: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn dynamic() -> Component<bool> {
                    *DYNAMIC
                }
                static KINEMATIC: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::kinematic"));
                #[doc = "**Kinematic**: If attached, and this entity is dynamic, this entity will also be kinematic (i.e. unable to be affected by other entities motion). Otherwise, it will receive forces normally.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn kinematic() -> Component<()> {
                    *KINEMATIC
                }
                static LINEAR_VELOCITY: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::linear_velocity")
                });
                #[doc = "**Linear velocity**: Linear velocity (meters/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's linear velocity in the physics scene.\n\n\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\n\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn linear_velocity() -> Component<Vec3> {
                    *LINEAR_VELOCITY
                }
                static MAKE_PHYSICS_STATIC: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::make_physics_static")
                });
                #[doc = "**Make physics static**: All physics objects will be made static when loaded.\n\n*Attributes*: Debuggable, Networked, Resource, Store"]
                pub fn make_physics_static() -> Component<bool> {
                    *MAKE_PHYSICS_STATIC
                }
                static MASS: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::mass"));
                #[doc = "**Mass**: The mass of this entity, measured in kilograms.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 1.0"]
                pub fn mass() -> Component<f32> {
                    *MASS
                }
                static PHYSICS_CONTROLLED: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::physics_controlled")
                });
                #[doc = "**Physics controlled**: If attached, this entity will be controlled by physics.\n\nNote that this requires the entity to have a collider.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn physics_controlled() -> Component<()> {
                    *PHYSICS_CONTROLLED
                }
                static PLANE_COLLIDER: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::plane_collider"));
                #[doc = "**Plane collider**: If attached, this entity will have a plane physics collider. A plane is an infinite, flat surface. If you need a bounded flat surface, consider using a cube collider instead.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn plane_collider() -> Component<()> {
                    *PLANE_COLLIDER
                }
                static REST_OFFSET: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::rest_offset"));
                #[doc = "**Rest offset**: Rest offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's rest offset for each attached shape in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn rest_offset() -> Component<f32> {
                    *REST_OFFSET
                }
                static SPHERE_COLLIDER: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::sphere_collider")
                });
                #[doc = "**Sphere collider**: If attached, this entity will have a sphere physics collider.\n\nThe value corresponds to the radius of the sphere.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn sphere_collider() -> Component<f32> {
                    *SPHERE_COLLIDER
                }
                static UNIT_MASS: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::unit_mass"));
                #[doc = "**Unit mass**: The mass of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn unit_mass() -> Component<f32> {
                    *UNIT_MASS
                }
                static UNIT_VELOCITY: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::unit_velocity"));
                #[doc = "**Unit velocity**: The velocity of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn unit_velocity() -> Component<Vec3> {
                    *UNIT_VELOCITY
                }
                static UNIT_YAW: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::physics::unit_yaw"));
                #[doc = "**Unit yaw**: The yaw of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn unit_yaw() -> Component<f32> {
                    *UNIT_YAW
                }
                static VISUALIZE_COLLIDER: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::physics::visualize_collider")
                });
                #[doc = "**Visualize collider**: If attached, the collider will be rendered.\n\n\n\n**Note**: this will continuously overwrite the `local_gizmos` component.\n\n\n\n*Attributes*: Debuggable, Networked"]
                pub fn visualize_collider() -> Component<()> {
                    *VISUALIZE_COLLIDER
                }
            }
            #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
            #[doc = r""]
            #[doc = r" They do not have any runtime representation outside of the components that compose them."]
            pub mod concepts {
                use crate::prelude::*;
                #[doc = "**Character Controller**: A capsule character controller. The capsule is defined as a position, a vertical height, and a radius. The height is the distance between the two sphere centers at the end of the capsule."]
                #[derive(Clone, Debug)]
                pub struct CharacterController {
                    #[doc = "**Component**: `ambient_core::physics::character_controller_height`\n\n**Suggested value**: `2f32`\n\n**Component description**: The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n\n"]
                    pub character_controller_height: f32,
                    #[doc = "**Component**: `ambient_core::physics::character_controller_radius`\n\n**Suggested value**: `0.5f32`\n\n**Component description**: The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n\n"]
                    pub character_controller_radius: f32,
                    #[doc = "**Component**: `ambient_core::physics::physics_controlled`\n\n**Suggested value**: `()`\n\n**Component description**: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n\n"]
                    pub physics_controlled: (),
                }
                impl crate::ecs::Concept for CharacterController {
                    fn make(self) -> Entity {
                        let mut entity = Entity :: new () . with (crate :: ambient_core :: physics :: components :: character_controller_height () , self . character_controller_height) . with (crate :: ambient_core :: physics :: components :: character_controller_radius () , self . character_controller_radius) . with (crate :: ambient_core :: physics :: components :: physics_controlled () , self . physics_controlled) ;
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some (Self { character_controller_height : entity :: get_component (id , crate :: ambient_core :: physics :: components :: character_controller_height ()) ? , character_controller_radius : entity :: get_component (id , crate :: ambient_core :: physics :: components :: character_controller_radius ()) ? , physics_controlled : entity :: get_component (id , crate :: ambient_core :: physics :: components :: physics_controlled ()) ? , })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some (Self { character_controller_height : entity . get (crate :: ambient_core :: physics :: components :: character_controller_height ()) ? , character_controller_radius : entity . get (crate :: ambient_core :: physics :: components :: character_controller_radius ()) ? , physics_controlled : entity . get (crate :: ambient_core :: physics :: components :: physics_controlled ()) ? , })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity :: has_components (id , & [& crate :: ambient_core :: physics :: components :: character_controller_height () , & crate :: ambient_core :: physics :: components :: character_controller_radius () , & crate :: ambient_core :: physics :: components :: physics_controlled ()])
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::physics::components::character_controller_height(
                            ),
                            &crate::ambient_core::physics::components::character_controller_radius(
                            ),
                            &crate::ambient_core::physics::components::physics_controlled(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for CharacterController {
                    fn suggested() -> Self {
                        Self {
                            character_controller_height: 2f32,
                            character_controller_radius: 0.5f32,
                            physics_controlled: (),
                        }
                    }
                }
            }
        }
        pub mod player {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static LOCAL_USER_ID: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::player::local_user_id"));
                #[doc = "**Local user ID**: The user ID of the local player.\n\n*Attributes*: Debuggable, Networked, Resource, Store"]
                pub fn local_user_id() -> Component<String> {
                    *LOCAL_USER_ID
                }
                static IS_PLAYER: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::player::is_player"));
                #[doc = "**Is player**: This entity is a player.\n\nNote that this is a logical construct; a player's body may be separate from the player itself.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn is_player() -> Component<()> {
                    *IS_PLAYER
                }
                static USER_ID: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::player::user_id"));
                #[doc = "**User ID**: An identifier attached to all things owned by a user, and supplied by the user.\n\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn user_id() -> Component<String> {
                    *USER_ID
                }
            }
        }
        pub mod prefab {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static PREFAB_FROM_URL: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::prefab::prefab_from_url"));
                #[doc = "**Prefab from URL**: Load and attach a prefab from a URL or relative path.\n\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity.\n\n*Attributes*: Debuggable, Store"]
                pub fn prefab_from_url() -> Component<String> {
                    *PREFAB_FROM_URL
                }
                static SPAWNED: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::prefab::spawned"));
                #[doc = "**Spawned**: If attached, this entity was built from a prefab that has finished spawning.\n\n*Attributes*: Debuggable"]
                pub fn spawned() -> Component<()> {
                    *SPAWNED
                }
            }
        }
        pub mod primitives {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static CUBE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::primitives::cube"));
                #[doc = "**Cube**: If attached to an entity, the entity will be converted to a cube primitive.\n\nThe cube is unit-sized (i.e. 0.5 metres out to each side).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn cube() -> Component<()> {
                    *CUBE
                }
                static QUAD: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::primitives::quad"));
                #[doc = "**Quad**: If attached to an entity, the entity will be converted to a quad primitive.\n\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn quad() -> Component<()> {
                    *QUAD
                }
                static SPHERE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::primitives::sphere"));
                #[doc = "**Sphere**: If attached to an entity alongside the other `sphere_*` components, the entity will be converted to a sphere primitive.\n\nTo easily instantiate a unit-diameter `sphere`, consider using the `sphere` concept (e.g. `make_sphere`).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn sphere() -> Component<()> {
                    *SPHERE
                }
                static SPHERE_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::sphere_radius")
                });
                #[doc = "**Sphere radius**: Set the radius of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 0.5"]
                pub fn sphere_radius() -> Component<f32> {
                    *SPHERE_RADIUS
                }
                static SPHERE_SECTORS: Lazy<Component<u32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::sphere_sectors")
                });
                #[doc = "**Sphere sectors**: Set the longitudinal sectors of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 36"]
                pub fn sphere_sectors() -> Component<u32> {
                    *SPHERE_SECTORS
                }
                static SPHERE_STACKS: Lazy<Component<u32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::sphere_stacks")
                });
                #[doc = "**Sphere stacks**: Set the latitudinal stacks of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 18"]
                pub fn sphere_stacks() -> Component<u32> {
                    *SPHERE_STACKS
                }
                static TORUS: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::primitives::torus"));
                #[doc = "**Torus**: If attached to an entity alongside the other `torus_*` components, the entity will be converted to a torus primitive.\n\nTo easily instantiate a default `torus`, consider using the `torus` concept (e.g. `make_torus`).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn torus() -> Component<()> {
                    *TORUS
                }
                static TORUS_INNER_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::torus_inner_radius")
                });
                #[doc = "**Torus inner radius**: Set the inner radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn torus_inner_radius() -> Component<f32> {
                    *TORUS_INNER_RADIUS
                }
                static TORUS_OUTER_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::torus_outer_radius")
                });
                #[doc = "**Torus outer radius**: Set the outer radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn torus_outer_radius() -> Component<f32> {
                    *TORUS_OUTER_RADIUS
                }
                static TORUS_LOOPS: Lazy<Component<u32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::primitives::torus_loops"));
                #[doc = "**Torus loops**: Set the loops of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn torus_loops() -> Component<u32> {
                    *TORUS_LOOPS
                }
                static TORUS_SLICES: Lazy<Component<u32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::torus_slices")
                });
                #[doc = "**Torus slices**: Set the slices of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn torus_slices() -> Component<u32> {
                    *TORUS_SLICES
                }
                static CAPSULE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::primitives::capsule"));
                #[doc = "**Capsule**: If attached to an entity alongside the other `capsule_*` components, the entity will be converted to a capsule primitive.\n\nTo easily instantiate a default `capsule`, consider using the `capsule` concept (e.g. `make_capsule`).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn capsule() -> Component<()> {
                    *CAPSULE
                }
                static CAPSULE_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::capsule_radius")
                });
                #[doc = "**Capsule radius**: Set the radius of a `capsule` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn capsule_radius() -> Component<f32> {
                    *CAPSULE_RADIUS
                }
                static CAPSULE_HALF_HEIGHT: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::capsule_half_height")
                });
                #[doc = "**Capsule half-height**: Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn capsule_half_height() -> Component<f32> {
                    *CAPSULE_HALF_HEIGHT
                }
                static CAPSULE_RINGS: Lazy<Component<u32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::capsule_rings")
                });
                #[doc = "**Capsule rings**: Set the number of sections between the caps.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn capsule_rings() -> Component<u32> {
                    *CAPSULE_RINGS
                }
                static CAPSULE_LATITUDES: Lazy<Component<u32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::capsule_latitudes")
                });
                #[doc = "**Capsule latitudes**: Set the number of latitudinal sections. Should be even.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn capsule_latitudes() -> Component<u32> {
                    *CAPSULE_LATITUDES
                }
                static CAPSULE_LONGITUDES: Lazy<Component<u32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::primitives::capsule_longitudes")
                });
                #[doc = "**Capsule longitudes**: Set the number of longitudinal sections.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn capsule_longitudes() -> Component<u32> {
                    *CAPSULE_LONGITUDES
                }
            }
            #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
            #[doc = r""]
            #[doc = r" They do not have any runtime representation outside of the components that compose them."]
            pub mod concepts {
                use crate::prelude::*;
                #[doc = "**Sphere**: A primitive sphere."]
                #[derive(Clone, Debug)]
                pub struct Sphere {
                    #[doc = "**Component**: `ambient_core::primitives::sphere`\n\n**Suggested value**: `()`\n\n**Component description**: If attached to an entity alongside the other `sphere_*` components, the entity will be converted to a sphere primitive.\nTo easily instantiate a unit-diameter `sphere`, consider using the `sphere` concept (e.g. `make_sphere`).\n\n"]
                    pub sphere: (),
                    #[doc = "**Component**: `ambient_core::primitives::sphere_radius`\n\n**Suggested value**: `0.5f32`\n\n**Component description**: Set the radius of a `sphere` entity.\n\n"]
                    pub sphere_radius: f32,
                    #[doc = "**Component**: `ambient_core::primitives::sphere_sectors`\n\n**Suggested value**: `36u32`\n\n**Component description**: Set the longitudinal sectors of a `sphere` entity.\n\n"]
                    pub sphere_sectors: u32,
                    #[doc = "**Component**: `ambient_core::primitives::sphere_stacks`\n\n**Suggested value**: `18u32`\n\n**Component description**: Set the latitudinal stacks of a `sphere` entity.\n\n"]
                    pub sphere_stacks: u32,
                }
                impl crate::ecs::Concept for Sphere {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::primitives::components::sphere(),
                                self.sphere,
                            )
                            .with(
                                crate::ambient_core::primitives::components::sphere_radius(),
                                self.sphere_radius,
                            )
                            .with(
                                crate::ambient_core::primitives::components::sphere_sectors(),
                                self.sphere_sectors,
                            )
                            .with(
                                crate::ambient_core::primitives::components::sphere_stacks(),
                                self.sphere_stacks,
                            );
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some(Self {
                            sphere: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::sphere(),
                            )?,
                            sphere_radius: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::sphere_radius(),
                            )?,
                            sphere_sectors: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::sphere_sectors(),
                            )?,
                            sphere_stacks: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::sphere_stacks(),
                            )?,
                        })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some(Self {
                            sphere: entity
                                .get(crate::ambient_core::primitives::components::sphere())?,
                            sphere_radius: entity
                                .get(crate::ambient_core::primitives::components::sphere_radius())?,
                            sphere_sectors: entity.get(
                                crate::ambient_core::primitives::components::sphere_sectors(),
                            )?,
                            sphere_stacks: entity
                                .get(crate::ambient_core::primitives::components::sphere_stacks())?,
                        })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::primitives::components::sphere(),
                                &crate::ambient_core::primitives::components::sphere_radius(),
                                &crate::ambient_core::primitives::components::sphere_sectors(),
                                &crate::ambient_core::primitives::components::sphere_stacks(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::primitives::components::sphere(),
                            &crate::ambient_core::primitives::components::sphere_radius(),
                            &crate::ambient_core::primitives::components::sphere_sectors(),
                            &crate::ambient_core::primitives::components::sphere_stacks(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for Sphere {
                    fn suggested() -> Self {
                        Self {
                            sphere: (),
                            sphere_radius: 0.5f32,
                            sphere_sectors: 36u32,
                            sphere_stacks: 18u32,
                        }
                    }
                }
                #[doc = "**Capsule**: A primitive capsule. Defined as a cylinder capped by hemispheres."]
                #[derive(Clone, Debug)]
                pub struct Capsule {
                    #[doc = "**Component**: `ambient_core::primitives::capsule`\n\n**Suggested value**: `()`\n\n**Component description**: If attached to an entity alongside the other `capsule_*` components, the entity will be converted to a capsule primitive.\nTo easily instantiate a default `capsule`, consider using the `capsule` concept (e.g. `make_capsule`).\n\n"]
                    pub capsule: (),
                    #[doc = "**Component**: `ambient_core::primitives::capsule_radius`\n\n**Suggested value**: `0.5f32`\n\n**Component description**: Set the radius of a `capsule` entity, spanning XY-plane.\n\n"]
                    pub capsule_radius: f32,
                    #[doc = "**Component**: `ambient_core::primitives::capsule_half_height`\n\n**Suggested value**: `0.5f32`\n\n**Component description**: Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps.\n\n"]
                    pub capsule_half_height: f32,
                    #[doc = "**Component**: `ambient_core::primitives::capsule_rings`\n\n**Suggested value**: `0u32`\n\n**Component description**: Set the number of sections between the caps.\n\n"]
                    pub capsule_rings: u32,
                    #[doc = "**Component**: `ambient_core::primitives::capsule_latitudes`\n\n**Suggested value**: `16u32`\n\n**Component description**: Set the number of latitudinal sections. Should be even.\n\n"]
                    pub capsule_latitudes: u32,
                    #[doc = "**Component**: `ambient_core::primitives::capsule_longitudes`\n\n**Suggested value**: `32u32`\n\n**Component description**: Set the number of longitudinal sections.\n\n"]
                    pub capsule_longitudes: u32,
                }
                impl crate::ecs::Concept for Capsule {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::primitives::components::capsule(),
                                self.capsule,
                            )
                            .with(
                                crate::ambient_core::primitives::components::capsule_radius(),
                                self.capsule_radius,
                            )
                            .with(
                                crate::ambient_core::primitives::components::capsule_half_height(),
                                self.capsule_half_height,
                            )
                            .with(
                                crate::ambient_core::primitives::components::capsule_rings(),
                                self.capsule_rings,
                            )
                            .with(
                                crate::ambient_core::primitives::components::capsule_latitudes(),
                                self.capsule_latitudes,
                            )
                            .with(
                                crate::ambient_core::primitives::components::capsule_longitudes(),
                                self.capsule_longitudes,
                            );
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some(Self {
                            capsule: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::capsule(),
                            )?,
                            capsule_radius: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::capsule_radius(),
                            )?,
                            capsule_half_height: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::capsule_half_height(),
                            )?,
                            capsule_rings: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::capsule_rings(),
                            )?,
                            capsule_latitudes: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::capsule_latitudes(),
                            )?,
                            capsule_longitudes: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::capsule_longitudes(),
                            )?,
                        })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some(Self {
                            capsule: entity
                                .get(crate::ambient_core::primitives::components::capsule())?,
                            capsule_radius: entity.get(
                                crate::ambient_core::primitives::components::capsule_radius(),
                            )?,
                            capsule_half_height: entity.get(
                                crate::ambient_core::primitives::components::capsule_half_height(),
                            )?,
                            capsule_rings: entity
                                .get(crate::ambient_core::primitives::components::capsule_rings())?,
                            capsule_latitudes: entity.get(
                                crate::ambient_core::primitives::components::capsule_latitudes(),
                            )?,
                            capsule_longitudes: entity.get(
                                crate::ambient_core::primitives::components::capsule_longitudes(),
                            )?,
                        })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::primitives::components::capsule(),
                                &crate::ambient_core::primitives::components::capsule_radius(),
                                &crate::ambient_core::primitives::components::capsule_half_height(),
                                &crate::ambient_core::primitives::components::capsule_rings(),
                                &crate::ambient_core::primitives::components::capsule_latitudes(),
                                &crate::ambient_core::primitives::components::capsule_longitudes(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::primitives::components::capsule(),
                            &crate::ambient_core::primitives::components::capsule_radius(),
                            &crate::ambient_core::primitives::components::capsule_half_height(),
                            &crate::ambient_core::primitives::components::capsule_rings(),
                            &crate::ambient_core::primitives::components::capsule_latitudes(),
                            &crate::ambient_core::primitives::components::capsule_longitudes(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for Capsule {
                    fn suggested() -> Self {
                        Self {
                            capsule: (),
                            capsule_radius: 0.5f32,
                            capsule_half_height: 0.5f32,
                            capsule_rings: 0u32,
                            capsule_latitudes: 16u32,
                            capsule_longitudes: 32u32,
                        }
                    }
                }
                #[doc = "**Torus**: A primitive Torus, surface of revolution generated by revolving a circle in three-dimensional space one full revolution."]
                #[derive(Clone, Debug)]
                pub struct Torus {
                    #[doc = "**Component**: `ambient_core::primitives::torus`\n\n**Suggested value**: `()`\n\n**Component description**: If attached to an entity alongside the other `torus_*` components, the entity will be converted to a torus primitive.\nTo easily instantiate a default `torus`, consider using the `torus` concept (e.g. `make_torus`).\n\n"]
                    pub torus: (),
                    #[doc = "**Component**: `ambient_core::primitives::torus_inner_radius`\n\n**Suggested value**: `0.25f32`\n\n**Component description**: Set the inner radius of a `torus` entity, spanning XY-plane.\n\n"]
                    pub torus_inner_radius: f32,
                    #[doc = "**Component**: `ambient_core::primitives::torus_outer_radius`\n\n**Suggested value**: `0.35f32`\n\n**Component description**: Set the outer radius of a `torus` entity, spanning XY-plane.\n\n"]
                    pub torus_outer_radius: f32,
                    #[doc = "**Component**: `ambient_core::primitives::torus_slices`\n\n**Suggested value**: `32u32`\n\n**Component description**: Set the slices of a `torus` entity, spanning XY-plane.\n\n"]
                    pub torus_slices: u32,
                    #[doc = "**Component**: `ambient_core::primitives::torus_loops`\n\n**Suggested value**: `16u32`\n\n**Component description**: Set the loops of a `torus` entity, spanning XY-plane.\n\n"]
                    pub torus_loops: u32,
                }
                impl crate::ecs::Concept for Torus {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new()
                            .with(
                                crate::ambient_core::primitives::components::torus(),
                                self.torus,
                            )
                            .with(
                                crate::ambient_core::primitives::components::torus_inner_radius(),
                                self.torus_inner_radius,
                            )
                            .with(
                                crate::ambient_core::primitives::components::torus_outer_radius(),
                                self.torus_outer_radius,
                            )
                            .with(
                                crate::ambient_core::primitives::components::torus_slices(),
                                self.torus_slices,
                            )
                            .with(
                                crate::ambient_core::primitives::components::torus_loops(),
                                self.torus_loops,
                            );
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some(Self {
                            torus: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::torus(),
                            )?,
                            torus_inner_radius: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::torus_inner_radius(),
                            )?,
                            torus_outer_radius: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::torus_outer_radius(),
                            )?,
                            torus_slices: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::torus_slices(),
                            )?,
                            torus_loops: entity::get_component(
                                id,
                                crate::ambient_core::primitives::components::torus_loops(),
                            )?,
                        })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some(Self {
                            torus: entity
                                .get(crate::ambient_core::primitives::components::torus())?,
                            torus_inner_radius: entity.get(
                                crate::ambient_core::primitives::components::torus_inner_radius(),
                            )?,
                            torus_outer_radius: entity.get(
                                crate::ambient_core::primitives::components::torus_outer_radius(),
                            )?,
                            torus_slices: entity
                                .get(crate::ambient_core::primitives::components::torus_slices())?,
                            torus_loops: entity
                                .get(crate::ambient_core::primitives::components::torus_loops())?,
                        })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[
                                &crate::ambient_core::primitives::components::torus(),
                                &crate::ambient_core::primitives::components::torus_inner_radius(),
                                &crate::ambient_core::primitives::components::torus_outer_radius(),
                                &crate::ambient_core::primitives::components::torus_slices(),
                                &crate::ambient_core::primitives::components::torus_loops(),
                            ],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::primitives::components::torus(),
                            &crate::ambient_core::primitives::components::torus_inner_radius(),
                            &crate::ambient_core::primitives::components::torus_outer_radius(),
                            &crate::ambient_core::primitives::components::torus_slices(),
                            &crate::ambient_core::primitives::components::torus_loops(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for Torus {
                    fn suggested() -> Self {
                        Self {
                            torus: (),
                            torus_inner_radius: 0.25f32,
                            torus_outer_radius: 0.35f32,
                            torus_slices: 32u32,
                            torus_loops: 16u32,
                        }
                    }
                }
            }
        }
        pub mod procedurals {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static PROCEDURAL_MESH: Lazy<Component<ProceduralMeshHandle>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::procedurals::procedural_mesh")
                });
                #[doc = "**Procedural mesh**: Attaches a procedural mesh to this entity\n\n*Attributes*: Debuggable, Store"]
                pub fn procedural_mesh() -> Component<ProceduralMeshHandle> {
                    *PROCEDURAL_MESH
                }
                static PROCEDURAL_MATERIAL: Lazy<Component<ProceduralMaterialHandle>> =
                    Lazy::new(|| {
                        __internal_get_component("ambient_core::procedurals::procedural_material")
                    });
                #[doc = "**Procedural material**: Attaches a procedural material to this entity\n\n*Attributes*: Debuggable, Store"]
                pub fn procedural_material() -> Component<ProceduralMaterialHandle> {
                    *PROCEDURAL_MATERIAL
                }
            }
        }
        pub mod rect {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static BACKGROUND_COLOR: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::background_color"));
                #[doc = "**Background color**: Background color of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn background_color() -> Component<Vec4> {
                    *BACKGROUND_COLOR
                }
                static BACKGROUND_URL: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::background_url"));
                #[doc = "**Background URL**: URL to an image asset.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn background_url() -> Component<String> {
                    *BACKGROUND_URL
                }
                static BORDER_COLOR: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::border_color"));
                #[doc = "**Border color**: Border color of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn border_color() -> Component<Vec4> {
                    *BORDER_COLOR
                }
                static BORDER_RADIUS: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::border_radius"));
                #[doc = "**Border radius**: Radius for each corner of an entity with a `rect` component.\n\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn border_radius() -> Component<Vec4> {
                    *BORDER_RADIUS
                }
                static BORDER_THICKNESS: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::border_thickness"));
                #[doc = "**Border thickness**: Border thickness of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn border_thickness() -> Component<f32> {
                    *BORDER_THICKNESS
                }
                static PIXEL_LINE_FROM: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::pixel_line_from"));
                #[doc = "**Pixel Line from**: Start point of a pixel sized line.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn pixel_line_from() -> Component<Vec3> {
                    *PIXEL_LINE_FROM
                }
                static PIXEL_LINE_TO: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::pixel_line_to"));
                #[doc = "**Pixel Line to**: End point of a pixel sized line.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn pixel_line_to() -> Component<Vec3> {
                    *PIXEL_LINE_TO
                }
                static LINE_FROM: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::line_from"));
                #[doc = "**Line from**: Start point of a line.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn line_from() -> Component<Vec3> {
                    *LINE_FROM
                }
                static LINE_TO: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::line_to"));
                #[doc = "**Line to**: End point of a line.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn line_to() -> Component<Vec3> {
                    *LINE_TO
                }
                static LINE_WIDTH: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::line_width"));
                #[doc = "**Line width**: Width of line.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn line_width() -> Component<f32> {
                    *LINE_WIDTH
                }
                static RECT: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rect::rect"));
                #[doc = "**Rect**: If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn rect() -> Component<()> {
                    *RECT
                }
                static SIZE_FROM_BACKGROUND_IMAGE: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rect::size_from_background_image")
                });
                #[doc = "**Size from background image**: Resize this rect based on the size of the background image.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn size_from_background_image() -> Component<()> {
                    *SIZE_FROM_BACKGROUND_IMAGE
                }
            }
        }
        pub mod rendering {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static CAST_SHADOWS: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::cast_shadows"));
                #[doc = "**Cast shadows**: If attached, this entity will cast shadows.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn cast_shadows() -> Component<()> {
                    *CAST_SHADOWS
                }
                static COLOR: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::color"));
                #[doc = "**Color**: This entity will be tinted with the specified color if the color is not black.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn color() -> Component<Vec4> {
                    *COLOR
                }
                static DOUBLE_SIDED: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::double_sided"));
                #[doc = "**Double-sided**: If attached, this controls whether or not the entity will be rendered with double-sided rendering. If not attached, the decision will fall back to the material.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn double_sided() -> Component<bool> {
                    *DOUBLE_SIDED
                }
                static FOG_COLOR: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::fog_color"));
                #[doc = "**Fog color**: The color of the fog for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn fog_color() -> Component<Vec3> {
                    *FOG_COLOR
                }
                static FOG_DENSITY: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::fog_density"));
                #[doc = "**Fog density**: The density of the fog for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn fog_density() -> Component<f32> {
                    *FOG_DENSITY
                }
                static FOG_HEIGHT_FALLOFF: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::fog_height_falloff")
                });
                #[doc = "**Fog height fall-off**: The height at which the fog will fall off (i.e. stop being visible) for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn fog_height_falloff() -> Component<f32> {
                    *FOG_HEIGHT_FALLOFF
                }
                static JOINT_MATRICES: Lazy<Component<Vec<Mat4>>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::joint_matrices")
                });
                #[doc = "**Joint Matrices**: Contains the matrices for each joint of this skinned mesh.\n\nThis should be used in combination with `joints`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn joint_matrices() -> Component<Vec<Mat4>> {
                    *JOINT_MATRICES
                }
                static JOINTS: Lazy<Component<Vec<EntityId>>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::joints"));
                #[doc = "**Joints**: Contains the joints that comprise this skinned mesh.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn joints() -> Component<Vec<EntityId>> {
                    *JOINTS
                }
                static LIGHT_AMBIENT: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::light_ambient")
                });
                #[doc = "**Light ambient**: The ambient light color of the `sun`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn light_ambient() -> Component<Vec3> {
                    *LIGHT_AMBIENT
                }
                static LIGHT_DIFFUSE: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::light_diffuse")
                });
                #[doc = "**Light diffuse**: The diffuse light color of the `sun`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn light_diffuse() -> Component<Vec3> {
                    *LIGHT_DIFFUSE
                }
                static OUTLINE: Lazy<Component<Vec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::outline"));
                #[doc = "**Outline**: If attached, this entity will be rendered with an outline with the color specified.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn outline() -> Component<Vec4> {
                    *OUTLINE
                }
                static OUTLINE_RECURSIVE: Lazy<Component<Vec4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::outline_recursive")
                });
                #[doc = "**Outline (recursive)**: If attached, this entity and all of its children will be rendered with an outline with the color specified.\n\nYou do not need to attach `outline` if you have attached `outline_recursive`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn outline_recursive() -> Component<Vec4> {
                    *OUTLINE_RECURSIVE
                }
                static OVERLAY: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::overlay"));
                #[doc = "**Overlay**: If attached, this entity will be rendered with an overlay.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn overlay() -> Component<()> {
                    *OVERLAY
                }
                static PBR_MATERIAL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::pbr_material_from_url")
                });
                #[doc = "**PBR material from URL**: Load a PBR material from the URL and attach it to this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn pbr_material_from_url() -> Component<String> {
                    *PBR_MATERIAL_FROM_URL
                }
                static SKY: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::sky"));
                #[doc = "**Sky**: Add a realistic skybox to the scene.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn sky() -> Component<()> {
                    *SKY
                }
                static SUN: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::sun"));
                #[doc = "**Sun**: Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\n\nThe entity with the highest `sun` value takes precedence.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn sun() -> Component<f32> {
                    *SUN
                }
                static TRANSPARENCY_GROUP: Lazy<Component<i32>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::transparency_group")
                });
                #[doc = "**Transparency group**: Controls when this transparent object will be rendered. Transparent objects are sorted by `(transparency_group, z-depth)`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn transparency_group() -> Component<i32> {
                    *TRANSPARENCY_GROUP
                }
                static WATER: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::water"));
                #[doc = "**Water**: Add a realistic water plane to this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn water() -> Component<()> {
                    *WATER
                }
                static DECAL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::decal_from_url")
                });
                #[doc = "**Decal material from URL**: Load a Decal material from the URL and attach it to this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn decal_from_url() -> Component<String> {
                    *DECAL_FROM_URL
                }
                static SCISSORS: Lazy<Component<UVec4>> =
                    Lazy::new(|| __internal_get_component("ambient_core::rendering::scissors"));
                #[doc = "**Scissors**: Apply a scissors test to this entity (anything outside the rect will be hidden).\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn scissors() -> Component<UVec4> {
                    *SCISSORS
                }
                static SCISSORS_RECURSIVE: Lazy<Component<UVec4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::rendering::scissors_recursive")
                });
                #[doc = "**Scissors (recursive)**: If attached, this entity and all of its children will be rendered with an scissor with the rect specified.\n\nYou do not need to attach `scissors` if you have attached `scissors_recursive`.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn scissors_recursive() -> Component<UVec4> {
                    *SCISSORS_RECURSIVE
                }
            }
        }
        pub mod text {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static FONT_FAMILY: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::text::font_family"));
                #[doc = "**Font family**: Font family to be used. Can either be 'Default', 'FontAwesome', 'FontAwesomeSolid', 'Code' or a url to a font.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn font_family() -> Component<String> {
                    *FONT_FAMILY
                }
                static FONT_SIZE: Lazy<Component<f32>> =
                    Lazy::new(|| __internal_get_component("ambient_core::text::font_size"));
                #[doc = "**Font size**: Size of the font.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn font_size() -> Component<f32> {
                    *FONT_SIZE
                }
                static FONT_STYLE: Lazy<Component<crate::ambient_core::text::types::FontStyle>> =
                    Lazy::new(|| __internal_get_component("ambient_core::text::font_style"));
                #[doc = "**Font style**: Style of the font.\n\n*Attributes*: Debuggable, Networked, Store, Enum"]
                pub fn font_style() -> Component<crate::ambient_core::text::types::FontStyle> {
                    *FONT_STYLE
                }
                static TEXT: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::text::text"));
                #[doc = "**Text**: Create a text mesh on this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn text() -> Component<String> {
                    *TEXT
                }
            }
            #[doc = r" Auto-generated type definitions."]
            pub mod types {
                use crate::{global::serde, message::*};
                #[derive(
                    Copy,
                    Clone,
                    Debug,
                    PartialEq,
                    Eq,
                    serde :: Serialize,
                    serde :: Deserialize,
                    Default,
                )]
                #[serde(crate = "self::serde")]
                #[doc = "**FontStyle**: Style of the font."]
                pub enum FontStyle {
                    #[default]
                    #[doc = "Use bold for this text."]
                    Bold,
                    #[doc = "Use bold italic for this text."]
                    BoldItalic,
                    #[doc = "Use medium for this text."]
                    Medium,
                    #[doc = "Use medium italic for this text."]
                    MediumItalic,
                    #[doc = "Use regular for this text."]
                    Regular,
                    #[doc = "Use italic for this text."]
                    Italic,
                    #[doc = "Use light for this text."]
                    Light,
                    #[doc = "Use light italic for this text."]
                    LightItalic,
                }
                impl crate::ecs::EnumComponent for FontStyle {
                    fn to_u32(&self) -> u32 {
                        match self {
                            Self::Bold => FontStyle::Bold as u32,
                            Self::BoldItalic => FontStyle::BoldItalic as u32,
                            Self::Medium => FontStyle::Medium as u32,
                            Self::MediumItalic => FontStyle::MediumItalic as u32,
                            Self::Regular => FontStyle::Regular as u32,
                            Self::Italic => FontStyle::Italic as u32,
                            Self::Light => FontStyle::Light as u32,
                            Self::LightItalic => FontStyle::LightItalic as u32,
                        }
                    }
                    fn from_u32(value: u32) -> Option<Self> {
                        if value == FontStyle::Bold as u32 {
                            return Some(Self::Bold);
                        }
                        if value == FontStyle::BoldItalic as u32 {
                            return Some(Self::BoldItalic);
                        }
                        if value == FontStyle::Medium as u32 {
                            return Some(Self::Medium);
                        }
                        if value == FontStyle::MediumItalic as u32 {
                            return Some(Self::MediumItalic);
                        }
                        if value == FontStyle::Regular as u32 {
                            return Some(Self::Regular);
                        }
                        if value == FontStyle::Italic as u32 {
                            return Some(Self::Italic);
                        }
                        if value == FontStyle::Light as u32 {
                            return Some(Self::Light);
                        }
                        if value == FontStyle::LightItalic as u32 {
                            return Some(Self::LightItalic);
                        }
                        None
                    }
                }
                impl crate::ecs::SupportedValue for FontStyle {
                    fn from_result(result: crate::ecs::WitComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_result(result).and_then(Self::from_u32)
                    }
                    fn into_result(self) -> crate::ecs::WitComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_result()
                    }
                    fn from_value(value: crate::ecs::ComponentValue) -> Option<Self> {
                        use crate::ecs::EnumComponent;
                        u32::from_value(value).and_then(Self::from_u32)
                    }
                    fn into_value(self) -> crate::ecs::ComponentValue {
                        use crate::ecs::EnumComponent;
                        self.to_u32().into_value()
                    }
                }
                impl MessageSerde for FontStyle {
                    fn serialize_message_part(
                        &self,
                        output: &mut Vec<u8>,
                    ) -> Result<(), MessageSerdeError> {
                        crate::ecs::EnumComponent::to_u32(self).serialize_message_part(output)
                    }
                    fn deserialize_message_part(
                        input: &mut dyn std::io::Read,
                    ) -> Result<Self, MessageSerdeError> {
                        crate::ecs::EnumComponent::from_u32(u32::deserialize_message_part(input)?)
                            .ok_or(MessageSerdeError::InvalidValue)
                    }
                }
            }
        }
        pub mod transform {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static CYLINDRICAL_BILLBOARD_Z: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::cylindrical_billboard_z")
                });
                #[doc = "**Cylindrical billboard Z**: If attached, this ensures this entity is always aligned with the camera, except on the Z-axis.\n\nThis is useful for decorations that the player will be looking at from roughly the same altitude.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn cylindrical_billboard_z() -> Component<()> {
                    *CYLINDRICAL_BILLBOARD_Z
                }
                static EULER_ROTATION: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::euler_rotation")
                });
                #[doc = "**Euler rotation**: The Euler rotation of this entity in ZYX order.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn euler_rotation() -> Component<Vec3> {
                    *EULER_ROTATION
                }
                static INV_LOCAL_TO_WORLD: Lazy<Component<Mat4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::inv_local_to_world")
                });
                #[doc = "**Inverse Local to World**: Converts a world position to a local position.\n\nThis is automatically updated.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn inv_local_to_world() -> Component<Mat4> {
                    *INV_LOCAL_TO_WORLD
                }
                static LOCAL_TO_PARENT: Lazy<Component<Mat4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::local_to_parent")
                });
                #[doc = "**Local to Parent**: Transformation from the entity's local space to the parent's space.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"]
                pub fn local_to_parent() -> Component<Mat4> {
                    *LOCAL_TO_PARENT
                }
                static LOCAL_TO_WORLD: Lazy<Component<Mat4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::local_to_world")
                });
                #[doc = "**Local to World**: Transformation from the entity's local space to worldspace.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn local_to_world() -> Component<Mat4> {
                    *LOCAL_TO_WORLD
                }
                static LOOKAT_TARGET: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::lookat_target")
                });
                #[doc = "**Look-at target**: The position that this entity should be looking at.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn lookat_target() -> Component<Vec3> {
                    *LOOKAT_TARGET
                }
                static LOOKAT_UP: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::transform::lookat_up"));
                #[doc = "**Look-at up**: When combined with `lookat_target`, the up vector for this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn lookat_up() -> Component<Vec3> {
                    *LOOKAT_UP
                }
                static MESH_TO_LOCAL: Lazy<Component<Mat4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::mesh_to_local")
                });
                #[doc = "**Mesh to Local**: Transformation from mesh-space to the entity's local space.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn mesh_to_local() -> Component<Mat4> {
                    *MESH_TO_LOCAL
                }
                static MESH_TO_WORLD: Lazy<Component<Mat4>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::mesh_to_world")
                });
                #[doc = "**Mesh to World**: Transformation from mesh-space to world space.\n\nThis is automatically updated when `mesh_to_local` and `local_to_world` change.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn mesh_to_world() -> Component<Mat4> {
                    *MESH_TO_WORLD
                }
                static RESET_SCALE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::transform::reset_scale"));
                #[doc = "**Reset scale**: If attached to a transform hierarchy, the scale will be reset at that point, with only rotation/translation considered.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn reset_scale() -> Component<()> {
                    *RESET_SCALE
                }
                static ROTATION: Lazy<Component<Quat>> =
                    Lazy::new(|| __internal_get_component("ambient_core::transform::rotation"));
                #[doc = "**Rotation**: The rotation of this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn rotation() -> Component<Quat> {
                    *ROTATION
                }
                static SCALE: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::transform::scale"));
                #[doc = "**Scale**: The scale of this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn scale() -> Component<Vec3> {
                    *SCALE
                }
                static SPHERICAL_BILLBOARD: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::transform::spherical_billboard")
                });
                #[doc = "**Spherical billboard**: If attached, this ensures that this entity is always aligned with the camera.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn spherical_billboard() -> Component<()> {
                    *SPHERICAL_BILLBOARD
                }
                static TRANSLATION: Lazy<Component<Vec3>> =
                    Lazy::new(|| __internal_get_component("ambient_core::transform::translation"));
                #[doc = "**Translation**: The translation/position of this entity.\n\n*Attributes*: Debuggable, Networked, Store"]
                pub fn translation() -> Component<Vec3> {
                    *TRANSLATION
                }
            }
            #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
            #[doc = r""]
            #[doc = r" They do not have any runtime representation outside of the components that compose them."]
            pub mod concepts {
                use crate::prelude::*;
                #[doc = "**Transformable**: Can be translated, rotated and scaled."]
                #[derive(Clone, Debug)]
                pub struct Transformable {
                    #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                    pub local_to_world: Mat4,
                    #[doc = r" Optional components."]
                    pub optional: TransformableOptional,
                }
                #[doc = "Optional part of [Transformable]."]
                #[derive(Clone, Debug, Default)]
                pub struct TransformableOptional {
                    #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                    pub translation: Option<Vec3>,
                    #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                    pub rotation: Option<Quat>,
                    #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                    pub scale: Option<Vec3>,
                }
                impl crate::ecs::Concept for Transformable {
                    fn make(self) -> Entity {
                        let mut entity = Entity::new().with(
                            crate::ambient_core::transform::components::local_to_world(),
                            self.local_to_world,
                        );
                        if let Some(translation) = self.optional.translation {
                            entity.set(
                                crate::ambient_core::transform::components::translation(),
                                translation,
                            );
                        }
                        if let Some(rotation) = self.optional.rotation {
                            entity.set(
                                crate::ambient_core::transform::components::rotation(),
                                rotation,
                            );
                        }
                        if let Some(scale) = self.optional.scale {
                            entity.set(crate::ambient_core::transform::components::scale(), scale);
                        }
                        entity
                    }
                    fn get_spawned(id: EntityId) -> Option<Self> {
                        Some(Self {
                            local_to_world: entity::get_component(
                                id,
                                crate::ambient_core::transform::components::local_to_world(),
                            )?,
                            optional: TransformableOptional {
                                translation: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::translation(),
                                ),
                                rotation: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::rotation(),
                                ),
                                scale: entity::get_component(
                                    id,
                                    crate::ambient_core::transform::components::scale(),
                                ),
                            },
                        })
                    }
                    fn get_unspawned(entity: &Entity) -> Option<Self> {
                        Some(Self {
                            local_to_world: entity
                                .get(crate::ambient_core::transform::components::local_to_world())?,
                            optional: TransformableOptional {
                                translation: entity
                                    .get(crate::ambient_core::transform::components::translation()),
                                rotation: entity
                                    .get(crate::ambient_core::transform::components::rotation()),
                                scale: entity
                                    .get(crate::ambient_core::transform::components::scale()),
                            },
                        })
                    }
                    fn contained_by_spawned(id: EntityId) -> bool {
                        entity::has_components(
                            id,
                            &[&crate::ambient_core::transform::components::local_to_world()],
                        )
                    }
                    fn contained_by_unspawned(entity: &Entity) -> bool {
                        entity.has_components(&[
                            &crate::ambient_core::transform::components::local_to_world(),
                        ])
                    }
                }
                impl crate::ecs::ConceptSuggested for Transformable {
                    fn suggested() -> Self {
                        Self {
                            local_to_world: Mat4::from_cols_array(&[
                                1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
                                0f32, 0f32, 0f32, 0f32, 1f32,
                            ]),
                            optional: Default::default(),
                        }
                    }
                }
            }
        }
        pub mod ui {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static FOCUS: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::ui::focus"));
                #[doc = "**Focus**: Currently focused object.\n\n*Attributes*: Debuggable, Networked, Resource"]
                pub fn focus() -> Component<String> {
                    *FOCUS
                }
                static FOCUSABLE: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::ui::focusable"));
                #[doc = "**Focus**: This entity can be focused. The value is the focus id.\n\n*Attributes*: Debuggable, Networked"]
                pub fn focusable() -> Component<String> {
                    *FOCUSABLE
                }
            }
            #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
            #[doc = r" and with other modules."]
            pub mod messages {
                use crate::{
                    message::{
                        Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                    },
                    prelude::*,
                };
                #[derive(Clone, Debug)]
                #[doc = "**FocusChanged**: Focus has been updated"]
                pub struct FocusChanged {
                    pub from_external: bool,
                    pub focus: String,
                }
                impl FocusChanged {
                    #[allow(clippy::too_many_arguments)]
                    pub fn new(from_external: impl Into<bool>, focus: impl Into<String>) -> Self {
                        Self {
                            from_external: from_external.into(),
                            focus: focus.into(),
                        }
                    }
                }
                impl Message for FocusChanged {
                    fn id() -> &'static str {
                        "FocusChanged"
                    }
                    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                        let mut output = vec![];
                        self.from_external.serialize_message_part(&mut output)?;
                        self.focus.serialize_message_part(&mut output)?;
                        Ok(output)
                    }
                    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                        Ok(Self {
                            from_external: bool::deserialize_message_part(&mut input)?,
                            focus: String::deserialize_message_part(&mut input)?,
                        })
                    }
                }
                impl ModuleMessage for FocusChanged {}
            }
        }
        pub mod wasm {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use crate::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_MODULE: Lazy<Component<()>> =
                    Lazy::new(|| __internal_get_component("ambient_core::wasm::is_module"));
                #[doc = "**Is module**: A module.\n\n*Attributes*: Networked, Store, Debuggable"]
                pub fn is_module() -> Component<()> {
                    *IS_MODULE
                }
                static IS_MODULE_ON_SERVER: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component("ambient_core::wasm::is_module_on_server")
                });
                #[doc = "**Is module on server**: Whether or not this module is on the server.\n\n*Attributes*: Networked, Store, Debuggable"]
                pub fn is_module_on_server() -> Component<()> {
                    *IS_MODULE_ON_SERVER
                }
                static BYTECODE_FROM_URL: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::wasm::bytecode_from_url"));
                #[doc = "**Bytecode from URL**: Asset URL for the bytecode of a WASM component.\n\n*Attributes*: Networked, Store, Debuggable"]
                pub fn bytecode_from_url() -> Component<String> {
                    *BYTECODE_FROM_URL
                }
                static MODULE_ENABLED: Lazy<Component<bool>> =
                    Lazy::new(|| __internal_get_component("ambient_core::wasm::module_enabled"));
                #[doc = "**Module enabled**: Whether or not this module is enabled.\n\n*Attributes*: Networked, Store, Debuggable"]
                pub fn module_enabled() -> Component<bool> {
                    *MODULE_ENABLED
                }
                static MODULE_NAME: Lazy<Component<String>> =
                    Lazy::new(|| __internal_get_component("ambient_core::wasm::module_name"));
                #[doc = "**Module name**: The name of this module.\n\n*Attributes*: Networked, Store, Debuggable"]
                pub fn module_name() -> Component<String> {
                    *MODULE_NAME
                }
                static PACKAGE_REF: Lazy<Component<EntityId>> =
                    Lazy::new(|| __internal_get_component("ambient_core::wasm::package_ref"));
                #[doc = "**Package reference**: The package that this module belongs to.\n\n*Attributes*: Networked, Store, Debuggable"]
                pub fn package_ref() -> Component<EntityId> {
                    *PACKAGE_REF
                }
            }
        }
        #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
        #[doc = r" and with other modules."]
        pub mod messages {
            use crate::{
                message::{
                    Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                },
                prelude::*,
            };
            #[derive(Clone, Debug)]
            #[doc = "**Frame**: Sent to all modules every frame."]
            pub struct Frame;
            impl Frame {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for Frame {
                fn id() -> &'static str {
                    "Frame"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for Frame {}
            impl Default for Frame {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**Collision**: Sent when a collision occurs."]
            pub struct Collision {
                pub ids: Vec<EntityId>,
            }
            impl Collision {
                #[allow(clippy::too_many_arguments)]
                pub fn new(ids: impl Into<Vec<EntityId>>) -> Self {
                    Self { ids: ids.into() }
                }
            }
            impl Message for Collision {
                fn id() -> &'static str {
                    "Collision"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.ids.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        ids: Vec::<EntityId>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for Collision {}
            #[derive(Clone, Debug)]
            #[doc = "**ColliderLoads**: Sent when colliders load."]
            pub struct ColliderLoads {
                pub ids: Vec<EntityId>,
            }
            impl ColliderLoads {
                #[allow(clippy::too_many_arguments)]
                pub fn new(ids: impl Into<Vec<EntityId>>) -> Self {
                    Self { ids: ids.into() }
                }
            }
            impl Message for ColliderLoads {
                fn id() -> &'static str {
                    "ColliderLoads"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.ids.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        ids: Vec::<EntityId>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for ColliderLoads {}
            #[derive(Clone, Debug)]
            #[doc = "**ModuleLoad**: Sent to a module when it loads."]
            pub struct ModuleLoad;
            impl ModuleLoad {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for ModuleLoad {
                fn id() -> &'static str {
                    "ModuleLoad"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for ModuleLoad {}
            impl Default for ModuleLoad {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**ModuleUnload**: Sent to a module when it unloads."]
            pub struct ModuleUnload;
            impl ModuleUnload {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for ModuleUnload {
                fn id() -> &'static str {
                    "ModuleUnload"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for ModuleUnload {}
            impl Default for ModuleUnload {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**WindowFocusChange**: Sent when the window gains or loses focus."]
            pub struct WindowFocusChange {
                pub focused: bool,
            }
            impl WindowFocusChange {
                #[allow(clippy::too_many_arguments)]
                pub fn new(focused: impl Into<bool>) -> Self {
                    Self {
                        focused: focused.into(),
                    }
                }
            }
            impl Message for WindowFocusChange {
                fn id() -> &'static str {
                    "WindowFocusChange"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.focused.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        focused: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowFocusChange {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowClose**: Sent when the window is closed."]
            pub struct WindowClose;
            impl WindowClose {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for WindowClose {
                fn id() -> &'static str {
                    "WindowClose"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl RuntimeMessage for WindowClose {}
            impl Default for WindowClose {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**WindowKeyboardCharacter**: Sent when the window receives a character from the keyboard."]
            pub struct WindowKeyboardCharacter {
                pub character: String,
            }
            impl WindowKeyboardCharacter {
                #[allow(clippy::too_many_arguments)]
                pub fn new(character: impl Into<String>) -> Self {
                    Self {
                        character: character.into(),
                    }
                }
            }
            impl Message for WindowKeyboardCharacter {
                fn id() -> &'static str {
                    "WindowKeyboardCharacter"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.character.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        character: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowKeyboardCharacter {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowKeyboardModifiersChange**: Sent when the window's keyboard modifiers change."]
            pub struct WindowKeyboardModifiersChange {
                pub modifiers: u32,
            }
            impl WindowKeyboardModifiersChange {
                #[allow(clippy::too_many_arguments)]
                pub fn new(modifiers: impl Into<u32>) -> Self {
                    Self {
                        modifiers: modifiers.into(),
                    }
                }
            }
            impl Message for WindowKeyboardModifiersChange {
                fn id() -> &'static str {
                    "WindowKeyboardModifiersChange"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.modifiers.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        modifiers: u32::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowKeyboardModifiersChange {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowKeyboardInput**: Sent when the window receives a keyboard input."]
            pub struct WindowKeyboardInput {
                pub pressed: bool,
                pub modifiers: u32,
                pub keycode: Option<String>,
            }
            impl WindowKeyboardInput {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    pressed: impl Into<bool>,
                    modifiers: impl Into<u32>,
                    keycode: impl Into<Option<String>>,
                ) -> Self {
                    Self {
                        pressed: pressed.into(),
                        modifiers: modifiers.into(),
                        keycode: keycode.into(),
                    }
                }
            }
            impl Message for WindowKeyboardInput {
                fn id() -> &'static str {
                    "WindowKeyboardInput"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.pressed.serialize_message_part(&mut output)?;
                    self.modifiers.serialize_message_part(&mut output)?;
                    self.keycode.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        pressed: bool::deserialize_message_part(&mut input)?,
                        modifiers: u32::deserialize_message_part(&mut input)?,
                        keycode: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowKeyboardInput {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowMouseInput**: Sent when the window receives a mouse input."]
            pub struct WindowMouseInput {
                pub pressed: bool,
                pub button: u32,
            }
            impl WindowMouseInput {
                #[allow(clippy::too_many_arguments)]
                pub fn new(pressed: impl Into<bool>, button: impl Into<u32>) -> Self {
                    Self {
                        pressed: pressed.into(),
                        button: button.into(),
                    }
                }
            }
            impl Message for WindowMouseInput {
                fn id() -> &'static str {
                    "WindowMouseInput"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.pressed.serialize_message_part(&mut output)?;
                    self.button.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        pressed: bool::deserialize_message_part(&mut input)?,
                        button: u32::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowMouseInput {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowMouseWheel**: Sent when the window receives a mouse wheel input."]
            pub struct WindowMouseWheel {
                pub delta: Vec2,
                pub pixels: bool,
            }
            impl WindowMouseWheel {
                #[allow(clippy::too_many_arguments)]
                pub fn new(delta: impl Into<Vec2>, pixels: impl Into<bool>) -> Self {
                    Self {
                        delta: delta.into(),
                        pixels: pixels.into(),
                    }
                }
            }
            impl Message for WindowMouseWheel {
                fn id() -> &'static str {
                    "WindowMouseWheel"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.delta.serialize_message_part(&mut output)?;
                    self.pixels.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        delta: Vec2::deserialize_message_part(&mut input)?,
                        pixels: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowMouseWheel {}
            #[derive(Clone, Debug)]
            #[doc = "**WindowMouseMotion**: Sent when the window receives a mouse motion input."]
            pub struct WindowMouseMotion {
                pub delta: Vec2,
            }
            impl WindowMouseMotion {
                #[allow(clippy::too_many_arguments)]
                pub fn new(delta: impl Into<Vec2>) -> Self {
                    Self {
                        delta: delta.into(),
                    }
                }
            }
            impl Message for WindowMouseMotion {
                fn id() -> &'static str {
                    "WindowMouseMotion"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.delta.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        delta: Vec2::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WindowMouseMotion {}
            #[derive(Clone, Debug)]
            #[doc = "**HttpResponse**: Sent when an HTTP response is received."]
            pub struct HttpResponse {
                pub url: String,
                pub status: u32,
                pub body: Vec<u8>,
                pub error: Option<String>,
            }
            impl HttpResponse {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    url: impl Into<String>,
                    status: impl Into<u32>,
                    body: impl Into<Vec<u8>>,
                    error: impl Into<Option<String>>,
                ) -> Self {
                    Self {
                        url: url.into(),
                        status: status.into(),
                        body: body.into(),
                        error: error.into(),
                    }
                }
            }
            impl Message for HttpResponse {
                fn id() -> &'static str {
                    "HttpResponse"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.url.serialize_message_part(&mut output)?;
                    self.status.serialize_message_part(&mut output)?;
                    self.body.serialize_message_part(&mut output)?;
                    self.error.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        url: String::deserialize_message_part(&mut input)?,
                        status: u32::deserialize_message_part(&mut input)?,
                        body: Vec::<u8>::deserialize_message_part(&mut input)?,
                        error: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for HttpResponse {}
            #[derive(Clone, Debug)]
            #[doc = "**WasmRebuild**: Sent when a request for WASM rebuilding is completed."]
            pub struct WasmRebuild {
                pub error: Option<String>,
            }
            impl WasmRebuild {
                #[allow(clippy::too_many_arguments)]
                pub fn new(error: impl Into<Option<String>>) -> Self {
                    Self {
                        error: error.into(),
                    }
                }
            }
            impl Message for WasmRebuild {
                fn id() -> &'static str {
                    "WasmRebuild"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.error.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        error: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl RuntimeMessage for WasmRebuild {}
        }
    }
}
