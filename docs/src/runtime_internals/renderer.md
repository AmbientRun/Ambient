# Renderer

The render is gpu-driven; i.e. culling happens on the gpu, and draw calls are created on the gpu (except on web and osx).

Rendering a frame roughly looks like this:

1. The [gpu ecs](./ecs.md) synchronizes any changed values to the gpu. Note; this only happens when values have changed, and is batched for performance.
2. The renderer run culling. Only some entities are cullable; for instance, if you spawn a character which has a bunch of sub-entities (like a sword and a shield),
   only the root entity will be culled. Culling happens entirely on the GPU.
3. We run the collect phase; this is per-primitive. Note that each entity may have multiple primitives. This also runs on the gpu, and the output is a compacted list
   of draw calls.
4. On native, we run a `multi_draw_indirect_count` call for each shader/material configuration. Note that on native, the CPU does very little work each frame; most work
   happens on the gpu and the cpu doesn't need to wait for it. On web and mac we currently don't have access to multi_draw_indirect_count, so we're currently dispatching
   draw calls one by one, but we're working on improvements to this.

Some performance details:
 - Per-entity data is _only_ uploaded in the gpu_ecs, when the data changes. The rest of the renderer basically just needs to bind a shader and a material, and then draw all
   objects.
 - The shadow renderer re-uses the same TreeRenderer for all cascades; it just switches which camera to use between them.
 - Culling is done for all entities and all renderer cameras (including the shadow cameras) in one compute shader pass.
 - Lodding is performed in the culling pass as well; it will select the lod level. Each lod is a separate primitive with a lod index associated. The collect phase then only
   picks the primitive with the lod matching the one picked in the cull phase.
 - We've stress tested the renderer with hundreds of thousands of objects previously; see [this video](https://www.youtube.com/watch?v=jgkhsY8aZO8) for instance.
