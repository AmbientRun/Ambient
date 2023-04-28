# Ambient guest bridge

This crate makes it possible to write code that can be used either internally in the Ambient runtime, or in a guest crate using `ambient_api`.

It does this by abstracting over both APIs and providing a common interface that can be targeted by downstream code to allow running that code in either context.
