#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
    pub mod uigiqyr7qugdncpzkyzinvwxh26daahx {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("uigiqyr7qugdncpzkyzinvwxh26daahx")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Auto-generated component definitions."]
        pub mod components {
            use ambient_api::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
                prelude::*,
            };
            static NEXT_PLAYER_HUE: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::next_player_hue")
            });
            #[doc = "**Next Player Hue**: Controls the hue (in degrees) to use for the next player's color.\n\n*Attributes*: Debuggable, Resource"]
            pub fn next_player_hue() -> Component<f32> {
                *NEXT_PLAYER_HUE
            }
            static IS_BALL: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::is_ball"));
            #[doc = "**Ball**: Used to tag a ball"]
            pub fn is_ball() -> Component<()> {
                *IS_BALL
            }
            static ORIGIN: Lazy<Component<Vec3>> =
                Lazy::new(|| __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::origin"));
            #[doc = "**Origin**: An object's origin in world space"]
            pub fn origin() -> Component<Vec3> {
                *ORIGIN
            }
            static PLAYER_BALL: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_ball")
            });
            #[doc = "**Player Ball**: Used to tag a player's ball"]
            pub fn player_ball() -> Component<EntityId> {
                *PLAYER_BALL
            }
            static PLAYER_RESTORE_POINT: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_restore_point")
            });
            #[doc = "**Player Restore Point**: A player's restore point"]
            pub fn player_restore_point() -> Component<Vec3> {
                *PLAYER_RESTORE_POINT
            }
            static PLAYER_STROKE_COUNT: Lazy<Component<u32>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_stroke_count")
            });
            #[doc = "**Player Stroke Count**: A player's stroke count"]
            pub fn player_stroke_count() -> Component<u32> {
                *PLAYER_STROKE_COUNT
            }
            static PLAYER_COLOR: Lazy<Component<Vec4>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_color")
            });
            #[doc = "**Player Color**: A player's color"]
            pub fn player_color() -> Component<Vec4> {
                *PLAYER_COLOR
            }
            static PLAYER_SHOOT_REQUESTED: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_shoot_requested")
            });
            #[doc = "**Player Shoot Requested**: Whether or not a player has requested to shoot the ball"]
            pub fn player_shoot_requested() -> Component<bool> {
                *PLAYER_SHOOT_REQUESTED
            }
            static PLAYER_INDICATOR: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_indicator")
            });
            #[doc = "**Player Indicator**: EntityId of a player's indicator"]
            pub fn player_indicator() -> Component<EntityId> {
                *PLAYER_INDICATOR
            }
            static PLAYER_INDICATOR_ARROW: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_indicator_arrow")
            });
            #[doc = "**Player Indicator Arrow**: EntityId of a player's indicator arrow"]
            pub fn player_indicator_arrow() -> Component<EntityId> {
                *PLAYER_INDICATOR_ARROW
            }
            static PLAYER_TEXT: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_text")
            });
            #[doc = "**Player Text**: EntityId of a player's text"]
            pub fn player_text() -> Component<EntityId> {
                *PLAYER_TEXT
            }
            static PLAYER_TEXT_CONTAINER: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_text_container")
            });
            #[doc = "**Player Text Container**: EntityId of a player's text container"]
            pub fn player_text_container() -> Component<EntityId> {
                *PLAYER_TEXT_CONTAINER
            }
            static PLAYER_CAMERA_PIVOT: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_pivot")
            });
            #[doc = "**Player Camera Pivot**: The pivot offset a player's camera pivots around"]
            pub fn player_camera_pivot() -> Component<Vec3> {
                *PLAYER_CAMERA_PIVOT
            }
            static PLAYER_CAMERA_POSITION: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_position")
            });
            #[doc = "**Player Camera Position**: The position at which a player's camera pivots around"]
            pub fn player_camera_position() -> Component<Vec3> {
                *PLAYER_CAMERA_POSITION
            }
            static PLAYER_CAMERA_RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_radius")
            });
            #[doc = "**Player Camera Radius**: The radius at which a player's camera pivots around"]
            pub fn player_camera_radius() -> Component<f32> {
                *PLAYER_CAMERA_RADIUS
            }
            static PLAYER_CAMERA_ROTATION: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_rotation")
            });
            #[doc = "**Player Camera Rotation**: The rotation (radians) of a player's camera"]
            pub fn player_camera_rotation() -> Component<Vec2> {
                *PLAYER_CAMERA_ROTATION
            }
            static PLAYER_CAMERA_STATE: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_state")
            });
            #[doc = "**Player Camera State**: EntityId of a player's camera state"]
            pub fn player_camera_state() -> Component<EntityId> {
                *PLAYER_CAMERA_STATE
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        #[doc = r""]
        #[doc = r" They do not have any runtime representation outside of the components that compose them."]
        pub mod concepts {
            use ambient_api::{
                global::serde::{self, Deserialize, Serialize},
                prelude::*,
            };
            #[doc = "**Player State**: A player's state\n\n**Required**:\n- `player_restore_point`: A player's restore point\n- `player_stroke_count`: A player's stroke count\n- `player_color`: A player's color"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct PlayerState {
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_restore_point`\n\n**Suggested value**: `Vec3::new(-5f32, 0f32, 20f32, )`\n\n**Component description**: A player's restore point\n\n"]
                pub player_restore_point: Vec3,
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_stroke_count`\n\n**Suggested value**: `0u32`\n\n**Component description**: A player's stroke count\n\n"]
                pub player_stroke_count: u32,
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_color`\n\n**Suggested value**: `Vec4::new(1f32, 1f32, 1f32, 1f32, )`\n\n**Component description**: A player's color\n\n"]
                pub player_color: Vec4,
            }
            impl Concept for PlayerState {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_restore_point () , self . player_restore_point) . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_stroke_count () , self . player_stroke_count) . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_color () , self . player_color) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { player_restore_point : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_restore_point ()) ? , player_stroke_count : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_stroke_count ()) ? , player_color : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_color ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { player_restore_point : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_restore_point ()) ? , player_stroke_count : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_stroke_count ()) ? , player_color : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_color ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_restore_point () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_stroke_count () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_color ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_restore_point () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_stroke_count () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_color ()])
                }
            }
            impl ConceptSuggested for PlayerState {
                #[doc = "```\nplayer_restore_point: Vec3::new(-5f32, 0f32, 20f32, ),\nplayer_stroke_count: 0u32,\nplayer_color: Vec4::new(1f32, 1f32, 1f32, 1f32, ),\n```"]
                fn suggested() -> Self {
                    Self {
                        player_restore_point: Vec3::new(-5f32, 0f32, 20f32),
                        player_stroke_count: 0u32,
                        player_color: Vec4::new(1f32, 1f32, 1f32, 1f32),
                    }
                }
            }
            impl ConceptComponents for PlayerState {
                type Required = (Component<Vec3>, Component<u32>, Component<Vec4>);
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_restore_point () , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_stroke_count () , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_color () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        player_restore_point: required.0,
                        player_stroke_count: required.1,
                        player_color: required.2,
                    }
                }
            }
            #[doc = "**Player Camera State**: A player's camera state\n\n**Required**:\n- `player_camera_pivot`: The pivot offset a player's camera pivots around\n- `player_camera_position`: The position at which a player's camera pivots around\n- `player_camera_radius`: The radius at which a player's camera pivots around\n- `player_camera_rotation`: The rotation (radians) of a player's camera"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct PlayerCameraState {
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_pivot`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 8f32, )`\n\n**Component description**: The pivot offset a player's camera pivots around\n\n"]
                pub player_camera_pivot: Vec3,
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_position`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The position at which a player's camera pivots around\n\n"]
                pub player_camera_position: Vec3,
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_radius`\n\n**Suggested value**: `15f32`\n\n**Component description**: The radius at which a player's camera pivots around\n\n"]
                pub player_camera_radius: f32,
                #[doc = "**Component**: `uigiqyr7qugdncpzkyzinvwxh26daahx::player_camera_rotation`\n\n**Suggested value**: `Vec2::new(3.1415927f32, 0.610865f32, )`\n\n**Component description**: The rotation (radians) of a player's camera\n\n"]
                pub player_camera_rotation: Vec2,
            }
            impl Concept for PlayerCameraState {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_pivot () , self . player_camera_pivot) . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_position () , self . player_camera_position) . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_radius () , self . player_camera_radius) . with (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_rotation () , self . player_camera_rotation) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { player_camera_pivot : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_pivot ()) ? , player_camera_position : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_position ()) ? , player_camera_radius : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_radius ()) ? , player_camera_rotation : entity :: get_component (id , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_rotation ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { player_camera_pivot : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_pivot ()) ? , player_camera_position : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_position ()) ? , player_camera_radius : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_radius ()) ? , player_camera_rotation : entity . get (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_rotation ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_pivot () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_position () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_radius () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_rotation ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_pivot () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_position () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_radius () , & crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_rotation ()])
                }
            }
            impl ConceptSuggested for PlayerCameraState {
                #[doc = "```\nplayer_camera_pivot: Vec3::new(0f32, 0f32, 8f32, ),\nplayer_camera_position: Vec3::new(0f32, 0f32, 0f32, ),\nplayer_camera_radius: 15f32,\nplayer_camera_rotation: Vec2::new(3.1415927f32, 0.610865f32, ),\n```"]
                fn suggested() -> Self {
                    Self {
                        player_camera_pivot: Vec3::new(0f32, 0f32, 8f32),
                        player_camera_position: Vec3::new(0f32, 0f32, 0f32),
                        player_camera_radius: 15f32,
                        player_camera_rotation: Vec2::new(3.1415927f32, 0.610865f32),
                    }
                }
            }
            impl ConceptComponents for PlayerCameraState {
                type Required = (
                    Component<Vec3>,
                    Component<Vec3>,
                    Component<f32>,
                    Component<Vec2>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_pivot () , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_position () , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_radius () , crate :: packages :: raw :: uigiqyr7qugdncpzkyzinvwxh26daahx :: components :: player_camera_rotation () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        player_camera_pivot: required.0,
                        player_camera_position: required.1,
                        player_camera_radius: required.2,
                        player_camera_rotation: required.3,
                    }
                }
            }
        }
        #[doc = r" Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,"]
        #[doc = r" and with other modules."]
        pub mod messages {
            use ambient_api::{
                message::{
                    Message, MessageSerde, MessageSerdeError, ModuleMessage, RuntimeMessage,
                },
                prelude::*,
            };
            #[derive(Clone, Debug)]
            #[doc = "**Input**: Player input"]
            pub struct Input {
                pub camera_rotation: Vec2,
                pub camera_zoom: f32,
                pub shoot: bool,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    camera_rotation: impl Into<Vec2>,
                    camera_zoom: impl Into<f32>,
                    shoot: impl Into<bool>,
                ) -> Self {
                    Self {
                        camera_rotation: camera_rotation.into(),
                        camera_zoom: camera_zoom.into(),
                        shoot: shoot.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "uigiqyr7qugdncpzkyzinvwxh26daahx::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.camera_rotation.serialize_message_part(&mut output)?;
                    self.camera_zoom.serialize_message_part(&mut output)?;
                    self.shoot.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        camera_rotation: Vec2::deserialize_message_part(&mut input)?,
                        camera_zoom: f32::deserialize_message_part(&mut input)?,
                        shoot: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
            #[derive(Clone, Debug)]
            #[doc = "**Hit**: Hit."]
            pub struct Hit {
                pub ball: EntityId,
            }
            impl Hit {
                #[allow(clippy::too_many_arguments)]
                pub fn new(ball: impl Into<EntityId>) -> Self {
                    Self { ball: ball.into() }
                }
            }
            impl Message for Hit {
                fn id() -> &'static str {
                    "uigiqyr7qugdncpzkyzinvwxh26daahx::Hit"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.ball.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        ball: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Hit {}
            #[derive(Clone, Debug)]
            #[doc = "**Bonk**: Collision between two objects."]
            pub struct Bonk {
                pub ball: EntityId,
            }
            impl Bonk {
                #[allow(clippy::too_many_arguments)]
                pub fn new(ball: impl Into<EntityId>) -> Self {
                    Self { ball: ball.into() }
                }
            }
            impl Message for Bonk {
                fn id() -> &'static str {
                    "uigiqyr7qugdncpzkyzinvwxh26daahx::Bonk"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.ball.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        ball: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Bonk {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod ambient_core {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ambient_core")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod xar372tfo2oswb4pkvx7h7o3rxi6tap6 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xar372tfo2oswb4pkvx7h7o3rxi6tap6")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
}
pub use raw::uigiqyr7qugdncpzkyzinvwxh26daahx as this;
pub use raw::xar372tfo2oswb4pkvx7h7o3rxi6tap6 as hide_cursor;
