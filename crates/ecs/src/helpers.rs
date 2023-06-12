use itertools::Itertools;

use crate::{
    children, query, ArchetypeFilter, Component, ComponentValue, DynSystem, Entity, MakeDefault,
    Query, SystemGroup,
};

pub fn ensure_has_component<X: ComponentValue + 'static, T: ComponentValue + Clone + 'static>(
    if_has_component: Component<X>,
    ensure_this_component_too: Component<T>,
    value: T,
) -> DynSystem {
    Query::new(
        ArchetypeFilter::new()
            .incl(if_has_component)
            .excl(ensure_this_component_too),
    )
    .to_system_with_name("ensure_has_component", move |q, world, qs, _| {
        let ids = q.iter(world, Some(qs)).map(|ea| ea.id()).collect_vec();
        for id in ids {
            world
                .add_component(id, ensure_this_component_too, value.clone())
                .unwrap();
        }
    })
}

pub fn ensure_has_component_with_default<
    X: ComponentValue + 'static,
    T: ComponentValue + Default + Clone + 'static,
>(
    if_has_component: Component<X>,
    ensure_this_component_too: Component<T>,
) -> DynSystem {
    ensure_has_component(if_has_component, ensure_this_component_too, T::default())
}

/// Uses the MakeDefault attribute. Will panic if this attribute is not present.
pub fn ensure_has_component_with_make_default<
    X: ComponentValue + 'static,
    T: ComponentValue + Clone + 'static,
>(
    if_has_component: Component<X>,
    ensure_this_component_too: Component<T>,
) -> DynSystem {
    let default = Entity::from_iter([ensure_this_component_too
        .attribute::<MakeDefault>()
        .unwrap()
        .make_default(ensure_this_component_too.desc())]);

    query(if_has_component)
        .excl(ensure_this_component_too)
        .to_system_with_name(
            "ensure_has_component_with_make_default",
            move |q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    world.add_components(id, default.clone()).unwrap();
                }
            },
        )
}

pub fn copy_component_recursive<T: ComponentValue + Clone + 'static>(
    label: &'static str,
    component_recursive: Component<T>,
    component: Component<T>,
) -> SystemGroup {
    SystemGroup::new(
        label,
        vec![
            query((component_recursive.changed(),)).to_system(move |q, world, qs, _| {
                for (id, (val,)) in q.collect_cloned(world, qs) {
                    world.add_component(id, component, val).ok();
                }
            }),
            query((component_recursive,))
                .despawned()
                .to_system(move |q, world, qs, _| {
                    for (id, _) in q.collect_cloned(world, qs) {
                        world.remove_component(id, component).ok();
                    }
                }),
            query((component_recursive, children().changed())).to_system(move |q, world, qs, _| {
                for (_, (val, childs)) in q.collect_cloned(world, qs) {
                    for c in childs {
                        world
                            .add_component(c, component_recursive, val.clone())
                            .ok();
                    }
                }
            }),
            query((component_recursive, children()))
                .despawned()
                .to_system(move |q, world, qs, _| {
                    for (_, (_, childs)) in q.collect_cloned(world, qs) {
                        for c in childs {
                            world.remove_component(c, component_recursive).ok();
                        }
                    }
                }),
        ],
    )
}
