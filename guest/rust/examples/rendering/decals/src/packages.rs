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
    pub mod nd2mwrslb3iyo2bw4jmwcbkuo4mtuyfr {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("nd2mwrslb3iyo2bw4jmwcbkuo4mtuyfr")
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
    pub mod tijz7x6fimbgu24sbbtp4nllhfxbgblp {
        pub fn entity() -> ambient_api::global::EntityId {
            use ambient_api::once_cell::sync::Lazy;
            static ENTITY: Lazy<ambient_api::global::EntityId> = Lazy::new(|| {
                ambient_api::package::get_entity_for_package_id("tijz7x6fimbgu24sbbtp4nllhfxbgblp")
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
            static IS_ORBIT_CAMERA: Lazy<Component<()>> = Lazy::new(|| {
                __internal_get_component("tijz7x6fimbgu24sbbtp4nllhfxbgblp::is_orbit_camera")
            });
            #[doc = "**is_orbit_camera**\n\n*Attributes*: Debuggable, Networked"]
            pub fn is_orbit_camera() -> Component<()> {
                *IS_ORBIT_CAMERA
            }
            static CAMERA_ANGLE: Lazy<Component<Vec2>> = Lazy::new(|| {
                __internal_get_component("tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_angle")
            });
            #[doc = "**camera_angle**: Camera angle specified in radians; x = yaw, y = pitch\n\n*Attributes*: Debuggable, Networked"]
            pub fn camera_angle() -> Component<Vec2> {
                *CAMERA_ANGLE
            }
            static CAMERA_DISTANCE: Lazy<Component<f32>> = Lazy::new(|| {
                __internal_get_component("tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_distance")
            });
            #[doc = "**camera_distance**: Camera distance specified in meters\n\n*Attributes*: Debuggable, Networked"]
            pub fn camera_distance() -> Component<f32> {
                *CAMERA_DISTANCE
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
            #[doc = "**Orbit Camera**: An orbit camera.\n\n**Required**:\n- `is_orbit_camera`: No description provided.\n\n\n**Optional**:\n- `camera_angle`: Camera angle specified in radians; x = yaw, y = pitch\n- `camera_distance`: Camera distance specified in meters\n- `lookat_target`: The position that this entity should be looking at."]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct OrbitCamera {
                #[doc = "**Component**: `tijz7x6fimbgu24sbbtp4nllhfxbgblp::is_orbit_camera`\n\n**Suggested value**: `()`\n\n"]
                pub is_orbit_camera: (),
                #[doc = r" Optional components."]
                pub optional: OrbitCameraOptional,
            }
            #[doc = "Optional part of [OrbitCamera]."]
            #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
            #[serde(crate = "self::serde")]
            pub struct OrbitCameraOptional {
                #[doc = "**Component**: `tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_angle`\n\n**Component description**: Camera angle specified in radians; x = yaw, y = pitch\n\n"]
                pub camera_angle: Option<Vec2>,
                #[doc = "**Component**: `tijz7x6fimbgu24sbbtp4nllhfxbgblp::camera_distance`\n\n**Component description**: Camera distance specified in meters\n\n"]
                pub camera_distance: Option<f32>,
                #[doc = "**Component**: `ambient_core::transform::lookat_target`\n\n**Component description**: The position that this entity should be looking at.\n\n"]
                pub lookat_target: Option<Vec3>,
            }
            impl Concept for OrbitCamera {
                fn make(self) -> Entity {
                    let mut entity = Entity :: new () . with (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera () , self . is_orbit_camera) ;
                    if let Some(camera_angle) = self.optional.camera_angle {
                        entity . set (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle () , camera_angle) ;
                    }
                    if let Some(camera_distance) = self.optional.camera_distance {
                        entity . set (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance () , camera_distance) ;
                    }
                    if let Some(lookat_target) = self.optional.lookat_target {
                        entity.set(
                            ambient_api::core::transform::components::lookat_target(),
                            lookat_target,
                        );
                    }
                    entity
                }
                fn get_spawned(id: EntityId) -> Option<Self> {
                    Some (Self { is_orbit_camera : entity :: get_component (id , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()) ? , optional : OrbitCameraOptional { camera_angle : entity :: get_component (id , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle ()) , camera_distance : entity :: get_component (id , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance ()) , lookat_target : entity :: get_component (id , ambient_api :: core :: transform :: components :: lookat_target ()) , } })
                }
                fn get_unspawned(entity: &Entity) -> Option<Self> {
                    Some (Self { is_orbit_camera : entity . get (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()) ? , optional : OrbitCameraOptional { camera_angle : entity . get (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle ()) , camera_distance : entity . get (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance ()) , lookat_target : entity . get (ambient_api :: core :: transform :: components :: lookat_target ()) , } })
                }
                fn contained_by_spawned(id: EntityId) -> bool {
                    entity :: has_components (id , & [& crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()])
                }
                fn contained_by_unspawned(entity: &Entity) -> bool {
                    entity . has_components (& [& crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera ()])
                }
            }
            impl ConceptSuggested for OrbitCamera {
                #[doc = "```\nis_orbit_camera: (),\n```"]
                fn suggested() -> Self {
                    Self {
                        is_orbit_camera: (),
                        optional: Default::default(),
                    }
                }
            }
            impl ConceptComponents for OrbitCamera {
                type Required = (Component<()>,);
                type Optional = (Component<Vec2>, Component<f32>, Component<Vec3>);
                fn required() -> Self::Required {
                    (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: is_orbit_camera () ,)
                }
                fn optional() -> Self::Optional {
                    (crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_angle () , crate :: packages :: raw :: tijz7x6fimbgu24sbbtp4nllhfxbgblp :: components :: camera_distance () , ambient_api :: core :: transform :: components :: lookat_target () ,)
                }
                fn from_required_data(required: <Self::Required as ComponentsTuple>::Data) -> Self {
                    Self {
                        is_orbit_camera: required.0,
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
}
pub use raw::nd2mwrslb3iyo2bw4jmwcbkuo4mtuyfr as this;
pub use raw::tijz7x6fimbgu24sbbtp4nllhfxbgblp as orbit_camera;
