#![allow(missing_docs)]
#[doc = r" Auto-generated component definitions. These come from `ambient.toml` in the root of the project."]
pub mod components {
    #[allow(unused)]
    pub mod core {
        #[allow(unused)]
        pub mod animation {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static ANIMATION_PLAYER: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/animation-player")
            });
            #[doc = "**Animation player**: This entity is treated as an animation player. Attach an animation node as a child for it to play.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn animation_player() -> Component<()> {
                *ANIMATION_PLAYER
            }
            static ANIMATION_ERRORS: Lazy<Component<Vec<String>>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/animation-errors")
            });
            #[doc = "**Animation errors**: A list of errors that were produced trying to play the animation.\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn animation_errors() -> Component<Vec<String>> {
                *ANIMATION_ERRORS
            }
            static APPLY_ANIMATION_PLAYER: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/apply-animation-player")
            });
            #[doc = "**Apply animation player**: Apply the designated animation player to this entity and its sub-tree.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn apply_animation_player() -> Component<EntityId> {
                *APPLY_ANIMATION_PLAYER
            }
            static PLAY_CLIP_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/play-clip-from-url")
            });
            #[doc = "**Play clip from URL**: Make this entity a 'play animation clip' node. The value is the URL to the clip we'd like to play.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn play_clip_from_url() -> Component<String> {
                *PLAY_CLIP_FROM_URL
            }
            static LOOPING: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/animation/looping"));
            #[doc = "**Looping**: When this is true, the animation clip will repeat infinitely.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn looping() -> Component<bool> {
                *LOOPING
            }
            static SPEED: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/animation/speed"));
            #[doc = "**Speed**: Animation playback speed. Default is 1, higher values speeds up the animation.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn speed() -> Component<f32> {
                *SPEED
            }
            static START_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/start-time")
            });
            #[doc = "**Start time**: Start time of an animation node.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn start_time() -> Component<Duration> {
                *START_TIME
            }
            static FREEZE_AT_PERCENTAGE: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/freeze-at-percentage")
            });
            #[doc = "**Freeze at percentage**: Sample the input animation at a certain percentage of the animation track length.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn freeze_at_percentage() -> Component<f32> {
                *FREEZE_AT_PERCENTAGE
            }
            static FREEZE_AT_TIME: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/freeze-at-time")
            });
            #[doc = "**Freeze at time**: Sample the input animation at a certain time (in seconds).\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn freeze_at_time() -> Component<f32> {
                *FREEZE_AT_TIME
            }
            static CLIP_DURATION: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/clip-duration")
            });
            #[doc = "**Clip duration**: The clip duration is loaded from the clip, and then applied to the entity.\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn clip_duration() -> Component<f32> {
                *CLIP_DURATION
            }
            static BLEND: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/animation/blend"));
            #[doc = "**Blend**: Blend two animations together. The values is the blend weight. Use `children` to set the animations. Blend 0 means we only sample from the first animation, 1 means only the second one, and values in between blend between them.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn blend() -> Component<f32> {
                *BLEND
            }
            static MASK_BIND_IDS: Lazy<Component<Vec<String>>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/mask-bind-ids")
            });
            #[doc = "**Mask bind ids**: List of bind ids that will be masked.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn mask_bind_ids() -> Component<Vec<String>> {
                *MASK_BIND_IDS
            }
            static MASK_WEIGHTS: Lazy<Component<Vec<f32>>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/mask-weights")
            });
            #[doc = "**Mask weights**: Weights for each bind id in `mask-bind-ids`.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn mask_weights() -> Component<Vec<f32>> {
                *MASK_WEIGHTS
            }
            static RETARGET_MODEL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/retarget-model-from-url")
            });
            #[doc = "**Retarget Model from URL**: Retarget the animation using the model at the given URL.\n\n*Attributes*: debuggable, networked, store"]
            pub fn retarget_model_from_url() -> Component<String> {
                *RETARGET_MODEL_FROM_URL
            }
            static RETARGET_ANIMATION_SCALED: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component(
                    "component:ambient/core/animation/retarget-animation-scaled",
                )
            });
            #[doc = "**Retarget animation scaled**: Retarget animation scaled. True means normalize hip.\n\n*Attributes*: debuggable, networked, store"]
            pub fn retarget_animation_scaled() -> Component<bool> {
                *RETARGET_ANIMATION_SCALED
            }
            static APPLY_BASE_POSE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/animation/apply-base-pose")
            });
            #[doc = "**Apply base pose**: Apply the base pose to this clip.\n\n*Attributes*: debuggable, networked, store"]
            pub fn apply_base_pose() -> Component<()> {
                *APPLY_BASE_POSE
            }
            static BIND_ID: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/animation/bind-id"));
            #[doc = "**Bind id**: Animation bind ID.\n\n*Attributes*: debuggable, networked, store"]
            pub fn bind_id() -> Component<String> {
                *BIND_ID
            }
            static BIND_IDS: Lazy<Component<Vec<String>>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/animation/bind-ids"));
            #[doc = "**Bind ids**: Animation bind IDs.\n\n*Attributes*: debuggable, store"]
            pub fn bind_ids() -> Component<Vec<String>> {
                *BIND_IDS
            }
        }
        #[allow(unused)]
        pub mod app {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static CURSOR_POSITION: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/app/cursor-position")
            });
            #[doc = "**Cursor position**: Absolute mouse cursor position in screen-space. This is the -logical- position. Multiply by the `window-scale-factor` to get the physical position.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn cursor_position() -> Component<Vec2> {
                *CURSOR_POSITION
            }
            static DELTA_TIME: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/delta-time"));
            #[doc = "**Delta time**: How long the previous tick took in seconds.\n\n*Attributes*: debuggable, resource"]
            pub fn delta_time() -> Component<f32> {
                *DELTA_TIME
            }
            static EPOCH_TIME: Lazy<Component<Duration>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/epoch-time"));
            #[doc = "**Epoch time**: Time since epoch (Jan 1, 1970). Non-monotonic.\n\n*Attributes*: debuggable, resource"]
            pub fn epoch_time() -> Component<Duration> {
                *EPOCH_TIME
            }
            static GAME_TIME: Lazy<Component<Duration>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/game-time"));
            #[doc = "**Game time**: Time since the game was started. Monotonic.\n\n*Attributes*: debuggable, resource"]
            pub fn game_time() -> Component<Duration> {
                *GAME_TIME
            }
            static ELEMENT: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/element"));
            #[doc = "**Element**: The identifier of the `Element` that controls this entity.\n\nThis is automatically generated by `ElementTree`.\n\n*Attributes*: debuggable, networked"]
            pub fn element() -> Component<String> {
                *ELEMENT
            }
            static ELEMENT_UNMANAGED_CHILDREN: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/app/element-unmanaged-children")
            });
            #[doc = "**Element unmanaged children**: If this is set, the user is expected to manage the children of the `Element` themselves.\n\n*Attributes*: debuggable, networked"]
            pub fn element_unmanaged_children() -> Component<()> {
                *ELEMENT_UNMANAGED_CHILDREN
            }
            static MAIN_SCENE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/main-scene"));
            #[doc = "**Main scene**: If attached, this entity belongs to the main scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn main_scene() -> Component<()> {
                *MAIN_SCENE
            }
            static MAP_SEED: Lazy<Component<u64>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/map-seed"));
            #[doc = "**Map seed**: A random number seed for this map.\n\n*Attributes*: debuggable, networked, store"]
            pub fn map_seed() -> Component<u64> {
                *MAP_SEED
            }
            static NAME: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/name"));
            #[doc = "**Name**: A human-friendly name for this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn name() -> Component<String> {
                *NAME
            }
            static DESCRIPTION: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/description"));
            #[doc = "**Description**: A human-friendly description for this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn description() -> Component<String> {
                *DESCRIPTION
            }
            static PROJECT_NAME: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/project-name"));
            #[doc = "**Project Name**: The name of the project, from the manifest.\n\nDefaults to \"Ambient\".\n\n*Attributes*: debuggable, resource"]
            pub fn project_name() -> Component<String> {
                *PROJECT_NAME
            }
            static SELECTABLE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/selectable"));
            #[doc = "**Selectable**: If attached, this object can be selected in the editor.\n\n*Attributes*: debuggable, networked, store"]
            pub fn selectable() -> Component<()> {
                *SELECTABLE
            }
            static SNAP_TO_GROUND: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/snap-to-ground"));
            #[doc = "**Snap to ground**: This object should automatically be moved with the terrain if the terrain is changed.\n\nThe value is the offset from the terrain.\n\n*Attributes*: debuggable, networked, store"]
            pub fn snap_to_ground() -> Component<f32> {
                *SNAP_TO_GROUND
            }
            static TAGS: Lazy<Component<Vec<String>>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/tags"));
            #[doc = "**Tags**: Tags for categorizing this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn tags() -> Component<Vec<String>> {
                *TAGS
            }
            static UI_SCENE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/ui-scene"));
            #[doc = "**UI scene**: If attached, this entity belongs to the UI scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn ui_scene() -> Component<()> {
                *UI_SCENE
            }
            static WINDOW_LOGICAL_SIZE: Lazy<Component<UVec2>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/app/window-logical-size")
            });
            #[doc = "**Window logical size**: The logical size is the physical size divided by the scale factor.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn window_logical_size() -> Component<UVec2> {
                *WINDOW_LOGICAL_SIZE
            }
            static WINDOW_PHYSICAL_SIZE: Lazy<Component<UVec2>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/app/window-physical-size")
            });
            #[doc = "**Window physical size**: The physical size is the actual number of pixels on the screen.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn window_physical_size() -> Component<UVec2> {
                *WINDOW_PHYSICAL_SIZE
            }
            static WINDOW_SCALE_FACTOR: Lazy<Component<f64>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/app/window-scale-factor")
            });
            #[doc = "**Window scale factor**: The DPI/pixel scale factor of the window.\n\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn window_scale_factor() -> Component<f64> {
                *WINDOW_SCALE_FACTOR
            }
            static REF_COUNT: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/app/ref-count"));
            #[doc = "**Reference count**: Ref-counted enity. If this entity doesn't have a `parent` component, and the the ref count reaches 0, it will be removed together with all its children recursively.\n\n*Attributes*: maybe-resource, debuggable, networked"]
            pub fn ref_count() -> Component<u32> {
                *REF_COUNT
            }
        }
        #[allow(unused)]
        pub mod audio {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static AUDIO_PLAYER: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/audio-player"));
            #[doc = "**Audio player**: The entity is an audio player.\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn audio_player() -> Component<()> {
                *AUDIO_PLAYER
            }
            static SPATIAL_AUDIO_PLAYER: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/audio/spatial-audio-player")
            });
            #[doc = "**Spatial audio player**: The entity is a spatial audio player.\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn spatial_audio_player() -> Component<()> {
                *SPATIAL_AUDIO_PLAYER
            }
            static SPATIAL_AUDIO_EMITTER: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/audio/spatial-audio-emitter")
            });
            #[doc = "**Spatial audio emitter**: The entity is a spatial audio emitter.\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn spatial_audio_emitter() -> Component<EntityId> {
                *SPATIAL_AUDIO_EMITTER
            }
            static SPATIAL_AUDIO_LISTENER: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/audio/spatial-audio-listener")
            });
            #[doc = "**Spatial audio listener**: The entity is a spatial audio listener.\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn spatial_audio_listener() -> Component<EntityId> {
                *SPATIAL_AUDIO_LISTENER
            }
            static LOOPING: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/looping"));
            #[doc = "**Looping**: Whether or not the audio should loop.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn looping() -> Component<bool> {
                *LOOPING
            }
            static PLAYING_SOUND: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/audio/playing-sound")
            });
            #[doc = "**Playing sound**: The entity with this comp is a playing sound.\n\nWe can attach other components to it to control the sound parameters.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn playing_sound() -> Component<()> {
                *PLAYING_SOUND
            }
            static AMPLITUDE: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/amplitude"));
            #[doc = "**Amplitude**: The amplitude of the audio.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn amplitude() -> Component<f32> {
                *AMPLITUDE
            }
            static PANNING: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/panning"));
            #[doc = "**Panning**: The panning of the audio.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn panning() -> Component<f32> {
                *PANNING
            }
            static LPF: Lazy<Component<Vec2>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/lpf"));
            #[doc = "**Low-pass filter**: Low pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn lpf() -> Component<Vec2> {
                *LPF
            }
            static HPF: Lazy<Component<Vec2>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/hpf"));
            #[doc = "**High-pass filter**: High pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn hpf() -> Component<Vec2> {
                *HPF
            }
            static AUDIO_URL: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/audio-url"));
            #[doc = "**Audio URL**: The URL of the assets.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn audio_url() -> Component<String> {
                *AUDIO_URL
            }
            static PLAY_NOW: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/play-now"));
            #[doc = "**Trigger at this frame**: The system will watch for this component and PLAY the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn play_now() -> Component<()> {
                *PLAY_NOW
            }
            static STOP_NOW: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/audio/stop-now"));
            #[doc = "**Stop at this frame**: The system will watch for this component and STOP the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: maybe-resource, debuggable"]
            pub fn stop_now() -> Component<()> {
                *STOP_NOW
            }
        }
        #[allow(unused)]
        pub mod camera {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static ACTIVE_CAMERA: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/active-camera")
            });
            #[doc = "**Active camera**: The camera with the highest `active-camera` value will be used for rendering. Cameras are also filtered by the `user-id`.\n\nIf there's no `user-id`, the camera is considered global and potentially applies to all users (if its `active-camera` value is high enough).\n\n*Attributes*: debuggable, networked, store"]
            pub fn active_camera() -> Component<f32> {
                *ACTIVE_CAMERA
            }
            static ASPECT_RATIO: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/aspect-ratio")
            });
            #[doc = "**Aspect ratio**: The aspect ratio of this camera.\n\nIf `aspect-ratio-from-window` is set, this will be automatically updated to match the window.\n\n*Attributes*: debuggable, networked, store"]
            pub fn aspect_ratio() -> Component<f32> {
                *ASPECT_RATIO
            }
            static ASPECT_RATIO_FROM_WINDOW: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/aspect-ratio-from-window")
            });
            #[doc = "**Aspect ratio from window**: If attached, the `aspect-ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window-physical-size` component.\n\n*Attributes*: debuggable, networked, store"]
            pub fn aspect_ratio_from_window() -> Component<EntityId> {
                *ASPECT_RATIO_FROM_WINDOW
            }
            static FAR: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/far"));
            #[doc = "**Far plane**: The far plane of this camera, measured in meters.\n\n*Attributes*: debuggable, networked, store"]
            pub fn far() -> Component<f32> {
                *FAR
            }
            static FOG: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/fog"));
            #[doc = "**Fog**: If attached, this camera will see/render fog.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fog() -> Component<()> {
                *FOG
            }
            static FOVY: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/fovy"));
            #[doc = "**Field of View Y**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fovy() -> Component<f32> {
                *FOVY
            }
            static NEAR: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/near"));
            #[doc = "**Near plane**: The near plane of this camera, measured in meters.\n\n*Attributes*: debuggable, networked, store"]
            pub fn near() -> Component<f32> {
                *NEAR
            }
            static ORTHOGRAPHIC: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/orthographic")
            });
            #[doc = "**Orthographic projection**: If attached, this camera will use a standard orthographic projection matrix.\n\nEnsure that the `orthographic-` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orthographic() -> Component<()> {
                *ORTHOGRAPHIC
            }
            static ORTHOGRAPHIC_BOTTOM: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/orthographic-bottom")
            });
            #[doc = "**Orthographic bottom**: The bottom bound for this `orthographic` camera.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orthographic_bottom() -> Component<f32> {
                *ORTHOGRAPHIC_BOTTOM
            }
            static ORTHOGRAPHIC_FROM_WINDOW: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/orthographic-from-window")
            });
            #[doc = "**Orthographic from window**: The bounds of this orthographic camera will be updated to match the window automatically. Should point to an entity with a `window-logical-size` component.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orthographic_from_window() -> Component<EntityId> {
                *ORTHOGRAPHIC_FROM_WINDOW
            }
            static ORTHOGRAPHIC_LEFT: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/orthographic-left")
            });
            #[doc = "**Orthographic left**: The left bound for this `orthographic` camera.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orthographic_left() -> Component<f32> {
                *ORTHOGRAPHIC_LEFT
            }
            static ORTHOGRAPHIC_RIGHT: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/orthographic-right")
            });
            #[doc = "**Orthographic right**: The right bound for this `orthographic` camera.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orthographic_right() -> Component<f32> {
                *ORTHOGRAPHIC_RIGHT
            }
            static ORTHOGRAPHIC_TOP: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/orthographic-top")
            });
            #[doc = "**Orthographic top**: The top bound for this `orthographic` camera.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orthographic_top() -> Component<f32> {
                *ORTHOGRAPHIC_TOP
            }
            static PERSPECTIVE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/perspective"));
            #[doc = "**Perspective projection**: If attached, this camera will use a standard perspective projection matrix.\n\nEnsure that `near` and `far` are set.\n\n*Attributes*: debuggable, networked, store"]
            pub fn perspective() -> Component<()> {
                *PERSPECTIVE
            }
            static PERSPECTIVE_INFINITE_REVERSE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component(
                    "component:ambient/core/camera/perspective-infinite-reverse",
                )
            });
            #[doc = "**Perspective-infinite-reverse projection**: If attached, this camera will use a perspective-infinite-reverse projection matrix.\n\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set.\n\n*Attributes*: debuggable, networked, store"]
            pub fn perspective_infinite_reverse() -> Component<()> {
                *PERSPECTIVE_INFINITE_REVERSE
            }
            static PROJECTION: Lazy<Component<Mat4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/projection"));
            #[doc = "**Projection**: The projection matrix of this camera.\n\nThis can be driven by other components, including `perspective` and `perspective-infinite-reverse`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn projection() -> Component<Mat4> {
                *PROJECTION
            }
            static PROJECTION_VIEW: Lazy<Component<Mat4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/camera/projection-view")
            });
            #[doc = "**Projection-view**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n*Attributes*: debuggable, networked, store"]
            pub fn projection_view() -> Component<Mat4> {
                *PROJECTION_VIEW
            }
            static SHADOWS_FAR: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/camera/shadows-far"));
            #[doc = "**Shadows far plane**: The far plane for the shadow camera, measured in meters.\n\n*Attributes*: debuggable, networked, store"]
            pub fn shadows_far() -> Component<f32> {
                *SHADOWS_FAR
            }
        }
        #[allow(unused)]
        pub mod ecs {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static CHILDREN: Lazy<Component<Vec<EntityId>>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/ecs/children"));
            #[doc = "**Children**: The children of this entity.\n\n*Attributes*: debuggable, networked, store, maybe-resource"]
            pub fn children() -> Component<Vec<EntityId>> {
                *CHILDREN
            }
            static DONT_DESPAWN_ON_UNLOAD: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/ecs/dont-despawn-on-unload")
            });
            #[doc = "**Don't automatically despawn on module unload**: Indicates that this entity shouldn't be despawned when the module that spawned it unloads.\n\n*Attributes*: debuggable, store"]
            pub fn dont_despawn_on_unload() -> Component<()> {
                *DONT_DESPAWN_ON_UNLOAD
            }
            static DONT_STORE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/ecs/dont-store"));
            #[doc = "**Don't store**: Indicates that this entity shouldn't be stored on disk.\n\n*Attributes*: debuggable, networked, store"]
            pub fn dont_store() -> Component<()> {
                *DONT_STORE
            }
            static ID: Lazy<Component<EntityId>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/ecs/id"));
            #[doc = "**ID**: The ID of the entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn id() -> Component<EntityId> {
                *ID
            }
            static PARENT: Lazy<Component<EntityId>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/ecs/parent"));
            #[doc = "**Parent**: The parent of this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn parent() -> Component<EntityId> {
                *PARENT
            }
        }
        #[allow(unused)]
        pub mod input {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static MOUSE_OVER: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/input/mouse-over"));
            #[doc = "**Mouse over**: The number of mouse cursors that are currently over this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn mouse_over() -> Component<u32> {
                *MOUSE_OVER
            }
            static MOUSE_PICKABLE_MAX: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/input/mouse-pickable-max")
            });
            #[doc = "**Mouse pickable max**: This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area.\n\n*Attributes*: debuggable, networked, store"]
            pub fn mouse_pickable_max() -> Component<Vec3> {
                *MOUSE_PICKABLE_MAX
            }
            static MOUSE_PICKABLE_MIN: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/input/mouse-pickable-min")
            });
            #[doc = "**Mouse pickable min**: This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area.\n\n*Attributes*: debuggable, networked, store"]
            pub fn mouse_pickable_min() -> Component<Vec3> {
                *MOUSE_PICKABLE_MIN
            }
        }
        #[allow(unused)]
        pub mod layout {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static ALIGN_HORIZONTAL: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/align-horizontal")
            });
            #[doc = "**Align horizontal**: Layout alignment: horizontal.\n\n*Attributes*: debuggable, networked, store"]
            pub fn align_horizontal() -> Component<u32> {
                *ALIGN_HORIZONTAL
            }
            static ALIGN_VERTICAL: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/align-vertical")
            });
            #[doc = "**Align vertical**: Layout alignment: vertical.\n\n*Attributes*: debuggable, networked, store"]
            pub fn align_vertical() -> Component<u32> {
                *ALIGN_VERTICAL
            }
            static DOCKING: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/docking"));
            #[doc = "**Docking**: Layout docking.\n\n*Attributes*: debuggable, networked, store"]
            pub fn docking() -> Component<u32> {
                *DOCKING
            }
            static FIT_HORIZONTAL: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/fit-horizontal")
            });
            #[doc = "**Fit horizontal**: Layout fit: horizontal.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fit_horizontal() -> Component<u32> {
                *FIT_HORIZONTAL
            }
            static FIT_VERTICAL: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/fit-vertical")
            });
            #[doc = "**Fit vertical**: Layout fit: vertical.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fit_vertical() -> Component<u32> {
                *FIT_VERTICAL
            }
            static LAYOUT: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/layout"));
            #[doc = "**Layout**: Layout.\n\n*Attributes*: debuggable, networked, store"]
            pub fn layout() -> Component<u32> {
                *LAYOUT
            }
            static ORIENTATION: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/orientation"));
            #[doc = "**Orientation**: Layout orientation.\n\n*Attributes*: debuggable, networked, store"]
            pub fn orientation() -> Component<u32> {
                *ORIENTATION
            }
            static IS_BOOK_FILE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/is-book-file")
            });
            #[doc = "**Is book file**: This is a file in a `layout-bookcase`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn is_book_file() -> Component<()> {
                *IS_BOOK_FILE
            }
            static MARGIN: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/margin"));
            #[doc = "**Margin**: Layout margin: [top, right, bottom, left].\n\n*Attributes*: debuggable, networked, store"]
            pub fn margin() -> Component<Vec4> {
                *MARGIN
            }
            static PADDING: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/padding"));
            #[doc = "**Padding**: Layout padding: [top, right, bottom, left].\n\n*Attributes*: debuggable, networked, store"]
            pub fn padding() -> Component<Vec4> {
                *PADDING
            }
            static MESH_TO_LOCAL_FROM_SIZE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/mesh-to-local-from-size")
            });
            #[doc = "**Mesh to local from size**: Update the `mesh-to-local` based on the width and height of this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn mesh_to_local_from_size() -> Component<()> {
                *MESH_TO_LOCAL_FROM_SIZE
            }
            static MIN_HEIGHT: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/min-height"));
            #[doc = "**Minimum height**: The minimum height of a UI element.\n\n*Attributes*: debuggable, networked, store"]
            pub fn min_height() -> Component<f32> {
                *MIN_HEIGHT
            }
            static MIN_WIDTH: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/min-width"));
            #[doc = "**Minimum width**: The minimum width of a UI element.\n\n*Attributes*: debuggable, networked, store"]
            pub fn min_width() -> Component<f32> {
                *MIN_WIDTH
            }
            static MAX_HEIGHT: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/max-height"));
            #[doc = "**Maximum height**: The maximum height of a UI element.\n\n*Attributes*: debuggable, networked, store"]
            pub fn max_height() -> Component<f32> {
                *MAX_HEIGHT
            }
            static MAX_WIDTH: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/max-width"));
            #[doc = "**Maximum width**: The maximum width of a UI element.\n\n*Attributes*: debuggable, networked, store"]
            pub fn max_width() -> Component<f32> {
                *MAX_WIDTH
            }
            static SCREEN: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/screen"));
            #[doc = "**Screen**: This entity will be treated as a screen. Used by the Screen ui component.\n\n*Attributes*: debuggable, networked, store"]
            pub fn screen() -> Component<()> {
                *SCREEN
            }
            static SPACE_BETWEEN_ITEMS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/layout/space-between-items")
            });
            #[doc = "**Space between items**: Space between items in a layout.\n\n*Attributes*: debuggable, networked, store"]
            pub fn space_between_items() -> Component<f32> {
                *SPACE_BETWEEN_ITEMS
            }
            static WIDTH: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/width"));
            #[doc = "**Width**: The width of a UI element.\n\n*Attributes*: debuggable, networked, store"]
            pub fn width() -> Component<f32> {
                *WIDTH
            }
            static HEIGHT: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/height"));
            #[doc = "**Height**: The height of a UI element.\n\n*Attributes*: debuggable, networked, store"]
            pub fn height() -> Component<f32> {
                *HEIGHT
            }
            static GPU_UI_SIZE: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/layout/gpu-ui-size"));
            #[doc = "**GPU UI size**: Upload the width and height of this UI element to the GPU.\n\n*Attributes*: debuggable, networked, store"]
            pub fn gpu_ui_size() -> Component<Vec4> {
                *GPU_UI_SIZE
            }
        }
        #[allow(unused)]
        pub mod model {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static MODEL_ANIMATABLE: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/model/model-animatable")
            });
            #[doc = "**Model animatable**: Controls whether this model can be animated.\n\n*Attributes*: maybe-resource, debuggable, networked, store"]
            pub fn model_animatable() -> Component<bool> {
                *MODEL_ANIMATABLE
            }
            static MODEL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/model/model-from-url")
            });
            #[doc = "**Model from URL**: Load a model from the given URL or relative path.\n\n*Attributes*: debuggable, networked, store"]
            pub fn model_from_url() -> Component<String> {
                *MODEL_FROM_URL
            }
            static MODEL_LOADED: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/model/model-loaded"));
            #[doc = "**Model loaded**: If attached, this entity has a model attached to it.\n\n*Attributes*: debuggable, networked, store"]
            pub fn model_loaded() -> Component<()> {
                *MODEL_LOADED
            }
        }
        #[allow(unused)]
        pub mod network {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static IS_REMOTE_ENTITY: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/network/is-remote-entity")
            });
            #[doc = "**Is remote entity**: If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server).\n\n*Attributes*: debuggable, networked"]
            pub fn is_remote_entity() -> Component<()> {
                *IS_REMOTE_ENTITY
            }
            static PERSISTENT_RESOURCES: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/network/persistent-resources")
            });
            #[doc = "**Persistent resources**: If attached, this entity contains global resources that are persisted to disk and synchronized to clients.\n\n*Attributes*: debuggable, networked"]
            pub fn persistent_resources() -> Component<()> {
                *PERSISTENT_RESOURCES
            }
            static SYNCED_RESOURCES: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/network/synced-resources")
            });
            #[doc = "**Synchronized resources**: If attached, this entity contains global resources that are synchronized to clients, but not persisted.\n\n*Attributes*: debuggable, networked"]
            pub fn synced_resources() -> Component<()> {
                *SYNCED_RESOURCES
            }
        }
        #[allow(unused)]
        pub mod physics {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static ANGULAR_VELOCITY: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/angular-velocity")
            });
            #[doc = "**Angular velocity**: Angular velocity (radians/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's angular velocity in the physics scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn angular_velocity() -> Component<Vec3> {
                *ANGULAR_VELOCITY
            }
            static CUBE_COLLIDER: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/cube-collider")
            });
            #[doc = "**Cube collider**: If attached, this entity will have a cube physics collider.\n\n`x, y, z` is the size of the cube.\n\n*Attributes*: debuggable, networked, store"]
            pub fn cube_collider() -> Component<Vec3> {
                *CUBE_COLLIDER
            }
            static CHARACTER_CONTROLLER_HEIGHT: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component(
                    "component:ambient/core/physics/character-controller-height",
                )
            });
            #[doc = "**Character controller height**: The height of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character-controller-radius`, it will be given a physical character collider.\n\n*Attributes*: debuggable, networked, store"]
            pub fn character_controller_height() -> Component<f32> {
                *CHARACTER_CONTROLLER_HEIGHT
            }
            static CHARACTER_CONTROLLER_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component(
                    "component:ambient/core/physics/character-controller-radius",
                )
            });
            #[doc = "**Character controller radius**: The radius of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character-controller-height`, it will be given a physical character collider.\n\n*Attributes*: debuggable, networked, store"]
            pub fn character_controller_radius() -> Component<f32> {
                *CHARACTER_CONTROLLER_RADIUS
            }
            static COLLIDER_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/collider-from-url")
            });
            #[doc = "**Collider from URL**: This entity will load its physics collider from the URL.\n\nThe value is the URL to load from.\n\n*Attributes*: debuggable, networked, store"]
            pub fn collider_from_url() -> Component<String> {
                *COLLIDER_FROM_URL
            }
            static COLLIDER_LOADED: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/collider-loaded")
            });
            #[doc = "**Collider loaded**: This component is automatically attached to an entity once the collider has been loaded (through e.g. `collider-from-url`).\n\n*Attributes*: debuggable, networked, store"]
            pub fn collider_loaded() -> Component<()> {
                *COLLIDER_LOADED
            }
            static COLLIDER_LOADS: Lazy<Component<Vec<EntityId>>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/collider-loads")
            });
            #[doc = "**Collider loads**: Contains all colliders that were loaded in this physics tick.\n\n*Attributes*: debuggable, networked, resource, store"]
            pub fn collider_loads() -> Component<Vec<EntityId>> {
                *COLLIDER_LOADS
            }
            static CONTACT_OFFSET: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/contact-offset")
            });
            #[doc = "**Contact offset**: Contact offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's contact offset for each attached shape in the physics scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn contact_offset() -> Component<f32> {
                *CONTACT_OFFSET
            }
            static DENSITY: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/physics/density"));
            #[doc = "**Density**: The density of this entity.\n\nThis is used to update the `mass` when the entity is rescaled.\n\n*Attributes*: debuggable, networked, store\n\n*Suggested Default*: F32(1.0)"]
            pub fn density() -> Component<f32> {
                *DENSITY
            }
            static DYNAMIC: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/physics/dynamic"));
            #[doc = "**Dynamic**: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n\n*Attributes*: debuggable, networked, store"]
            pub fn dynamic() -> Component<bool> {
                *DYNAMIC
            }
            static KINEMATIC: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/physics/kinematic"));
            #[doc = "**Kinematic**: If attached, and this entity is dynamic, this entity will also be kinematic (i.e. unable to be affected by other entities motion). Otherwise, it will receive forces normally.\n\n*Attributes*: debuggable, networked, store"]
            pub fn kinematic() -> Component<()> {
                *KINEMATIC
            }
            static LINEAR_VELOCITY: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/linear-velocity")
            });
            #[doc = "**Linear velocity**: Linear velocity (meters/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's linear velocity in the physics scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn linear_velocity() -> Component<Vec3> {
                *LINEAR_VELOCITY
            }
            static MAKE_PHYSICS_STATIC: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/make-physics-static")
            });
            #[doc = "**Make physics static**: All physics objects will be made static when loaded.\n\n*Attributes*: debuggable, networked, resource, store"]
            pub fn make_physics_static() -> Component<bool> {
                *MAKE_PHYSICS_STATIC
            }
            static MASS: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/physics/mass"));
            #[doc = "**Mass**: The mass of this entity, measured in kilograms.\n\n*Attributes*: debuggable, networked, store\n\n*Suggested Default*: F32(1.0)"]
            pub fn mass() -> Component<f32> {
                *MASS
            }
            static PHYSICS_CONTROLLED: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/physics-controlled")
            });
            #[doc = "**Physics controlled**: If attached, this entity will be controlled by physics.\n\nNote that this requires the entity to have a collider.\n\n*Attributes*: debuggable, networked, store"]
            pub fn physics_controlled() -> Component<()> {
                *PHYSICS_CONTROLLED
            }
            static PLANE_COLLIDER: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/plane-collider")
            });
            #[doc = "**Plane collider**: If attached, this entity will have a plane physics collider.\n\n*Attributes*: debuggable, networked, store"]
            pub fn plane_collider() -> Component<()> {
                *PLANE_COLLIDER
            }
            static REST_OFFSET: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/rest-offset")
            });
            #[doc = "**Rest offset**: Rest offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's rest offset for each attached shape in the physics scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn rest_offset() -> Component<f32> {
                *REST_OFFSET
            }
            static SPHERE_COLLIDER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/sphere-collider")
            });
            #[doc = "**Sphere collider**: If attached, this entity will have a sphere physics collider.\n\nThe value corresponds to the radius of the sphere.\n\n*Attributes*: debuggable, networked, store"]
            pub fn sphere_collider() -> Component<f32> {
                *SPHERE_COLLIDER
            }
            static UNIT_MASS: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/physics/unit-mass"));
            #[doc = "**Unit mass**: The mass of a character/unit.\n\n*Attributes*: debuggable, networked, store"]
            pub fn unit_mass() -> Component<f32> {
                *UNIT_MASS
            }
            static UNIT_VELOCITY: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/unit-velocity")
            });
            #[doc = "**Unit velocity**: The velocity of a character/unit.\n\n*Attributes*: debuggable, networked, store"]
            pub fn unit_velocity() -> Component<Vec3> {
                *UNIT_VELOCITY
            }
            static UNIT_YAW: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/physics/unit-yaw"));
            #[doc = "**Unit yaw**: The yaw of a character/unit.\n\n*Attributes*: debuggable, networked, store"]
            pub fn unit_yaw() -> Component<f32> {
                *UNIT_YAW
            }
            static VISUALIZE_COLLIDER: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/physics/visualize-collider")
            });
            #[doc = "**Visualize collider**: If attached, the collider will be rendered.\n\n*Attributes*: debuggable, networked"]
            pub fn visualize_collider() -> Component<()> {
                *VISUALIZE_COLLIDER
            }
        }
        #[allow(unused)]
        pub mod player {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static LOCAL_USER_ID: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/player/local-user-id")
            });
            #[doc = "**Local user ID**: The user ID of the local player.\n\n*Attributes*: debuggable, networked, resource, store"]
            pub fn local_user_id() -> Component<String> {
                *LOCAL_USER_ID
            }
            static PLAYER: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/player/player"));
            #[doc = "**Player**: This entity is a player.\n\nNote that this is a logical construct; a player's body may be separate from the player itself.\n\n*Attributes*: debuggable, networked, store"]
            pub fn player() -> Component<()> {
                *PLAYER
            }
            static USER_ID: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/player/user-id"));
            #[doc = "**User ID**: An identifier attached to all things owned by a user, and supplied by the user.\n\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n*Attributes*: debuggable, networked, store"]
            pub fn user_id() -> Component<String> {
                *USER_ID
            }
        }
        #[allow(unused)]
        pub mod prefab {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static PREFAB_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/prefab/prefab-from-url")
            });
            #[doc = "**Prefab from URL**: Load and attach a prefab from a URL or relative path.\n\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity.\n\n*Attributes*: debuggable, store"]
            pub fn prefab_from_url() -> Component<String> {
                *PREFAB_FROM_URL
            }
            static SPAWNED: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/prefab/spawned"));
            #[doc = "**Spawned**: If attached, this entity was built from a prefab that has finished spawning.\n\n*Attributes*: debuggable"]
            pub fn spawned() -> Component<()> {
                *SPAWNED
            }
        }
        #[allow(unused)]
        pub mod primitives {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static CUBE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/primitives/cube"));
            #[doc = "**Cube**: If attached to an entity, the entity will be converted to a cube primitive.\n\nThe cube is unit-sized (i.e. 0.5 metres out to each side).\n\n*Attributes*: debuggable, networked, store"]
            pub fn cube() -> Component<()> {
                *CUBE
            }
            static QUAD: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/primitives/quad"));
            #[doc = "**Quad**: If attached to an entity, the entity will be converted to a quad primitive.\n\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes).\n\n*Attributes*: debuggable, networked, store"]
            pub fn quad() -> Component<()> {
                *QUAD
            }
            static SPHERE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/primitives/sphere"));
            #[doc = "**Sphere**: If attached to an entity alongside the other `sphere-*` components, the entity will be converted to a sphere primitive.\n\nTo easily instantiate a unit-diameter `sphere`, consider using the `sphere` concept (e.g. `make-sphere`).\n\n*Attributes*: debuggable, networked, store"]
            pub fn sphere() -> Component<()> {
                *SPHERE
            }
            static SPHERE_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/sphere-radius")
            });
            #[doc = "**Sphere radius**: Set the radius of a `sphere` entity.\n\n*Attributes*: debuggable, networked, store\n\n*Suggested Default*: F32(0.5)"]
            pub fn sphere_radius() -> Component<f32> {
                *SPHERE_RADIUS
            }
            static SPHERE_SECTORS: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/sphere-sectors")
            });
            #[doc = "**Sphere sectors**: Set the longitudinal sectors of a `sphere` entity.\n\n*Attributes*: debuggable, networked, store\n\n*Suggested Default*: U32(36)"]
            pub fn sphere_sectors() -> Component<u32> {
                *SPHERE_SECTORS
            }
            static SPHERE_STACKS: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/sphere-stacks")
            });
            #[doc = "**Sphere stacks**: Set the latitudinal stacks of a `sphere` entity.\n\n*Attributes*: debuggable, networked, store\n\n*Suggested Default*: U32(18)"]
            pub fn sphere_stacks() -> Component<u32> {
                *SPHERE_STACKS
            }
            static TORUS: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/primitives/torus"));
            #[doc = "**Torus**: If attached to an entity alongside the other `torus-*` components, the entity will be converted to a torus primitive.\n\nTo easily instantiate a default `torus`, consider using the `torus` concept (e.g. `make-torus`).\n\n*Attributes*: debuggable, networked, store"]
            pub fn torus() -> Component<()> {
                *TORUS
            }
            static TORUS_INNER_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/torus-inner-radius")
            });
            #[doc = "**Torus inner radius**: Set the inner radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: debuggable, networked, store"]
            pub fn torus_inner_radius() -> Component<f32> {
                *TORUS_INNER_RADIUS
            }
            static TORUS_OUTER_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/torus-outer-radius")
            });
            #[doc = "**Torus outer radius**: Set the outer radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: debuggable, networked, store"]
            pub fn torus_outer_radius() -> Component<f32> {
                *TORUS_OUTER_RADIUS
            }
            static TORUS_LOOPS: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/torus-loops")
            });
            #[doc = "**Torus loops**: Set the loops of a `torus` entity, spanning XY-plane.\n\n*Attributes*: debuggable, networked, store"]
            pub fn torus_loops() -> Component<u32> {
                *TORUS_LOOPS
            }
            static TORUS_SLICES: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/torus-slices")
            });
            #[doc = "**Torus slices**: Set the slices of a `torus` entity, spanning XY-plane.\n\n*Attributes*: debuggable, networked, store"]
            pub fn torus_slices() -> Component<u32> {
                *TORUS_SLICES
            }
            static CAPSULE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/primitives/capsule"));
            #[doc = "**Capsule**: If attached to an entity alongside the other `capsule-*` components, the entity will be converted to a capsule primitive.\n\nTo easily instantiate a default `capsule`, consider using the `capsule` concept (e.g. `make-capsule`).\n\n*Attributes*: debuggable, networked, store"]
            pub fn capsule() -> Component<()> {
                *CAPSULE
            }
            static CAPSULE_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/capsule-radius")
            });
            #[doc = "**Capsule radius**: Set the radius of a `capsule` entity, spanning XY-plane.\n\n*Attributes*: debuggable, networked, store"]
            pub fn capsule_radius() -> Component<f32> {
                *CAPSULE_RADIUS
            }
            static CAPSULE_HALF_HEIGHT: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/capsule-half-height")
            });
            #[doc = "**Capsule half-height**: Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps.\n\n*Attributes*: debuggable, networked, store"]
            pub fn capsule_half_height() -> Component<f32> {
                *CAPSULE_HALF_HEIGHT
            }
            static CAPSULE_RINGS: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/capsule-rings")
            });
            #[doc = "**Capsule rings**: Set the number of sections between the caps.\n\n*Attributes*: debuggable, networked, store"]
            pub fn capsule_rings() -> Component<u32> {
                *CAPSULE_RINGS
            }
            static CAPSULE_LATITUDES: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/capsule-latitudes")
            });
            #[doc = "**Capsule latitudes**: Set the number of latitudinal sections. Should be even.\n\n*Attributes*: debuggable, networked, store"]
            pub fn capsule_latitudes() -> Component<u32> {
                *CAPSULE_LATITUDES
            }
            static CAPSULE_LONGITUDES: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/primitives/capsule-longitudes")
            });
            #[doc = "**Capsule longitudes**: Set the number of longitudinal sections.\n\n*Attributes*: debuggable, networked, store"]
            pub fn capsule_longitudes() -> Component<u32> {
                *CAPSULE_LONGITUDES
            }
        }
        #[allow(unused)]
        pub mod procedurals {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static PROCEDURAL_MESH: Lazy<Component<ProceduralMeshHandle>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/procedurals/procedural-mesh")
            });
            #[doc = "**Procedural mesh**: Attaches a procedural mesh to this entity\n\n*Attributes*: debuggable, store"]
            pub fn procedural_mesh() -> Component<ProceduralMeshHandle> {
                *PROCEDURAL_MESH
            }
            static PROCEDURAL_MATERIAL: Lazy<Component<ProceduralMaterialHandle>> =
                Lazy::new(|| {
                    __internal_get_component(
                        "component:ambient/core/procedurals/procedural-material",
                    )
                });
            #[doc = "**Procedural material**: Attaches a procedural material to this entity\n\n*Attributes*: debuggable, store"]
            pub fn procedural_material() -> Component<ProceduralMaterialHandle> {
                *PROCEDURAL_MATERIAL
            }
        }
        #[allow(unused)]
        pub mod rect {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static BACKGROUND_COLOR: Lazy<Component<Vec4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rect/background-color")
            });
            #[doc = "**Background color**: Background color of an entity with a `rect` component.\n\n*Attributes*: debuggable, networked, store"]
            pub fn background_color() -> Component<Vec4> {
                *BACKGROUND_COLOR
            }
            static BACKGROUND_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rect/background-url")
            });
            #[doc = "**Background URL**: URL to an image asset.\n\n*Attributes*: debuggable, networked, store"]
            pub fn background_url() -> Component<String> {
                *BACKGROUND_URL
            }
            static BORDER_COLOR: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rect/border-color"));
            #[doc = "**Border color**: Border color of an entity with a `rect` component.\n\n*Attributes*: debuggable, networked, store"]
            pub fn border_color() -> Component<Vec4> {
                *BORDER_COLOR
            }
            static BORDER_RADIUS: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rect/border-radius"));
            #[doc = "**Border radius**: Radius for each corner of an entity with a `rect` component.\n\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right.\n\n*Attributes*: debuggable, networked, store"]
            pub fn border_radius() -> Component<Vec4> {
                *BORDER_RADIUS
            }
            static BORDER_THICKNESS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rect/border-thickness")
            });
            #[doc = "**Border thickness**: Border thickness of an entity with a `rect` component.\n\n*Attributes*: debuggable, networked, store"]
            pub fn border_thickness() -> Component<f32> {
                *BORDER_THICKNESS
            }
            static LINE_FROM: Lazy<Component<Vec3>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rect/line-from"));
            #[doc = "**Line from**: Start point of a line.\n\n*Attributes*: debuggable, networked, store"]
            pub fn line_from() -> Component<Vec3> {
                *LINE_FROM
            }
            static LINE_TO: Lazy<Component<Vec3>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rect/line-to"));
            #[doc = "**Line to**: End point of a line.\n\n*Attributes*: debuggable, networked, store"]
            pub fn line_to() -> Component<Vec3> {
                *LINE_TO
            }
            static LINE_WIDTH: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rect/line-width"));
            #[doc = "**Line width**: Width of line.\n\n*Attributes*: debuggable, networked, store"]
            pub fn line_width() -> Component<f32> {
                *LINE_WIDTH
            }
            static RECT: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rect/rect"));
            #[doc = "**Rect**: If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders.\n\n*Attributes*: debuggable, networked, store"]
            pub fn rect() -> Component<()> {
                *RECT
            }
            static SIZE_FROM_BACKGROUND_IMAGE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rect/size-from-background-image")
            });
            #[doc = "**Size from background image**: Resize this rect based on the size of the background image.\n\n*Attributes*: debuggable, networked, store"]
            pub fn size_from_background_image() -> Component<()> {
                *SIZE_FROM_BACKGROUND_IMAGE
            }
        }
        #[allow(unused)]
        pub mod rendering {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static CAST_SHADOWS: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/cast-shadows")
            });
            #[doc = "**Cast shadows**: If attached, this entity will cast shadows.\n\n*Attributes*: debuggable, networked, store"]
            pub fn cast_shadows() -> Component<()> {
                *CAST_SHADOWS
            }
            static COLOR: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/color"));
            #[doc = "**Color**: This entity will be tinted with the specified color if the color is not black.\n\n*Attributes*: debuggable, networked, store"]
            pub fn color() -> Component<Vec4> {
                *COLOR
            }
            static DOUBLE_SIDED: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/double-sided")
            });
            #[doc = "**Double-sided**: If this is set, the entity will be rendered with double-sided rendering.\n\n*Attributes*: debuggable, networked, store"]
            pub fn double_sided() -> Component<bool> {
                *DOUBLE_SIDED
            }
            static FOG_COLOR: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/fog-color")
            });
            #[doc = "**Fog color**: The color of the fog for this `sun`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fog_color() -> Component<Vec3> {
                *FOG_COLOR
            }
            static FOG_DENSITY: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/fog-density")
            });
            #[doc = "**Fog density**: The density of the fog for this `sun`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fog_density() -> Component<f32> {
                *FOG_DENSITY
            }
            static FOG_HEIGHT_FALLOFF: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/fog-height-falloff")
            });
            #[doc = "**Fog height fall-off**: The height at which the fog will fall off (i.e. stop being visible) for this `sun`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn fog_height_falloff() -> Component<f32> {
                *FOG_HEIGHT_FALLOFF
            }
            static JOINT_MATRICES: Lazy<Component<Vec<Mat4>>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/joint-matrices")
            });
            #[doc = "**Joint Matrices**: Contains the matrices for each joint of this skinned mesh.\n\nThis should be used in combination with `joints`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn joint_matrices() -> Component<Vec<Mat4>> {
                *JOINT_MATRICES
            }
            static JOINTS: Lazy<Component<Vec<EntityId>>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/joints"));
            #[doc = "**Joints**: Contains the joints that comprise this skinned mesh.\n\n*Attributes*: debuggable, networked, store"]
            pub fn joints() -> Component<Vec<EntityId>> {
                *JOINTS
            }
            static LIGHT_AMBIENT: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/light-ambient")
            });
            #[doc = "**Light ambient**: The ambient light color of the `sun`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn light_ambient() -> Component<Vec3> {
                *LIGHT_AMBIENT
            }
            static LIGHT_DIFFUSE: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/light-diffuse")
            });
            #[doc = "**Light diffuse**: The diffuse light color of the `sun`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn light_diffuse() -> Component<Vec3> {
                *LIGHT_DIFFUSE
            }
            static OUTLINE: Lazy<Component<Vec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/outline"));
            #[doc = "**Outline**: If attached, this entity will be rendered with an outline with the color specified.\n\n*Attributes*: debuggable, networked, store"]
            pub fn outline() -> Component<Vec4> {
                *OUTLINE
            }
            static OUTLINE_RECURSIVE: Lazy<Component<Vec4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/outline-recursive")
            });
            #[doc = "**Outline (recursive)**: If attached, this entity and all of its children will be rendered with an outline with the color specified.\n\nYou do not need to attach `outline` if you have attached `outline-recursive`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn outline_recursive() -> Component<Vec4> {
                *OUTLINE_RECURSIVE
            }
            static OVERLAY: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/overlay"));
            #[doc = "**Overlay**: If attached, this entity will be rendered with an overlay.\n\n*Attributes*: debuggable, networked, store"]
            pub fn overlay() -> Component<()> {
                *OVERLAY
            }
            static PBR_MATERIAL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/pbr-material-from-url")
            });
            #[doc = "**PBR material from URL**: Load a PBR material from the URL and attach it to this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn pbr_material_from_url() -> Component<String> {
                *PBR_MATERIAL_FROM_URL
            }
            static SKY: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/sky"));
            #[doc = "**Sky**: Add a realistic skybox to the scene.\n\n*Attributes*: debuggable, networked, store"]
            pub fn sky() -> Component<()> {
                *SKY
            }
            static SUN: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/sun"));
            #[doc = "**Sun**: Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\n\nThe entity with the highest `sun` value takes precedence.\n\n*Attributes*: debuggable, networked, store"]
            pub fn sun() -> Component<f32> {
                *SUN
            }
            static TRANSPARENCY_GROUP: Lazy<Component<i32>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/transparency-group")
            });
            #[doc = "**Transparency group**: Controls when this transparent object will be rendered. Transparent objects are sorted by `(transparency-group, z-depth)`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn transparency_group() -> Component<i32> {
                *TRANSPARENCY_GROUP
            }
            static WATER: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/water"));
            #[doc = "**Water**: Add a realistic water plane to this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn water() -> Component<()> {
                *WATER
            }
            static DECAL_FROM_URL: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/decal-from-url")
            });
            #[doc = "**Decal material from URL**: Load a Decal material from the URL and attach it to this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn decal_from_url() -> Component<String> {
                *DECAL_FROM_URL
            }
            static SCISSORS: Lazy<Component<UVec4>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/rendering/scissors"));
            #[doc = "**Scissors**: Apply a scissors test to this entity (anything outside the rect will be hidden).\n\n*Attributes*: debuggable, networked, store"]
            pub fn scissors() -> Component<UVec4> {
                *SCISSORS
            }
            static SCISSORS_RECURSIVE: Lazy<Component<UVec4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/rendering/scissors-recursive")
            });
            #[doc = "**Scissors (recursive)**: If attached, this entity and all of its children will be rendered with an scissor with the rect specified.\n\nYou do not need to attach `scissors` if you have attached `scissors-recursive`.\n\n*Attributes*: debuggable, networked, store"]
            pub fn scissors_recursive() -> Component<UVec4> {
                *SCISSORS_RECURSIVE
            }
        }
        #[allow(unused)]
        pub mod text {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static FONT_FAMILY: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/text/font-family"));
            #[doc = "**Font family**: Font family to be used. Can either be 'Default', 'FontAwesome', 'FontAwesomeSolid', 'Code' or a url to a font.\n\n*Attributes*: debuggable, networked, store"]
            pub fn font_family() -> Component<String> {
                *FONT_FAMILY
            }
            static FONT_SIZE: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/text/font-size"));
            #[doc = "**Font size**: Size of the font.\n\n*Attributes*: debuggable, networked, store"]
            pub fn font_size() -> Component<f32> {
                *FONT_SIZE
            }
            static FONT_STYLE: Lazy<Component<u32>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/text/font-style"));
            #[doc = "**Font style**: Style of the font.\n\n*Attributes*: debuggable, networked, store"]
            pub fn font_style() -> Component<u32> {
                *FONT_STYLE
            }
            static TEXT: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/text/text"));
            #[doc = "**Text**: Create a text mesh on this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn text() -> Component<String> {
                *TEXT
            }
        }
        #[allow(unused)]
        pub mod transform {
            use crate::global::{
                Duration, EntityId, IVec2, IVec3, IVec4, Mat4, ProceduralMaterialHandle,
                ProceduralMeshHandle, ProceduralSamplerHandle, ProceduralTextureHandle, Quat,
                UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
            };
            use crate::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
            };
            static CYLINDRICAL_BILLBOARD_Z: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/cylindrical-billboard-z")
            });
            #[doc = "**Cylindrical billboard Z**: If attached, this ensures this entity is always aligned with the camera, except on the Z-axis.\n\nThis is useful for decorations that the player will be looking at from roughly the same altitude.\n\n*Attributes*: debuggable, networked, store"]
            pub fn cylindrical_billboard_z() -> Component<()> {
                *CYLINDRICAL_BILLBOARD_Z
            }
            static EULER_ROTATION: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/euler-rotation")
            });
            #[doc = "**Euler rotation**: The Euler rotation of this entity in ZYX order.\n\n*Attributes*: debuggable, networked, store"]
            pub fn euler_rotation() -> Component<Vec3> {
                *EULER_ROTATION
            }
            static INV_LOCAL_TO_WORLD: Lazy<Component<Mat4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/inv-local-to-world")
            });
            #[doc = "**Inverse Local to World**: Converts a world position to a local position.\n\nThis is automatically updated.\n\n*Attributes*: debuggable, networked, store"]
            pub fn inv_local_to_world() -> Component<Mat4> {
                *INV_LOCAL_TO_WORLD
            }
            static LOCAL_TO_PARENT: Lazy<Component<Mat4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/local-to-parent")
            });
            #[doc = "**Local to Parent**: Transformation from the entity's local space to the parent's space.\n\n*Attributes*: debuggable, networked, store, maybe-resource"]
            pub fn local_to_parent() -> Component<Mat4> {
                *LOCAL_TO_PARENT
            }
            static LOCAL_TO_WORLD: Lazy<Component<Mat4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/local-to-world")
            });
            #[doc = "**Local to World**: Transformation from the entity's local space to worldspace.\n\n*Attributes*: debuggable, networked, store"]
            pub fn local_to_world() -> Component<Mat4> {
                *LOCAL_TO_WORLD
            }
            static LOOKAT_TARGET: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/lookat-target")
            });
            #[doc = "**Look-at target**: The position that this entity should be looking at.\n\n*Attributes*: debuggable, networked, store"]
            pub fn lookat_target() -> Component<Vec3> {
                *LOOKAT_TARGET
            }
            static LOOKAT_UP: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/lookat-up")
            });
            #[doc = "**Look-at up**: When combined with `lookat-target`, the up vector for this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn lookat_up() -> Component<Vec3> {
                *LOOKAT_UP
            }
            static MESH_TO_LOCAL: Lazy<Component<Mat4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/mesh-to-local")
            });
            #[doc = "**Mesh to Local**: Transformation from mesh-space to the entity's local space.\n\n*Attributes*: debuggable, networked, store"]
            pub fn mesh_to_local() -> Component<Mat4> {
                *MESH_TO_LOCAL
            }
            static MESH_TO_WORLD: Lazy<Component<Mat4>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/mesh-to-world")
            });
            #[doc = "**Mesh to World**: Transformation from mesh-space to world space.\n\nThis is automatically updated when `mesh-to-local` and `local-to-world` change.\n\n*Attributes*: debuggable, networked, store"]
            pub fn mesh_to_world() -> Component<Mat4> {
                *MESH_TO_WORLD
            }
            static RESET_SCALE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/reset-scale")
            });
            #[doc = "**Reset scale**: If attached to a transform hierarchy, the scale will be reset at that point, with only rotation/translation considered.\n\n*Attributes*: debuggable, networked, store"]
            pub fn reset_scale() -> Component<()> {
                *RESET_SCALE
            }
            static ROTATION: Lazy<Component<Quat>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/transform/rotation"));
            #[doc = "**Rotation**: The rotation of this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn rotation() -> Component<Quat> {
                *ROTATION
            }
            static SCALE: Lazy<Component<Vec3>> =
                Lazy::new(|| __internal_get_component("component:ambient/core/transform/scale"));
            #[doc = "**Scale**: The scale of this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn scale() -> Component<Vec3> {
                *SCALE
            }
            static SPHERICAL_BILLBOARD: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/spherical-billboard")
            });
            #[doc = "**Spherical billboard**: If attached, this ensures that this entity is always aligned with the camera.\n\n*Attributes*: debuggable, networked, store"]
            pub fn spherical_billboard() -> Component<()> {
                *SPHERICAL_BILLBOARD
            }
            static TRANSLATION: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("component:ambient/core/transform/translation")
            });
            #[doc = "**Translation**: The translation/position of this entity.\n\n*Attributes*: debuggable, networked, store"]
            pub fn translation() -> Component<Vec3> {
                *TRANSLATION
            }
        }
    }
}
#[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
#[doc = r""]
#[doc = r" They do not have any runtime representation outside of the components that compose them."]
pub mod concepts {}
#[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
#[doc = r" and with other modules."]
pub mod messages {
    #[allow(unused)]
    pub mod core {
        use crate::{
            message::{Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage},
            prelude::*,
        };
        #[derive(Clone, Debug)]
        #[doc = "**Frame**: Sent to all modules every frame."]
        pub struct Frame {}
        impl Frame {
            pub fn new() -> Self {
                Self {}
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
        #[derive(Clone, Debug)]
        #[doc = "**Collision**: Sent when a collision occurs."]
        pub struct Collision {
            pub ids: Vec<EntityId>,
        }
        impl Collision {
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
        pub struct ModuleLoad {}
        impl ModuleLoad {
            pub fn new() -> Self {
                Self {}
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
        #[derive(Clone, Debug)]
        #[doc = "**ModuleUnload**: Sent to a module when it unloads."]
        pub struct ModuleUnload {}
        impl ModuleUnload {
            pub fn new() -> Self {
                Self {}
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
        #[derive(Clone, Debug)]
        #[doc = "**WindowFocusChange**: Sent when the window gains or loses focus."]
        pub struct WindowFocusChange {
            pub focused: bool,
        }
        impl WindowFocusChange {
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
        pub struct WindowClose {}
        impl WindowClose {
            pub fn new() -> Self {
                Self {}
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
        #[derive(Clone, Debug)]
        #[doc = "**WindowKeyboardCharacter**: Sent when the window receives a character from the keyboard."]
        pub struct WindowKeyboardCharacter {
            pub character: String,
        }
        impl WindowKeyboardCharacter {
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
    }
}
