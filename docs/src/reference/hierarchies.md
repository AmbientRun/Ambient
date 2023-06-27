# Hierarchies and transforms

Ambient supports hierarchies of entities using the `parent` and `children` components. Both need to be present for a hierarchy to be valid - as an example, the following entities in the ECS

```yml
entity a:
  - children: [b, c]
entity b:
  - parent: a
entity c:
  - parent: a
```

will produce the hierarchy:

```
entity a
    entity b
    entity c
```

If you are creating hierachies yourself, you need to make sure that both `parent` and `children` exists and are correct for the hierarchy to work.

The `entity::add_child` and `entity::remove_child` functions can be used to add and remove children from a parent.

When using the `model_from_url` or `prefab_from_url` components, the entire model sub-tree will be spawned in, with the root of the sub-tree being added as a child to the entity with the component. Each entity in the sub-tree will be part of the hierarchy using their own `parent` and `children` components.

## Transforms in hierarchies

Hierarchies are common to use for transforms where a root entity is moved around and all its children should move with it.
To apply transforms to a hierarchy, `local_to_parent` must be used:

```yml
entity a:
  - children: [b]
  - local_to_world: Mat4(..)
entity b:
  - parent: a
  - local_to_parent: Mat4(..)
  - local_to_world: Mat4(..)
```

In this case, `b.local_to_world` will be calculated as `a.local_to_world * b.local_to_parent`.

`local_to_world` and `local_to_parent` are the only matrices necessary here. However, it is often more convenient to work with `translation`, `rotation` and `scale` components:

```yml
entity a:
  - children: [b]
  - local_to_world: Mat4(..)
  - translation: vec3(5., 2., 9.)
  - rotation: quat(..)
  - scale: vec3(0.5, 0.5, 1.)
entity b:
  - parent: a
  - local_to_parent: Mat4(..)
  - local_to_world: Mat4(..)
  - translation: vec3(-2., 0., 0.)
  - rotation: quat(..)
  - scale: vec3(1., 2., 1.)
```

In this case, the `local_to_world` and `local_to_parent` will automatically be recalculated from `translation`, `rotation` and `scale` whenever they change; the following computations will happen in this order:

```rust
a.local_to_world = mat4_from(a.scale, a.rotation, a.translation);
b.local_to_parent = mat4_from(b.scale, b.rotation, b.translation);
b.local_to_world = a.local_to_world * b.local_to_parent;
```
