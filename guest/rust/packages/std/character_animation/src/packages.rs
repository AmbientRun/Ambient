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
}
pub use raw::afl5yv5ya35vbuaj3aido22cwjzat25z as unit_schema;
pub use raw::d3y3wbexrclipsykysumem3fthkudwx2 as this;
pub use raw::hvxms7i2px7krvkm23sxfjxsjqlcmtb5 as game_object;
