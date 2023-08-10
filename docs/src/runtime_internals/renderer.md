# Renderer

Rendering a frame roughly looks like this:

1. The [gpu ecs](./ecs.md) synchronizes any changed values to the gpu. Note; this only happens when values have changed, and is batched for performance.
2. The renderer run culling. Only some entities are cullable; for instance, if you spawn a character which has a bunch of sub-entities (like a sword and a shield),
   only the root entity will be culled. Culling happens entirely on the GPU.
3. We run the collect phase; this is per-primitive. Note that each entity may have multiple primitives. This also runs on the gpu, and the output is a compacted list
   of draw calls.
4. On native, we run a `multi_draw_indirect_count` call for each shader/material configuration. Note that on native, the CPU does very little work each frame; most work
   happens on the gpu and the cpu doesn't need to wait for it. On web and mac we currently don't have access to multi_draw_indirect_count, so we're currently dispatching
   draw calls one by one, but we're working on improvements to this.

