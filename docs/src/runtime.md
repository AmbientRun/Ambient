# Runtime

## Coordinate system

Ambient uses a right-handed coordinate system for NDC with z from 1 to 0; i.e. reverse-z (near plane z=1, far plane z=0). For world coordinates, it uses a left-handed system. We consider x to be right, y to be back, and z to be up (same as Unreal). This means that the default camera (no view transform) is lying on its stomach, facing downwards.

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

WebGPU uses +y is up in NDC, and z from 0 to 1 (https://gpuweb.github.io/gpuweb/#coordinate-systems), i.e. left-handed.
Overview of which programs use which coordinate systems: https://twitter.com/freyaholmer/status/1325556229410861056
More info on reverse-z: https://developer.nvidia.com/content/depth-precision-visualized https://www.danielecarbone.com/reverse-depth-buffer-in-opengl/
