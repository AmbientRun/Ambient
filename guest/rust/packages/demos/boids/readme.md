A boids-based benchmark which invokes as many of Ambient's elements as possible in a relatively organic videogame-simulation-like way.

So far what's covered is:
- Lots of entities
- Many models
- Server to client syncing (many server-side changes)
- Shadows
- Queries, change_queries, spawn_queries, despawn_queries
- Entity spawning and despawning
- High number of transparent objects in same transparency group

What's missing:
- Physics
- Animations
- High volume of client to server messages
- High number of packages (There are only 3 being used here)
- High number of transparent objects in different transparency groups (?)
- High UI complexity, high number of UI elements
- Audio