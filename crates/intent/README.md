# Elements intents

Intents are a general undo/redo system. Each action ("intent") is stored as an entity.

A single intent contains data which causes an effect onto the world.

An intent defines an action, which uses the data to modify the world, and an
undo action, to undo the effect on the world.

Intents can only be applied and reverted in order.

## Draft intent

Creating a draft intent blocks other intents from being pushed onto the intent
stack.

The head _draft_ intent is then allowed to be appended.

Finalizing a draft intent converts it to a normal _immutable_ intent.

Any intents which attempted to be applied will be queued and then applied, in
order, at a later date.
