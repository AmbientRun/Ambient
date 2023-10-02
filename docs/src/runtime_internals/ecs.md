# ECS

The ECS is archetypal; each combination of components (e.g. `(translation, name, scale)` and `(translation, name, hitpoints)`) is a separate archetype, and each component is stored as a `Vec<T>` within the archetype.

## Change detection

At a conceptual level, we keep an circular buffer of all changes for each component/archetype. That means that doing a change query is extremely fast;
it will only need to iterate over the changes. However, a component can change twice or more in a frame, which should produce only one change event. To ensure this, we also keep track of the content version of each component for each entity.

## GPU ECS

The Ambient ECS also supports storing data on the GPU, through the `gpu_ecs` crate. This gives you a way to define components that live on the GPU,
and ways to synchronize data to those components.

CPU-to-GPU syncs are chunked, so in many cases it takes the same time to update one element as it does `CHUNK_SIZE` elements (currently 256).

## `components!` macro

At the root of the repository, there is an `ambient.toml` that defines all of the guest-visible components for Ambient. This is what runtime developers will typically add to when they want to add new components to Ambient.

However, there are some components that are not visible to guest code, but are still defined in host code. These components are defined using the `components!` macro. It is used like this:

```rust
components!("app", {
    @[MakeDefault[default_title], Debuggable, MaybeResource]
    window_title: String,
    fps_stats: FpsSample,
});
```

Unlike `ambient.toml`, components can be of any type that meet a set of requirements. Additionally, the components defined here will not be visible to guest code. The attributes available are a superset of those available to `ambient.toml`.

These component definitions are primarily useful for internal data that needs to be attached to entities, but should not be or cannot be visible to guest code. For example, the `FpsSample` struct in the example above is a complex type and cannot be stored in a component in guest code, but it can be stored in a component in host code.
