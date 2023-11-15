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
    pub mod c72h7qyqnp4njboj7tu4vomadoj2zu6e {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("c72h7qyqnp4njboj7tu4vomadoj2zu6e")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
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
            #[doc = "**VehicleSpawnRequest**: Sent by the client to request a vehicle spawn."]
            pub struct VehicleSpawnRequest {
                pub def_id: EntityId,
            }
            impl VehicleSpawnRequest {
                #[allow(clippy::too_many_arguments)]
                pub fn new(def_id: impl Into<EntityId>) -> Self {
                    Self {
                        def_id: def_id.into(),
                    }
                }
            }
            impl Message for VehicleSpawnRequest {
                fn id() -> &'static str {
                    "c72h7qyqnp4njboj7tu4vomadoj2zu6e::VehicleSpawnRequest"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.def_id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        def_id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for VehicleSpawnRequest {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod cneomdouziieskjvs3szwmigzotofjzs {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("cneomdouziieskjvs3szwmigzotofjzs")
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
            static IS_EXPLOSION: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("cneomdouziieskjvs3szwmigzotofjzs::is_explosion")
            });
            #[doc = "**Is Explosion**: Is an explosion\n\n*Attributes*: Networked"]
            pub fn is_explosion() -> Component<()> {
                *IS_EXPLOSION
            }
            static RADIUS: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("cneomdouziieskjvs3szwmigzotofjzs::radius"));
            #[doc = "**Radius**: Radius of the explosion\n\n*Attributes*: Networked"]
            pub fn radius() -> Component<f32> {
                *RADIUS
            }
            static DAMAGE: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("cneomdouziieskjvs3szwmigzotofjzs::damage"));
            #[doc = "**Damage**: Damage of the explosion\n\n*Attributes*: Networked"]
            pub fn damage() -> Component<f32> {
                *DAMAGE
            }
            static CREATED_AT: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("cneomdouziieskjvs3szwmigzotofjzs::created_at")
            });
            #[doc = "**Created At**: Time the explosion was created. Must be manually attached using a spawn_query as time is not synchronized between client and server at time of writing."]
            pub fn created_at() -> Component<Duration> {
                *CREATED_AT
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
            #[doc = "**Explosion**: An explosion\n\n**Required**:\n- `is_explosion`: Is an explosion\n- `radius`: Radius of the explosion\n- `damage`: Damage of the explosion\n- `translation`: The translation/position of this entity.\n\n\n**Optional**:\n- `created_at`: Time the explosion was created. Must be manually attached using a spawn_query as time is not synchronized between client and server at time of writing."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Explosion {
                #[doc = "**Component**: `cneomdouziieskjvs3szwmigzotofjzs::is_explosion`\n\n**Component description**: Is an explosion\n\n"]
                pub is_explosion: (),
                #[doc = "**Component**: `cneomdouziieskjvs3szwmigzotofjzs::radius`\n\n**Component description**: Radius of the explosion\n\n"]
                pub radius: f32,
                #[doc = "**Component**: `cneomdouziieskjvs3szwmigzotofjzs::damage`\n\n**Component description**: Damage of the explosion\n\n"]
                pub damage: f32,
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = r" Optional components."]
                pub optional: ExplosionOptional,
            }
            #[doc = "Optional part of [Explosion]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct ExplosionOptional {
                #[doc = "**Component**: `cneomdouziieskjvs3szwmigzotofjzs::created_at`\n\n**Component description**: Time the explosion was created. Must be manually attached using a spawn_query as time is not synchronized between client and server at time of writing.\n\n"]
                pub created_at: Option<Duration>,
            }
            impl Concept for Explosion {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: is_explosion () , self . is_explosion) . with (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: radius () , self . radius) . with (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: damage () , self . damage) . with (ambient_api :: core :: transform :: components :: translation () , self . translation) ;
                    if let Some(created_at) = self.optional.created_at {
                        entity . set (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: created_at () , created_at) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_explosion : entity :: get_component (id , crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: is_explosion ()) ? , radius : entity :: get_component (id , crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: radius ()) ? , damage : entity :: get_component (id , crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: damage ()) ? , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , optional : ExplosionOptional { created_at : entity :: get_component (id , crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: created_at ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_explosion : entity . get (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: is_explosion ()) ? , radius : entity . get (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: radius ()) ? , damage : entity . get (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: damage ()) ? , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , optional : ExplosionOptional { created_at : entity . get (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: created_at ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: is_explosion () , & crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: radius () , & crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: damage () , & ambient_api :: core :: transform :: components :: translation ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: is_explosion () , & crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: radius () , & crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: damage () , & ambient_api :: core :: transform :: components :: translation ()])
                }
            }
            impl ConceptComponents for Explosion {
                type Required = (
                    Component<()>,
                    Component<f32>,
                    Component<f32>,
                    Component<Vec3>,
                );
                type Optional = (Component<Duration>,);
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: is_explosion () , crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: radius () , crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: damage () , ambient_api :: core :: transform :: components :: translation () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: cneomdouziieskjvs3szwmigzotofjzs :: components :: created_at () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_explosion: required.0,
                        radius: required.1,
                        damage: required.2,
                        translation: required.3,
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
    pub mod d7rxxncafgtwf7c3brhsw7oh7h2ccfip {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("d7rxxncafgtwf7c3brhsw7oh7h2ccfip")
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
    pub mod e4unr4x2lz2ov7dsd5vnjylbykwixvv2 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("e4unr4x2lz2ov7dsd5vnjylbykwixvv2")
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
    pub mod fvn74ns4ozf3gn42vmowphmvmpsfklbi {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("fvn74ns4ozf3gn42vmowphmvmpsfklbi")
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
    pub mod ggu2h7bk4jrvshq7zteboipyut7wuib2 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ggu2h7bk4jrvshq7zteboipyut7wuib2")
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
    pub mod gzbamly2shtnz3siisf3mdzglsi67vul {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("gzbamly2shtnz3siisf3mdzglsi67vul")
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
            static INCLUDE_CORNERS: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("gzbamly2shtnz3siisf3mdzglsi67vul::include_corners")
            });
            #[doc = "**Include Corners**: Whether or not the corner spawnpoints are created.\n\n*Attributes*: Debuggable, Networked\n\n*Suggested Default*: true"]
            pub fn include_corners() -> Component<bool> {
                *INCLUDE_CORNERS
            }
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod hr4pxz7kfhzgimicoyh65ydel3aehuhk {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("hr4pxz7kfhzgimicoyh65ydel3aehuhk")
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
            static MOD_MANAGER_FOR: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("hr4pxz7kfhzgimicoyh65ydel3aehuhk::mod_manager_for")
            });
            #[doc = "**Mod Manager For**: Package config component. Attach this component to this package's entity to make it a mod manager for the given package.\n\n*Attributes*: Networked, Debuggable"]
            pub fn mod_manager_for() -> Component<EntityId> {
                *MOD_MANAGER_FOR
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
            #[doc = "**PackageLoad**"]
            pub struct PackageLoad {
                pub url: String,
                pub enabled: bool,
            }
            impl PackageLoad {
                #[allow(clippy::too_many_arguments)]
                pub fn new(url: impl Into<String>, enabled: impl Into<bool>) -> Self {
                    Self {
                        url: url.into(),
                        enabled: enabled.into(),
                    }
                }
            }
            impl Message for PackageLoad {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoad"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.url.serialize_message_part(&mut output)?;
                    self.enabled.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        url: String::deserialize_message_part(&mut input)?,
                        enabled: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageLoad {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageLoadSuccess**"]
            pub struct PackageLoadSuccess {
                pub id: EntityId,
                pub name: String,
            }
            impl PackageLoadSuccess {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>, name: impl Into<String>) -> Self {
                    Self {
                        id: id.into(),
                        name: name.into(),
                    }
                }
            }
            impl Message for PackageLoadSuccess {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoadSuccess"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    self.name.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                        name: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageLoadSuccess {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageLoadFailure**"]
            pub struct PackageLoadFailure {
                pub reason: String,
            }
            impl PackageLoadFailure {
                #[allow(clippy::too_many_arguments)]
                pub fn new(reason: impl Into<String>) -> Self {
                    Self {
                        reason: reason.into(),
                    }
                }
            }
            impl Message for PackageLoadFailure {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoadFailure"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.reason.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        reason: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageLoadFailure {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageSetEnabled**"]
            pub struct PackageSetEnabled {
                pub id: EntityId,
                pub enabled: bool,
            }
            impl PackageSetEnabled {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>, enabled: impl Into<bool>) -> Self {
                    Self {
                        id: id.into(),
                        enabled: enabled.into(),
                    }
                }
            }
            impl Message for PackageSetEnabled {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageSetEnabled"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    self.enabled.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                        enabled: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageSetEnabled {}
            #[derive(Clone, Debug)]
            #[doc = "**WasmSetEnabled**"]
            pub struct WasmSetEnabled {
                pub id: EntityId,
                pub enabled: bool,
            }
            impl WasmSetEnabled {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>, enabled: impl Into<bool>) -> Self {
                    Self {
                        id: id.into(),
                        enabled: enabled.into(),
                    }
                }
            }
            impl Message for WasmSetEnabled {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::WasmSetEnabled"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    self.enabled.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                        enabled: bool::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for WasmSetEnabled {}
            #[derive(Clone, Debug)]
            #[doc = "**WasmReload**"]
            pub struct WasmReload {
                pub id: EntityId,
            }
            impl WasmReload {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>) -> Self {
                    Self { id: id.into() }
                }
            }
            impl Message for WasmReload {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::WasmReload"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for WasmReload {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageShow**: Shows the package view for a package"]
            pub struct PackageShow {
                pub id: EntityId,
            }
            impl PackageShow {
                #[allow(clippy::too_many_arguments)]
                pub fn new(id: impl Into<EntityId>) -> Self {
                    Self { id: id.into() }
                }
            }
            impl Message for PackageShow {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageShow"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageShow {}
            #[derive(Clone, Debug)]
            #[doc = "**PackageLoadShow**: Shows the package load view"]
            pub struct PackageLoadShow;
            impl PackageLoadShow {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for PackageLoadShow {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageLoadShow"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for PackageLoadShow {}
            impl Default for PackageLoadShow {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**PackageRemoteRequest**: Requests all relevant remote packages"]
            pub struct PackageRemoteRequest;
            impl PackageRemoteRequest {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for PackageRemoteRequest {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageRemoteRequest"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for PackageRemoteRequest {}
            impl Default for PackageRemoteRequest {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**PackageRemoteResponse**: Response to a remote package request"]
            pub struct PackageRemoteResponse {
                pub packages: Vec<String>,
                pub error: Option<String>,
            }
            impl PackageRemoteResponse {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    packages: impl Into<Vec<String>>,
                    error: impl Into<Option<String>>,
                ) -> Self {
                    Self {
                        packages: packages.into(),
                        error: error.into(),
                    }
                }
            }
            impl Message for PackageRemoteResponse {
                fn id() -> &'static str {
                    "hr4pxz7kfhzgimicoyh65ydel3aehuhk::PackageRemoteResponse"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.packages.serialize_message_part(&mut output)?;
                    self.error.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        packages: Vec::<String>::deserialize_message_part(&mut input)?,
                        error: Option::<String>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for PackageRemoteResponse {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod hs7ygpw4pmpsixtcohdcvzxwmrfzubvi {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("hs7ygpw4pmpsixtcohdcvzxwmrfzubvi")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
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
            #[doc = "**ClassSetRequest**: Sent by the client to request a class set change."]
            pub struct ClassSetRequest {
                pub class_id: EntityId,
            }
            impl ClassSetRequest {
                #[allow(clippy::too_many_arguments)]
                pub fn new(class_id: impl Into<EntityId>) -> Self {
                    Self {
                        class_id: class_id.into(),
                    }
                }
            }
            impl Message for ClassSetRequest {
                fn id() -> &'static str {
                    "hs7ygpw4pmpsixtcohdcvzxwmrfzubvi::ClassSetRequest"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.class_id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        class_id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for ClassSetRequest {}
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
    pub mod ianwyihfsaiuc26xjldmwd3duidju2tb {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ianwyihfsaiuc26xjldmwd3duidju2tb")
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
    pub mod itzh3wovmdje4ttrmo6wrravaaxp6b52 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("itzh3wovmdje4ttrmo6wrravaaxp6b52")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
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
            #[doc = "**Input**: Input"]
            pub struct Input {
                pub direction: Vec2,
                pub jump: bool,
                pub fire: bool,
                pub use_button: bool,
                pub sprint: bool,
                pub respawn: bool,
                pub aim_direction: Vec2,
                pub aim_ray_origin: Vec3,
                pub aim_ray_direction: Vec3,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    direction: impl Into<Vec2>,
                    jump: impl Into<bool>,
                    fire: impl Into<bool>,
                    use_button: impl Into<bool>,
                    sprint: impl Into<bool>,
                    respawn: impl Into<bool>,
                    aim_direction: impl Into<Vec2>,
                    aim_ray_origin: impl Into<Vec3>,
                    aim_ray_direction: impl Into<Vec3>,
                ) -> Self {
                    Self {
                        direction: direction.into(),
                        jump: jump.into(),
                        fire: fire.into(),
                        use_button: use_button.into(),
                        sprint: sprint.into(),
                        respawn: respawn.into(),
                        aim_direction: aim_direction.into(),
                        aim_ray_origin: aim_ray_origin.into(),
                        aim_ray_direction: aim_ray_direction.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "itzh3wovmdje4ttrmo6wrravaaxp6b52::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.direction.serialize_message_part(&mut output)?;
                    self.jump.serialize_message_part(&mut output)?;
                    self.fire.serialize_message_part(&mut output)?;
                    self.use_button.serialize_message_part(&mut output)?;
                    self.sprint.serialize_message_part(&mut output)?;
                    self.respawn.serialize_message_part(&mut output)?;
                    self.aim_direction.serialize_message_part(&mut output)?;
                    self.aim_ray_origin.serialize_message_part(&mut output)?;
                    self.aim_ray_direction.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        direction: Vec2::deserialize_message_part(&mut input)?,
                        jump: bool::deserialize_message_part(&mut input)?,
                        fire: bool::deserialize_message_part(&mut input)?,
                        use_button: bool::deserialize_message_part(&mut input)?,
                        sprint: bool::deserialize_message_part(&mut input)?,
                        respawn: bool::deserialize_message_part(&mut input)?,
                        aim_direction: Vec2::deserialize_message_part(&mut input)?,
                        aim_ray_origin: Vec3::deserialize_message_part(&mut input)?,
                        aim_ray_direction: Vec3::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
            #[derive(Clone, Debug)]
            #[doc = "**UseFailed**: Sent from the server to the client when a use action fails."]
            pub struct UseFailed;
            impl UseFailed {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for UseFailed {
                fn id() -> &'static str {
                    "itzh3wovmdje4ttrmo6wrravaaxp6b52::UseFailed"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for UseFailed {}
            impl Default for UseFailed {
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
    pub mod j32xi2gg5rvgob2cu7uirdbtj5ce4jw7 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("j32xi2gg5rvgob2cu7uirdbtj5ce4jw7")
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
    pub mod jyp2hh3fpjfe7kaos36zbdztfypqip3m {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("jyp2hh3fpjfe7kaos36zbdztfypqip3m")
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
            static IS_GUN_LASER: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("jyp2hh3fpjfe7kaos36zbdztfypqip3m::is_gun_laser")
            });
            #[doc = "**Is Gun (Laser)**: This entity is a laser gun."]
            pub fn is_gun_laser() -> Component<()> {
                *IS_GUN_LASER
            }
            static DAMAGE: Lazy<Component<f32>> =
                Lazy::new(|| __internal_get_component("jyp2hh3fpjfe7kaos36zbdztfypqip3m::damage"));
            #[doc = "**Damage**: The amount of damage this gun does."]
            pub fn damage() -> Component<f32> {
                *DAMAGE
            }
            static TIME_BETWEEN_SHOTS: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("jyp2hh3fpjfe7kaos36zbdztfypqip3m::time_between_shots")
            });
            #[doc = "**Time Between Shots**: The amount of time between shots."]
            pub fn time_between_shots() -> Component<Duration> {
                *TIME_BETWEEN_SHOTS
            }
            static LAST_SHOT_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                __internal_get_component("jyp2hh3fpjfe7kaos36zbdztfypqip3m::last_shot_time")
            });
            #[doc = "**Last Shot Time**: The time of the last shot."]
            pub fn last_shot_time() -> Component<Duration> {
                *LAST_SHOT_TIME
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
            #[doc = "**Gun Laser**: A laser gun.\n\n**Extends**: `ambient_core::transform::Transformable`\n\n**Required**:\n- `local_to_world`: Transformation from the entity's local space to worldspace.\n- `is_gun_laser`: This entity is a laser gun.\n- `damage`: The amount of damage this gun does.\n- `time_between_shots`: The amount of time between shots.\n\n\n**Optional**:\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity.\n- `scale`: The scale of this entity.\n- `last_shot_time`: The time of the last shot."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct GunLaser {
                #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Suggested value**: `Mat4::from_cols_array(&[1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, ])`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                pub local_to_world: Mat4,
                #[doc = "**Component**: `jyp2hh3fpjfe7kaos36zbdztfypqip3m::is_gun_laser`\n\n**Component description**: This entity is a laser gun.\n\n"]
                pub is_gun_laser: (),
                #[doc = "**Component**: `jyp2hh3fpjfe7kaos36zbdztfypqip3m::damage`\n\n**Component description**: The amount of damage this gun does.\n\n"]
                pub damage: f32,
                #[doc = "**Component**: `jyp2hh3fpjfe7kaos36zbdztfypqip3m::time_between_shots`\n\n**Component description**: The amount of time between shots.\n\n"]
                pub time_between_shots: Duration,
                #[doc = r" Optional components."]
                pub optional: GunLaserOptional,
            }
            #[doc = "Optional part of [GunLaser]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct GunLaserOptional {
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Suggested value**: `Vec3::new(0f32, 0f32, 0f32, )`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Option<Vec3>,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Suggested value**: `Quat::from_xyzw(0f32, 0f32, 0f32, 1f32, )`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Option<Quat>,
                #[doc = "**Component**: `ambient_core::transform::scale`\n\n**Suggested value**: `Vec3::new(1f32, 1f32, 1f32, )`\n\n**Component description**: The scale of this entity.\n\n"]
                pub scale: Option<Vec3>,
                #[doc = "**Component**: `jyp2hh3fpjfe7kaos36zbdztfypqip3m::last_shot_time`\n\n**Component description**: The time of the last shot.\n\n"]
                pub last_shot_time: Option<Duration>,
            }
            impl Concept for GunLaser {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: transform :: components :: local_to_world () , self . local_to_world) . with (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: is_gun_laser () , self . is_gun_laser) . with (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: damage () , self . damage) . with (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: time_between_shots () , self . time_between_shots) ;
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
                    if let Some(last_shot_time) = self.optional.last_shot_time {
                        entity . set (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: last_shot_time () , last_shot_time) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { local_to_world : entity :: get_component (id , ambient_api :: core :: transform :: components :: local_to_world ()) ? , is_gun_laser : entity :: get_component (id , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: is_gun_laser ()) ? , damage : entity :: get_component (id , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: damage ()) ? , time_between_shots : entity :: get_component (id , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: time_between_shots ()) ? , optional : GunLaserOptional { translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) , scale : entity :: get_component (id , ambient_api :: core :: transform :: components :: scale ()) , last_shot_time : entity :: get_component (id , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: last_shot_time ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { local_to_world : entity . get (ambient_api :: core :: transform :: components :: local_to_world ()) ? , is_gun_laser : entity . get (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: is_gun_laser ()) ? , damage : entity . get (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: damage ()) ? , time_between_shots : entity . get (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: time_between_shots ()) ? , optional : GunLaserOptional { translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) , scale : entity . get (ambient_api :: core :: transform :: components :: scale ()) , last_shot_time : entity . get (crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: last_shot_time ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: transform :: components :: local_to_world () , & crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: is_gun_laser () , & crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: damage () , & crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: time_between_shots ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: transform :: components :: local_to_world () , & crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: is_gun_laser () , & crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: damage () , & crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: time_between_shots ()])
                }
            }
            impl ConceptComponents for GunLaser {
                type Required = (
                    Component<Mat4>,
                    Component<()>,
                    Component<f32>,
                    Component<Duration>,
                );
                type Optional = (
                    Component<Vec3>,
                    Component<Quat>,
                    Component<Vec3>,
                    Component<Duration>,
                );
                fn required() -> Self::Required {
                    (ambient_api :: core :: transform :: components :: local_to_world () , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: is_gun_laser () , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: damage () , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: time_between_shots () ,)
                }
                fn optional() -> Self::Optional {
                    (ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () , ambient_api :: core :: transform :: components :: scale () , crate :: packages :: raw :: jyp2hh3fpjfe7kaos36zbdztfypqip3m :: components :: last_shot_time () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        local_to_world: required.0,
                        is_gun_laser: required.1,
                        damage: required.2,
                        time_between_shots: required.3,
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
            #[doc = "**Fire**: Sent to the client-side when a gun is fired."]
            pub struct Fire {
                pub weapon_id: EntityId,
            }
            impl Fire {
                #[allow(clippy::too_many_arguments)]
                pub fn new(weapon_id: impl Into<EntityId>) -> Self {
                    Self {
                        weapon_id: weapon_id.into(),
                    }
                }
            }
            impl Message for Fire {
                fn id() -> &'static str {
                    "jyp2hh3fpjfe7kaos36zbdztfypqip3m::Fire"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.weapon_id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        weapon_id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Fire {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod k7svgbw5j6orlwzj45koeownlodsdbth {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("k7svgbw5j6orlwzj45koeownlodsdbth")
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
    pub mod llhdryqkfsr6gy4f26wbolh4kfwnzn3c {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("llhdryqkfsr6gy4f26wbolh4kfwnzn3c")
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
    pub mod mkd4mhans4bdn3mvmt45vxqbxepdhys3 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("mkd4mhans4bdn3mvmt45vxqbxepdhys3")
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
    pub mod mnm43qv33k7kx235bz5hcgoguokwxzwo {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("mnm43qv33k7kx235bz5hcgoguokwxzwo")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
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
            #[doc = "**OnCollision**: Sent to the client when a vehicle collides with something."]
            pub struct OnCollision {
                pub position: Vec3,
                pub speed: f32,
                pub vehicle_id: EntityId,
            }
            impl OnCollision {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    position: impl Into<Vec3>,
                    speed: impl Into<f32>,
                    vehicle_id: impl Into<EntityId>,
                ) -> Self {
                    Self {
                        position: position.into(),
                        speed: speed.into(),
                        vehicle_id: vehicle_id.into(),
                    }
                }
            }
            impl Message for OnCollision {
                fn id() -> &'static str {
                    "mnm43qv33k7kx235bz5hcgoguokwxzwo::OnCollision"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.position.serialize_message_part(&mut output)?;
                    self.speed.serialize_message_part(&mut output)?;
                    self.vehicle_id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        position: Vec3::deserialize_message_part(&mut input)?,
                        speed: f32::deserialize_message_part(&mut input)?,
                        vehicle_id: EntityId::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for OnCollision {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod mwrcsok65na7owrbdococ5sthr3ccskc {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("mwrcsok65na7owrbdococ5sthr3ccskc")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
        }
        pub mod player {
            pub mod class {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static IS_CLASS: Lazy<Component<()>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::is_class",
                        )
                    });
                    #[doc = "**Is Class**: Is a player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn is_class() -> Component<()> {
                        *IS_CLASS
                    }
                    static NAME: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::name",
                        )
                    });
                    #[doc = "**Name**: Name of the player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn name() -> Component<String> {
                        *NAME
                    }
                    static DESCRIPTION: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::description",
                        )
                    });
                    #[doc = "**Description**: Description of the player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn description() -> Component<String> {
                        *DESCRIPTION
                    }
                    static ICON_URL: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::icon_url",
                        )
                    });
                    #[doc = "**Icon URL**: URL of the icon for the player class\n\n*Attributes*: Networked, Debuggable"]
                    pub fn icon_url() -> Component<String> {
                        *ICON_URL
                    }
                    static DEF_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::player::class::def_ref",
                        )
                    });
                    #[doc = "**Class Definition Ref**: When attached to a class, indicates that it should draw its properties from this entity.\n\n*Attributes*: Networked, Debuggable"]
                    pub fn def_ref() -> Component<EntityId> {
                        *DEF_REF
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
                static VEHICLE_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::vehicle_ref",
                    )
                });
                #[doc = "**Player's Vehicle**: The player's vehicle\n\n*Attributes*: Networked, Debuggable"]
                pub fn vehicle_ref() -> Component<EntityId> {
                    *VEHICLE_REF
                }
                static CHARACTER_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::character_ref",
                    )
                });
                #[doc = "**Player's Character**: The player's character\n\n*Attributes*: Networked, Debuggable"]
                pub fn character_ref() -> Component<EntityId> {
                    *CHARACTER_REF
                }
                static CLASS_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::class_ref")
                });
                #[doc = "**Class**: The player's class\n\n*Attributes*: Networked, Debuggable"]
                pub fn class_ref() -> Component<EntityId> {
                    *CLASS_REF
                }
                static INPUT_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_direction",
                    )
                });
                #[doc = "**Input Direction**: Input direction\n\n*Attributes*: Debuggable"]
                pub fn input_direction() -> Component<Vec2> {
                    *INPUT_DIRECTION
                }
                static INPUT_JUMP: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::input_jump")
                });
                #[doc = "**Jump**: Jump\n\n*Attributes*: Debuggable"]
                pub fn input_jump() -> Component<bool> {
                    *INPUT_JUMP
                }
                static INPUT_FIRE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::input_fire")
                });
                #[doc = "**Fire**: Fire\n\n*Attributes*: Debuggable"]
                pub fn input_fire() -> Component<bool> {
                    *INPUT_FIRE
                }
                static INPUT_USE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::player::input_use")
                });
                #[doc = "**Use**: Use\n\n*Attributes*: Debuggable"]
                pub fn input_use() -> Component<bool> {
                    *INPUT_USE
                }
                static INPUT_SPRINT: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_sprint",
                    )
                });
                #[doc = "**Sprint**: Sprint\n\n*Attributes*: Debuggable"]
                pub fn input_sprint() -> Component<bool> {
                    *INPUT_SPRINT
                }
                static INPUT_RESPAWN: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_respawn",
                    )
                });
                #[doc = "**Respawn**: Respawn\n\n*Attributes*: Debuggable"]
                pub fn input_respawn() -> Component<bool> {
                    *INPUT_RESPAWN
                }
                static INPUT_AIM_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_aim_direction",
                    )
                });
                #[doc = "**Aim Direction**: Aim direction in degrees from the centre\n\n*Attributes*: Debuggable"]
                pub fn input_aim_direction() -> Component<Vec2> {
                    *INPUT_AIM_DIRECTION
                }
                static INPUT_AIM_RAY_ORIGIN: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_aim_ray_origin",
                    )
                });
                #[doc = "**Aim Ray Origin**: Origin of the aim ray\n\n*Attributes*: Debuggable"]
                pub fn input_aim_ray_origin() -> Component<Vec3> {
                    *INPUT_AIM_RAY_ORIGIN
                }
                static INPUT_AIM_RAY_DIRECTION: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::player::input_aim_ray_direction",
                    )
                });
                #[doc = "**Aim Ray Direction**: Direction of the aim ray\n\n*Attributes*: Debuggable"]
                pub fn input_aim_ray_direction() -> Component<Vec3> {
                    *INPUT_AIM_RAY_DIRECTION
                }
            }
        }
        pub mod character {
            pub mod head {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static CAMERA_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::character::head::camera_ref",
                        )
                    });
                    #[doc = "**Head Camera**: The camera attached to the character's head\n\n*Attributes*: Networked, Debuggable"]
                    pub fn camera_ref() -> Component<EntityId> {
                        *CAMERA_REF
                    }
                }
            }
            pub mod def {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static MODEL_URL: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::character::def::model_url",
                        )
                    });
                    #[doc = "**Model URL**: URL of the model for the character def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn model_url() -> Component<String> {
                        *MODEL_URL
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
                static IS_CHARACTER: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::character::is_character",
                    )
                });
                #[doc = "**Is Character**: Is a player character\n\n*Attributes*: Networked, Debuggable"]
                pub fn is_character() -> Component<()> {
                    *IS_CHARACTER
                }
                static PLAYER_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::character::player_ref",
                    )
                });
                #[doc = "**Character's Player**: The player controlling the character\n\n*Attributes*: Networked, Debuggable"]
                pub fn player_ref() -> Component<EntityId> {
                    *PLAYER_REF
                }
                static LAST_USE_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::character::last_use_time",
                    )
                });
                #[doc = "**Last Use Time**: The last time the player tried to use something\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_use_time() -> Component<Duration> {
                    *LAST_USE_TIME
                }
                static DEF_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::character::def_ref")
                });
                #[doc = "**Character Definition Ref**: When attached to a character, indicates that it should draw its properties from this entity.\n\n*Attributes*: Networked, Debuggable"]
                pub fn def_ref() -> Component<EntityId> {
                    *DEF_REF
                }
            }
        }
        pub mod vehicle {
            pub mod def {
                pub mod thruster {
                    #[doc = r" Auto-generated component definitions."]
                    pub mod components {
                        use ambient_api::{
                            ecs::{Component, __internal_get_component},
                            once_cell::sync::Lazy,
                            prelude::*,
                        };
                        static OFFSETS: Lazy<Component<Vec<Vec2>>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::offsets",
                            )
                        });
                        #[doc = "**Thruster Offsets**: Offsets of the thrusters from the center of the vehicle\n\n*Attributes*: Networked, Debuggable"]
                        pub fn offsets() -> Component<Vec<Vec2>> {
                            *OFFSETS
                        }
                        static K_P: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_p",
                            )
                        });
                        #[doc = "**Thruster K_p**: Proportional gain for the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn k_p() -> Component<f32> {
                            *K_P
                        }
                        static K_D: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_d",
                            )
                        });
                        #[doc = "**Thruster K_d**: Derivative gain for the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn k_d() -> Component<f32> {
                            *K_D
                        }
                        static TARGET: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::target",
                            )
                        });
                        #[doc = "**Thruster Target Altitude**: Target altitude for the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn target() -> Component<f32> {
                            *TARGET
                        }
                        static MAX_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::max_strength")
                        });
                        #[doc = "**Thruster Max Strength**: Maximum strength of the thrusters\n\n*Attributes*: Networked, Debuggable"]
                        pub fn max_strength() -> Component<f32> {
                            *MAX_STRENGTH
                        }
                    }
                }
                pub mod input {
                    #[doc = r" Auto-generated component definitions."]
                    pub mod components {
                        use ambient_api::{
                            ecs::{Component, __internal_get_component},
                            once_cell::sync::Lazy,
                            prelude::*,
                        };
                        static FORWARD_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_force")
                        });
                        #[doc = "**Forward Force**: Forward force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn forward_force() -> Component<f32> {
                            *FORWARD_FORCE
                        }
                        static BACKWARD_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::backward_force")
                        });
                        #[doc = "**Backward Force**: Backward force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn backward_force() -> Component<f32> {
                            *BACKWARD_FORCE
                        }
                        static FORWARD_OFFSET: Lazy<Component<Vec2>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_offset")
                        });
                        #[doc = "**Forward Offset**: Offset of the forward force from the center of the vehicle. Typically at the back of the vehicle.\n\n*Attributes*: Networked, Debuggable"]
                        pub fn forward_offset() -> Component<Vec2> {
                            *FORWARD_OFFSET
                        }
                        static SIDE_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_force",
                            )
                        });
                        #[doc = "**Side Force**: Side force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn side_force() -> Component<f32> {
                            *SIDE_FORCE
                        }
                        static SIDE_OFFSET: Lazy<Component<Vec2>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_offset")
                        });
                        #[doc = "**Side Offset**: Offset of the side force from the center of the vehicle. Typically at the front of the vehicle.\n\n*Attributes*: Networked, Debuggable"]
                        pub fn side_offset() -> Component<Vec2> {
                            *SIDE_OFFSET
                        }
                        static JUMP_FORCE: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component(
                                "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_force",
                            )
                        });
                        #[doc = "**Jump Force**: Jump force applied to the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn jump_force() -> Component<f32> {
                            *JUMP_FORCE
                        }
                        static PITCH_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::pitch_strength")
                        });
                        #[doc = "**Pitch Strength**: Strength of the pitch applied to the applicable thrusters of the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn pitch_strength() -> Component<f32> {
                            *PITCH_STRENGTH
                        }
                        static TURNING_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::turning_strength")
                        });
                        #[doc = "**Turning Strength**: Strength of the turning applied to the applicable thrusters of the vehicle on input\n\n*Attributes*: Networked, Debuggable"]
                        pub fn turning_strength() -> Component<f32> {
                            *TURNING_STRENGTH
                        }
                        static JUMP_TIMEOUT: Lazy<Component<Duration>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_timeout")
                        });
                        #[doc = "**Jump Timeout**: Timeout between jumps\n\n*Attributes*: Networked, Debuggable"]
                        pub fn jump_timeout() -> Component<Duration> {
                            *JUMP_TIMEOUT
                        }
                        static AIM_DIRECTION_LIMITS: Lazy<Component<Vec2>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::aim_direction_limits")
                        });
                        #[doc = "**Aim Direction Limits**: Limits of the aim direction in degrees from the centre\n\n*Attributes*: Networked, Debuggable"]
                        pub fn aim_direction_limits() -> Component<Vec2> {
                            *AIM_DIRECTION_LIMITS
                        }
                    }
                }
                pub mod slowdown {
                    #[doc = r" Auto-generated component definitions."]
                    pub mod components {
                        use ambient_api::{
                            ecs::{Component, __internal_get_component},
                            once_cell::sync::Lazy,
                            prelude::*,
                        };
                        static LINEAR_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::linear_strength")
                        });
                        #[doc = "**Linear Strength**: Strength of the linear slowdown applied to the vehicle\n\n*Attributes*: Networked, Debuggable"]
                        pub fn linear_strength() -> Component<f32> {
                            *LINEAR_STRENGTH
                        }
                        static ANGULAR_STRENGTH: Lazy<Component<f32>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_strength")
                        });
                        #[doc = "**Angular Strength**: Strength of the angular slowdown applied to the vehicle\n\n*Attributes*: Networked, Debuggable"]
                        pub fn angular_strength() -> Component<f32> {
                            *ANGULAR_STRENGTH
                        }
                        static ANGULAR_DELAY: Lazy<Component<Duration>> = Lazy::new(|| {
                            __internal_get_component ("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_delay")
                        });
                        #[doc = "**Angular Delay**: Time to wait before applying angular slowdown\n\n*Attributes*: Networked, Debuggable"]
                        pub fn angular_delay() -> Component<Duration> {
                            *ANGULAR_DELAY
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
                    static IS_DEF: Lazy<Component<()>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::is_def",
                        )
                    });
                    #[doc = "**Is Def**: Is a vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn is_def() -> Component<()> {
                        *IS_DEF
                    }
                    static NAME: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::name",
                        )
                    });
                    #[doc = "**Name**: Name of the vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn name() -> Component<String> {
                        *NAME
                    }
                    static MODEL_URL: Lazy<Component<String>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_url",
                        )
                    });
                    #[doc = "**Model URL**: URL of the model for the vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn model_url() -> Component<String> {
                        *MODEL_URL
                    }
                    static MODEL_SCALE: Lazy<Component<f32>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_scale",
                        )
                    });
                    #[doc = "**Model Scale**: Scale of the model for the vehicle def\n\n*Attributes*: Networked, Debuggable"]
                    pub fn model_scale() -> Component<f32> {
                        *MODEL_SCALE
                    }
                }
            }
            pub mod client {
                #[doc = r" Auto-generated component definitions."]
                pub mod components {
                    use ambient_api::{
                        ecs::{Component, __internal_get_component},
                        once_cell::sync::Lazy,
                        prelude::*,
                    };
                    static SPEED_KPH: Lazy<Component<f32>> = Lazy::new(|| {
                        __internal_get_component(
                            "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::client::speed_kph",
                        )
                    });
                    #[doc = "**Speed (KPH)**: Speed of the vehicle in kilometers per hour."]
                    pub fn speed_kph() -> Component<f32> {
                        *SPEED_KPH
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
                static IS_VEHICLE: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::is_vehicle",
                    )
                });
                #[doc = "**Is Vehicle**: Is a vehicle\n\n*Attributes*: Networked, Debuggable"]
                pub fn is_vehicle() -> Component<()> {
                    *IS_VEHICLE
                }
                static DRIVER_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::driver_ref",
                    )
                });
                #[doc = "**Vehicle Driver**: A vehicle's driver, if present\n\n*Attributes*: Networked, Debuggable"]
                pub fn driver_ref() -> Component<EntityId> {
                    *DRIVER_REF
                }
                static DEF_REF: Lazy<Component<EntityId>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def_ref")
                });
                #[doc = "**Vehicle Definition Ref**: When attached to a vehicle entity, indicates that it should draw its properties from this entity.\n\n*Attributes*: Networked, Debuggable"]
                pub fn def_ref() -> Component<EntityId> {
                    *DEF_REF
                }
                static AIMABLE_WEAPON_REFS: Lazy<Component<Vec<EntityId>>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aimable_weapon_refs",
                    )
                });
                #[doc = "**Aimable Weapon Refs**: All aimable weapons for this vehicle. If present in this array, their rotations will be automatically updated to match the aim position. Assumes that the weapons are a child of the vehicle.\n\n*Attributes*: Networked, Debuggable"]
                pub fn aimable_weapon_refs() -> Component<Vec<EntityId>> {
                    *AIMABLE_WEAPON_REFS
                }
                static LAST_DISTANCES: Lazy<Component<Vec<f32>>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_distances",
                    )
                });
                #[doc = "**Last Distances**: Distances from the ground for each vehicle probe\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_distances() -> Component<Vec<f32>> {
                    *LAST_DISTANCES
                }
                static LAST_JUMP_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_jump_time",
                    )
                });
                #[doc = "**Last Jump Time**: The last time the player jumped\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_jump_time() -> Component<Duration> {
                    *LAST_JUMP_TIME
                }
                static LAST_SLOWDOWN_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_slowdown_time",
                    )
                });
                #[doc = "**Last Slowdown Time**: The last time the player's vehicle was slowed down\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_slowdown_time() -> Component<Duration> {
                    *LAST_SLOWDOWN_TIME
                }
                static LAST_UPSIDE_DOWN_TIME: Lazy<Component<Duration>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_upside_down_time",
                    )
                });
                #[doc = "**Last Upside Down Time**: The last time the player's vehicle was known to be upside down. Added when first detected, removed when no longer the case. Used to respawn the vehicle after some time.\n\n*Attributes*: Networked, Debuggable"]
                pub fn last_upside_down_time() -> Component<Duration> {
                    *LAST_UPSIDE_DOWN_TIME
                }
                static INPUT_DIRECTION: Lazy<Component<Vec2>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_direction",
                    )
                });
                #[doc = "**Input Direction**: Input direction\n\n*Attributes*: Debuggable"]
                pub fn input_direction() -> Component<Vec2> {
                    *INPUT_DIRECTION
                }
                static INPUT_JUMP: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_jump",
                    )
                });
                #[doc = "**Jump**: Jump\n\n*Attributes*: Debuggable"]
                pub fn input_jump() -> Component<bool> {
                    *INPUT_JUMP
                }
                static INPUT_FIRE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_fire",
                    )
                });
                #[doc = "**Fire**: Fire\n\n*Attributes*: Debuggable"]
                pub fn input_fire() -> Component<bool> {
                    *INPUT_FIRE
                }
                static AIM_POSITION: Lazy<Component<Vec3>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aim_position",
                    )
                });
                #[doc = "**Aim Position**: Position of the aim target\n\n*Attributes*: Debuggable"]
                pub fn aim_position() -> Component<Vec3> {
                    *AIM_POSITION
                }
            }
        }
        pub mod spawnpoint {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static IS_SPAWNPOINT: Lazy<Component<()>> = Lazy::new(|| {
                    __internal_get_component(
                        "mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::is_spawnpoint",
                    )
                });
                #[doc = "**Is Spawnpoint**: Is a spawnpoint\n\n*Attributes*: Networked, Debuggable"]
                pub fn is_spawnpoint() -> Component<()> {
                    *IS_SPAWNPOINT
                }
                static RADIUS: Lazy<Component<f32>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::radius")
                });
                #[doc = "**Radius**: Radius of the spawnpoint\n\n*Attributes*: Networked, Debuggable"]
                pub fn radius() -> Component<f32> {
                    *RADIUS
                }
            }
        }
        pub mod weapon {
            #[doc = r" Auto-generated component definitions."]
            pub mod components {
                use ambient_api::{
                    ecs::{Component, __internal_get_component},
                    once_cell::sync::Lazy,
                    prelude::*,
                };
                static FIRE: Lazy<Component<bool>> = Lazy::new(|| {
                    __internal_get_component("mwrcsok65na7owrbdococ5sthr3ccskc::weapon::fire")
                });
                #[doc = "**Fire**: When on, the weapon will attempt to fire.\n\n*Attributes*: Networked, Debuggable"]
                pub fn fire() -> Component<bool> {
                    *FIRE
                }
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
            #[doc = "**Player Class**: A player class\n\n**Required**:\n- `is_class`: Is a player class\n- `name`: Name of the player class\n- `description`: Description of the player class\n- `icon_url`: URL of the icon for the player class\n- `def_ref`: When attached to a class, indicates that it should draw its properties from this entity."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct PlayerClass {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::is_class`\n\n**Component description**: Is a player class\n\n"]
                pub is_class: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::name`\n\n**Component description**: Name of the player class\n\n"]
                pub name: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::description`\n\n**Component description**: Description of the player class\n\n"]
                pub description: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::icon_url`\n\n**Component description**: URL of the icon for the player class\n\n"]
                pub icon_url: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::player::class::def_ref`\n\n**Component description**: When attached to a class, indicates that it should draw its properties from this entity.\n\n"]
                pub def_ref: EntityId,
            }
            impl Concept for PlayerClass {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , self . is_class) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , self . name) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , self . description) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , self . icon_url) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref () , self . def_ref) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_class : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class ()) ? , name : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name ()) ? , description : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description ()) ? , icon_url : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url ()) ? , def_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_class : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class ()) ? , name : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name ()) ? , description : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description ()) ? , icon_url : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url ()) ? , def_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref ()])
                }
            }
            impl ConceptComponents for PlayerClass {
                type Required = (
                    Component<()>,
                    Component<String>,
                    Component<String>,
                    Component<String>,
                    Component<EntityId>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: is_class () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: name () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: description () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: icon_url () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: player :: class :: components :: def_ref () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_class: required.0,
                        name: required.1,
                        description: required.2,
                        icon_url: required.3,
                        def_ref: required.4,
                    }
                }
            }
            #[doc = "**Character**: A character with a physical representation\n\n**Required**:\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity.\n- `health`: This game object's health. \"Standard\" health is 100 HP.\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `is_character`: Is a player character\n- `player_ref`: The player controlling the character\n- `def_ref`: When attached to a character, indicates that it should draw its properties from this entity.\n\n\n**Optional**:\n- `last_use_time`: The last time the player tried to use something"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Character {
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::health`\n\n**Component description**: This game object's health. \"Standard\" health is 100 HP.\n\n"]
                pub health: f32,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::is_character`\n\n**Component description**: Is a player character\n\n"]
                pub is_character: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::player_ref`\n\n**Component description**: The player controlling the character\n\n"]
                pub player_ref: EntityId,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::def_ref`\n\n**Component description**: When attached to a character, indicates that it should draw its properties from this entity.\n\n"]
                pub def_ref: EntityId,
                #[doc = r" Optional components."]
                pub optional: CharacterOptional,
            }
            #[doc = "Optional part of [Character]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct CharacterOptional {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::last_use_time`\n\n**Component description**: The last time the player tried to use something\n\n"]
                pub last_use_time: Option<Duration>,
            }
            impl Concept for Character {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , self . health) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , self . is_character) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , self . player_ref) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref () , self . def_ref) ;
                    if let Some(last_use_time) = self.optional.last_use_time {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time () , last_use_time) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_character : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character ()) ? , player_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref ()) ? , def_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()) ? , optional : CharacterOptional { last_use_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_character : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character ()) ? , player_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref ()) ? , def_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()) ? , optional : CharacterOptional { last_use_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref ()])
                }
            }
            impl ConceptComponents for Character {
                type Required = (
                    Component<Vec3>,
                    Component<Quat>,
                    Component<f32>,
                    Component<f32>,
                    Component<()>,
                    Component<EntityId>,
                    Component<EntityId>,
                );
                type Optional = (Component<Duration>,);
                fn required() -> Self::Required {
                    (ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: is_character () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: player_ref () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: def_ref () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: components :: last_use_time () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        translation: required.0,
                        rotation: required.1,
                        health: required.2,
                        max_health: required.3,
                        is_character: required.4,
                        player_ref: required.5,
                        def_ref: required.6,
                        optional: Default::default(),
                    }
                }
            }
            #[doc = "**Character Definition**: Definition for a character\n\n**Required**:\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `model_url`: URL of the model for the character def\n- `speed`: The speed the unit can walk at\n- `run_speed_multiplier`: The speed the unit can run at\n- `strafe_speed_multiplier`: The speed the unit can strafe at"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct CharacterDef {
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::character::def::model_url`\n\n**Component description**: URL of the model for the character def\n\n"]
                pub model_url: String,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::speed`\n\n**Component description**: The speed the unit can walk at\n\n"]
                pub speed: f32,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::run_speed_multiplier`\n\n**Component description**: The speed the unit can run at\n\n"]
                pub run_speed_multiplier: f32,
                #[doc = "**Component**: `afl5yv5ya35vbuaj3aido22cwjzat25z::strafe_speed_multiplier`\n\n**Component description**: The speed the unit can strafe at\n\n"]
                pub strafe_speed_multiplier: f32,
            }
            impl Concept for CharacterDef {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , self . model_url) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , self . speed) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , self . run_speed_multiplier) . with (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () , self . strafe_speed_multiplier) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , model_url : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url ()) ? , speed : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) ? , run_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) ? , strafe_speed_multiplier : entity :: get_component (id , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , model_url : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url ()) ? , speed : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed ()) ? , run_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier ()) ? , strafe_speed_multiplier : entity . get (crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , & crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier ()])
                }
            }
            impl ConceptComponents for CharacterDef {
                type Required = (
                    Component<f32>,
                    Component<String>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: character :: def :: components :: model_url () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: speed () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: run_speed_multiplier () , crate :: packages :: raw :: afl5yv5ya35vbuaj3aido22cwjzat25z :: components :: strafe_speed_multiplier () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        max_health: required.0,
                        model_url: required.1,
                        speed: required.2,
                        run_speed_multiplier: required.3,
                        strafe_speed_multiplier: required.4,
                    }
                }
            }
            #[doc = "**Vehicle Definition**: Definition for a vehicle\n\n**Required**:\n- `density`: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n- `cube_collider`: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `offsets`: Offsets of the thrusters from the center of the vehicle\n- `k_p`: Proportional gain for the thrusters\n- `k_d`: Derivative gain for the thrusters\n- `target`: Target altitude for the thrusters\n- `max_strength`: Maximum strength of the thrusters\n- `forward_force`: Forward force applied to the vehicle on input\n- `backward_force`: Backward force applied to the vehicle on input\n- `forward_offset`: Offset of the forward force from the center of the vehicle. Typically at the back of the vehicle.\n- `side_force`: Side force applied to the vehicle on input\n- `side_offset`: Offset of the side force from the center of the vehicle. Typically at the front of the vehicle.\n- `jump_force`: Jump force applied to the vehicle on input\n- `aim_direction_limits`: Limits of the aim direction in degrees from the centre\n- `pitch_strength`: Strength of the pitch applied to the applicable thrusters of the vehicle on input\n- `turning_strength`: Strength of the turning applied to the applicable thrusters of the vehicle on input\n- `jump_timeout`: Timeout between jumps\n- `linear_strength`: Strength of the linear slowdown applied to the vehicle\n- `angular_strength`: Strength of the angular slowdown applied to the vehicle\n- `angular_delay`: Time to wait before applying angular slowdown\n- `is_def`: Is a vehicle def\n- `name`: Name of the vehicle def\n- `model_url`: URL of the model for the vehicle def\n- `model_scale`: Scale of the model for the vehicle def"]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct VehicleDef {
                #[doc = "**Component**: `ambient_core::physics::density`\n\n**Component description**: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n\n"]
                pub density: f32,
                #[doc = "**Component**: `ambient_core::physics::cube_collider`\n\n**Component description**: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n\n"]
                pub cube_collider: Vec3,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::offsets`\n\n**Component description**: Offsets of the thrusters from the center of the vehicle\n\n"]
                pub offsets: Vec<Vec2>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_p`\n\n**Component description**: Proportional gain for the thrusters\n\n"]
                pub k_p: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::k_d`\n\n**Component description**: Derivative gain for the thrusters\n\n"]
                pub k_d: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::target`\n\n**Component description**: Target altitude for the thrusters\n\n"]
                pub target: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::thruster::max_strength`\n\n**Component description**: Maximum strength of the thrusters\n\n"]
                pub max_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_force`\n\n**Component description**: Forward force applied to the vehicle on input\n\n"]
                pub forward_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::backward_force`\n\n**Component description**: Backward force applied to the vehicle on input\n\n"]
                pub backward_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::forward_offset`\n\n**Component description**: Offset of the forward force from the center of the vehicle. Typically at the back of the vehicle.\n\n"]
                pub forward_offset: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_force`\n\n**Component description**: Side force applied to the vehicle on input\n\n"]
                pub side_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::side_offset`\n\n**Component description**: Offset of the side force from the center of the vehicle. Typically at the front of the vehicle.\n\n"]
                pub side_offset: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_force`\n\n**Component description**: Jump force applied to the vehicle on input\n\n"]
                pub jump_force: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::aim_direction_limits`\n\n**Component description**: Limits of the aim direction in degrees from the centre\n\n"]
                pub aim_direction_limits: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::pitch_strength`\n\n**Component description**: Strength of the pitch applied to the applicable thrusters of the vehicle on input\n\n"]
                pub pitch_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::turning_strength`\n\n**Component description**: Strength of the turning applied to the applicable thrusters of the vehicle on input\n\n"]
                pub turning_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::input::jump_timeout`\n\n**Component description**: Timeout between jumps\n\n"]
                pub jump_timeout: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::linear_strength`\n\n**Component description**: Strength of the linear slowdown applied to the vehicle\n\n"]
                pub linear_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_strength`\n\n**Component description**: Strength of the angular slowdown applied to the vehicle\n\n"]
                pub angular_strength: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::slowdown::angular_delay`\n\n**Component description**: Time to wait before applying angular slowdown\n\n"]
                pub angular_delay: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::is_def`\n\n**Component description**: Is a vehicle def\n\n"]
                pub is_def: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::name`\n\n**Component description**: Name of the vehicle def\n\n"]
                pub name: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_url`\n\n**Component description**: URL of the model for the vehicle def\n\n"]
                pub model_url: String,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def::model_scale`\n\n**Component description**: Scale of the model for the vehicle def\n\n"]
                pub model_scale: f32,
            }
            impl Concept for VehicleDef {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: physics :: components :: density () , self . density) . with (ambient_api :: core :: physics :: components :: cube_collider () , self . cube_collider) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , self . offsets) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , self . k_p) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , self . k_d) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , self . target) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , self . max_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , self . forward_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , self . backward_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , self . forward_offset) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , self . side_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , self . side_offset) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , self . jump_force) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , self . aim_direction_limits) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , self . pitch_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , self . turning_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , self . jump_timeout) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , self . linear_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , self . angular_strength) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , self . angular_delay) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , self . is_def) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , self . name) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , self . model_url) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale () , self . model_scale) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { density : entity :: get_component (id , ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity :: get_component (id , ambient_api :: core :: physics :: components :: cube_collider ()) ? , max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , offsets : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets ()) ? , k_p : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p ()) ? , k_d : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d ()) ? , target : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target ()) ? , max_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength ()) ? , forward_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force ()) ? , backward_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force ()) ? , forward_offset : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset ()) ? , side_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force ()) ? , side_offset : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset ()) ? , jump_force : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force ()) ? , aim_direction_limits : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits ()) ? , pitch_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength ()) ? , turning_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength ()) ? , jump_timeout : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout ()) ? , linear_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength ()) ? , angular_strength : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength ()) ? , angular_delay : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay ()) ? , is_def : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def ()) ? , name : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name ()) ? , model_url : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url ()) ? , model_scale : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { density : entity . get (ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity . get (ambient_api :: core :: physics :: components :: cube_collider ()) ? , max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , offsets : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets ()) ? , k_p : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p ()) ? , k_d : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d ()) ? , target : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target ()) ? , max_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength ()) ? , forward_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force ()) ? , backward_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force ()) ? , forward_offset : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset ()) ? , side_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force ()) ? , side_offset : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset ()) ? , jump_force : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force ()) ? , aim_direction_limits : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits ()) ? , pitch_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength ()) ? , turning_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength ()) ? , jump_timeout : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout ()) ? , linear_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength ()) ? , angular_strength : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength ()) ? , angular_delay : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay ()) ? , is_def : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def ()) ? , name : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name ()) ? , model_url : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url ()) ? , model_scale : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale ()])
                }
            }
            impl ConceptComponents for VehicleDef {
                type Required = (
                    Component<f32>,
                    Component<Vec3>,
                    Component<f32>,
                    Component<Vec<Vec2>>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<f32>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<Vec2>,
                    Component<f32>,
                    Component<f32>,
                    Component<Duration>,
                    Component<f32>,
                    Component<f32>,
                    Component<Duration>,
                    Component<()>,
                    Component<String>,
                    Component<String>,
                    Component<f32>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (ambient_api :: core :: physics :: components :: density () , ambient_api :: core :: physics :: components :: cube_collider () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: offsets () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_p () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: k_d () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: target () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: thruster :: components :: max_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: backward_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: forward_offset () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: side_offset () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_force () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: aim_direction_limits () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: pitch_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: turning_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: input :: components :: jump_timeout () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: linear_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_strength () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: slowdown :: components :: angular_delay () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: is_def () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: name () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_url () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: def :: components :: model_scale () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        density: required.0,
                        cube_collider: required.1,
                        max_health: required.2,
                        offsets: required.3,
                        k_p: required.4,
                        k_d: required.5,
                        target: required.6,
                        max_strength: required.7,
                        forward_force: required.8,
                        backward_force: required.9,
                        forward_offset: required.10,
                        side_force: required.11,
                        side_offset: required.12,
                        jump_force: required.13,
                        aim_direction_limits: required.14,
                        pitch_strength: required.15,
                        turning_strength: required.16,
                        jump_timeout: required.17,
                        linear_strength: required.18,
                        angular_strength: required.19,
                        angular_delay: required.20,
                        is_def: required.21,
                        name: required.22,
                        model_url: required.23,
                        model_scale: required.24,
                    }
                }
            }
            #[doc = "**Vehicle**: A vehicle with a physical representation\n\n**Required**:\n- `linear_velocity`: Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead.\n- `angular_velocity`: Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead.\n- `physics_controlled`: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n- `dynamic`: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n- `density`: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n- `cube_collider`: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n- `local_to_world`: Transformation from the entity's local space to worldspace.\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity.\n- `health`: This game object's health. \"Standard\" health is 100 HP.\n- `max_health`: Maximum health of the object. 100 HP is \"standard.\"\n- `is_vehicle`: Is a vehicle\n- `last_distances`: Distances from the ground for each vehicle probe\n- `last_jump_time`: The last time the player jumped\n- `last_slowdown_time`: The last time the player's vehicle was slowed down\n- `def_ref`: When attached to a vehicle entity, indicates that it should draw its properties from this entity.\n- `input_direction`: Input direction\n- `input_jump`: Jump\n- `input_fire`: Fire\n\n\n**Optional**:\n- `driver_ref`: A vehicle's driver, if present\n- `last_upside_down_time`: The last time the player's vehicle was known to be upside down. Added when first detected, removed when no longer the case. Used to respawn the vehicle after some time.\n- `aim_position`: Position of the aim target\n- `aimable_weapon_refs`: All aimable weapons for this vehicle. If present in this array, their rotations will be automatically updated to match the aim position. Assumes that the weapons are a child of the vehicle."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Vehicle {
                #[doc = "**Component**: `ambient_core::physics::linear_velocity`\n\n**Component description**: Linear velocity (meters/second) of this entity in the physics scene.\nUpdating this component will update the entity's linear velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like gravity not working or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying a force using `physics` functions instead.\n\n"]
                pub linear_velocity: Vec3,
                #[doc = "**Component**: `ambient_core::physics::angular_velocity`\n\n**Component description**: Angular velocity (radians/second) of this entity in the physics scene.\nUpdating this component will update the entity's angular velocity in the physics scene.\n\nNote that changing this component will forcibly set the velocity; changing the velocity every frame may lead to unexpected behavior, like improper physics or collisions failing.\n\nIf you need to adjust the velocity each frame, consider applying an impulse using `physics` functions instead.\n\n"]
                pub angular_velocity: Vec3,
                #[doc = "**Component**: `ambient_core::physics::physics_controlled`\n\n**Component description**: If attached, this entity will be controlled by physics.\nNote that this requires the entity to have a collider.\n\n"]
                pub physics_controlled: (),
                #[doc = "**Component**: `ambient_core::physics::dynamic`\n\n**Component description**: If this is true, the entity will be dynamic (i.e. be able to move). Otherwise, it will be static.\n\n"]
                pub dynamic: bool,
                #[doc = "**Component**: `ambient_core::physics::density`\n\n**Component description**: The density of this entity.\nThis is used to update the `mass` when the entity is rescaled.\n\n"]
                pub density: f32,
                #[doc = "**Component**: `ambient_core::physics::cube_collider`\n\n**Component description**: If attached, this entity will have a cube physics collider.\n`x, y, z` is the size of the cube.\n\n"]
                pub cube_collider: Vec3,
                #[doc = "**Component**: `ambient_core::transform::local_to_world`\n\n**Component description**: Transformation from the entity's local space to worldspace.\n\n"]
                pub local_to_world: Mat4,
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::health`\n\n**Component description**: This game object's health. \"Standard\" health is 100 HP.\n\n"]
                pub health: f32,
                #[doc = "**Component**: `hvxms7i2px7krvkm23sxfjxsjqlcmtb5::max_health`\n\n**Component description**: Maximum health of the object. 100 HP is \"standard.\"\n\n"]
                pub max_health: f32,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::is_vehicle`\n\n**Component description**: Is a vehicle\n\n"]
                pub is_vehicle: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_distances`\n\n**Component description**: Distances from the ground for each vehicle probe\n\n"]
                pub last_distances: Vec<f32>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_jump_time`\n\n**Component description**: The last time the player jumped\n\n"]
                pub last_jump_time: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_slowdown_time`\n\n**Component description**: The last time the player's vehicle was slowed down\n\n"]
                pub last_slowdown_time: Duration,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::def_ref`\n\n**Component description**: When attached to a vehicle entity, indicates that it should draw its properties from this entity.\n\n"]
                pub def_ref: EntityId,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_direction`\n\n**Component description**: Input direction\n\n"]
                pub input_direction: Vec2,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_jump`\n\n**Component description**: Jump\n\n"]
                pub input_jump: bool,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::input_fire`\n\n**Component description**: Fire\n\n"]
                pub input_fire: bool,
                #[doc = r" Optional components."]
                pub optional: VehicleOptional,
            }
            #[doc = "Optional part of [Vehicle]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct VehicleOptional {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::driver_ref`\n\n**Component description**: A vehicle's driver, if present\n\n"]
                pub driver_ref: Option<EntityId>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::last_upside_down_time`\n\n**Component description**: The last time the player's vehicle was known to be upside down. Added when first detected, removed when no longer the case. Used to respawn the vehicle after some time.\n\n"]
                pub last_upside_down_time: Option<Duration>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aim_position`\n\n**Component description**: Position of the aim target\n\n"]
                pub aim_position: Option<Vec3>,
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::vehicle::aimable_weapon_refs`\n\n**Component description**: All aimable weapons for this vehicle. If present in this array, their rotations will be automatically updated to match the aim position. Assumes that the weapons are a child of the vehicle.\n\n"]
                pub aimable_weapon_refs: Option<Vec<EntityId>>,
            }
            impl Concept for Vehicle {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (ambient_api :: core :: physics :: components :: linear_velocity () , self . linear_velocity) . with (ambient_api :: core :: physics :: components :: angular_velocity () , self . angular_velocity) . with (ambient_api :: core :: physics :: components :: physics_controlled () , self . physics_controlled) . with (ambient_api :: core :: physics :: components :: dynamic () , self . dynamic) . with (ambient_api :: core :: physics :: components :: density () , self . density) . with (ambient_api :: core :: physics :: components :: cube_collider () , self . cube_collider) . with (ambient_api :: core :: transform :: components :: local_to_world () , self . local_to_world) . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , self . health) . with (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , self . max_health) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , self . is_vehicle) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , self . last_distances) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , self . last_jump_time) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , self . last_slowdown_time) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , self . def_ref) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , self . input_direction) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , self . input_jump) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire () , self . input_fire) ;
                    if let Some(driver_ref) = self.optional.driver_ref {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref () , driver_ref) ;
                    }
                    if let Some(last_upside_down_time) = self.optional.last_upside_down_time {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time () , last_upside_down_time) ;
                    }
                    if let Some(aim_position) = self.optional.aim_position {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position () , aim_position) ;
                    }
                    if let Some(aimable_weapon_refs) = self.optional.aimable_weapon_refs {
                        entity . set (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs () , aimable_weapon_refs) ;
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { linear_velocity : entity :: get_component (id , ambient_api :: core :: physics :: components :: linear_velocity ()) ? , angular_velocity : entity :: get_component (id , ambient_api :: core :: physics :: components :: angular_velocity ()) ? , physics_controlled : entity :: get_component (id , ambient_api :: core :: physics :: components :: physics_controlled ()) ? , dynamic : entity :: get_component (id , ambient_api :: core :: physics :: components :: dynamic ()) ? , density : entity :: get_component (id , ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity :: get_component (id , ambient_api :: core :: physics :: components :: cube_collider ()) ? , local_to_world : entity :: get_component (id , ambient_api :: core :: transform :: components :: local_to_world ()) ? , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity :: get_component (id , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_vehicle : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle ()) ? , last_distances : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances ()) ? , last_jump_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time ()) ? , last_slowdown_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time ()) ? , def_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref ()) ? , input_direction : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction ()) ? , input_jump : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump ()) ? , input_fire : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()) ? , optional : VehicleOptional { driver_ref : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref ()) , last_upside_down_time : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time ()) , aim_position : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position ()) , aimable_weapon_refs : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { linear_velocity : entity . get (ambient_api :: core :: physics :: components :: linear_velocity ()) ? , angular_velocity : entity . get (ambient_api :: core :: physics :: components :: angular_velocity ()) ? , physics_controlled : entity . get (ambient_api :: core :: physics :: components :: physics_controlled ()) ? , dynamic : entity . get (ambient_api :: core :: physics :: components :: dynamic ()) ? , density : entity . get (ambient_api :: core :: physics :: components :: density ()) ? , cube_collider : entity . get (ambient_api :: core :: physics :: components :: cube_collider ()) ? , local_to_world : entity . get (ambient_api :: core :: transform :: components :: local_to_world ()) ? , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health ()) ? , max_health : entity . get (crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health ()) ? , is_vehicle : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle ()) ? , last_distances : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances ()) ? , last_jump_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time ()) ? , last_slowdown_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time ()) ? , def_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref ()) ? , input_direction : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction ()) ? , input_jump : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump ()) ? , input_fire : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()) ? , optional : VehicleOptional { driver_ref : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref ()) , last_upside_down_time : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time ()) , aim_position : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position ()) , aimable_weapon_refs : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& ambient_api :: core :: physics :: components :: linear_velocity () , & ambient_api :: core :: physics :: components :: angular_velocity () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: physics :: components :: dynamic () , & ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & ambient_api :: core :: transform :: components :: local_to_world () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& ambient_api :: core :: physics :: components :: linear_velocity () , & ambient_api :: core :: physics :: components :: angular_velocity () , & ambient_api :: core :: physics :: components :: physics_controlled () , & ambient_api :: core :: physics :: components :: dynamic () , & ambient_api :: core :: physics :: components :: density () , & ambient_api :: core :: physics :: components :: cube_collider () , & ambient_api :: core :: transform :: components :: local_to_world () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , & crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire ()])
                }
            }
            impl ConceptComponents for Vehicle {
                type Required = (
                    Component<Vec3>,
                    Component<Vec3>,
                    Component<()>,
                    Component<bool>,
                    Component<f32>,
                    Component<Vec3>,
                    Component<Mat4>,
                    Component<Vec3>,
                    Component<Quat>,
                    Component<f32>,
                    Component<f32>,
                    Component<()>,
                    Component<Vec<f32>>,
                    Component<Duration>,
                    Component<Duration>,
                    Component<EntityId>,
                    Component<Vec2>,
                    Component<bool>,
                    Component<bool>,
                );
                type Optional = (
                    Component<EntityId>,
                    Component<Duration>,
                    Component<Vec3>,
                    Component<Vec<EntityId>>,
                );
                fn required() -> Self::Required {
                    (ambient_api :: core :: physics :: components :: linear_velocity () , ambient_api :: core :: physics :: components :: angular_velocity () , ambient_api :: core :: physics :: components :: physics_controlled () , ambient_api :: core :: physics :: components :: dynamic () , ambient_api :: core :: physics :: components :: density () , ambient_api :: core :: physics :: components :: cube_collider () , ambient_api :: core :: transform :: components :: local_to_world () , ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: health () , crate :: packages :: raw :: hvxms7i2px7krvkm23sxfjxsjqlcmtb5 :: components :: max_health () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: is_vehicle () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_distances () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_jump_time () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_slowdown_time () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: def_ref () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_direction () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_jump () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: input_fire () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: driver_ref () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: last_upside_down_time () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aim_position () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: vehicle :: components :: aimable_weapon_refs () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        linear_velocity: required.0,
                        angular_velocity: required.1,
                        physics_controlled: required.2,
                        dynamic: required.3,
                        density: required.4,
                        cube_collider: required.5,
                        local_to_world: required.6,
                        translation: required.7,
                        rotation: required.8,
                        health: required.9,
                        max_health: required.10,
                        is_vehicle: required.11,
                        last_distances: required.12,
                        last_jump_time: required.13,
                        last_slowdown_time: required.14,
                        def_ref: required.15,
                        input_direction: required.16,
                        input_jump: required.17,
                        input_fire: required.18,
                        optional: Default::default(),
                    }
                }
            }
            #[doc = "**Spawnpoint**: A spawnpoint\n\n**Required**:\n- `is_spawnpoint`: Is a spawnpoint\n- `radius`: Radius of the spawnpoint\n- `translation`: The translation/position of this entity.\n- `color`: This entity will be tinted with the specified color if the color is not black."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct Spawnpoint {
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::is_spawnpoint`\n\n**Component description**: Is a spawnpoint\n\n"]
                pub is_spawnpoint: (),
                #[doc = "**Component**: `mwrcsok65na7owrbdococ5sthr3ccskc::spawnpoint::radius`\n\n**Component description**: Radius of the spawnpoint\n\n"]
                pub radius: f32,
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::rendering::color`\n\n**Component description**: This entity will be tinted with the specified color if the color is not black.\n\n"]
                pub color: Vec4,
            }
            impl Concept for Spawnpoint {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , self . is_spawnpoint) . with (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , self . radius) . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: rendering :: components :: color () , self . color) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_spawnpoint : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint ()) ? , radius : entity :: get_component (id , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius ()) ? , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , color : entity :: get_component (id , ambient_api :: core :: rendering :: components :: color ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_spawnpoint : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint ()) ? , radius : entity . get (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius ()) ? , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , color : entity . get (ambient_api :: core :: rendering :: components :: color ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: rendering :: components :: color ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , & crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: rendering :: components :: color ()])
                }
            }
            impl ConceptComponents for Spawnpoint {
                type Required = (
                    Component<()>,
                    Component<f32>,
                    Component<Vec3>,
                    Component<Vec4>,
                );
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: is_spawnpoint () , crate :: packages :: raw :: mwrcsok65na7owrbdococ5sthr3ccskc :: spawnpoint :: components :: radius () , ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: rendering :: components :: color () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_spawnpoint: required.0,
                        radius: required.1,
                        translation: required.2,
                        color: required.3,
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
    pub mod mx4o7x2s4zqc6pxmsfcb7hznbv4chxe5 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("mx4o7x2s4zqc6pxmsfcb7hznbv4chxe5")
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
    pub mod n7xfnlfzdmnvj7bqasfdhqftbtdi27ah {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("n7xfnlfzdmnvj7bqasfdhqftbtdi27ah")
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
            static IN_EDITOR: Lazy<Component<bool>> = Lazy::new(|| {
                __internal_get_component("n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::in_editor")
            });
            #[doc = "**in_editor**\n\n*Attributes*: Networked, Debuggable"]
            pub fn in_editor() -> Component<bool> {
                *IN_EDITOR
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
            #[doc = "**EditorLoad**: Sent by the editor when it has loaded to signal to other modules to add themselves."]
            pub struct EditorLoad;
            impl EditorLoad {
                pub fn new() -> Self {
                    Self
                }
            }
            impl Message for EditorLoad {
                fn id() -> &'static str {
                    "n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::EditorLoad"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {})
                }
            }
            impl ModuleMessage for EditorLoad {}
            impl Default for EditorLoad {
                fn default() -> Self {
                    Self::new()
                }
            }
            #[derive(Clone, Debug)]
            #[doc = "**EditorMenuBarAdd**: Sent by other modules to add themselves to the editor's menubar."]
            pub struct EditorMenuBarAdd {
                pub name: String,
            }
            impl EditorMenuBarAdd {
                #[allow(clippy::too_many_arguments)]
                pub fn new(name: impl Into<String>) -> Self {
                    Self { name: name.into() }
                }
            }
            impl Message for EditorMenuBarAdd {
                fn id() -> &'static str {
                    "n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::EditorMenuBarAdd"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.name.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        name: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for EditorMenuBarAdd {}
            #[derive(Clone, Debug)]
            #[doc = "**EditorMenuBarClick**: Sent by the editor when a menu item is clicked."]
            pub struct EditorMenuBarClick {
                pub name: String,
            }
            impl EditorMenuBarClick {
                #[allow(clippy::too_many_arguments)]
                pub fn new(name: impl Into<String>) -> Self {
                    Self { name: name.into() }
                }
            }
            impl Message for EditorMenuBarClick {
                fn id() -> &'static str {
                    "n7xfnlfzdmnvj7bqasfdhqftbtdi27ah::EditorMenuBarClick"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.name.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        name: String::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for EditorMenuBarClick {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod oa46hyuls6l24bmqapdm3iqzh3p37di6 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("oa46hyuls6l24bmqapdm3iqzh3p37di6")
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
    pub mod roosvvawp6sjvlolokk5qyafl5vp2su7 {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("roosvvawp6sjvlolokk5qyafl5vp2su7")
                    .expect("Failed to get package entity - was it despawned?")
            });
            *ENTITY
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
            #[doc = "**VehicleSpawn**: Spawns the vehicle from the given definition."]
            pub struct VehicleSpawn {
                pub def_id: EntityId,
                pub position: Vec3,
                pub rotation: Option<Quat>,
                pub driver_id: Option<EntityId>,
            }
            impl VehicleSpawn {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    def_id: impl Into<EntityId>,
                    position: impl Into<Vec3>,
                    rotation: impl Into<Option<Quat>>,
                    driver_id: impl Into<Option<EntityId>>,
                ) -> Self {
                    Self {
                        def_id: def_id.into(),
                        position: position.into(),
                        rotation: rotation.into(),
                        driver_id: driver_id.into(),
                    }
                }
            }
            impl Message for VehicleSpawn {
                fn id() -> &'static str {
                    "roosvvawp6sjvlolokk5qyafl5vp2su7::VehicleSpawn"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.def_id.serialize_message_part(&mut output)?;
                    self.position.serialize_message_part(&mut output)?;
                    self.rotation.serialize_message_part(&mut output)?;
                    self.driver_id.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        def_id: EntityId::deserialize_message_part(&mut input)?,
                        position: Vec3::deserialize_message_part(&mut input)?,
                        rotation: Option::<Quat>::deserialize_message_part(&mut input)?,
                        driver_id: Option::<EntityId>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for VehicleSpawn {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod skpc6fwjkbidr7a6pmx4mab6zl37oiut {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("skpc6fwjkbidr7a6pmx4mab6zl37oiut")
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
            static IS_HEALTH_PICKUP: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("skpc6fwjkbidr7a6pmx4mab6zl37oiut::is_health_pickup")
            });
            #[doc = "**Is Health Pickup**: This entity is a health pickup."]
            pub fn is_health_pickup() -> Component<()> {
                *IS_HEALTH_PICKUP
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
            #[doc = "**Health Pickup**: This entity is a health pickup.\n\n**Required**:\n- `is_health_pickup`: This entity is a health pickup.\n- `translation`: The translation/position of this entity.\n- `rotation`: The rotation of this entity."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct HealthPickup {
                #[doc = "**Component**: `skpc6fwjkbidr7a6pmx4mab6zl37oiut::is_health_pickup`\n\n**Component description**: This entity is a health pickup.\n\n"]
                pub is_health_pickup: (),
                #[doc = "**Component**: `ambient_core::transform::translation`\n\n**Component description**: The translation/position of this entity.\n\n"]
                pub translation: Vec3,
                #[doc = "**Component**: `ambient_core::transform::rotation`\n\n**Component description**: The rotation of this entity.\n\n"]
                pub rotation: Quat,
            }
            impl Concept for HealthPickup {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: skpc6fwjkbidr7a6pmx4mab6zl37oiut :: components :: is_health_pickup () , self . is_health_pickup) . with (ambient_api :: core :: transform :: components :: translation () , self . translation) . with (ambient_api :: core :: transform :: components :: rotation () , self . rotation) ;
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_health_pickup : entity :: get_component (id , crate :: packages :: raw :: skpc6fwjkbidr7a6pmx4mab6zl37oiut :: components :: is_health_pickup ()) ? , translation : entity :: get_component (id , ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity :: get_component (id , ambient_api :: core :: transform :: components :: rotation ()) ? , })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_health_pickup : entity . get (crate :: packages :: raw :: skpc6fwjkbidr7a6pmx4mab6zl37oiut :: components :: is_health_pickup ()) ? , translation : entity . get (ambient_api :: core :: transform :: components :: translation ()) ? , rotation : entity . get (ambient_api :: core :: transform :: components :: rotation ()) ? , })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: skpc6fwjkbidr7a6pmx4mab6zl37oiut :: components :: is_health_pickup () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: skpc6fwjkbidr7a6pmx4mab6zl37oiut :: components :: is_health_pickup () , & ambient_api :: core :: transform :: components :: translation () , & ambient_api :: core :: transform :: components :: rotation ()])
                }
            }
            impl ConceptComponents for HealthPickup {
                type Required = (Component<()>, Component<Vec3>, Component<Quat>);
                type Optional = ();
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: skpc6fwjkbidr7a6pmx4mab6zl37oiut :: components :: is_health_pickup () , ambient_api :: core :: transform :: components :: translation () , ambient_api :: core :: transform :: components :: rotation () ,)
                }
                fn optional() -> Self::Optional {
                    ()
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_health_pickup: required.0,
                        translation: required.1,
                        rotation: required.2,
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
            #[doc = "**OnHealthPickup**: Sent to the client when a health pickup is picked up."]
            pub struct OnHealthPickup {
                pub position: Vec3,
            }
            impl OnHealthPickup {
                #[allow(clippy::too_many_arguments)]
                pub fn new(position: impl Into<Vec3>) -> Self {
                    Self {
                        position: position.into(),
                    }
                }
            }
            impl Message for OnHealthPickup {
                fn id() -> &'static str {
                    "skpc6fwjkbidr7a6pmx4mab6zl37oiut::OnHealthPickup"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.position.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        position: Vec3::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for OnHealthPickup {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod t5qdqwpkoxtelvafs7qpvzhaoperwfpt {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("t5qdqwpkoxtelvafs7qpvzhaoperwfpt")
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
    pub mod vvuixyn4jy3xv4ij4hi75atzfwk2j37k {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("vvuixyn4jy3xv4ij4hi75atzfwk2j37k")
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
    pub mod xadcwjtmzuagnoydry5h4psaph3hccbw {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xadcwjtmzuagnoydry5h4psaph3hccbw")
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
    pub mod xgafi5ckk5vhxscb6tqubfvy2fwfyqxo {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xgafi5ckk5vhxscb6tqubfvy2fwfyqxo")
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
    pub mod xr6whcy65gn3vlzrp2ssyn7udcbxb6mu {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu")
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
            static EDITOR_CAMERA: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::editor_camera")
            });
            #[doc = "**editor_camera**\n\n*Attributes*: Networked, Debuggable"]
            pub fn editor_camera() -> Component<EntityId> {
                *EDITOR_CAMERA
            }
            static MOUSEOVER_POSITION: Lazy<Component<Vec3>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::mouseover_position")
            });
            #[doc = "**mouseover_position**\n\n*Attributes*: Networked, Debuggable"]
            pub fn mouseover_position() -> Component<Vec3> {
                *MOUSEOVER_POSITION
            }
            static MOUSEOVER_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::mouseover_entity")
            });
            #[doc = "**mouseover_entity**\n\n*Attributes*: Networked, Debuggable"]
            pub fn mouseover_entity() -> Component<EntityId> {
                *MOUSEOVER_ENTITY
            }
            static SELECTED_ENTITY: Lazy<Component<EntityId>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::selected_entity")
            });
            #[doc = "**selected_entity**\n\n*Attributes*: Networked, Debuggable"]
            pub fn selected_entity() -> Component<EntityId> {
                *SELECTED_ENTITY
            }
            static HAS_SAMPLE_SCENE: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::has_sample_scene")
            });
            #[doc = "**has_sample_scene**\n\n*Attributes*: Networked, MaybeResource"]
            pub fn has_sample_scene() -> Component<()> {
                *HAS_SAMPLE_SCENE
            }
            static CAMERA_ANGLE: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::camera_angle")
            });
            #[doc = "**camera_angle**: X is yaw, Y is pitch\n\n*Attributes*: Networked, Debuggable"]
            pub fn camera_angle() -> Component<Vec2> {
                *CAMERA_ANGLE
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
            #[doc = "**ToggleEditor**"]
            pub struct ToggleEditor {
                pub camera_transform: Option<Mat4>,
            }
            impl ToggleEditor {
                #[allow(clippy::too_many_arguments)]
                pub fn new(camera_transform: impl Into<Option<Mat4>>) -> Self {
                    Self {
                        camera_transform: camera_transform.into(),
                    }
                }
            }
            impl Message for ToggleEditor {
                fn id() -> &'static str {
                    "xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::ToggleEditor"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.camera_transform.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        camera_transform: Option::<Mat4>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for ToggleEditor {}
            #[derive(Clone, Debug)]
            #[doc = "**Input**"]
            pub struct Input {
                pub aim_delta: Vec2,
                pub movement: Vec2,
                pub boost: bool,
                pub ray_origin: Vec3,
                pub ray_direction: Vec3,
                pub select: bool,
                pub freeze: bool,
                pub translate_to: Option<Vec3>,
            }
            impl Input {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    aim_delta: impl Into<Vec2>,
                    movement: impl Into<Vec2>,
                    boost: impl Into<bool>,
                    ray_origin: impl Into<Vec3>,
                    ray_direction: impl Into<Vec3>,
                    select: impl Into<bool>,
                    freeze: impl Into<bool>,
                    translate_to: impl Into<Option<Vec3>>,
                ) -> Self {
                    Self {
                        aim_delta: aim_delta.into(),
                        movement: movement.into(),
                        boost: boost.into(),
                        ray_origin: ray_origin.into(),
                        ray_direction: ray_direction.into(),
                        select: select.into(),
                        freeze: freeze.into(),
                        translate_to: translate_to.into(),
                    }
                }
            }
            impl Message for Input {
                fn id() -> &'static str {
                    "xr6whcy65gn3vlzrp2ssyn7udcbxb6mu::Input"
                }
                fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
                    let mut output = vec![];
                    self.aim_delta.serialize_message_part(&mut output)?;
                    self.movement.serialize_message_part(&mut output)?;
                    self.boost.serialize_message_part(&mut output)?;
                    self.ray_origin.serialize_message_part(&mut output)?;
                    self.ray_direction.serialize_message_part(&mut output)?;
                    self.select.serialize_message_part(&mut output)?;
                    self.freeze.serialize_message_part(&mut output)?;
                    self.translate_to.serialize_message_part(&mut output)?;
                    Ok(output)
                }
                fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
                    Ok(Self {
                        aim_delta: Vec2::deserialize_message_part(&mut input)?,
                        movement: Vec2::deserialize_message_part(&mut input)?,
                        boost: bool::deserialize_message_part(&mut input)?,
                        ray_origin: Vec3::deserialize_message_part(&mut input)?,
                        ray_direction: Vec3::deserialize_message_part(&mut input)?,
                        select: bool::deserialize_message_part(&mut input)?,
                        freeze: bool::deserialize_message_part(&mut input)?,
                        translate_to: Option::<Vec3>::deserialize_message_part(&mut input)?,
                    })
                }
            }
            impl ModuleMessage for Input {}
        }
        #[doc = r" Helpers for accessing the assets for this package."]
        pub mod assets {
            pub fn url(path: &str) -> String {
                ambient_api::asset::url_for_package_asset(super::entity(), path).unwrap()
            }
        }
    }
    pub mod ys6g5noe72fbhnoj6l3psjq75knd7gko {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("ys6g5noe72fbhnoj6l3psjq75knd7gko")
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
    pub mod zlu324bqcibov3o4co42eriyfhcnzsus {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("zlu324bqcibov3o4co42eriyfhcnzsus")
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
pub use raw::c72h7qyqnp4njboj7tu4vomadoj2zu6e as ui_spawn_menu;
pub use raw::d7rxxncafgtwf7c3brhsw7oh7h2ccfip as gameplay_daynight;
pub use raw::fvn74ns4ozf3gn42vmowphmvmpsfklbi as class_scout;
pub use raw::ggu2h7bk4jrvshq7zteboipyut7wuib2 as level_cubicide_onespawn;
pub use raw::gzbamly2shtnz3siisf3mdzglsi67vul as level_cubicide;
pub use raw::hr4pxz7kfhzgimicoyh65ydel3aehuhk as package_manager;
pub use raw::hs7ygpw4pmpsixtcohdcvzxwmrfzubvi as ui_class_selection;
pub use raw::ianwyihfsaiuc26xjldmwd3duidju2tb as gameplay_firerain;
pub use raw::itzh3wovmdje4ttrmo6wrravaaxp6b52 as core;
pub use raw::j32xi2gg5rvgob2cu7uirdbtj5ce4jw7 as class_assault;
pub use raw::mkd4mhans4bdn3mvmt45vxqbxepdhys3 as camera_follow;
pub use raw::mnm43qv33k7kx235bz5hcgoguokwxzwo as behavior_vehicle;
pub use raw::mx4o7x2s4zqc6pxmsfcb7hznbv4chxe5 as vehicle_standard;
pub use raw::oa46hyuls6l24bmqapdm3iqzh3p37di6 as reskin_classes;
pub use raw::skpc6fwjkbidr7a6pmx4mab6zl37oiut as pickup_health;
pub use raw::t5qdqwpkoxtelvafs7qpvzhaoperwfpt as this;
pub use raw::vvuixyn4jy3xv4ij4hi75atzfwk2j37k as camera_topdown;
pub use raw::xadcwjtmzuagnoydry5h4psaph3hccbw as ui_holohud;
pub use raw::xar372tfo2oswb4pkvx7h7o3rxi6tap6 as hide_cursor;
pub use raw::xgafi5ckk5vhxscb6tqubfvy2fwfyqxo as behavior_character;
pub use raw::xr6whcy65gn3vlzrp2ssyn7udcbxb6mu as editor;
pub use raw::ys6g5noe72fbhnoj6l3psjq75knd7gko as class_tank;
pub use raw::zlu324bqcibov3o4co42eriyfhcnzsus as ui_flathud;
