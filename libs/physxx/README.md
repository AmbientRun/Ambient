# physxx

This crate provides a semi-safe wrapper around the PhysX C++ API.

This crate is similar to Embark's PhysX crate, except it maps closer to the underlying PhysX API.
It doesn't do any lifetime handling for you; once you get an object from PhysX, you need to call `.release`
on it to destroy it. This is because the objects are actually "owned" by PhysX, and the pointers
PhysX returns are only references to those objects.

This crate also differs from the Embark one in that it doesn't place any type restrictions on rigid bodies;
i.e. you can have have a RigidStatic with multiple types of shapes attached to it.
