# Debugging

## Physics / physx

Ambient uses physx 4.1 from Nvidia for the physics simulation. You can visualize the entire phyiscs scene using pvd from nvidia.
Essentially, you just need to install PVD, start it and then start a Ambient project, and it should show the project.
See https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/VisualDebugger.html for more details.

## Assets

If the assets aren't updating, you probably need to clear the cache. Delete the `tmp/http` folder to clear it.
