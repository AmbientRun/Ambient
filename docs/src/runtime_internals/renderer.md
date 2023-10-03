# Renderer

The renderer is designed to be GPU-driven, where culling happens and draw calls are issued on the GPU. However, due to complications with some of our target platforms, we are currently handling these processes on the CPU, and the following documentation may not apply. We are working to rectify this.

Rendering a frame roughly looks like this:

1. The [GPU ECS](./ecs.md) synchronizes any changed values to the GPU. Note: this only happens when values have changed, and is batched for performance.
2. The renderer runs culling. Only some entities are cullable; for instance, if you spawn a character which has a bunch of sub-entities (like a sword and a shield),
   only the root entity will be culled. Culling happens entirely on the GPU.
3. We run the collect phase; this is per-primitive. Note that each entity may have multiple primitives. This also runs on the GPU, and the output is a compacted list
   of draw calls.
4. On native, we run a `multi_draw_indirect_count` call for each shader/material configuration. Note that on native, the CPU does very little work each frame; most work
   happens on the GPU and the CPU doesn't need to wait for it. On web and macOS we currently don't have access to `multi_draw_indirect_count`, so we're currently dispatching
   draw calls one by one, but we're working on improvements to this.

Some performance details:

- Per-entity data is _only_ uploaded in the GPU ECS when the data changes. The rest of the renderer basically just needs to bind a shader and a material, and then draw all
  objects.
- The shadow renderer re-uses the same `TreeRenderer` for all cascades; it just switches which camera to use between them.
- Culling is done for all entities and all renderer cameras (including the shadow cameras) in one compute shader pass.
- Level-of-detail (LOD) selection is performed in the culling pass as well; it will select the LOD level. Each LOD is a separate primitive with a LOD index associated. The collect phase then only
  picks the primitive with the LOD matching the one picked in the cull phase.
- The renderer has been stress-tested in the past with hundreds of thousands of objects; see [this video](https://www.youtube.com/watch?v=jgkhsY8aZO8) for an example. We hope to construct an example
  that shows this off in the future, and to continue improving it.
