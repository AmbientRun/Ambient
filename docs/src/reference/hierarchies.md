# Hierarchies and transforms

Ambient supports hierarchies of entities using the `parent` and `children` components. The user only specifies the `parent` component, the `children` are automatically derived from the existing parents.
As an example, the following entities in the ECS

```yml
entity a:
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

The `entity::add_child` and `entity::remove_child` functions can be used to add and remove children from a parent.

When using the `model_from_url` or `prefab_from_url` components, the entire model sub-tree will be spawned in, with the root of the sub-tree being added as a child to the entity with the component. Each entity in the sub-tree will be part of the hierarchy using their own `parent` and `children` components.

## Transforms in hierarchies

Hierarchies are common to use for transforms where a root entity is moved around and all its children should move with it.
To apply transforms to a hierarchy, `local_to_parent` must be used:

```yml
entity a:
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

### Mesh transforms

The above will let you express any transform hierarchy, but to reduce the number of entities, you can also use
`mesh_to_local` and `mesh_to_world`. When `mesh_to_world` exists, it replaces `local_to_world` as the "final"
transform for the renderered mesh. It's calculated as follows:

```yml
entity a:
    - local_to_world: Mat4(..)
    - mesh_to_local: Mat4(..)
    - mesh_to_world: Mat4(..)
```

```rust
mesh_to_world = local_to_world * mesh_to_local
```

This also means that you can attach a mesh in the middle of a hierarchy, with an offset. For instance, if you have
a bone hierarchy on a character, you can attach an mesh to the upper arm bone, but without `mesh_to_local/world` it
would be rendered at the center of the arm (inside the arm), so by using `mesh_to_local/world` you can offset it.

## Opting out of automatically derived children

If you whish to manage the `children` component yourself, you can attach an `unmanaged_children` component to your
entity. This stops `children` from being automatically created, and it's now up to you to populate the `children`
component to create a valid hierarchy.
