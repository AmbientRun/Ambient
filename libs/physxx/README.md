This repo is similar to Embarks physx package, except it maps closer to the underlying PhysX api.
It doesn't do any lifetime handling for you; once you get an object from Physx, you need to call .release
on it to destroy it. This is because the objects are actually "owned" by Physx, and the pointers
Physx returns are only references to those objects.

This repo also differs from the Embark one in that it doesn't place any type restrictions on rigid bodies;
i.e. you can have have a RigidStatic with multiple types of shapes attached to it.
