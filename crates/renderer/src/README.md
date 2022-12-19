Different types of LODs:

### CPU LOD group

This toggle an entire subtree on the cpu side. For example:

```yaml
- "root":
      lod_cutoffs: [0.5, 0.3]
      cpu_lod: 0
    children:
      - "a":
          lod_group: ()
          lod_visible: true
      - "b":
          lod_group: ()
          lod_visible: false
```

### Mesh lod