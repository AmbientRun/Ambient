# Runtime

## Coordinate system

By default, Ambient uses a right-handed coordinate system for NDC with `z` from 1 to 0 (i.e. reverse-z with the near plane at `z=1`, and the far plane at `z=0`).

For world coordinates, it uses a left-handed system. We consider `x` to be right, `y` to be back, and `z` to be up (same as Unreal). This means that the default camera without any transformation is lying on its stomach and facing downwards.

```
NDC:
   y
   |
   |
   0 ---> x
 /
z (coming out of the screen)

World:
  z (up)
  |
  0 --- x (right)
 /
y
```

WebGPU uses positive-`y` as up in its NDC, and `z` from 0 to 1 (https://gpuweb.github.io/gpuweb/#coordinate-systems) - that is, it is left-handed.

Freya Holmér has produced an overview of which programs use which coordinate systems, which can be found [here](https://twitter.com/freyaholmer/status/1325556229410861056).

For more information on our use of reverse-z, consult the following links:

- https://developer.nvidia.com/content/depth-precision-visualized
- https://www.danielecarbone.com/reverse-depth-buffer-in-opengl/
