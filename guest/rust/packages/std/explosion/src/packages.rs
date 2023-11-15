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
pub use raw::cneomdouziieskjvs3szwmigzotofjzs as this;
pub use raw::hvxms7i2px7krvkm23sxfjxsjqlcmtb5 as game_object;
