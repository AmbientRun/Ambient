#[allow(
    unused,
    clippy::unit_arg,
    clippy::let_and_return,
    clippy::approx_constant,
    clippy::unused_unit
)]
mod raw {
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
}
pub use raw::afl5yv5ya35vbuaj3aido22cwjzat25z as unit_schema;
pub use raw::lktsfudbjw2qikhyumt573ozxhadkiwm as this;
