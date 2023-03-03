use ambient_ecs::{
    primitive_component_definitions, ComponentDesc, ComponentEntry, ComponentRegistry, ComponentValue, DefaultValue, EntityId,
    ExternalComponentAttributes, PrimitiveComponentType,
};
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

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

    doc.insert("components", toml_edit::Item::Table(make_components()));
    doc.insert("concepts", toml_edit::Item::Table(make_concepts()));

    doc
}

fn make_components() -> toml_edit::Table {
    let mut components = toml_edit::Table::new();
    components.set_implicit(true);

    let namespaces = [
        ("core", "Core", "Contains all core components for the Ambient Runtime."),
        ("core::app", "App", "High-level state relevant to the application (including the in-development Editor)."),
        ("core::camera", "Camera", "Camera matrices, types, parameters, and more."),
        ("core::ecs", "Entity Component System", "Core components for the ECS and entities."),
        ("core::game_objects", "Game Objects", "Pre-defined game objects that implement specific behaviours."),
        ("core::model", "Model", "Information about models attached to entities."),
        ("core::network", "Network", "Network-related state."),
        ("core::prefab", "Prefab", "Prefab-related state, including loading of prefabs."),
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
        ("core::ui", "UI", "Anything related to UI and text."),
    ];

    for (path, name, description) in namespaces {
        use toml_edit::value;

        let mut table = toml_edit::Table::new();
        table.insert("name", value(name));
        table.insert("description", value(description));
        components.insert(path, toml_edit::Item::Table(table));
    }

    let component_registry = ambient_ecs::ComponentRegistry::get();
    let mut all_primitive = component_registry.all_primitive().collect::<Vec<_>>();
    all_primitive.sort_by_key(|pc| pc.desc.path());
    for component in all_primitive {
        if let Some(table) = make_component_table(component) {
            components.insert(&component.desc.path(), toml_edit::Item::Table(table));
        }
    }
    components
}

fn make_component_table(component: &ambient_ecs::PrimitiveComponent) -> Option<toml_edit::Table> {
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
    if let Some(default) = get_toml_default_for_primitive_component(component) {
        table.insert("default", default);
    }
    table.insert("attributes", {
        let attrs = ExternalComponentAttributes::from_existing_component(desc);
        value(toml_edit::Array::from_iter(attrs.flags.iter()))
    });

    Some(table)
}

macro_rules! make_get_toml_default_for_primitive_component {
    ($(($value:ident, $type:ty)),*) => { paste::paste! {
        fn get_toml_default_for_primitive_component(component: &ambient_ecs::PrimitiveComponent) -> Option<toml_edit::Item> {
            fn dispatch_default<T: ToToml>(desc: ComponentDesc) -> Option<toml_edit::Item> {
                Some(toml_edit::value(desc.attribute::<DefaultValue<T>>().and_then(|attr| attr.0.to_toml())?))
            }

            let desc = component.desc;
            match component.ty {
                $(PrimitiveComponentType::$value => dispatch_default::<$type>(desc),)*
                $(PrimitiveComponentType::[< Vec $value >] => dispatch_default::<Vec<$type>>(desc),)*
                $(PrimitiveComponentType::[< Option$value >] => dispatch_default::<Option<$type>>(desc),)*
            }
        }
    } };
}
primitive_component_definitions!(make_get_toml_default_for_primitive_component);

fn make_concepts() -> toml_edit::Table {
    let mut concepts = toml_edit::Table::new();
    concepts.set_implicit(true);

    for concept in super::concepts() {
        use toml_edit::value;
        let mut table = toml_edit::Table::new();

        table.insert("name", value(concept.name));
        table.insert("description", value(concept.description));
        if !concept.extends.is_empty() {
            table.insert("extends", value(toml_edit::Array::from_iter(concept.extends.iter().cloned())));
        }

        let mut components_table = toml_edit::Table::new();
        for entry in concept.data {
            let path = entry.path();
            let default = convert_entry_to_toml(&entry);
            match default {
                Some(default) => components_table.insert(&path, value(default)),
                _ => panic!("invalid toml default for {path:?}"),
            };
        }
        table.insert("components", toml_edit::Item::Table(components_table));

        concepts.insert(&concept.id, toml_edit::Item::Table(table));
    }

    concepts
}

macro_rules! make_convert_entry_to_toml {
    ($(($value:ident, $type:ty)),*) => { paste::paste! {
        fn convert_entry_to_toml(entry: &ComponentEntry) -> Option<toml_edit::Value> {
            let desc = entry.desc();
            let pct = ComponentRegistry::get().get_primitive_component(desc.index())?.ty;
            match pct {
                $(PrimitiveComponentType::$value => entry.downcast_ref::<$type>().to_toml(),)*
                $(PrimitiveComponentType::[< Vec $value >] => entry.downcast_ref::<Vec<$type>>().to_toml(),)*
                $(PrimitiveComponentType::[< Option$value >] => entry.downcast_ref::<Option<$type>>().to_toml(),)*
            }
        }
    } };
}
primitive_component_definitions!(make_convert_entry_to_toml);

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
        convert_array_f32(self.to_cols_array())
    }
}
impl ToToml for i32 {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        Some((*self as i64).into())
    }
}
impl ToToml for Quat {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        convert_array_f32(self.to_array())
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
        convert_array_f32(self.to_array())
    }
}
impl ToToml for Vec3 {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        convert_array_f32(self.to_array())
    }
}
impl ToToml for Vec4 {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        convert_array_f32(self.to_array())
    }
}
impl ToToml for UVec2 {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        convert_array_u32(self.to_array())
    }
}
impl ToToml for UVec3 {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        convert_array_u32(self.to_array())
    }
}
impl ToToml for UVec4 {
    fn to_toml(&self) -> Option<toml_edit::Value> {
        convert_array_u32(self.to_array())
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

fn convert_array_f32<const N: usize>(arr: [f32; N]) -> Option<toml_edit::Value> {
    Some(toml_edit::Array::from_iter(arr.map(|v| v as f64)).into())
}

fn convert_array_u32<const N: usize>(arr: [u32; N]) -> Option<toml_edit::Value> {
    Some(toml_edit::Array::from_iter(arr.map(|v| v as i64)).into())
}
