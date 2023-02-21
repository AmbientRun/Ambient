use ambient_ecs::{
    primitive_component_definitions, ComponentDesc, ComponentValue, DefaultValue, EntityId, ExternalComponentAttributes,
    PrimitiveComponentType,
};
use ambient_std::asset_url::ObjectRef;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

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

fn make_concepts() -> toml_edit::Table {
    let defs = [
        (
            ("transformable", "Transformable"),
            "Can be translated, rotated and scaled.",
            vec![],
            vec![
                (ambient_core::transform::translation().desc(), Vec3::ZERO.to_toml()),
                (ambient_core::transform::rotation().desc(), Quat::IDENTITY.to_toml()),
                (ambient_core::transform::scale().desc(), Vec3::ONE.to_toml()),
            ],
        ),
        (
            ("sphere", "Sphere"),
            "A primitive sphere.",
            vec![],
            vec![
                (ambient_primitives::sphere().desc(), ().to_toml()),
                (ambient_primitives::sphere_radius().desc(), 0.5f32.to_toml()),
                (ambient_primitives::sphere_sectors().desc(), 36u32.to_toml()),
                (ambient_primitives::sphere_stacks().desc(), 18u32.to_toml()),
            ],
        ),
        (
            ("camera", "Camera"),
            "Base components for a camera. You will need other components to make a fully-functioning camera.",
            vec!["transformable"],
            vec![
                (ambient_core::camera::projection().desc(), glam::Mat4::IDENTITY.to_toml()),
                (ambient_core::camera::projection_view().desc(), glam::Mat4::IDENTITY.to_toml()),
                (ambient_core::camera::near().desc(), 0.1f32.to_toml()),
            ],
        ),
        (
            ("perspective_common_camera", "Perspective Common Camera"),
            "Base components for a perspective camera. Consider `perspective_camera` or `perspective_infinite_reverse_camera`.",
            vec!["camera"],
            vec![(ambient_core::camera::aspect_ratio().desc(), 1.0f32.to_toml()), (ambient_core::camera::fovy().desc(), 1.0f32.to_toml())],
        ),
        (
            ("perspective_camera", "Perspective Camera"),
            "A perspective camera.",
            vec!["perspective_common_camera"],
            vec![(ambient_core::camera::perspective().desc(), ().to_toml()), (ambient_core::camera::far().desc(), 1_000f32.to_toml())],
        ),
        (
            ("perspective_infinite_reverse_camera", "Perspective-Infinite-Reverse Camera"),
            "A perspective-infinite-reverse camera. This is recommended for most use-cases.",
            vec!["perspective_common_camera"],
            vec![(ambient_core::camera::perspective_infinite_reverse().desc(), ().to_toml())],
        ),
        (
            ("orthographic_camera", "Orthographic Camera"),
            "An orthographic camera.",
            vec!["camera"],
            vec![
                (ambient_core::camera::orthographic().desc(), ().to_toml()),
                (ambient_core::camera::orthographic_left().desc(), (-1.0f32).to_toml()),
                (ambient_core::camera::orthographic_right().desc(), 1.0f32.to_toml()),
                (ambient_core::camera::orthographic_top().desc(), 1.0f32.to_toml()),
                (ambient_core::camera::orthographic_bottom().desc(), (-1.0f32).to_toml()),
                (ambient_core::camera::far().desc(), 1_000f32.to_toml()),
            ],
        ),
    ];

    let mut concepts = toml_edit::Table::new();
    concepts.set_implicit(true);
    for ((id, name), description, extends, components) in defs {
        concepts.insert(id, make_concept(name, description, &extends, &components));
    }
    concepts
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
    if let Some(default) = make_component_default(component) {
        table.insert("default", default);
    }
    table.insert("attributes", {
        let attrs = ExternalComponentAttributes::from_existing_component(desc);
        value(toml_edit::Array::from_iter(attrs.flags.iter()))
    });

    Some(table)
}

fn make_concept(
    name: &str,
    description: &str,
    extends: &[&str],
    components: &[(ComponentDesc, Option<toml_edit::Value>)],
) -> toml_edit::Item {
    use toml_edit::value;

    let mut table = toml_edit::Table::new();
    table.insert("name", value(name));
    table.insert("description", value(description));
    if !extends.is_empty() {
        table.insert("extends", value(toml_edit::Array::from_iter(extends.iter().cloned())));
    }
    let mut components_table = toml_edit::Table::new();
    for (component, default) in components {
        match default {
            Some(default) => components_table.insert(&component.path(), value(default)),
            _ => panic!("invalid toml default for {component:?}"),
        };
    }
    table.insert("components", toml_edit::Item::Table(components_table));
    toml_edit::Item::Table(table)
}

macro_rules! make_make_component_default {
    ($(($value:ident, $type:ty)),*) => { paste::paste! {
        fn make_component_default(component: &ambient_ecs::PrimitiveComponent) -> Option<toml_edit::Item> {
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
