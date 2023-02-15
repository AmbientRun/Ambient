pub(crate) fn init() -> anyhow::Result<()> {
    kiwi_app::init_all_components();
    kiwi_network::init_all_components();
    kiwi_physics::init_all_components();
    kiwi_wasm::shared::init_components();
    kiwi_decals::init_components();
    kiwi_world_audio::init_components();
    kiwi_primitives::init_components();
    kiwi_project::init_components();
    kiwi_object::init_components();

    crate::player::init_all_components();

    Ok(())
}

#[cfg(not(feature = "production"))]
pub(crate) mod dev {
    use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
    use kiwi_ecs::{
        primitive_component_definitions, ComponentDesc, ComponentValue, DefaultValue, EntityId, ExternalComponentAttributes,
        PrimitiveComponentType,
    };
    use kiwi_std::asset_url::ObjectRef;

    pub fn build_components_toml() -> toml_edit::Document {
        let mut doc = toml_edit::Document::new();
        {
            let mut project = toml_edit::Table::new();
            project.decor_mut().set_prefix("# The following file is auto-generated. Please do not change this file.\n");
            project.insert("id", toml_edit::value("runtime_components"));
            project.insert("name", toml_edit::value("Runtime Components"));
            project.insert("version", toml_edit::value(env!("CARGO_PKG_VERSION")));
            doc.insert("project", toml_edit::Item::Table(project));
        }

        {
            let mut components = toml_edit::Table::new();
            components.set_implicit(true);

            let namespaces = [
                ("core", "Core", "Contains all core components for the Kiwi Runtime."),
                ("core::app", "App", "High-level state relevant to the application (including the in-development Editor)."),
                ("core::camera", "Camera", "Camera matrices, types, parameters, and more."),
                ("core::ecs", "Entity Component System", "Core components for the ECS and entities."),
                ("core::game_objects", "Game Objects", "Pre-defined game objects that implement specific behaviours."),
                ("core::model", "Model", "Information about models attached to entities."),
                ("core::network", "Network", "Network-related state."),
                ("core::object", "Object", "External object related state (e.g. drawing objects from remote URLs)"),
                ("core::physics", "Physics", "Physics functionality and state."),
                ("core::player", "Player", "Components that are attached to player entities."),
                (
                    "core::primitives",
                    "Primitives",
                    "Components that create primitive (in the geometric sense) objects from their attached entities.",
                ),
                ("core::rendering", "Rendering", "Rendering-related state, including global rendering parameters and per-entity state."),
                (
                    "core::transform",
                    "Transform",
                    "Entity transform state (including translation, rotation and scale), as well as other transformations for this entity.",
                ),
                ("core::ui", "Ui", "Anything related to ui and text."),
            ];

            for (path, name, description) in namespaces {
                use toml_edit::value;

                let mut table = toml_edit::Table::new();
                table.insert("name", value(name));
                table.insert("description", value(description));
                components.insert(path, toml_edit::Item::Table(table));
            }

            let component_registry = kiwi_ecs::ComponentRegistry::get();
            let mut all_primitive = component_registry.all_primitive().collect::<Vec<_>>();
            all_primitive.sort_by_key(|pc| pc.desc.path());
            for component in all_primitive {
                if let Some(table) = make_component_table(component) {
                    components.insert(&component.desc.path(), toml_edit::Item::Table(table));
                }
            }
            doc.insert("components", toml_edit::Item::Table(components));
        }

        doc
    }

    fn make_component_table(component: &kiwi_ecs::PrimitiveComponent) -> Option<toml_edit::Table> {
        use toml_edit::value;

        let desc = component.desc;
        let Some(name) = desc.name() else { return None; };
        let Some(description) = desc.description() else { return None; };

        if !description.ends_with('.') {
            log::warn!("`{}`'s description did not end in a full stop. Is it grammatical?", component.desc.path());
        }

        let mut table = toml_edit::Table::new();
        table.insert(
            "type",
            match component.ty.decompose_container_type() {
                Some((container_type, element_type)) => value(toml_edit::InlineTable::from_iter([
                    ("type", container_type.as_str()),
                    ("element_type", element_type.as_str().expect("invalid container type")),
                ])),
                None => value(component.ty.as_str().expect("invalid component type")),
            },
        );
        table.insert("name", value(name));
        table.insert("description", value(description));
        if let Some(default) = make_component_default(component) {
            table.insert("default", default);
        }
        table.insert("attributes", {
            let attrs = ExternalComponentAttributes::from_existing_component(desc);
            value(toml_edit::Array::from_iter(attrs.flags.iter()))
        });

        Some(table)
    }

    macro_rules! make_make_component_default {
        ($(($value:ident, $type:ty)),*) => { paste::paste! {
            fn make_component_default(component: &kiwi_ecs::PrimitiveComponent) -> Option<toml_edit::Item> {
                let desc = component.desc;
                match component.ty {
                    $(PrimitiveComponentType::$value => dispatch_default::<$type>(desc),)*
                    $(PrimitiveComponentType::[< Vec $value >] => dispatch_default::<Vec<$type>>(desc),)*
                    $(PrimitiveComponentType::[< Option$value >] => dispatch_default::<Option<$type>>(desc),)*
                }
            }
        } };
    }
    primitive_component_definitions!(make_make_component_default);

    fn dispatch_default<T: ToToml>(desc: ComponentDesc) -> Option<toml_edit::Item> {
        Some(toml_edit::value(desc.attribute::<DefaultValue<T>>().and_then(|attr| attr.0.to_toml())?))
    }

    trait ToToml: ComponentValue {
        fn to_toml(&self) -> Option<toml_edit::Value>;
    }
    impl ToToml for () {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(toml_edit::InlineTable::new().into())
        }
    }
    impl ToToml for bool {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some((*self).into())
        }
    }
    impl ToToml for EntityId {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(self.to_string().into())
        }
    }
    impl ToToml for f32 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some((*self as f64).into())
        }
    }
    impl ToToml for f64 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some((*self).into())
        }
    }
    impl ToToml for Mat4 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            convert_array(self.to_cols_array())
        }
    }
    impl ToToml for i32 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some((*self as i64).into())
        }
    }
    impl ToToml for Quat {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            convert_array(self.to_array())
        }
    }
    impl ToToml for String {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(self.as_str().into())
        }
    }
    impl ToToml for u32 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some((*self as i64).into())
        }
    }
    impl ToToml for u64 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(self.to_string().into())
        }
    }
    impl ToToml for Vec2 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            convert_array(self.to_array())
        }
    }
    impl ToToml for Vec3 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            convert_array(self.to_array())
        }
    }
    impl ToToml for Vec4 {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            convert_array(self.to_array())
        }
    }
    impl ToToml for ObjectRef {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(self.to_string().into())
        }
    }

    impl<T: ToToml> ToToml for Vec<T> {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(toml_edit::Array::from_iter(self.iter().flat_map(|v| v.to_toml())).into())
        }
    }
    impl<T: ToToml> ToToml for Option<T> {
        fn to_toml(&self) -> Option<toml_edit::Value> {
            Some(toml_edit::Array::from_iter(self.iter().flat_map(|v| v.to_toml())).into())
        }
    }

    fn convert_array<const N: usize>(arr: [f32; N]) -> Option<toml_edit::Value> {
        Some(toml_edit::Array::from_iter(arr.map(|v| v as f64)).into())
    }
}
