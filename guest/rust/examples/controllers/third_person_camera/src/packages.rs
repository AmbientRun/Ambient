#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
    pub mod afl5yv5ya35vbuaj3aido22cwjzat25z {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("afl5yv5ya35vbuaj3aido22cwjzat25z")
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
            static RUN_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::run_direction")
            });
            #[doc = "**run_direction**\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_direction() -> Component<Vec2> {
                *RUN_DIRECTION
            }
            static SPEED: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::speed"));
            #[doc = "**speed**: The speed the unit can walk at\n\n*Attributes*: Debuggable, Networked"]
            pub fn speed() -> Component<f32> {
                *SPEED
            }
            static RUN_SPEED_MULTIPLIER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::run_speed_multiplier")
            });
            #[doc = "**run_speed_multiplier**: The speed the unit can run at\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_speed_multiplier() -> Component<f32> {
                *RUN_SPEED_MULTIPLIER
            }
            static STRAFE_SPEED_MULTIPLIER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component(
                    "afl5yv5ya35vbuaj3aido22cwjzat25z::strafe_speed_multiplier",
                )
            });
            #[doc = "**strafe_speed_multiplier**: The speed the unit can strafe at\n\n*Attributes*: Debuggable, Networked"]
            pub fn strafe_speed_multiplier() -> Component<f32> {
                *STRAFE_SPEED_MULTIPLIER
            }
            static AIR_SPEED_MULTIPLIER: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::air_speed_multiplier")
            });
            #[doc = "**air_speed_multiplier**: When the unit is in the air; how much can it control its movement? If this is 0, it can't control it at all. If it's 1 it's the same as on the ground.\n\n*Attributes*: Debuggable, Networked"]
            pub fn air_speed_multiplier() -> Component<f32> {
                *AIR_SPEED_MULTIPLIER
            }
            static UNIT_DISPLACEMENT: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::unit_displacement")
            });
            #[doc = "**unit_displacement**: The distance the unit tried to move last frame (though it may have collided so the actual distance may be shorter).\n\n*Attributes*: Debuggable, Networked"]
            pub fn unit_displacement() -> Component<Vec3> {
                *UNIT_DISPLACEMENT
            }
            static JUMPING: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::jumping"));
            #[doc = "**jumping**\n\n*Attributes*: Debuggable, Networked"]
            pub fn jumping() -> Component<bool> {
                *JUMPING
            }
            static RUNNING: Lazy<Component<bool>> =
                Lazy::new(|| __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::running"));
            #[doc = "**running**\n\n*Attributes*: Debuggable, Networked"]
            pub fn running() -> Component<bool> {
                *RUNNING
            }
            static SHOOTING: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::shooting")
            });
            #[doc = "**shooting**\n\n*Attributes*: Debuggable, Networked"]
            pub fn shooting() -> Component<bool> {
                *SHOOTING
            }
            static VERTICAL_VELOCITY: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::vertical_velocity")
            });
            #[doc = "**Unit vertical speed**: The units's vertical speed.\n\n*Attributes*: Debuggable, Networked"]
            pub fn vertical_velocity() -> Component<f32> {
                *VERTICAL_VELOCITY
            }
            static IS_ON_GROUND: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::is_on_ground")
            });
            #[doc = "**is_on_ground**\n\n*Attributes*: Debuggable, Networked"]
            pub fn is_on_ground() -> Component<bool> {
                *IS_ON_GROUND
            }
            static HEAD_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("afl5yv5ya35vbuaj3aido22cwjzat25z::head_ref")
            });
            #[doc = "**head_ref**\n\n*Attributes*: Debuggable, Networked"]
            pub fn head_ref() -> Component<EntityId> {
                *HEAD_REF
            }
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
    pub mod d3y3wbexrclipsykysumem3fthkudwx2 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("d3y3wbexrclipsykysumem3fthkudwx2")
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
            static BASIC_CHARACTER_ANIMATIONS: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component(
                    "d3y3wbexrclipsykysumem3fthkudwx2::basic_character_animations",
                )
            });
            #[doc = "**basic_character_animations**: Apply animations to the model this points to. Parameters such as health etc. is read from the entity this component is attached to.\n\n*Attributes*: Debuggable, Networked"]
            pub fn basic_character_animations() -> Component<EntityId> {
                *BASIC_CHARACTER_ANIMATIONS
            }
            static USE_RIFLE_ANIMATIONS: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::use_rifle_animations")
            });
            #[doc = "**use_rifle_animations**: Use rifle animations instead of no-weapon animations\n\n*Attributes*: Debuggable, Networked"]
            pub fn use_rifle_animations() -> Component<()> {
                *USE_RIFLE_ANIMATIONS
            }
            static WALK_FORWARD: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_forward")
            });
            #[doc = "**walk_forward**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_forward() -> Component<String> {
                *WALK_FORWARD
            }
            static WALK_BACKWARD: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_backward")
            });
            #[doc = "**walk_backward**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_backward() -> Component<String> {
                *WALK_BACKWARD
            }
            static WALK_LEFT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_left")
            });
            #[doc = "**walk_left**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_left() -> Component<String> {
                *WALK_LEFT
            }
            static WALK_RIGHT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_right")
            });
            #[doc = "**walk_right**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_right() -> Component<String> {
                *WALK_RIGHT
            }
            static WALK_FORWARD_LEFT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_forward_left")
            });
            #[doc = "**walk_forward_left**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_forward_left() -> Component<String> {
                *WALK_FORWARD_LEFT
            }
            static WALK_FORWARD_RIGHT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_forward_right")
            });
            #[doc = "**walk_forward_right**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_forward_right() -> Component<String> {
                *WALK_FORWARD_RIGHT
            }
            static WALK_BACKWARD_LEFT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_backward_left")
            });
            #[doc = "**walk_backward_left**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_backward_left() -> Component<String> {
                *WALK_BACKWARD_LEFT
            }
            static WALK_BACKWARD_RIGHT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::walk_backward_right")
            });
            #[doc = "**walk_backward_right**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn walk_backward_right() -> Component<String> {
                *WALK_BACKWARD_RIGHT
            }
            static RUN_FORWARD: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_forward")
            });
            #[doc = "**run_forward**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_forward() -> Component<String> {
                *RUN_FORWARD
            }
            static RUN_BACKWARD: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_backward")
            });
            #[doc = "**run_backward**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_backward() -> Component<String> {
                *RUN_BACKWARD
            }
            static RUN_LEFT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_left")
            });
            #[doc = "**run_left**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_left() -> Component<String> {
                *RUN_LEFT
            }
            static RUN_RIGHT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_right")
            });
            #[doc = "**run_right**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_right() -> Component<String> {
                *RUN_RIGHT
            }
            static RUN_FORWARD_LEFT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_forward_left")
            });
            #[doc = "**run_forward_left**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_forward_left() -> Component<String> {
                *RUN_FORWARD_LEFT
            }
            static RUN_FORWARD_RIGHT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_forward_right")
            });
            #[doc = "**run_forward_right**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_forward_right() -> Component<String> {
                *RUN_FORWARD_RIGHT
            }
            static RUN_BACKWARD_LEFT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_backward_left")
            });
            #[doc = "**run_backward_left**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_backward_left() -> Component<String> {
                *RUN_BACKWARD_LEFT
            }
            static RUN_BACKWARD_RIGHT: Lazy<Component<String>> = Lazy::new(|| {
                __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::run_backward_right")
            });
            #[doc = "**run_backward_right**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn run_backward_right() -> Component<String> {
                *RUN_BACKWARD_RIGHT
            }
            static IDLE: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::idle"));
            #[doc = "**idle**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn idle() -> Component<String> {
                *IDLE
            }
            static DEATH: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::death"));
            #[doc = "**death**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn death() -> Component<String> {
                *DEATH
            }
            static JUMP: Lazy<Component<String>> =
                Lazy::new(|| __internal_get_component("d3y3wbexrclipsykysumem3fthkudwx2::jump"));
            #[doc = "**jump**: URL to animation\n\n*Attributes*: Debuggable, Networked"]
            pub fn jump() -> Component<String> {
                *JUMP
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod hvxms7i2px7krvkm23sxfjxsjqlcmtb5 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("hvxms7i2px7krvkm23sxfjxsjqlcmtb5")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        pub mod player {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static CONTROL_OF_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "hvxms7i2px7krvkm23sxfjxsjqlcmtb5::player::control_of_entity",
                    )
                });
                #[doc = "**Player's Control-of Entity**: The entity that this player is controlling.\n\n*Attributes*: Debuggable, Networked"]
                pub fn control_of_entity() -> Component<EntityId> {
                    *CONTROL_OF_ENTITY
                }
            }
        }
        #[doc = r" Auto-generated component definitions."]
        pub mod components {
            use ambient_api::{
                ecs::{Component, __internal_get_component},
                once_cell::sync::Lazy,
                prelude::*,
            };
            static HEALTH: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("hvxms7i2px7krvkm23sxfjxsjqlcmtb5::health"));
            #[doc = "**Health**: This game object's health. \"Standard\" health is 100 HP.\n\n*Attributes*: Debuggable, Networked"]
            pub fn health() -> Component<f32> {
                *HEALTH
            }
            static MAX_HEALTH: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health")
            });
            #[doc = "**Max Health**: Maximum health of the object. 100 HP is \"standard.\"\n\n*Attributes*: Debuggable, Networked"]
            pub fn max_health() -> Component<f32> {
                *MAX_HEALTH
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod ie4hdvkvui6t36yzniyex2hc5i2bradc {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ie4hdvkvui6t36yzniyex2hc5i2bradc")
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
    pub mod lktsfudbjw2qikhyumt573ozxhadkiwm {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("lktsfudbjw2qikhyumt573ozxhadkiwm")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        #[doc = r""]
        #[doc = r" They do not have any runtime representation outside of the components that compose them."]
        pub mod concepts {
            use ambient_api::{
                global::serde::{self, Deserialize, Serialize},
                prelude::*,
            };
            #[doc = "**CharacterMovement**\n\n**Extends**: `ambient_core::physics::CharacterController`\n\n**Required**:\n- `character_controller_height`: The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n- `character_controller_radius`: The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n- `physics_controlled`: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n- `rotation`: The rotation of this entity.\n- `run_direction`: No description provided.\n- `vertical_velocity`: The units's vertical speed.\n- `running`: No description provided.\n- `jumping`: No description provided.\n- `is_on_ground`: No description provided.\n\n\n**Optional**:\n- `run_speed_multiplier`: The speed the unit can run at\n- `speed`: The speed the unit can walk at\n- `strafe_speed_multiplier`: The speed the unit can strafe at\n- `air_speed_multiplier`: When the unit is in the air; how much can it control its movement? If this is 0, it can't control it at all. If it's 1 it's the same as on the ground."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct CharacterMovement {
                #[doc = "**Component**: `ambient_core::physics::character_controller_height`\n\n**Suggested value**: `2f32`\n\n**Component description**: The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n\n"]
                pub character_controller_height: f32,
                #[doc = "**Component**: `ambient_core::physics::character_controller_radius`\n\n**Suggested value**: `0.5f32`\n\n**Component description**: The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n\n"]
                pub character_controller_radius: f32,
                #[doc = "**Component**: `ambient_core::physics::physics_controlled`\n\n**Suggested value**: `()`\n\n**Component description**: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n\n"]
                pub physics_controlled: (),
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::run_direction`\n\n**Suggested value**: `Vec2::new(0f32, 0f32, )`\n\n"]
                pub run_direction: Vec2,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::vertical_velocity`\n\n**Suggested value**: `0f32`\n\n**Component description**: The units's vertical speed.\n\n"]
                pub vertical_velocity: f32,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::running`\n\n**Suggested value**: `false`\n\n"]
                pub running: bool,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::jumping`\n\n**Suggested value**: `false`\n\n"]
                pub jumping: bool,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::is_on_ground`\n\n**Suggested value**: `true`\n\n"]
                pub is_on_ground: bool,
                #[doc = r" Optional components."]
                pub optional: CharacterMovementOptional,
            }
            #[doc = "Optional part of [CharacterMovement]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct CharacterMovementOptional {
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::run_speed_multiplier`\n\n**Component description**: The speed the unit can run at\n\n"]
                pub run_speed_multiplier: Option<f32>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::speed`\n\n**Component description**: The speed the unit can walk at\n\n"]
                pub speed: Option<f32>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::strafe_speed_multiplier`\n\n**Component description**: The speed the unit can strafe at\n\n"]
                pub strafe_speed_multiplier: Option<f32>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::air_speed_multiplier`\n\n**Component description**: When the unit is in the air; how much can it control its movement? If this is 0, it can't control it at all. If it's 1 it's the same as on the ground.\n\n"]
                pub air_speed_multiplier: Option<f32>,
            }
            impl Concept for CharacterMovement {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: physics :: components :: character_controller_height () , self . character_controller_height) . with (ambient_api :: core :: physics :: components :: character_controller_radius () , self . character_controller_radius) . with (ambient_api :: core :: physics :: components :: physics_controlled () , self . physics_controlled) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , self . run_direction) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , self . vertical_velocity) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , self . running) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , self . jumping) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground () , self . is_on_ground) ;
                    if let Some(run_speed_multiplier) = self.optional.run_speed_multiplier {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , run_speed_multiplier) ;
                    }
                    if let Some(speed) = self.optional.speed {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , speed) ;
                    }
                    if let Some(strafe_speed_multiplier) = self.optional.strafe_speed_multiplier {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () , strafe_speed_multiplier) ;
                    }
                    if let Some(air_speed_multiplier) = self.optional.air_speed_multiplier {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier () , air_speed_multiplier) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { character_controller_height : entity :: get_component (id , ambient_api :: core :: physics :: components :: character_controller_height ()) ? , character_controller_radius : entity :: get_component (id , ambient_api :: core :: physics :: components :: character_controller_radius ()) ? , physics_controlled : entity :: get_component (id , ambient_api :: core :: physics :: components :: physics_controlled ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , run_direction : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction ()) ? , vertical_velocity : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity ()) ? , running : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running ()) ? , jumping : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping ()) ? , is_on_ground : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground ()) ? , optional : CharacterMovementOptional { run_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) , speed : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) , strafe_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) , air_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { character_controller_height : entity . get (ambient_api :: core :: physics :: components :: character_controller_height ()) ? , character_controller_radius : entity . get (ambient_api :: core :: physics :: components :: character_controller_radius ()) ? , physics_controlled : entity . get (ambient_api :: core :: physics :: components :: physics_controlled ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , run_direction : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction ()) ? , vertical_velocity : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity ()) ? , running : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running ()) ? , jumping : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping ()) ? , is_on_ground : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground ()) ? , optional : CharacterMovementOptional { run_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) , speed : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) , strafe_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) , air_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: physics :: components :: character_controller_height () , & ambient_api :: core :: physics :: components :: character_controller_radius () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: physics :: components :: character_controller_height () , & ambient_api :: core :: physics :: components :: character_controller_radius () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground ()])
                }
            }
            impl ConceptSuggested for CharacterMovement {
                #[doc = "```\ncharacter_controller_height: 2f32,\ncharacter_controller_radius: 0.5f32,\nphysics_controlled: (),\nrotation: Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, ),\nrun_direction: Vec2::new(0f32, 0f32, ),\nvertical_velocity: 0f32,\nrunning: false,\njumping: false,\nis_on_ground: true,\n```"]
                fn suggested() -> Self {
                    Self {
                        character_controller_height: 2f32,
                        character_controller_radius: 0.5f32,
                        physics_controlled: (),
                        rotation: Quat::from_xyzw(0f32, 0f32, 0f32, 1f32),
                        run_direction: Vec2::new(0f32, 0f32),
                        vertical_velocity: 0f32,
                        running: false,
                        jumping: false,
                        is_on_ground: true,
                        optional: Default::default(),
                    }
                }
            }
            impl ConceptComponents for CharacterMovement {
                type Required = (
                    Component<f32>,
                    Component<f32>,
                    Component<()>,
                    Component<Quat>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<bool>,
                    Component<bool>,
                    Component<bool>,
                );
                type Optional = (
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                );
                fn required() -> Self::Required {
                    (ambient_api :: core :: physics :: components :: character_controller_height () , ambient_api :: core :: physics :: components :: character_controller_radius () , ambient_api :: core :: physics :: components :: physics_controlled () , ambient_api :: core :: transform :: components :: rotation () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        character_controller_height: required.0,
                        character_controller_radius: required.1,
                        physics_controlled: required.2,
                        rotation: required.3,
                        run_direction: required.4,
                        vertical_velocity: required.5,
                        running: required.6,
                        jumping: required.7,
                        is_on_ground: required.8,
                        optional: Default::default(),
                    }
                }
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod n7a4j7htvenss35tsnfvegbhxuwij5il {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("n7a4j7htvenss35tsnfvegbhxuwij5il")
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
    pub mod per6j2iqhj3jz4da3fqr75jcj2kqjooo {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("per6j2iqhj3jz4da3fqr75jcj2kqjooo")
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
            static HEIGHT_OFFSET: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("per6j2iqhj3jz4da3fqr75jcj2kqjooo::height_offset")
            });
            #[doc = "**Nameplate Height Offset**: The height offset from the base of this entity at which to render a nameplate. If not specified, it will default to this entity's local bounding AABB Z, and if that's not available, it will default to a constant.\n\n*Attributes*: Debuggable, Networked"]
            pub fn height_offset() -> Component<f32> {
                *HEIGHT_OFFSET
            }
            static TEXT_SIZE: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("per6j2iqhj3jz4da3fqr75jcj2kqjooo::text_size")
            });
            #[doc = "**Nameplate Text Size**: The text size of the nameplate to render. If not specified, it will default to 2.0.\n\n*Attributes*: Debuggable, Networked"]
            pub fn text_size() -> Component<f32> {
                *TEXT_SIZE
            }
            static HIDE: Lazy<Component<()>> =
                Lazy::new(|| __internal_get_component("per6j2iqhj3jz4da3fqr75jcj2kqjooo::hide"));
            #[doc = "**Nameplate Hide**: If attached to a player, hide the nameplate for that player.\n\n*Attributes*: Networked, Debuggable"]
            pub fn hide() -> Component<()> {
                *HIDE
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod vuph6dqdj6li4apmcgomn3faudcbfz56 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("vuph6dqdj6li4apmcgomn3faudcbfz56")
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
            static USE_THIRD_PERSON_CONTROLLER: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component(
                    "vuph6dqdj6li4apmcgomn3faudcbfz56::use_third_person_controller",
                )
            });
            #[doc = "**Use Third-Person Controller**: Use a third-person controller to drive this entity.\n\n*Attributes*: Debuggable, Networked"]
            pub fn use_third_person_controller() -> Component<()> {
                *USE_THIRD_PERSON_CONTROLLER
            }
            static PLAYER_CAMERA_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("vuph6dqdj6li4apmcgomn3faudcbfz56::player_camera_ref")
            });
            #[doc = "**Player camera ref**: The player's camera.\n\n*Attributes*: Debuggable"]
            pub fn player_camera_ref() -> Component<EntityId> {
                *PLAYER_CAMERA_REF
            }
            static CAMERA_DISTANCE: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("vuph6dqdj6li4apmcgomn3faudcbfz56::camera_distance")
            });
            #[doc = "**Camera distance**: The distance of the camera from the player's head.\n\n*Attributes*: Debuggable, Networked"]
            pub fn camera_distance() -> Component<f32> {
                *CAMERA_DISTANCE
            }
            static PLAYER_INTERMEDIATE_ROTATION: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component(
                    "vuph6dqdj6li4apmcgomn3faudcbfz56::player_intermediate_rotation",
                )
            });
            #[doc = "**player_intermediate_rotation**\n\n*Attributes*: Debuggable"]
            pub fn player_intermediate_rotation() -> Component<Vec2> {
                *PLAYER_INTERMEDIATE_ROTATION
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
            #[doc = "**ThirdPersonController**\n\n**Extends**: `lktsfudbjw2qikhyumt573ozxhadkiwm::CharacterMovement`, `ambient_core::transform::Transformable`\n\n**Required**:\n- `character_controller_height`: The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n- `character_controller_radius`: The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n- `physics_controlled`: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n- `rotation`: The rotation of this entity.\n- `run_direction`: No description provided.\n- `vertical_velocity`: The units's vertical speed.\n- `running`: No description provided.\n- `jumping`: No description provided.\n- `is_on_ground`: No description provided.\n- `local_to_world`: Transformation from the entity's local space to worldspace.\n- `use_third_person_controller`: Use a third-person controller to drive this entity.\n- `shooting`: No description provided.\n\n\n**Optional**:\n- `run_speed_multiplier`: The speed the unit can run at\n- `speed`: The speed the unit can walk at\n- `strafe_speed_multiplier`: The speed the unit can strafe at\n- `air_speed_multiplier`: When the unit is in the air; how much can it control its movement? If this is 0, it can't control it at all. If it's 1 it's the same as on the ground.\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity.\n- `scale`: The scale of this entity.\n- `head_ref`: No description provided."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct ThirdPersonController {
                #[doc = "**Component**: `ambient_core::physics::character_controller_height`\n\n**Suggested value**: `2f32`\n\n**Component description**: The height of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_radius`, it will be given a physical character collider.\n\n"]
                pub character_controller_height: f32,
                #[doc = "**Component**: `ambient_core::physics::character_controller_radius`\n\n**Suggested value**: `0.5f32`\n\n**Component description**: The radius of the physics character controller attached to this entity.\nIf an entity has both this and a `character_controller_height`, it will be given a physical character collider.\n\n"]
                pub character_controller_radius: f32,
                #[doc = "**Component**: `ambient_core::physics::physics_controlled`\n\n**Suggested value**: `()`\n\n**Component description**: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n\n"]
                pub physics_controlled: (),
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::run_direction`\n\n**Suggested value**: `Vec2::new(0f32, 0f32, )`\n\n"]
                pub run_direction: Vec2,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::vertical_velocity`\n\n**Suggested value**: `0f32`\n\n**Component description**: The units's vertical speed.\n\n"]
                pub vertical_velocity: f32,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::running`\n\n**Suggested value**: `false`\n\n"]
                pub running: bool,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::jumping`\n\n**Suggested value**: `false`\n\n"]
                pub jumping: bool,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::is_on_ground`\n\n**Suggested value**: `true`\n\n"]
                pub is_on_ground: bool,
                #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                pub local_to_world: Mat4,
                #[doc = "**Component**: `vuph6dqdj6li4apmcgomn3faudcbfz56::use_third_person_controller`\n\n**Suggested value**: `()`\n\n**Component description**: Use a third-person controller to drive this entity.\n\n"]
                pub use_third_person_controller: (),
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::shooting`\n\n**Suggested value**: `false`\n\n"]
                pub shooting: bool,
                #[doc = r" Optional components."]
                pub optional: ThirdPersonControllerOptional,
            }
            #[doc = "Optional part of [ThirdPersonController]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct ThirdPersonControllerOptional {
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::run_speed_multiplier`\n\n**Component description**: The speed the unit can run at\n\n"]
                pub run_speed_multiplier: Option<f32>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::speed`\n\n**Component description**: The speed the unit can walk at\n\n"]
                pub speed: Option<f32>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::strafe_speed_multiplier`\n\n**Component description**: The speed the unit can strafe at\n\n"]
                pub strafe_speed_multiplier: Option<f32>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::air_speed_multiplier`\n\n**Component description**: When the unit is in the air; how much can it control its movement? If this is 0, it can't control it at all. If it's 1 it's the same as on the ground.\n\n"]
                pub air_speed_multiplier: Option<f32>,
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Option<Vec3>,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Option<Quat>,
                #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                pub scale: Option<Vec3>,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::head_ref`\n\n"]
                pub head_ref: Option<EntityId>,
            }
            impl Concept for ThirdPersonController {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: physics :: components :: character_controller_height () , self . character_controller_height) . with (ambient_api :: core :: physics :: components :: character_controller_radius () , self . character_controller_radius) . with (ambient_api :: core :: physics :: components :: physics_controlled () , self . physics_controlled) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , self . run_direction) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , self . vertical_velocity) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , self . running) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , self . jumping) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground () , self . is_on_ground) . with (ambient_api :: core :: transform :: components :: local_to_world () , self . local_to_world) . with (crate :: packages :: raw :: vuph6dqdj6li4apmcgomn3faudcbfz56 :: components :: use_third_person_controller () , self . use_third_person_controller) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: shooting () , self . shooting) ;
                    if let Some(run_speed_multiplier) = self.optional.run_speed_multiplier {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , run_speed_multiplier) ;
                    }
                    if let Some(speed) = self.optional.speed {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , speed) ;
                    }
                    if let Some(strafe_speed_multiplier) = self.optional.strafe_speed_multiplier {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () , strafe_speed_multiplier) ;
                    }
                    if let Some(air_speed_multiplier) = self.optional.air_speed_multiplier {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier () , air_speed_multiplier) ;
                    }
                    if let Some(translation) = self.optional.translation {
                        entity.set(
                            ambient_api::core::transform::components::translation(),
                            translation,
                        );
                    }
                    if let Some(rotation) = self.optional.rotation {
                        entity.set(
                            ambient_api::core::transform::components::rotation(),
                            rotation,
                        );
                    }
                    if let Some(scale) = self.optional.scale {
                        entity.set(ambient_api::core::transform::components::scale(), scale);
                    }
                    if let Some(head_ref) = self.optional.head_ref {
                        entity . set (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: head_ref () , head_ref) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { character_controller_height : entity :: get_component (id , ambient_api :: core :: physics :: components :: character_controller_height ()) ? , character_controller_radius : entity :: get_component (id , ambient_api :: core :: physics :: components :: character_controller_radius ()) ? , physics_controlled : entity :: get_component (id , ambient_api :: core :: physics :: components :: physics_controlled ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , run_direction : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction ()) ? , vertical_velocity : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity ()) ? , running : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running ()) ? , jumping : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping ()) ? , is_on_ground : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground ()) ? , local_to_world : entity :: get_component (id , ambient_api :: core :: transform :: components :: local_to_world ()) ? , use_third_person_controller : entity :: get_component (id , crate :: packages :: raw :: vuph6dqdj6li4apmcgomn3faudcbfz56 :: components :: use_third_person_controller ()) ? , shooting : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: shooting ()) ? , optional : ThirdPersonControllerOptional { run_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) , speed : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) , strafe_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) , air_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier ()) , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) , scale : entity :: get_component (id , ambient_api :: core :: transform :: components :: scale ()) , head_ref : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: head_ref ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { character_controller_height : entity . get (ambient_api :: core :: physics :: components :: character_controller_height ()) ? , character_controller_radius : entity . get (ambient_api :: core :: physics :: components :: character_controller_radius ()) ? , physics_controlled : entity . get (ambient_api :: core :: physics :: components :: physics_controlled ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , run_direction : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction ()) ? , vertical_velocity : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity ()) ? , running : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running ()) ? , jumping : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping ()) ? , is_on_ground : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground ()) ? , local_to_world : entity . get (ambient_api :: core :: transform :: components :: local_to_world ()) ? , use_third_person_controller : entity . get (crate :: packages :: raw :: vuph6dqdj6li4apmcgomn3faudcbfz56 :: components :: use_third_person_controller ()) ? , shooting : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: shooting ()) ? , optional : ThirdPersonControllerOptional { run_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) , speed : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) , strafe_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) , air_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier ()) , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) , scale : entity . get (ambient_api :: core :: transform :: components :: scale ()) , head_ref : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: head_ref ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: physics :: components :: character_controller_height () , & ambient_api :: core :: physics :: components :: character_controller_radius () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground () , & ambient_api :: core :: transform :: components :: local_to_world () , & crate :: packages :: raw :: vuph6dqdj6li4apmcgomn3faudcbfz56 :: components :: use_third_person_controller () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: shooting ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: physics :: components :: character_controller_height () , & ambient_api :: core :: physics :: components :: character_controller_radius () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground () , & ambient_api :: core :: transform :: components :: local_to_world () , & crate :: packages :: raw :: vuph6dqdj6li4apmcgomn3faudcbfz56 :: components :: use_third_person_controller () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: shooting ()])
                }
            }
            impl ConceptSuggested for ThirdPersonController {
                #[doc = "```\ncharacter_controller_height: 2f32,\ncharacter_controller_radius: 0.5f32,\nphysics_controlled: (),\nrotation: Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, ),\nrun_direction: Vec2::new(0f32, 0f32, ),\nvertical_velocity: 0f32,\nrunning: false,\njumping: false,\nis_on_ground: true,\nlocal_to_world: Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ]),\nuse_third_person_controller: (),\nshooting: false,\n```"]
                fn suggested() -> Self {
                    Self {
                        character_controller_height: 2f32,
                        character_controller_radius: 0.5f32,
                        physics_controlled: (),
                        rotation: Quat::from_xyzw(0f32, 0f32, 0f32, 1f32),
                        run_direction: Vec2::new(0f32, 0f32),
                        vertical_velocity: 0f32,
                        running: false,
                        jumping: false,
                        is_on_ground: true,
                        local_to_world: Mat4::from_cols_array(&[
                            1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
                            0f32, 0f32, 0f32, 1f32,
                        ]),
                        use_third_person_controller: (),
                        shooting: false,
                        optional: Default::default(),
                    }
                }
            }
            impl ConceptComponents for ThirdPersonController {
                type Required = (
                    Component<f32>,
                    Component<f32>,
                    Component<()>,
                    Component<Quat>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<bool>,
                    Component<bool>,
                    Component<bool>,
                    Component<Mat4>,
                    Component<()>,
                    Component<bool>,
                );
                type Optional = (
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<Vec3>,
                    Component<Quat>,
                    Component<Vec3>,
                    Component<EntityId>,
                );
                fn required() -> Self::Required {
                    (ambient_api :: core :: physics :: components :: character_controller_height () , ambient_api :: core :: physics :: components :: character_controller_radius () , ambient_api :: core :: physics :: components :: physics_controlled () , ambient_api :: core :: transform :: components :: rotation () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_direction () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: vertical_velocity () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: running () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: jumping () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: is_on_ground () , ambient_api :: core :: transform :: components :: local_to_world () , crate :: packages :: raw :: vuph6dqdj6li4apmcgomn3faudcbfz56 :: components :: use_third_person_controller () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: shooting () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: air_speed_multiplier () , ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () , ambient_api :: core :: transform :: components :: scale () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: head_ref () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        character_controller_height: required.0,
                        character_controller_radius: required.1,
                        physics_controlled: required.2,
                        rotation: required.3,
                        run_direction: required.4,
                        vertical_velocity: required.5,
                        running: required.6,
                        jumping: required.7,
                        is_on_ground: required.8,
                        local_to_world: required.9,
                        use_third_person_controller: required.10,
                        shooting: required.11,
                        optional: Default::default(),
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
            #[doc = "**Input**: Describes the input state of the player."]
            pub struct Input {
                pub run_direction: Vec2,
                pub body_yaw: f32,
                pub head_pitch: f32,
                pub shooting: bool,
                pub ducking: bool,
                pub running: bool,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    run_direction: impl Into<Vec2>,
                    body_yaw: impl Into<f32>,
                    head_pitch: impl Into<f32>,
                    shooting: impl Into<bool>,
                    ducking: impl Into<bool>,
                    running: impl Into<bool>,
                ) -> Self {
                    Self {
                        run_direction: run_direction.into(),
                        body_yaw: body_yaw.into(),
                        head_pitch: head_pitch.into(),
                        shooting: shooting.into(),
                        ducking: ducking.into(),
                        running: running.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "vuph6dqdj6li4apmcgomn3faudcbfz56::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.run_direction.serialize_message_part(&mut output)?;
                    self.body_yaw.serialize_message_part(&mut output)?;
                    self.head_pitch.serialize_message_part(&mut output)?;
                    self.shooting.serialize_message_part(&mut output)?;
                    self.ducking.serialize_message_part(&mut output)?;
                    self.running.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        run_direction: Vec2::deserialize_message_part(&mut input)?,
                        body_yaw: f32::deserialize_message_part(&mut input)?,
                        head_pitch: f32::deserialize_message_part(&mut input)?,
                        shooting: bool::deserialize_message_part(&mut input)?,
                        ducking: bool::deserialize_message_part(&mut input)?,
                        running: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
            #[derive(Clone, Debug)]
            #[doc = "**Jump**"]
            pub struct Jump;
            impl Jump {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for Jump {
                fn id() -> &'static str {
                    "vuph6dqdj6li4apmcgomn3faudcbfz56::Jump"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for Jump {}
            impl Default for Jump {
                fn default() -> Self {
                    Self::new()
                }
            }
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
pub use raw::d3y3wbexrclipsykysumem3fthkudwx2 as character_animation;
pub use raw::ie4hdvkvui6t36yzniyex2hc5i2bradc as this;
pub use raw::n7a4j7htvenss35tsnfvegbhxuwij5il as base_assets;
pub use raw::per6j2iqhj3jz4da3fqr75jcj2kqjooo as nameplates;
pub use raw::vuph6dqdj6li4apmcgomn3faudcbfz56 as third_person_controller;
pub use raw::xar372tfo2oswb4pkvx7h7o3rxi6tap6 as hide_cursor;
