# Hierarchies

Ambient supports hierarchies of entities, using the `parent` and `children` components. Both need to be present for a hierarchy to be valid, i.e.:

```yml
entity a:
    - children: [b, c]
entity b:
    - parent: a
entity c:
    - parent: a
```

Will produce the following hierachy:
```
entity a
    entity b
    entity c
```

If you are creating hierachies yourself, you need to make sure that both `parent` and `children` exists and are correct for the hierachy to work.
When working with models (i.e. `model_from_url` or `prefab_from_url`), the entire model sub-tree will be spawned with correctly wired up `parent` and `children` components.

## Transforms in hierarchies

Hierachies are common to use for transforms; i.e. you have a root entity which you move around, and all its children will move with it.
To apply transforms to a hierarchy, you need to use `local_to_parent`:

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

`local_to_world` and `local_to_parent` are the only matrices necessary here. However, it's often more conventient to work with `translation`, `scale` and `rotation` components, i.e.;

```yml
entity a:
    - children: [b]
    - local_to_world: Mat4(..)
    - translation: vec3(5., 2., 9.)
    - scale: vec3(0.5, 0.5, 1.)
    - rotation: quat(..)
entity b:
    - parent: a
    - local_to_parent: Mat4(..)
    - local_to_world: Mat4(..)
    - translation: vec3(-2., 0., 0.)
    - scale: vec3(1., 2., 1.)
    - rotation: quat(..)
```

In this case, the `local_to_world` and `local_to_parent` will be updated using the `translation`, `scale` and `rotation` whenever they change. I.e. the following computations will happen, in this order:

```rust
a.local_to_world = mat4_from(a.scale, a.rotation, a.translation);
b.local_to_parent = mat4_from(b.scale, b.rotation, b.translation);
b.local_to_world = a.local_to_world * b.local_to_parent;
```
