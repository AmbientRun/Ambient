# Entity Component System (ECS)

An entity component system (ECS) is an architectural pattern that is used in game development to organize the logic of a game. It is a data-oriented approach to programming, which means that it focuses on the data that is being processed, rather than the logic that is processing it.

The ECS pattern is based on three concepts: _entities_, _components_, and _systems_. Entities are the objects that exist in the game world. Components are the data that describe the entities. Systems are the logic that processes the components.

Conceptually, the ECS can be considered to be a database, where the entities are the rows, the components are the columns, and the systems are the queries. The ECS is designed to be fast and efficient, and is used in many modern game engines.

In addition to the three core concepts, Ambient also supports _concepts_, which are a way of defining a collection of components that correspond to some concept in the game world. For example, a `Player` concept might be defined as a collection of components that describe the player's health, inventory, and position.

## Entities

Entities are the objects that exist in the game world. They consist of a unique identifier (an `EntityId`, which is 128 bits) and a set of components. Entities are created and destroyed dynamically during runtime.

## Components

Components are pieces of data that can be attached to entities. They store information like health, position, velocity, and more. Components are defined in the package manifest, and are attached to entities at runtime.

They are defined in the manifest (and not your codebase) so that other packages that depend on your package can use them when interacting with the ECS. Additionally, this means that component definitions are not tied to a specific language, and can be used in any language that supports the runtime.

For more detail on what components can be, see the [package manifest reference](package.md#components--components). Note that component types cannot be nested - you cannot have a component that is a `Vec` of `Vec`s.

### Attributes

Components can have attributes that modify their behavior. These attributes are defined in the package manifest, and are used by the runtime to determine how to handle the component.

#### `Debuggable`

This component can have its debug value printed. This is most often used for ECS dumps, but can also be used for debugging purposes.

#### `Networked`

This component is networked to the client. This means that the component's value will be sent to the client when the component is created, and whenever the component's value changes.

Note that a component that is `Networked` on the client will _not_ be networked to the server. Ambient's ECS networking is strictly server to client; to send data from the client to the server, you must use [messages](package.md#messages--messages).

#### `Resource`

This component will only ever be attached to the `entity::resources()` entity, which is always present in the world. This is useful for storing global state that is not tied to a specific entity.

This component will error when attached to any other entity. Note that the resources entity is not networked; if you want networked global state, consider using `entity::synchronized_resources()`.

#### `MaybeResource`

This component can be used as either a resource or as a component. This is useful for components that are traditionally attached to entities, but are sometimes attached to the resource entity.

This is most commonly used for components that are used in the resources of a prefab to provide metadata about the prefab. It is unlikely you will need to interact with this directly as a user.

#### `Store`

This component's value will be stored in the world file. This is useful for components that store persistent state, like the player's inventory.

At present, Ambient does not support persistency. This functionality will be added in the future.

## Systems

Systems are the logic that processes the components. Ambient guest code cannot directly define systems; instead, they rely on queries that run every frame. These function identically to systems for now, but systems may be formally introduced in the future to allow for more advanced functionality, including automatic parallelism of the ECS.

Queries are powerful, and can be used to query for entities that have a specific component, or a specific set of components. At present, they are entirely structural, so they cannot be used to query for entities that have a specific value for a component.

There are three types of queries in Ambient at present: general queries, (de)spawn queries, and change queries.

General queries are the most common type of query. They are used to query for entities that have a specific set of components:

```rust
query((player(), player_camera_ref(), translation(), rotation())).each_frame(move |players| {
    for (_, (_, camera_id, pos, rot)) in players {
        let forward = rot * Vec3::X;
        entity::set_component(camera_id, lookat_target(), pos);
        entity::set_component(camera_id, translation(), pos - forward * 4. + Vec3::Z * 2.);
    }
});
```

Spawn queries are used to query for when specific components are added to entities (including the entire entity being spawned). They are useful for spawning entities when a player joins the game, for example:

```rust
spawn_query(player()).bind(move |players| {
    // For each player joining, spawn a random colored box somewhere
    for _ in players {
        Entity::new()
            .with_merge(make_transformable())
            .with(cube(), ())
            .with(translation(), rand::random())
            .with(color(), rand::random::<Vec3>().extend(1.0))
            .spawn();
    }
});
```

Despawn queries are similar to spawn queries, but track the removal of components from entities (including the entire entity being despawned). They are useful for cleaning up entities when a player leaves the game, for example:

```rust
despawn_query(user_id()).requires(player()).bind(move |players| {
    for (_, user_id) in players {
        println!("Player {user_id} left");
    }
});
```

Finally, change queries are activated when one of the components they track change. Note that the components that are returned by the query are separate from the components that are tracked; this allows you to get more information about the entity than just the components that changed.

```rust
change_query((user_id(), health())).track_change(health()).requires(player()).bind(move |players| {
    for (_, (user_id, health)) in players {
        println!("Player {user_id} now has {health} health");
    }
});
```

In addition to specifying components in the query, you can also specify components that must be needed using `.requires` or components that must not be present using `.excludes`. These are useful for filtering out entities that should not be processed by the query.

## Concepts

Concepts are defined in the package manifest, and are used to define a collection of components that correspond to some concept in the game world. For example, a `Player` concept might be defined as a collection of components that describe the player's health, inventory, and position.

Concepts have an ID (specified as the name of their TOML table), a name, a description, and a list of components with defaults. Additionally, they can extend other concepts, which will cause them to inherit the components and defaults of the concept they extend.

For illustration, here are two concepts that are defined as part of Ambient's default manifest:

```toml
[concepts.transformable]
name = "Transformable"
description = "Can be translated, rotated and scaled."

[concepts.transformable.components]
translation = [0.0, 0.0, 0.0]
scale = [1.0, 1.0, 1.0]
rotation = [0.0, 0.0, 0.0, 1.0]

[concepts.camera]
name = "Camera"
description = "Base components for a camera. You will need other components to make a fully-functioning camera."
extends = ["transform::transformable"]

[concepts.camera.components]
near = 0.1
projection = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
projection_view = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
active_camera = 0.0
"transform::local_to_world" = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
"transform::inv_local_to_world" = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
```

In this example, the "camera" concept contains all of the components from a transformable, as well as components of its own. This means that any entity that has the "camera" concept will also have the components from the "transformable" concept.

Concepts are exposed to your Rust code in three ways, using `camera` as an example:

- `camera()`: returns a tuple of the components that are part of the `camera` concept. This can be used within queries to query for entities that have the `camera` concept.
- `make_camera()`: makes a `Entity` with the components of the `camera` concept, which can then be spawned.
- `is_camera(id)`: returns true if the entity with the given ID contains all of the components of the `camera` concept.
