A boids-based benchmark which invokes as many of Ambient's elements as possible in a relatively organic videogame-simulation-like way.

So far what's covered is:
- Lots of entities
- Many models
- Server to client syncing (many server-side changes)
- Shadows
- Queries, change_queries, spawn_queries, despawn_queries
- Entity spawning and despawning
- High number of transparent objects in same transparency group
- Animations, speed varying by entity
- High number of frequently-updating UI elements (they can be disabled by sliding "Neighbour Count Opacity" to 0)

What's missing:
- Physics
- High volume of client to server messages
- High number of packages (There are only a small number being used here)
- High number of transparent objects in different transparency groups (?)
- Audio