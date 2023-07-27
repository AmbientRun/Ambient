#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused)]
use std::io::Read;

use ambient_project_rt::message_serde::{MessageSerde, MessageSerdeError};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

impl MessageSerde for crate::EntityId {
    fn serialize_message_part(&self, output: &mut Vec<u8>) -> Result<(), MessageSerdeError> {
        let (id0, id1) = self.to_u64s();
        output.write_u64::<BigEndian>(id0)?;
        output.write_u64::<BigEndian>(id1)?;
        Ok(())
    }

    fn deserialize_message_part(input: &mut dyn Read) -> Result<Self, MessageSerdeError> {
        let (id0, id1) = (
            input.read_u64::<BigEndian>()?,
            input.read_u64::<BigEndian>()?,
        );
        Ok(Self::from_u64s(id0, id1))
    }
}

#[allow(unused)]
pub mod animation {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("animation" , { # [doc = "**Animation player**: This entity is treated as an animation player. Attach an animation node as a child for it to play.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Animation player"] , Description ["This entity is treated as an animation player. Attach an animation node as a child for it to play."]] animation_player : () , # [doc = "**Animation errors**: A list of errors that were produced trying to play the animation.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Animation errors"] , Description ["A list of errors that were produced trying to play the animation."]] animation_errors : Vec :: < String > , # [doc = "**Apply animation player**: Apply the designated animation player to this entity and its sub-tree.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Apply animation player"] , Description ["Apply the designated animation player to this entity and its sub-tree."]] apply_animation_player : EntityId , # [doc = "**Play clip from URL**: Make this entity a 'play animation clip' node. The value is the URL to the clip we'd like to play.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Play clip from URL"] , Description ["Make this entity a 'play animation clip' node. The value is the URL to the clip we'd like to play."]] play_clip_from_url : String , # [doc = "**Looping**: When this is true, the animation clip will repeat infinitely.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Looping"] , Description ["When this is true, the animation clip will repeat infinitely."]] looping : bool , # [doc = "**Speed**: Animation playback speed. Default is 1, higher values speeds up the animation.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Speed"] , Description ["Animation playback speed. Default is 1, higher values speeds up the animation."]] speed : f32 , # [doc = "**Start time**: Start time of an animation node.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Start time"] , Description ["Start time of an animation node."]] start_time : Duration , # [doc = "**Freeze at percentage**: Sample the input animation at a certain percentage of the animation track length.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Freeze at percentage"] , Description ["Sample the input animation at a certain percentage of the animation track length."]] freeze_at_percentage : f32 , # [doc = "**Freeze at time**: Sample the input animation at a certain time (in seconds).\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Freeze at time"] , Description ["Sample the input animation at a certain time (in seconds)."]] freeze_at_time : f32 , # [doc = "**Clip duration**: The clip duration is loaded from the clip, and then applied to the entity.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Clip duration"] , Description ["The clip duration is loaded from the clip, and then applied to the entity."]] clip_duration : f32 , # [doc = "**Blend**: Blend two animations together. The values is the blend weight. Use `children` to set the animations. Blend 0 means we only sample from the first animation, 1 means only the second one, and values in between blend between them.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Blend"] , Description ["Blend two animations together. The values is the blend weight. Use `children` to set the animations. Blend 0 means we only sample from the first animation, 1 means only the second one, and values in between blend between them."]] blend : f32 , # [doc = "**Mask bind ids**: List of bind ids that will be masked.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Mask bind ids"] , Description ["List of bind ids that will be masked."]] mask_bind_ids : Vec :: < String > , # [doc = "**Mask weights**: Weights for each bind id in `mask_bind_ids`.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Mask weights"] , Description ["Weights for each bind id in `mask_bind_ids`."]] mask_weights : Vec :: < f32 > , # [doc = "**Retarget Model from URL**: Retarget the animation using the model at the given URL.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Retarget Model from URL"] , Description ["Retarget the animation using the model at the given URL."]] retarget_model_from_url : String , # [doc = "**Retarget animation scaled**: Retarget animation scaled. True means normalize hip.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Retarget animation scaled"] , Description ["Retarget animation scaled. True means normalize hip."]] retarget_animation_scaled : bool , # [doc = "**Apply base pose**: Apply the base pose to this clip.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Apply base pose"] , Description ["Apply the base pose to this clip."]] apply_base_pose : () , # [doc = "**Bind id**: Animation bind ID.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Bind id"] , Description ["Animation bind ID."]] bind_id : String , # [doc = "**Bind ids**: Animation bind IDs.\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Bind ids"] , Description ["Animation bind IDs."]] bind_ids : Vec :: < String > , });
    }
}
#[allow(unused)]
pub mod app {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("app" , { # [doc = "**Cursor position**: Absolute mouse cursor position in screen-space. This is the *logical* position. Multiply by the `window_scale_factor` to get the physical position.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Cursor position"] , Description ["Absolute mouse cursor position in screen-space. This is the *logical* position. Multiply by the `window_scale_factor` to get the physical position."]] cursor_position : Vec2 , # [doc = "**Delta time**: How long the previous tick took in seconds.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Delta time"] , Description ["How long the previous tick took in seconds."]] delta_time : f32 , # [doc = "**Epoch time**: Time since epoch (Jan 1, 1970). Non_monotonic.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Epoch time"] , Description ["Time since epoch (Jan 1, 1970). Non_monotonic."]] epoch_time : Duration , # [doc = "**Game time**: Time since the game was started. Monotonic.\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Game time"] , Description ["Time since the game was started. Monotonic."]] game_time : Duration , # [doc = "**Element**: The identifier of the `Element` that controls this entity.\n\nThis is automatically generated by `ElementTree`.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Element"] , Description ["The identifier of the `Element` that controls this entity.\nThis is automatically generated by `ElementTree`."]] element : String , # [doc = "**Element unmanaged children**: If this is set, the user is expected to manage the children of the `Element` themselves.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Element unmanaged children"] , Description ["If this is set, the user is expected to manage the children of the `Element` themselves."]] element_unmanaged_children : () , # [doc = "**Main scene**: If attached, this entity belongs to the main scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Main scene"] , Description ["If attached, this entity belongs to the main scene."]] main_scene : () , # [doc = "**Map seed**: A random number seed for this map.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Map seed"] , Description ["A random number seed for this map."]] map_seed : u64 , # [doc = "**Name**: A human-friendly name for this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Name"] , Description ["A human-friendly name for this entity."]] name : String , # [doc = "**Description**: A human-friendly description for this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Description"] , Description ["A human-friendly description for this entity."]] description : String , # [doc = "**Project Name**: The name of the project, from the manifest.\n\nDefaults to \"Ambient\".\n\n*Attributes*: Debuggable, Resource"] @ [Debuggable , Resource , Name ["Project Name"] , Description ["The name of the project, from the manifest.\nDefaults to \"Ambient\"."]] project_name : String , # [doc = "**Selectable**: If attached, this object can be selected in the editor.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Selectable"] , Description ["If attached, this object can be selected in the editor."]] selectable : () , # [doc = "**Snap to ground**: This object should automatically be moved with the terrain if the terrain is changed.\n\nThe value is the offset from the terrain.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Snap to ground"] , Description ["This object should automatically be moved with the terrain if the terrain is changed.\nThe value is the offset from the terrain."]] snap_to_ground : f32 , # [doc = "**Tags**: Tags for categorizing this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Tags"] , Description ["Tags for categorizing this entity."]] tags : Vec :: < String > , # [doc = "**UI scene**: If attached, this entity belongs to the UI scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["UI scene"] , Description ["If attached, this entity belongs to the UI scene."]] ui_scene : () , # [doc = "**Window logical size**: The logical size is the physical size divided by the scale factor.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Window logical size"] , Description ["The logical size is the physical size divided by the scale factor."]] window_logical_size : UVec2 , # [doc = "**Window physical size**: The physical size is the actual number of pixels on the screen.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Window physical size"] , Description ["The physical size is the actual number of pixels on the screen."]] window_physical_size : UVec2 , # [doc = "**Window scale factor**: The DPI/pixel scale factor of the window.\n\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Window scale factor"] , Description ["The DPI/pixel scale factor of the window.\nOn standard displays, this is 1, but it can be higher on high-DPI displays like Apple Retina displays."]] window_scale_factor : f64 , # [doc = "**Reference count**: Ref-counted enity. If this entity doesn't have a `parent` component, and the ref count reaches 0, it will be removed together with all its children recursively.\n\n*Attributes*: MaybeResource, Debuggable, Networked"] @ [MaybeResource , Debuggable , Networked , Name ["Reference count"] , Description ["Ref-counted enity. If this entity doesn't have a `parent` component, and the ref count reaches 0, it will be removed together with all its children recursively."]] ref_count : u32 , });
    }
}
#[allow(unused)]
pub mod audio {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("audio" , { # [doc = "**Audio player**: The entity is an audio player.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Audio player"] , Description ["The entity is an audio player."]] audio_player : () , # [doc = "**Spatial audio player**: The entity is a spatial audio player.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Spatial audio player"] , Description ["The entity is a spatial audio player."]] spatial_audio_player : () , # [doc = "**Spatial audio emitter**: The entity is a spatial audio emitter.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Spatial audio emitter"] , Description ["The entity is a spatial audio emitter."]] spatial_audio_emitter : EntityId , # [doc = "**Spatial audio listener**: The entity is a spatial audio listener.\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Spatial audio listener"] , Description ["The entity is a spatial audio listener."]] spatial_audio_listener : EntityId , # [doc = "**Looping**: Whether or not the audio should loop.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Looping"] , Description ["Whether or not the audio should loop.\n"]] looping : bool , # [doc = "**One pole low pass filter**: With this component, the audio will be filtered with a one pole low pass filter.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["One pole low pass filter"] , Description ["With this component, the audio will be filtered with a one pole low pass filter.\n"]] onepole_lpf : f32 , # [doc = "**Playing sound**: The entity with this comp is a playing sound.\n\nWe can attach other components to it to control the sound parameters.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Playing sound"] , Description ["The entity with this comp is a playing sound.\nWe can attach other components to it to control the sound parameters.\n"]] playing_sound : () , # [doc = "**Amplitude**: The amplitude of the audio.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Amplitude"] , Description ["The amplitude of the audio.\n"]] amplitude : f32 , # [doc = "**Panning**: The panning of the audio.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Panning"] , Description ["The panning of the audio.\n"]] panning : f32 , # [doc = "**Low_pass filter**: Low pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Low_pass filter"] , Description ["Low pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n"]] lpf : Vec2 , # [doc = "**High_pass filter**: High pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["High_pass filter"] , Description ["High pass filter. The first value is the cutoff frequency, the second is the bandwidth.\n"]] hpf : Vec2 , # [doc = "**Audio URL**: The URL of the assets.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Audio URL"] , Description ["The URL of the assets.\n"]] audio_url : String , # [doc = "**Trigger at this frame**: The system will watch for this component and PLAY the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Trigger at this frame"] , Description ["The system will watch for this component and PLAY the audio at this frame,\nusing the other components as parameters.\nThen set it back to false.\n"]] play_now : () , # [doc = "**Stop at this frame**: The system will watch for this component and STOP the audio at this frame,\n\nusing the other components as parameters.\n\nThen set it back to false.\n\n\n\n*Attributes*: MaybeResource, Debuggable"] @ [MaybeResource , Debuggable , Name ["Stop at this frame"] , Description ["The system will watch for this component and STOP the audio at this frame,\nusing the other components as parameters.\nThen set it back to false.\n"]] stop_now : () , });
    }
}
#[allow(unused)]
pub mod camera {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("camera" , { # [doc = "**Active camera**: The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\n\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Active camera"] , Description ["The camera with the highest `active_camera` value will be used for rendering. Cameras are also filtered by the `user_id`.\nIf there's no `user_id`, the camera is considered global and potentially applies to all users (if its `active_camera` value is high enough)."]] active_camera : f32 , # [doc = "**Aspect ratio**: The aspect ratio of this camera.\n\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Aspect ratio"] , Description ["The aspect ratio of this camera.\nIf `aspect_ratio_from_window` is set, this will be automatically updated to match the window."]] aspect_ratio : f32 , # [doc = "**Aspect ratio from window**: If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Aspect ratio from window"] , Description ["If attached, the `aspect_ratio` component will be automatically updated to match the aspect ratio of the window. Should point to an entity with a `window_physical_size` component."]] aspect_ratio_from_window : EntityId , # [doc = "**Far plane**: The far plane of this camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Far plane"] , Description ["The far plane of this camera, measured in meters."]] far : f32 , # [doc = "**Fog**: If attached, this camera will see/render fog.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog"] , Description ["If attached, this camera will see/render fog."]] fog : () , # [doc = "**Field of View Y**: The field of view of this camera in the Y/vertical direction, measured in radians.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Field of View Y"] , Description ["The field of view of this camera in the Y/vertical direction, measured in radians."]] fovy : f32 , # [doc = "**Near plane**: The near plane of this camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Near plane"] , Description ["The near plane of this camera, measured in meters."]] near : f32 , # [doc = "**Orthographic projection**: If attached, this camera will use a standard orthographic projection matrix.\n\nEnsure that the `orthographic_` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic projection"] , Description ["If attached, this camera will use a standard orthographic projection matrix.\nEnsure that the `orthographic_` components are set, including `left`, right`, `top` and `bottom`, as well as `near` and `far`."]] orthographic : () , # [doc = "**Orthographic bottom**: The bottom bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic bottom"] , Description ["The bottom bound for this `orthographic` camera."]] orthographic_bottom : f32 , # [doc = "**Orthographic from window**: The bounds of this orthographic camera will be updated to match the window automatically. Should point to an entity with a `window_logical_size` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic from window"] , Description ["The bounds of this orthographic camera will be updated to match the window automatically. Should point to an entity with a `window_logical_size` component."]] orthographic_from_window : EntityId , # [doc = "**Orthographic left**: The left bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic left"] , Description ["The left bound for this `orthographic` camera."]] orthographic_left : f32 , # [doc = "**Orthographic right**: The right bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic right"] , Description ["The right bound for this `orthographic` camera."]] orthographic_right : f32 , # [doc = "**Orthographic top**: The top bound for this `orthographic` camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orthographic top"] , Description ["The top bound for this `orthographic` camera."]] orthographic_top : f32 , # [doc = "**Perspective projection**: If attached, this camera will use a standard perspective projection matrix.\n\nEnsure that `near` and `far` are set.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Perspective projection"] , Description ["If attached, this camera will use a standard perspective projection matrix.\nEnsure that `near` and `far` are set."]] perspective : () , # [doc = "**Perspective-infinite-reverse projection**: If attached, this camera will use a perspective-infinite-reverse projection matrix.\n\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Perspective-infinite-reverse projection"] , Description ["If attached, this camera will use a perspective-infinite-reverse projection matrix.\nThis is well-suited for rendering large worlds as it has no far plane. Ensure `near` is set."]] perspective_infinite_reverse : () , # [doc = "**Projection**: The projection matrix of this camera.\n\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Projection"] , Description ["The projection matrix of this camera.\nThis can be driven by other components, including `perspective` and `perspective_infinite_reverse`."]] projection : Mat4 , # [doc = "**Projection-view**: The composition of the projection and view (inverse-local-to-world) matrices.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Projection-view"] , Description ["The composition of the projection and view (inverse-local-to-world) matrices."]] projection_view : Mat4 , # [doc = "**Shadows far plane**: The far plane for the shadow camera, measured in meters.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Shadows far plane"] , Description ["The far plane for the shadow camera, measured in meters."]] shadows_far : f32 , });
    }
    #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
    #[doc = r""]
    #[doc = r" They do not have any runtime representation outside of the components that compose them."]
    pub mod concepts {
        use crate::{Component, Entity, EntityId};
        use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Camera*.\n\nBase components for a camera. You will need other components to make a fully-functioning camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::near\": f32 = 0.1,\n  \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::camera::active_camera\": f32 = 0.0,\n  \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::transform::transformable\": { // Concept.\n    \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n    \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n    \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n  },\n}\n```\n"]
        pub fn make_camera() -> Entity {
            Entity::new()
                .with_merge(crate::generated::transform::concepts::make_transformable())
                .with(crate::generated::camera::components::near(), 0.1f32)
                .with(
                    crate::generated::camera::components::projection(),
                    Mat4::from_cols_array(&[
                        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
                        0f32, 0f32, 0f32, 1f32,
                    ]),
                )
                .with(
                    crate::generated::camera::components::projection_view(),
                    Mat4::from_cols_array(&[
                        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
                        0f32, 0f32, 0f32, 1f32,
                    ]),
                )
                .with(crate::generated::camera::components::active_camera(), 0f32)
                .with(
                    crate::generated::transform::components::local_to_world(),
                    Mat4::from_cols_array(&[
                        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
                        0f32, 0f32, 0f32, 1f32,
                    ]),
                )
                .with(
                    crate::generated::transform::components::inv_local_to_world(),
                    Mat4::from_cols_array(&[
                        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
                        0f32, 0f32, 0f32, 1f32,
                    ]),
                )
        }
        #[doc = "Checks if the entity is a *Camera*.\n\nBase components for a camera. You will need other components to make a fully-functioning camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::near\": f32 = 0.1,\n  \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::camera::active_camera\": f32 = 0.0,\n  \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::transform::transformable\": { // Concept.\n    \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n    \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n    \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n  },\n}\n```\n"]
        pub fn is_camera(world: &crate::World, id: EntityId) -> bool {
            crate::generated::transform::concepts::is_transformable(world, id)
                && world.has_components(id, &{
                    let mut set = crate::ComponentSet::new();
                    set.insert(crate::generated::camera::components::near().desc());
                    set.insert(crate::generated::camera::components::projection().desc());
                    set.insert(crate::generated::camera::components::projection_view().desc());
                    set.insert(crate::generated::camera::components::active_camera().desc());
                    set.insert(crate::generated::transform::components::local_to_world().desc());
                    set.insert(
                        crate::generated::transform::components::inv_local_to_world().desc(),
                    );
                    set
                })
        }
        #[doc = "Returns the components that comprise *Camera* as a tuple.\n\nBase components for a camera. You will need other components to make a fully-functioning camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::near\": f32 = 0.1,\n  \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::camera::active_camera\": f32 = 0.0,\n  \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n  \"ambient_core::transform::transformable\": { // Concept.\n    \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n    \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n    \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n  },\n}\n```\n"]
        pub fn camera() -> (
            Component<f32>,
            Component<Mat4>,
            Component<Mat4>,
            Component<f32>,
            Component<Mat4>,
            Component<Mat4>,
        ) {
            (
                crate::generated::camera::components::near(),
                crate::generated::camera::components::projection(),
                crate::generated::camera::components::projection_view(),
                crate::generated::camera::components::active_camera(),
                crate::generated::transform::components::local_to_world(),
                crate::generated::transform::components::inv_local_to_world(),
            )
        }
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Perspective Common Camera*.\n\nBase components for a perspective camera. Consider `perspective_camera` or `perspective_infinite_reverse_camera`.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::fovy\": f32 = 1.0,\n  \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n  \"ambient_core::camera::camera\": { // Concept.\n    \"ambient_core::camera::near\": f32 = 0.1,\n    \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::active_camera\": f32 = 0.0,\n    \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::transformable\": { // Concept.\n      \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n      \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n      \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n    },\n  },\n}\n```\n"]
        pub fn make_perspective_common_camera() -> Entity {
            Entity::new()
                .with_merge(crate::generated::camera::concepts::make_camera())
                .with(crate::generated::camera::components::fovy(), 1f32)
                .with(crate::generated::camera::components::aspect_ratio(), 1f32)
        }
        #[doc = "Checks if the entity is a *Perspective Common Camera*.\n\nBase components for a perspective camera. Consider `perspective_camera` or `perspective_infinite_reverse_camera`.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::fovy\": f32 = 1.0,\n  \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n  \"ambient_core::camera::camera\": { // Concept.\n    \"ambient_core::camera::near\": f32 = 0.1,\n    \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::active_camera\": f32 = 0.0,\n    \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::transformable\": { // Concept.\n      \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n      \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n      \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n    },\n  },\n}\n```\n"]
        pub fn is_perspective_common_camera(world: &crate::World, id: EntityId) -> bool {
            crate::generated::camera::concepts::is_camera(world, id)
                && world.has_components(id, &{
                    let mut set = crate::ComponentSet::new();
                    set.insert(crate::generated::camera::components::fovy().desc());
                    set.insert(crate::generated::camera::components::aspect_ratio().desc());
                    set
                })
        }
        #[doc = "Returns the components that comprise *Perspective Common Camera* as a tuple.\n\nBase components for a perspective camera. Consider `perspective_camera` or `perspective_infinite_reverse_camera`.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::fovy\": f32 = 1.0,\n  \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n  \"ambient_core::camera::camera\": { // Concept.\n    \"ambient_core::camera::near\": f32 = 0.1,\n    \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::active_camera\": f32 = 0.0,\n    \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::transformable\": { // Concept.\n      \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n      \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n      \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n    },\n  },\n}\n```\n"]
        pub fn perspective_common_camera() -> (Component<f32>, Component<f32>) {
            (
                crate::generated::camera::components::fovy(),
                crate::generated::camera::components::aspect_ratio(),
            )
        }
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Perspective Camera*.\n\nA perspective camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::perspective\": () = (),\n  \"ambient_core::camera::far\": f32 = 1000.0,\n  \"ambient_core::camera::perspective_common_camera\": { // Concept.\n    \"ambient_core::camera::fovy\": f32 = 1.0,\n    \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n    \"ambient_core::camera::camera\": { // Concept.\n      \"ambient_core::camera::near\": f32 = 0.1,\n      \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::active_camera\": f32 = 0.0,\n      \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::transformable\": { // Concept.\n        \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n        \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n        \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n      },\n    },\n  },\n}\n```\n"]
        pub fn make_perspective_camera() -> Entity {
            Entity::new()
                .with_merge(crate::generated::camera::concepts::make_perspective_common_camera())
                .with(crate::generated::camera::components::perspective(), ())
                .with(crate::generated::camera::components::far(), 1000f32)
        }
        #[doc = "Checks if the entity is a *Perspective Camera*.\n\nA perspective camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::perspective\": () = (),\n  \"ambient_core::camera::far\": f32 = 1000.0,\n  \"ambient_core::camera::perspective_common_camera\": { // Concept.\n    \"ambient_core::camera::fovy\": f32 = 1.0,\n    \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n    \"ambient_core::camera::camera\": { // Concept.\n      \"ambient_core::camera::near\": f32 = 0.1,\n      \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::active_camera\": f32 = 0.0,\n      \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::transformable\": { // Concept.\n        \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n        \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n        \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n      },\n    },\n  },\n}\n```\n"]
        pub fn is_perspective_camera(world: &crate::World, id: EntityId) -> bool {
            crate::generated::camera::concepts::is_perspective_common_camera(world, id)
                && world.has_components(id, &{
                    let mut set = crate::ComponentSet::new();
                    set.insert(crate::generated::camera::components::perspective().desc());
                    set.insert(crate::generated::camera::components::far().desc());
                    set
                })
        }
        #[doc = "Returns the components that comprise *Perspective Camera* as a tuple.\n\nA perspective camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::perspective\": () = (),\n  \"ambient_core::camera::far\": f32 = 1000.0,\n  \"ambient_core::camera::perspective_common_camera\": { // Concept.\n    \"ambient_core::camera::fovy\": f32 = 1.0,\n    \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n    \"ambient_core::camera::camera\": { // Concept.\n      \"ambient_core::camera::near\": f32 = 0.1,\n      \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::active_camera\": f32 = 0.0,\n      \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::transformable\": { // Concept.\n        \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n        \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n        \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n      },\n    },\n  },\n}\n```\n"]
        pub fn perspective_camera() -> (Component<()>, Component<f32>) {
            (
                crate::generated::camera::components::perspective(),
                crate::generated::camera::components::far(),
            )
        }
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Perspective-Infinite-Reverse Camera*.\n\nA perspective-infinite-reverse camera. This is recommended for most use-cases.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::perspective_infinite_reverse\": () = (),\n  \"ambient_core::camera::perspective_common_camera\": { // Concept.\n    \"ambient_core::camera::fovy\": f32 = 1.0,\n    \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n    \"ambient_core::camera::camera\": { // Concept.\n      \"ambient_core::camera::near\": f32 = 0.1,\n      \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::active_camera\": f32 = 0.0,\n      \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::transformable\": { // Concept.\n        \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n        \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n        \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n      },\n    },\n  },\n}\n```\n"]
        pub fn make_perspective_infinite_reverse_camera() -> Entity {
            Entity::new()
                .with_merge(crate::generated::camera::concepts::make_perspective_common_camera())
                .with(
                    crate::generated::camera::components::perspective_infinite_reverse(),
                    (),
                )
        }
        #[doc = "Checks if the entity is a *Perspective-Infinite-Reverse Camera*.\n\nA perspective-infinite-reverse camera. This is recommended for most use-cases.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::perspective_infinite_reverse\": () = (),\n  \"ambient_core::camera::perspective_common_camera\": { // Concept.\n    \"ambient_core::camera::fovy\": f32 = 1.0,\n    \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n    \"ambient_core::camera::camera\": { // Concept.\n      \"ambient_core::camera::near\": f32 = 0.1,\n      \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::active_camera\": f32 = 0.0,\n      \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::transformable\": { // Concept.\n        \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n        \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n        \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n      },\n    },\n  },\n}\n```\n"]
        pub fn is_perspective_infinite_reverse_camera(world: &crate::World, id: EntityId) -> bool {
            crate::generated::camera::concepts::is_perspective_common_camera(world, id)
                && world.has_components(id, &{
                    let mut set = crate::ComponentSet::new();
                    set.insert(
                        crate::generated::camera::components::perspective_infinite_reverse().desc(),
                    );
                    set
                })
        }
        #[doc = "Returns the components that comprise *Perspective-Infinite-Reverse Camera* as a tuple.\n\nA perspective-infinite-reverse camera. This is recommended for most use-cases.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::perspective_infinite_reverse\": () = (),\n  \"ambient_core::camera::perspective_common_camera\": { // Concept.\n    \"ambient_core::camera::fovy\": f32 = 1.0,\n    \"ambient_core::camera::aspect_ratio\": f32 = 1.0,\n    \"ambient_core::camera::camera\": { // Concept.\n      \"ambient_core::camera::near\": f32 = 0.1,\n      \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::camera::active_camera\": f32 = 0.0,\n      \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n      \"ambient_core::transform::transformable\": { // Concept.\n        \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n        \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n        \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n      },\n    },\n  },\n}\n```\n"]
        pub fn perspective_infinite_reverse_camera() -> (Component<()>) {
            (crate::generated::camera::components::perspective_infinite_reverse())
        }
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Orthographic Camera*.\n\nAn orthographic camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::orthographic\": () = (),\n  \"ambient_core::camera::orthographic_left\": f32 = -1.0,\n  \"ambient_core::camera::orthographic_right\": f32 = 1.0,\n  \"ambient_core::camera::orthographic_top\": f32 = 1.0,\n  \"ambient_core::camera::orthographic_bottom\": f32 = -1.0,\n  \"ambient_core::camera::near\": f32 = -1.0,\n  \"ambient_core::camera::far\": f32 = 1.0,\n  \"ambient_core::camera::camera\": { // Concept.\n    \"ambient_core::camera::near\": f32 = 0.1,\n    \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::active_camera\": f32 = 0.0,\n    \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::transformable\": { // Concept.\n      \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n      \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n      \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n    },\n  },\n}\n```\n"]
        pub fn make_orthographic_camera() -> Entity {
            Entity::new()
                .with_merge(crate::generated::camera::concepts::make_camera())
                .with(crate::generated::camera::components::orthographic(), ())
                .with(
                    crate::generated::camera::components::orthographic_left(),
                    -1f32,
                )
                .with(
                    crate::generated::camera::components::orthographic_right(),
                    1f32,
                )
                .with(
                    crate::generated::camera::components::orthographic_top(),
                    1f32,
                )
                .with(
                    crate::generated::camera::components::orthographic_bottom(),
                    -1f32,
                )
                .with(crate::generated::camera::components::near(), -1f32)
                .with(crate::generated::camera::components::far(), 1f32)
        }
        #[doc = "Checks if the entity is a *Orthographic Camera*.\n\nAn orthographic camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::orthographic\": () = (),\n  \"ambient_core::camera::orthographic_left\": f32 = -1.0,\n  \"ambient_core::camera::orthographic_right\": f32 = 1.0,\n  \"ambient_core::camera::orthographic_top\": f32 = 1.0,\n  \"ambient_core::camera::orthographic_bottom\": f32 = -1.0,\n  \"ambient_core::camera::near\": f32 = -1.0,\n  \"ambient_core::camera::far\": f32 = 1.0,\n  \"ambient_core::camera::camera\": { // Concept.\n    \"ambient_core::camera::near\": f32 = 0.1,\n    \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::active_camera\": f32 = 0.0,\n    \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::transformable\": { // Concept.\n      \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n      \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n      \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n    },\n  },\n}\n```\n"]
        pub fn is_orthographic_camera(world: &crate::World, id: EntityId) -> bool {
            crate::generated::camera::concepts::is_camera(world, id)
                && world.has_components(id, &{
                    let mut set = crate::ComponentSet::new();
                    set.insert(crate::generated::camera::components::orthographic().desc());
                    set.insert(crate::generated::camera::components::orthographic_left().desc());
                    set.insert(crate::generated::camera::components::orthographic_right().desc());
                    set.insert(crate::generated::camera::components::orthographic_top().desc());
                    set.insert(crate::generated::camera::components::orthographic_bottom().desc());
                    set.insert(crate::generated::camera::components::near().desc());
                    set.insert(crate::generated::camera::components::far().desc());
                    set
                })
        }
        #[doc = "Returns the components that comprise *Orthographic Camera* as a tuple.\n\nAn orthographic camera.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::camera::orthographic\": () = (),\n  \"ambient_core::camera::orthographic_left\": f32 = -1.0,\n  \"ambient_core::camera::orthographic_right\": f32 = 1.0,\n  \"ambient_core::camera::orthographic_top\": f32 = 1.0,\n  \"ambient_core::camera::orthographic_bottom\": f32 = -1.0,\n  \"ambient_core::camera::near\": f32 = -1.0,\n  \"ambient_core::camera::far\": f32 = 1.0,\n  \"ambient_core::camera::camera\": { // Concept.\n    \"ambient_core::camera::near\": f32 = 0.1,\n    \"ambient_core::camera::projection\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::projection_view\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::camera::active_camera\": f32 = 0.0,\n    \"ambient_core::transform::local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::inv_local_to_world\": Mat4 = Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, 0.0, 0.0, 1.0) },\n    \"ambient_core::transform::transformable\": { // Concept.\n      \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n      \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n      \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n    },\n  },\n}\n```\n"]
        pub fn orthographic_camera() -> (
            Component<()>,
            Component<f32>,
            Component<f32>,
            Component<f32>,
            Component<f32>,
            Component<f32>,
            Component<f32>,
        ) {
            (
                crate::generated::camera::components::orthographic(),
                crate::generated::camera::components::orthographic_left(),
                crate::generated::camera::components::orthographic_right(),
                crate::generated::camera::components::orthographic_top(),
                crate::generated::camera::components::orthographic_bottom(),
                crate::generated::camera::components::near(),
                crate::generated::camera::components::far(),
            )
        }
    }
}
#[allow(unused)]
pub mod ecs {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("ecs" , { # [doc = "**Children**: The children of this entity.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"] @ [Debuggable , Networked , Store , MaybeResource , Name ["Children"] , Description ["The children of this entity."]] children : Vec :: < EntityId > , # [doc = "**Don't automatically despawn on module unload**: Indicates that this entity shouldn't be despawned when the module that spawned it unloads.\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Don't automatically despawn on module unload"] , Description ["Indicates that this entity shouldn't be despawned when the module that spawned it unloads."]] dont_despawn_on_unload : () , # [doc = "**Don't store**: Indicates that this entity shouldn't be stored on disk.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Don't store"] , Description ["Indicates that this entity shouldn't be stored on disk."]] dont_store : () , # [doc = "**ID**: The ID of the entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["ID"] , Description ["The ID of the entity."]] id : EntityId , # [doc = "**Parent**: The parent of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Parent"] , Description ["The parent of this entity."]] parent : EntityId , });
    }
}
#[allow(unused)]
pub mod input {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("input" , { # [doc = "**Mouse over**: The number of mouse cursors that are currently over this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mouse over"] , Description ["The number of mouse cursors that are currently over this entity."]] mouse_over : u32 , # [doc = "**Mouse pickable max**: This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mouse pickable max"] , Description ["This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area."]] mouse_pickable_max : Vec3 , # [doc = "**Mouse pickable min**: This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mouse pickable min"] , Description ["This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area."]] mouse_pickable_min : Vec3 , });
    }
}
#[allow(unused)]
pub mod layout {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("layout" , { # [doc = "**Align horizontal**: Layout alignment: horizontal.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Align horizontal"] , Description ["Layout alignment: horizontal."]] align_horizontal : u32 , # [doc = "**Align vertical**: Layout alignment: vertical.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Align vertical"] , Description ["Layout alignment: vertical."]] align_vertical : u32 , # [doc = "**Docking**: Layout docking.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Docking"] , Description ["Layout docking."]] docking : u32 , # [doc = "**Fit horizontal**: Layout fit: horizontal.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fit horizontal"] , Description ["Layout fit: horizontal."]] fit_horizontal : u32 , # [doc = "**Fit vertical**: Layout fit: vertical.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fit vertical"] , Description ["Layout fit: vertical."]] fit_vertical : u32 , # [doc = "**Layout**: Layout.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Layout"] , Description ["Layout."]] layout : u32 , # [doc = "**Orientation**: Layout orientation.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Orientation"] , Description ["Layout orientation."]] orientation : u32 , # [doc = "**Is book file**: This is a file in a `layout_bookcase`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Is book file"] , Description ["This is a file in a `layout_bookcase`."]] is_book_file : () , # [doc = "**Margin**: Layout margin: [top, right, bottom, left].\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Margin"] , Description ["Layout margin: [top, right, bottom, left]."]] margin : Vec4 , # [doc = "**Padding**: Layout padding: [top, right, bottom, left].\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Padding"] , Description ["Layout padding: [top, right, bottom, left]."]] padding : Vec4 , # [doc = "**Mesh to local from size**: Update the `mesh_to_local` based on the width and height of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mesh to local from size"] , Description ["Update the `mesh_to_local` based on the width and height of this entity."]] mesh_to_local_from_size : () , # [doc = "**Minimum height**: The minimum height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Minimum height"] , Description ["The minimum height of a UI element."]] min_height : f32 , # [doc = "**Minimum width**: The minimum width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Minimum width"] , Description ["The minimum width of a UI element."]] min_width : f32 , # [doc = "**Maximum height**: The maximum height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Maximum height"] , Description ["The maximum height of a UI element."]] max_height : f32 , # [doc = "**Maximum width**: The maximum width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Maximum width"] , Description ["The maximum width of a UI element."]] max_width : f32 , # [doc = "**Screen**: This entity will be treated as a screen. Used by the Screen ui component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Screen"] , Description ["This entity will be treated as a screen. Used by the Screen ui component."]] screen : () , # [doc = "**Space between items**: Space between items in a layout.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Space between items"] , Description ["Space between items in a layout."]] space_between_items : f32 , # [doc = "**Width**: The width of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Width"] , Description ["The width of a UI element."]] width : f32 , # [doc = "**Height**: The height of a UI element.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Height"] , Description ["The height of a UI element."]] height : f32 , # [doc = "**GPU UI size**: Upload the width and height of this UI element to the GPU.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["GPU UI size"] , Description ["Upload the width and height of this UI element to the GPU."]] gpu_ui_size : Vec4 , });
    }
}
#[allow(unused)]
pub mod model {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("model" , { # [doc = "**Model animatable**: Controls whether this model can be animated.\n\n*Attributes*: MaybeResource, Debuggable, Networked, Store"] @ [MaybeResource , Debuggable , Networked , Store , Name ["Model animatable"] , Description ["Controls whether this model can be animated."]] model_animatable : bool , # [doc = "**Model from URL**: Load a model from the given URL or relative path.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Model from URL"] , Description ["Load a model from the given URL or relative path."]] model_from_url : String , # [doc = "**Model loaded**: If attached, this entity has a model attached to it.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Model loaded"] , Description ["If attached, this entity has a model attached to it."]] model_loaded : () , });
    }
}
#[allow(unused)]
pub mod network {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("network" , { # [doc = "**Is remote entity**: If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server).\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Is remote entity"] , Description ["If attached, this entity was not spawned locally (e.g. if this is the client, it was spawned by the server)."]] is_remote_entity : () , # [doc = "**Persistent resources**: If attached, this entity contains global resources that are persisted to disk and synchronized to clients.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Persistent resources"] , Description ["If attached, this entity contains global resources that are persisted to disk and synchronized to clients."]] persistent_resources : () , # [doc = "**Synchronized resources**: If attached, this entity contains global resources that are synchronized to clients, but not persisted.\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Synchronized resources"] , Description ["If attached, this entity contains global resources that are synchronized to clients, but not persisted."]] synced_resources : () , });
    }
}
#[allow(unused)]
pub mod physics {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("physics" , { # [doc = "**Angular velocity**: Angular velocity (radians/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's angular velocity in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Angular velocity"] , Description ["Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene."]] angular_velocity : Vec3 , # [doc = "**Cube collider**: If attached, this entity will have a cube physics collider.\n\n`x, y, z` is the size of the cube.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cube collider"] , Description ["If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube."]] cube_collider : Vec3 , # [doc = "**Character controller height**: The height of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Character controller height"] , Description ["The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider."]] character_controller_height : f32 , # [doc = "**Character controller radius**: The radius of the physics character controller attached to this entity.\n\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Character controller radius"] , Description ["The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider."]] character_controller_radius : f32 , # [doc = "**Collider from URL**: This entity will load its physics collider from the URL.\n\nThe value is the URL to load from.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Collider from URL"] , Description ["This entity will load its physics collider from the URL.\nThe value is the URL to load from."]] collider_from_url : String , # [doc = "**Collider loaded**: This component is automatically attached to an entity once the collider has been loaded (through e.g. `collider_from_url`).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Collider loaded"] , Description ["This component is automatically attached to an entity once the collider has been loaded (through e.g. `collider_from_url`)."]] collider_loaded : () , # [doc = "**Collider loads**: Contains all colliders that were loaded in this physics tick.\n\n*Attributes*: Debuggable, Networked, Resource, Store"] @ [Debuggable , Networked , Resource , Store , Name ["Collider loads"] , Description ["Contains all colliders that were loaded in this physics tick."]] collider_loads : Vec :: < EntityId > , # [doc = "**Contact offset**: Contact offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's contact offset for each attached shape in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Contact offset"] , Description ["Contact offset (in meters) of this entity in the physics scene.\nUpdating this component will update the entity's contact offset for each attached shape in the physics scene."]] contact_offset : f32 , # [doc = "**Density**: The density of this entity.\n\nThis is used to update the `mass` when the entity is rescaled.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 1.0"] @ [Debuggable , Networked , Store , Name ["Density"] , Description ["The density of this entity.\nThis is used to update the `mass` when the entity is rescaled."]] density : f32 , # [doc = "**Dynamic**: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Dynamic"] , Description ["If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static."]] dynamic : bool , # [doc = "**Kinematic**: If attached, and this entity is dynamic, this entity will also be kinematic (i.e. unable to be affected by other entities motion). Otherwise, it will receive forces normally.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Kinematic"] , Description ["If attached, and this entity is dynamic, this entity will also be kinematic (i.e. unable to be affected by other entities motion). Otherwise, it will receive forces normally."]] kinematic : () , # [doc = "**Linear velocity**: Linear velocity (meters/second) of this entity in the physics scene.\n\nUpdating this component will update the entity's linear velocity in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Linear velocity"] , Description ["Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene."]] linear_velocity : Vec3 , # [doc = "**Make physics static**: All physics objects will be made static when loaded.\n\n*Attributes*: Debuggable, Networked, Resource, Store"] @ [Debuggable , Networked , Resource , Store , Name ["Make physics static"] , Description ["All physics objects will be made static when loaded."]] make_physics_static : bool , # [doc = "**Mass**: The mass of this entity, measured in kilograms.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 1.0"] @ [Debuggable , Networked , Store , Name ["Mass"] , Description ["The mass of this entity, measured in kilograms."]] mass : f32 , # [doc = "**Physics controlled**: If attached, this entity will be controlled by physics.\n\nNote that this requires the entity to have a collider.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Physics controlled"] , Description ["If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider."]] physics_controlled : () , # [doc = "**Plane collider**: If attached, this entity will have a plane physics collider. A plane is an infinite, flat surface. If you need a bounded flat surface, consider using a cube collider instead.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Plane collider"] , Description ["If attached, this entity will have a plane physics collider. A plane is an infinite, flat surface. If you need a bounded flat surface, consider using a cube collider instead."]] plane_collider : () , # [doc = "**Rest offset**: Rest offset (in meters) of this entity in the physics scene.\n\nUpdating this component will update the entity's rest offset for each attached shape in the physics scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Rest offset"] , Description ["Rest offset (in meters) of this entity in the physics scene.\nUpdating this component will update the entity's rest offset for each attached shape in the physics scene."]] rest_offset : f32 , # [doc = "**Sphere collider**: If attached, this entity will have a sphere physics collider.\n\nThe value corresponds to the radius of the sphere.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sphere collider"] , Description ["If attached, this entity will have a sphere physics collider.\nThe value corresponds to the radius of the sphere."]] sphere_collider : f32 , # [doc = "**Unit mass**: The mass of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Unit mass"] , Description ["The mass of a character/unit."]] unit_mass : f32 , # [doc = "**Unit velocity**: The velocity of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Unit velocity"] , Description ["The velocity of a character/unit."]] unit_velocity : Vec3 , # [doc = "**Unit yaw**: The yaw of a character/unit.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Unit yaw"] , Description ["The yaw of a character/unit."]] unit_yaw : f32 , # [doc = "**Visualize collider**: If attached, the collider will be rendered.\n\n\n\n**Note**: this will continuously overwrite the `local_gizmos` component.\n\n\n\n*Attributes*: Debuggable, Networked"] @ [Debuggable , Networked , Name ["Visualize collider"] , Description ["If attached, the collider will be rendered.\n\n**Note**: this will continuously overwrite the `local_gizmos` component.\n"]] visualize_collider : () , });
    }
}
#[allow(unused)]
pub mod player {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("player" , { # [doc = "**Local user ID**: The user ID of the local player.\n\n*Attributes*: Debuggable, Networked, Resource, Store"] @ [Debuggable , Networked , Resource , Store , Name ["Local user ID"] , Description ["The user ID of the local player."]] local_user_id : String , # [doc = "**Player**: This entity is a player.\n\nNote that this is a logical construct; a player's body may be separate from the player itself.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Player"] , Description ["This entity is a player.\nNote that this is a logical construct; a player's body may be separate from the player itself."]] player : () , # [doc = "**User ID**: An identifier attached to all things owned by a user, and supplied by the user.\n\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["User ID"] , Description ["An identifier attached to all things owned by a user, and supplied by the user.\nThis can be attached to more than just the player; by convention, it is also attached to related entities, including their camera and body."]] user_id : String , });
    }
}
#[allow(unused)]
pub mod prefab {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("prefab" , { # [doc = "**Prefab from URL**: Load and attach a prefab from a URL or relative path.\n\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity.\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Prefab from URL"] , Description ["Load and attach a prefab from a URL or relative path.\nWhen loaded, the components from this prefab will add to or replace the existing components for the entity."]] prefab_from_url : String , # [doc = "**Spawned**: If attached, this entity was built from a prefab that has finished spawning.\n\n*Attributes*: Debuggable"] @ [Debuggable , Name ["Spawned"] , Description ["If attached, this entity was built from a prefab that has finished spawning."]] spawned : () , });
    }
}
#[allow(unused)]
pub mod primitives {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("primitives" , { # [doc = "**Cube**: If attached to an entity, the entity will be converted to a cube primitive.\n\nThe cube is unit-sized (i.e. 0.5 metres out to each side).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cube"] , Description ["If attached to an entity, the entity will be converted to a cube primitive.\nThe cube is unit-sized (i.e. 0.5 metres out to each side)."]] cube : () , # [doc = "**Quad**: If attached to an entity, the entity will be converted to a quad primitive.\n\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Quad"] , Description ["If attached to an entity, the entity will be converted to a quad primitive.\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes)."]] quad : () , # [doc = "**Sphere**: If attached to an entity alongside the other `sphere_*` components, the entity will be converted to a sphere primitive.\n\nTo easily instantiate a unit-diameter `sphere`, consider using the `sphere` concept (e.g. `make_sphere`).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sphere"] , Description ["If attached to an entity alongside the other `sphere_*` components, the entity will be converted to a sphere primitive.\nTo easily instantiate a unit-diameter `sphere`, consider using the `sphere` concept (e.g. `make_sphere`)."]] sphere : () , # [doc = "**Sphere radius**: Set the radius of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 0.5"] @ [Debuggable , Networked , Store , Name ["Sphere radius"] , Description ["Set the radius of a `sphere` entity."]] sphere_radius : f32 , # [doc = "**Sphere sectors**: Set the longitudinal sectors of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 36"] @ [Debuggable , Networked , Store , Name ["Sphere sectors"] , Description ["Set the longitudinal sectors of a `sphere` entity."]] sphere_sectors : u32 , # [doc = "**Sphere stacks**: Set the latitudinal stacks of a `sphere` entity.\n\n*Attributes*: Debuggable, Networked, Store\n\n*Suggested Default*: 18"] @ [Debuggable , Networked , Store , Name ["Sphere stacks"] , Description ["Set the latitudinal stacks of a `sphere` entity."]] sphere_stacks : u32 , # [doc = "**Torus**: If attached to an entity alongside the other `torus_*` components, the entity will be converted to a torus primitive.\n\nTo easily instantiate a default `torus`, consider using the `torus` concept (e.g. `make_torus`).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus"] , Description ["If attached to an entity alongside the other `torus_*` components, the entity will be converted to a torus primitive.\nTo easily instantiate a default `torus`, consider using the `torus` concept (e.g. `make_torus`)."]] torus : () , # [doc = "**Torus inner radius**: Set the inner radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus inner radius"] , Description ["Set the inner radius of a `torus` entity, spanning XY-plane."]] torus_inner_radius : f32 , # [doc = "**Torus outer radius**: Set the outer radius of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus outer radius"] , Description ["Set the outer radius of a `torus` entity, spanning XY-plane."]] torus_outer_radius : f32 , # [doc = "**Torus loops**: Set the loops of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus loops"] , Description ["Set the loops of a `torus` entity, spanning XY-plane."]] torus_loops : u32 , # [doc = "**Torus slices**: Set the slices of a `torus` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Torus slices"] , Description ["Set the slices of a `torus` entity, spanning XY-plane."]] torus_slices : u32 , # [doc = "**Capsule**: If attached to an entity alongside the other `capsule_*` components, the entity will be converted to a capsule primitive.\n\nTo easily instantiate a default `capsule`, consider using the `capsule` concept (e.g. `make_capsule`).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule"] , Description ["If attached to an entity alongside the other `capsule_*` components, the entity will be converted to a capsule primitive.\nTo easily instantiate a default `capsule`, consider using the `capsule` concept (e.g. `make_capsule`)."]] capsule : () , # [doc = "**Capsule radius**: Set the radius of a `capsule` entity, spanning XY-plane.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule radius"] , Description ["Set the radius of a `capsule` entity, spanning XY-plane."]] capsule_radius : f32 , # [doc = "**Capsule half-height**: Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule half-height"] , Description ["Set the half-height of the `capsule` entity, spanning Z-axis, excluding the caps."]] capsule_half_height : f32 , # [doc = "**Capsule rings**: Set the number of sections between the caps.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule rings"] , Description ["Set the number of sections between the caps."]] capsule_rings : u32 , # [doc = "**Capsule latitudes**: Set the number of latitudinal sections. Should be even.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule latitudes"] , Description ["Set the number of latitudinal sections. Should be even."]] capsule_latitudes : u32 , # [doc = "**Capsule longitudes**: Set the number of longitudinal sections.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Capsule longitudes"] , Description ["Set the number of longitudinal sections."]] capsule_longitudes : u32 , });
    }
    #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
    #[doc = r""]
    #[doc = r" They do not have any runtime representation outside of the components that compose them."]
    pub mod concepts {
        use crate::{Component, Entity, EntityId};
        use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Sphere*.\n\nA primitive sphere.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::sphere\": () = (),\n  \"ambient_core::primitives::sphere_radius\": f32 = 0.5,\n  \"ambient_core::primitives::sphere_sectors\": u32 = 36,\n  \"ambient_core::primitives::sphere_stacks\": u32 = 18,\n}\n```\n"]
        pub fn make_sphere() -> Entity {
            Entity::new()
                .with(crate::generated::primitives::components::sphere(), ())
                .with(
                    crate::generated::primitives::components::sphere_radius(),
                    0.5f32,
                )
                .with(
                    crate::generated::primitives::components::sphere_sectors(),
                    36u32,
                )
                .with(
                    crate::generated::primitives::components::sphere_stacks(),
                    18u32,
                )
        }
        #[doc = "Checks if the entity is a *Sphere*.\n\nA primitive sphere.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::sphere\": () = (),\n  \"ambient_core::primitives::sphere_radius\": f32 = 0.5,\n  \"ambient_core::primitives::sphere_sectors\": u32 = 36,\n  \"ambient_core::primitives::sphere_stacks\": u32 = 18,\n}\n```\n"]
        pub fn is_sphere(world: &crate::World, id: EntityId) -> bool {
            world.has_components(id, &{
                let mut set = crate::ComponentSet::new();
                set.insert(crate::generated::primitives::components::sphere().desc());
                set.insert(crate::generated::primitives::components::sphere_radius().desc());
                set.insert(crate::generated::primitives::components::sphere_sectors().desc());
                set.insert(crate::generated::primitives::components::sphere_stacks().desc());
                set
            })
        }
        #[doc = "Returns the components that comprise *Sphere* as a tuple.\n\nA primitive sphere.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::sphere\": () = (),\n  \"ambient_core::primitives::sphere_radius\": f32 = 0.5,\n  \"ambient_core::primitives::sphere_sectors\": u32 = 36,\n  \"ambient_core::primitives::sphere_stacks\": u32 = 18,\n}\n```\n"]
        pub fn sphere() -> (
            Component<()>,
            Component<f32>,
            Component<u32>,
            Component<u32>,
        ) {
            (
                crate::generated::primitives::components::sphere(),
                crate::generated::primitives::components::sphere_radius(),
                crate::generated::primitives::components::sphere_sectors(),
                crate::generated::primitives::components::sphere_stacks(),
            )
        }
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Capsule*.\n\nA primitive capsule. Defined as a cylinder capped by hemispheres.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::capsule\": () = (),\n  \"ambient_core::primitives::capsule_radius\": f32 = 0.5,\n  \"ambient_core::primitives::capsule_half_height\": f32 = 0.5,\n  \"ambient_core::primitives::capsule_rings\": u32 = 0,\n  \"ambient_core::primitives::capsule_latitudes\": u32 = 16,\n  \"ambient_core::primitives::capsule_longitudes\": u32 = 32,\n}\n```\n"]
        pub fn make_capsule() -> Entity {
            Entity::new()
                .with(crate::generated::primitives::components::capsule(), ())
                .with(
                    crate::generated::primitives::components::capsule_radius(),
                    0.5f32,
                )
                .with(
                    crate::generated::primitives::components::capsule_half_height(),
                    0.5f32,
                )
                .with(
                    crate::generated::primitives::components::capsule_rings(),
                    0u32,
                )
                .with(
                    crate::generated::primitives::components::capsule_latitudes(),
                    16u32,
                )
                .with(
                    crate::generated::primitives::components::capsule_longitudes(),
                    32u32,
                )
        }
        #[doc = "Checks if the entity is a *Capsule*.\n\nA primitive capsule. Defined as a cylinder capped by hemispheres.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::capsule\": () = (),\n  \"ambient_core::primitives::capsule_radius\": f32 = 0.5,\n  \"ambient_core::primitives::capsule_half_height\": f32 = 0.5,\n  \"ambient_core::primitives::capsule_rings\": u32 = 0,\n  \"ambient_core::primitives::capsule_latitudes\": u32 = 16,\n  \"ambient_core::primitives::capsule_longitudes\": u32 = 32,\n}\n```\n"]
        pub fn is_capsule(world: &crate::World, id: EntityId) -> bool {
            world.has_components(id, &{
                let mut set = crate::ComponentSet::new();
                set.insert(crate::generated::primitives::components::capsule().desc());
                set.insert(crate::generated::primitives::components::capsule_radius().desc());
                set.insert(crate::generated::primitives::components::capsule_half_height().desc());
                set.insert(crate::generated::primitives::components::capsule_rings().desc());
                set.insert(crate::generated::primitives::components::capsule_latitudes().desc());
                set.insert(crate::generated::primitives::components::capsule_longitudes().desc());
                set
            })
        }
        #[doc = "Returns the components that comprise *Capsule* as a tuple.\n\nA primitive capsule. Defined as a cylinder capped by hemispheres.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::capsule\": () = (),\n  \"ambient_core::primitives::capsule_radius\": f32 = 0.5,\n  \"ambient_core::primitives::capsule_half_height\": f32 = 0.5,\n  \"ambient_core::primitives::capsule_rings\": u32 = 0,\n  \"ambient_core::primitives::capsule_latitudes\": u32 = 16,\n  \"ambient_core::primitives::capsule_longitudes\": u32 = 32,\n}\n```\n"]
        pub fn capsule() -> (
            Component<()>,
            Component<f32>,
            Component<f32>,
            Component<u32>,
            Component<u32>,
            Component<u32>,
        ) {
            (
                crate::generated::primitives::components::capsule(),
                crate::generated::primitives::components::capsule_radius(),
                crate::generated::primitives::components::capsule_half_height(),
                crate::generated::primitives::components::capsule_rings(),
                crate::generated::primitives::components::capsule_latitudes(),
                crate::generated::primitives::components::capsule_longitudes(),
            )
        }
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Torus*.\n\nA primitive Torus, surface of revolution generated by revolving a circle in three-dimensional space one full revolution.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::torus\": () = (),\n  \"ambient_core::primitives::torus_inner_radius\": f32 = 0.25,\n  \"ambient_core::primitives::torus_outer_radius\": f32 = 0.35,\n  \"ambient_core::primitives::torus_slices\": u32 = 32,\n  \"ambient_core::primitives::torus_loops\": u32 = 16,\n}\n```\n"]
        pub fn make_torus() -> Entity {
            Entity::new()
                .with(crate::generated::primitives::components::torus(), ())
                .with(
                    crate::generated::primitives::components::torus_inner_radius(),
                    0.25f32,
                )
                .with(
                    crate::generated::primitives::components::torus_outer_radius(),
                    0.35f32,
                )
                .with(
                    crate::generated::primitives::components::torus_slices(),
                    32u32,
                )
                .with(
                    crate::generated::primitives::components::torus_loops(),
                    16u32,
                )
        }
        #[doc = "Checks if the entity is a *Torus*.\n\nA primitive Torus, surface of revolution generated by revolving a circle in three-dimensional space one full revolution.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::torus\": () = (),\n  \"ambient_core::primitives::torus_inner_radius\": f32 = 0.25,\n  \"ambient_core::primitives::torus_outer_radius\": f32 = 0.35,\n  \"ambient_core::primitives::torus_slices\": u32 = 32,\n  \"ambient_core::primitives::torus_loops\": u32 = 16,\n}\n```\n"]
        pub fn is_torus(world: &crate::World, id: EntityId) -> bool {
            world.has_components(id, &{
                let mut set = crate::ComponentSet::new();
                set.insert(crate::generated::primitives::components::torus().desc());
                set.insert(crate::generated::primitives::components::torus_inner_radius().desc());
                set.insert(crate::generated::primitives::components::torus_outer_radius().desc());
                set.insert(crate::generated::primitives::components::torus_slices().desc());
                set.insert(crate::generated::primitives::components::torus_loops().desc());
                set
            })
        }
        #[doc = "Returns the components that comprise *Torus* as a tuple.\n\nA primitive Torus, surface of revolution generated by revolving a circle in three-dimensional space one full revolution.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::primitives::torus\": () = (),\n  \"ambient_core::primitives::torus_inner_radius\": f32 = 0.25,\n  \"ambient_core::primitives::torus_outer_radius\": f32 = 0.35,\n  \"ambient_core::primitives::torus_slices\": u32 = 32,\n  \"ambient_core::primitives::torus_loops\": u32 = 16,\n}\n```\n"]
        pub fn torus() -> (
            Component<()>,
            Component<f32>,
            Component<f32>,
            Component<u32>,
            Component<u32>,
        ) {
            (
                crate::generated::primitives::components::torus(),
                crate::generated::primitives::components::torus_inner_radius(),
                crate::generated::primitives::components::torus_outer_radius(),
                crate::generated::primitives::components::torus_slices(),
                crate::generated::primitives::components::torus_loops(),
            )
        }
    }
}
#[allow(unused)]
pub mod procedurals {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("procedurals" , { # [doc = "**Procedural mesh**: Attaches a procedural mesh to this entity\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Procedural mesh"] , Description ["Attaches a procedural mesh to this entity"]] procedural_mesh : ProceduralMeshHandle , # [doc = "**Procedural material**: Attaches a procedural material to this entity\n\n*Attributes*: Debuggable, Store"] @ [Debuggable , Store , Name ["Procedural material"] , Description ["Attaches a procedural material to this entity"]] procedural_material : ProceduralMaterialHandle , });
    }
}
#[allow(unused)]
pub mod rect {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("rect" , { # [doc = "**Background color**: Background color of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Background color"] , Description ["Background color of an entity with a `rect` component."]] background_color : Vec4 , # [doc = "**Background URL**: URL to an image asset.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Background URL"] , Description ["URL to an image asset."]] background_url : String , # [doc = "**Border color**: Border color of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Border color"] , Description ["Border color of an entity with a `rect` component."]] border_color : Vec4 , # [doc = "**Border radius**: Radius for each corner of an entity with a `rect` component.\n\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Border radius"] , Description ["Radius for each corner of an entity with a `rect` component.\n`x` = top-left, `y` = top-right, `z` = bottom-left, `w` = bottom-right."]] border_radius : Vec4 , # [doc = "**Border thickness**: Border thickness of an entity with a `rect` component.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Border thickness"] , Description ["Border thickness of an entity with a `rect` component."]] border_thickness : f32 , # [doc = "**Pixel Line from**: Start point of a pixel sized line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Pixel Line from"] , Description ["Start point of a pixel sized line."]] pixel_line_from : Vec3 , # [doc = "**Pixel Line to**: End point of a pixel sized line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Pixel Line to"] , Description ["End point of a pixel sized line."]] pixel_line_to : Vec3 , # [doc = "**Line from**: Start point of a line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Line from"] , Description ["Start point of a line."]] line_from : Vec3 , # [doc = "**Line to**: End point of a line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Line to"] , Description ["End point of a line."]] line_to : Vec3 , # [doc = "**Line width**: Width of line.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Line width"] , Description ["Width of line."]] line_width : f32 , # [doc = "**Rect**: If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Rect"] , Description ["If attached to an entity, the entity will be converted to a UI rectangle, with optionally rounded corners and borders."]] rect : () , # [doc = "**Size from background image**: Resize this rect based on the size of the background image.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Size from background image"] , Description ["Resize this rect based on the size of the background image."]] size_from_background_image : () , });
    }
}
#[allow(unused)]
pub mod rendering {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("rendering" , { # [doc = "**Cast shadows**: If attached, this entity will cast shadows.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cast shadows"] , Description ["If attached, this entity will cast shadows."]] cast_shadows : () , # [doc = "**Color**: This entity will be tinted with the specified color if the color is not black.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Color"] , Description ["This entity will be tinted with the specified color if the color is not black."]] color : Vec4 , # [doc = "**Double-sided**: If this is set, the entity will be rendered with double-sided rendering.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Double-sided"] , Description ["If this is set, the entity will be rendered with double-sided rendering."]] double_sided : bool , # [doc = "**Fog color**: The color of the fog for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog color"] , Description ["The color of the fog for this `sun`."]] fog_color : Vec3 , # [doc = "**Fog density**: The density of the fog for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog density"] , Description ["The density of the fog for this `sun`."]] fog_density : f32 , # [doc = "**Fog height fall-off**: The height at which the fog will fall off (i.e. stop being visible) for this `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Fog height fall-off"] , Description ["The height at which the fog will fall off (i.e. stop being visible) for this `sun`."]] fog_height_falloff : f32 , # [doc = "**Joint Matrices**: Contains the matrices for each joint of this skinned mesh.\n\nThis should be used in combination with `joints`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Joint Matrices"] , Description ["Contains the matrices for each joint of this skinned mesh.\nThis should be used in combination with `joints`."]] joint_matrices : Vec :: < Mat4 > , # [doc = "**Joints**: Contains the joints that comprise this skinned mesh.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Joints"] , Description ["Contains the joints that comprise this skinned mesh."]] joints : Vec :: < EntityId > , # [doc = "**Light ambient**: The ambient light color of the `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Light ambient"] , Description ["The ambient light color of the `sun`."]] light_ambient : Vec3 , # [doc = "**Light diffuse**: The diffuse light color of the `sun`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Light diffuse"] , Description ["The diffuse light color of the `sun`."]] light_diffuse : Vec3 , # [doc = "**Outline**: If attached, this entity will be rendered with an outline with the color specified.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Outline"] , Description ["If attached, this entity will be rendered with an outline with the color specified."]] outline : Vec4 , # [doc = "**Outline (recursive)**: If attached, this entity and all of its children will be rendered with an outline with the color specified.\n\nYou do not need to attach `outline` if you have attached `outline_recursive`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Outline (recursive)"] , Description ["If attached, this entity and all of its children will be rendered with an outline with the color specified.\nYou do not need to attach `outline` if you have attached `outline_recursive`."]] outline_recursive : Vec4 , # [doc = "**Overlay**: If attached, this entity will be rendered with an overlay.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Overlay"] , Description ["If attached, this entity will be rendered with an overlay."]] overlay : () , # [doc = "**PBR material from URL**: Load a PBR material from the URL and attach it to this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["PBR material from URL"] , Description ["Load a PBR material from the URL and attach it to this entity."]] pbr_material_from_url : String , # [doc = "**Sky**: Add a realistic skybox to the scene.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sky"] , Description ["Add a realistic skybox to the scene."]] sky : () , # [doc = "**Sun**: Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\n\nThe entity with the highest `sun` value takes precedence.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Sun"] , Description ["Marks this entity as a sun (i.e. its rotation will be used to control the global light direction).\nThe entity with the highest `sun` value takes precedence."]] sun : f32 , # [doc = "**Transparency group**: Controls when this transparent object will be rendered. Transparent objects are sorted by `(transparency_group, z-depth)`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Transparency group"] , Description ["Controls when this transparent object will be rendered. Transparent objects are sorted by `(transparency_group, z-depth)`."]] transparency_group : i32 , # [doc = "**Water**: Add a realistic water plane to this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Water"] , Description ["Add a realistic water plane to this entity."]] water : () , # [doc = "**Decal material from URL**: Load a Decal material from the URL and attach it to this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Decal material from URL"] , Description ["Load a Decal material from the URL and attach it to this entity."]] decal_from_url : String , # [doc = "**Scissors**: Apply a scissors test to this entity (anything outside the rect will be hidden).\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Scissors"] , Description ["Apply a scissors test to this entity (anything outside the rect will be hidden)."]] scissors : UVec4 , # [doc = "**Scissors (recursive)**: If attached, this entity and all of its children will be rendered with an scissor with the rect specified.\n\nYou do not need to attach `scissors` if you have attached `scissors_recursive`.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Scissors (recursive)"] , Description ["If attached, this entity and all of its children will be rendered with an scissor with the rect specified.\nYou do not need to attach `scissors` if you have attached `scissors_recursive`."]] scissors_recursive : UVec4 , });
    }
}
#[allow(unused)]
pub mod text {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("text" , { # [doc = "**Font family**: Font family to be used. Can either be 'Default', 'FontAwesome', 'FontAwesomeSolid', 'Code' or a url to a font.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Font family"] , Description ["Font family to be used. Can either be 'Default', 'FontAwesome', 'FontAwesomeSolid', 'Code' or a url to a font."]] font_family : String , # [doc = "**Font size**: Size of the font.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Font size"] , Description ["Size of the font."]] font_size : f32 , # [doc = "**Font style**: Style of the font.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Font style"] , Description ["Style of the font."]] font_style : u32 , # [doc = "**Text**: Create a text mesh on this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Text"] , Description ["Create a text mesh on this entity."]] text : String , });
    }
}
#[allow(unused)]
pub mod transform {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("transform" , { # [doc = "**Cylindrical billboard Z**: If attached, this ensures this entity is always aligned with the camera, except on the Z-axis.\n\nThis is useful for decorations that the player will be looking at from roughly the same altitude.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Cylindrical billboard Z"] , Description ["If attached, this ensures this entity is always aligned with the camera, except on the Z-axis.\nThis is useful for decorations that the player will be looking at from roughly the same altitude."]] cylindrical_billboard_z : () , # [doc = "**Euler rotation**: The Euler rotation of this entity in ZYX order.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Euler rotation"] , Description ["The Euler rotation of this entity in ZYX order."]] euler_rotation : Vec3 , # [doc = "**Inverse Local to World**: Converts a world position to a local position.\n\nThis is automatically updated.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Inverse Local to World"] , Description ["Converts a world position to a local position.\nThis is automatically updated."]] inv_local_to_world : Mat4 , # [doc = "**Local to Parent**: Transformation from the entity's local space to the parent's space.\n\n*Attributes*: Debuggable, Networked, Store, MaybeResource"] @ [Debuggable , Networked , Store , MaybeResource , Name ["Local to Parent"] , Description ["Transformation from the entity's local space to the parent's space."]] local_to_parent : Mat4 , # [doc = "**Local to World**: Transformation from the entity's local space to worldspace.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Local to World"] , Description ["Transformation from the entity's local space to worldspace."]] local_to_world : Mat4 , # [doc = "**Look-at target**: The position that this entity should be looking at.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Look-at target"] , Description ["The position that this entity should be looking at."]] lookat_target : Vec3 , # [doc = "**Look-at up**: When combined with `lookat_target`, the up vector for this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Look-at up"] , Description ["When combined with `lookat_target`, the up vector for this entity."]] lookat_up : Vec3 , # [doc = "**Mesh to Local**: Transformation from mesh-space to the entity's local space.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mesh to Local"] , Description ["Transformation from mesh-space to the entity's local space."]] mesh_to_local : Mat4 , # [doc = "**Mesh to World**: Transformation from mesh-space to world space.\n\nThis is automatically updated when `mesh_to_local` and `local_to_world` change.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Mesh to World"] , Description ["Transformation from mesh-space to world space.\nThis is automatically updated when `mesh_to_local` and `local_to_world` change."]] mesh_to_world : Mat4 , # [doc = "**Reset scale**: If attached to a transform hierarchy, the scale will be reset at that point, with only rotation/translation considered.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Reset scale"] , Description ["If attached to a transform hierarchy, the scale will be reset at that point, with only rotation/translation considered."]] reset_scale : () , # [doc = "**Rotation**: The rotation of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Rotation"] , Description ["The rotation of this entity."]] rotation : Quat , # [doc = "**Scale**: The scale of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Scale"] , Description ["The scale of this entity."]] scale : Vec3 , # [doc = "**Spherical billboard**: If attached, this ensures that this entity is always aligned with the camera.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Spherical billboard"] , Description ["If attached, this ensures that this entity is always aligned with the camera."]] spherical_billboard : () , # [doc = "**Translation**: The translation/position of this entity.\n\n*Attributes*: Debuggable, Networked, Store"] @ [Debuggable , Networked , Store , Name ["Translation"] , Description ["The translation/position of this entity."]] translation : Vec3 , });
    }
    #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
    #[doc = r""]
    #[doc = r" They do not have any runtime representation outside of the components that compose them."]
    pub mod concepts {
        use crate::{Component, Entity, EntityId};
        use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        #[allow(clippy::approx_constant)]
        #[doc = "Makes a *Transformable*.\n\nCan be translated, rotated and scaled.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n  \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n  \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n}\n```\n"]
        pub fn make_transformable() -> Entity {
            Entity::new()
                .with(
                    crate::generated::transform::components::translation(),
                    Vec3::new(0f32, 0f32, 0f32),
                )
                .with(
                    crate::generated::transform::components::rotation(),
                    Quat::from_xyzw(0f32, 0f32, 0f32, 1f32),
                )
                .with(
                    crate::generated::transform::components::scale(),
                    Vec3::new(1f32, 1f32, 1f32),
                )
        }
        #[doc = "Checks if the entity is a *Transformable*.\n\nCan be translated, rotated and scaled.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n  \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n  \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n}\n```\n"]
        pub fn is_transformable(world: &crate::World, id: EntityId) -> bool {
            world.has_components(id, &{
                let mut set = crate::ComponentSet::new();
                set.insert(crate::generated::transform::components::translation().desc());
                set.insert(crate::generated::transform::components::rotation().desc());
                set.insert(crate::generated::transform::components::scale().desc());
                set
            })
        }
        #[doc = "Returns the components that comprise *Transformable* as a tuple.\n\nCan be translated, rotated and scaled.\n\n*Definition*:\n\n```ignore\n{\n  \"ambient_core::transform::translation\": Vec3 = Vec3(0.0, 0.0, 0.0),\n  \"ambient_core::transform::rotation\": Quat = Quat(0.0, 0.0, 0.0, 1.0),\n  \"ambient_core::transform::scale\": Vec3 = Vec3(1.0, 1.0, 1.0),\n}\n```\n"]
        pub fn transformable() -> (Component<Vec3>, Component<Quat>, Component<Vec3>) {
            (
                crate::generated::transform::components::translation(),
                crate::generated::transform::components::rotation(),
                crate::generated::transform::components::scale(),
            )
        }
    }
}
#[allow(unused)]
pub mod wasm {
    #[doc = r" Auto-generated component definitions."]
    pub mod components {
        use crate::{
            components, Debuggable, Description, EntityId, MaybeResource, Name, Networked,
            Resource, Store,
        };
        use ambient_shared_types::{
            ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
            ProceduralTextureHandle,
        };
        use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
        use std::time::Duration;
        components ! ("wasm" , { # [doc = "**Module**: A module.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Module"] , Description ["A module."]] module : () , # [doc = "**Module on server**: Whether or not this module is on the server.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Module on server"] , Description ["Whether or not this module is on the server."]] module_on_server : () , # [doc = "**Bytecode from URL**: Asset URL for the bytecode of a WASM component.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Bytecode from URL"] , Description ["Asset URL for the bytecode of a WASM component."]] bytecode_from_url : String , # [doc = "**Module enabled**: Whether or not this module is enabled.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Module enabled"] , Description ["Whether or not this module is enabled."]] module_enabled : bool , # [doc = "**Module name**: The name of this module.\n\n*Attributes*: Networked, Store, Debuggable"] @ [Networked , Store , Debuggable , Name ["Module name"] , Description ["The name of this module."]] module_name : String , });
    }
}
#[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
#[doc = r" and with other modules."]
pub mod messages {
    use crate::{Entity, EntityId};
    use ambient_project_rt::message_serde::{
        Message, MessageSerde, MessageSerdeError, RuntimeMessage,
    };
    use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
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
    #[derive(Clone, Debug)]
    #[doc = "**HttpResponse**: Sent when an HTTP response is received."]
    pub struct HttpResponse {
        pub url: String,
        pub status: u32,
        pub body: Vec<u8>,
        pub error: Option<String>,
    }
    impl HttpResponse {
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
pub fn init() {
    animation::components::init_components();
    app::components::init_components();
    audio::components::init_components();
    camera::components::init_components();
    ecs::components::init_components();
    input::components::init_components();
    layout::components::init_components();
    model::components::init_components();
    network::components::init_components();
    physics::components::init_components();
    player::components::init_components();
    prefab::components::init_components();
    primitives::components::init_components();
    procedurals::components::init_components();
    rect::components::init_components();
    rendering::components::init_components();
    text::components::init_components();
    transform::components::init_components();
    wasm::components::init_components();
}
