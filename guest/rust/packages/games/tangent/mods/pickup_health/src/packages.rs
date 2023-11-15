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
}
pub use raw::hvxms7i2px7krvkm23sxfjxsjqlcmtb5 as game_object;
pub use raw::k7svgbw5j6orlwzj45koeownlodsdbth as kenney_digital_audio;
pub use raw::skpc6fwjkbidr7a6pmx4mab6zl37oiut as this;
