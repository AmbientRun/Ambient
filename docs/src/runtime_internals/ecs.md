# ECS

## What is the `components!` macro in host code?

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
