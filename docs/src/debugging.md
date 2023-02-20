# Debugging

## Physics

Ambient uses PhysX 4.1 from Nvidia for physics simulation. As a result, the entire physics scene can be visualized using the [PhysX Visual Debugger (PVD)](https://developer.nvidia.com/physx-visual-debugger).

By default, physics debugging is on. To debug your scene, install and start PVD, then start an Ambient project. Your project's scene should automatically be visible within PVD. For more details on how to use PVD, see the [guide](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/VisualDebugger.html).

## Assets

When assets are compiled by the assets pipeline, the resulting artifacts will be output to the `build` directory in your project. These can be examined to determine whether or not your source was accurately compiled by the asset pipeline.

Additionally, if there are fatal errors or warnings, the asset pipeline will report them during the compilation process.
