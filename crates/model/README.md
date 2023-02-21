## Transforms

There are two categories of models; "Model as scene" or "Model as single mesh".
We prefer the "Model as single mesh" since it will only require a single entity
when spawned, but at the cost of not being able to handle animations and only
supporting a single object in the scene.

### Models as scenes

```
     [Model entity]
            |
    [(Optional transform node)]
        /              \
[Scene root node 1]  [Scene root node 2] ...
```

These models can consist of multiple entities, and may have animations. In order for
these to have consistent animations when the entire model is transformed, they need
to have some kind of extra "transform" node inserted at the root.

### Models as a single meshes

```
[Model entity]
```

In this case we want the resulting spawn to just be a single entity, with the mesh attached.
All model transforms will be applied as `mesh_to_local` transforms.

## Notes on cordinate system conversion

There are two ways to handle coordinate system conversions and scaling of models;

1. Try to do it everywhere inside of the model loading
2. Just put a root node that wraps everything that does the conversion

I tried 1. first and just couldn't get it to work. This code does 2.
Also it seems like this is kind of how some exporters have solved it; the
model may internally use one system and then the exporter has put some transforms
at the root, which makes it even harder to try to do the conversions with 1.
