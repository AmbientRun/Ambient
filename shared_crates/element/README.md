# Ambient Element

Element is a React-inspired virtual tree library for the Ambient runtime.

It is backed by the Ambient ECS; the virtual tree is converted into a real tree of entities and components.
When the tree is updated, it is compared to the previous tree, and only the differences are applied to the ECS.
This can be used for UI, as well as any other tree-like data structure that you want to be able to update efficiently.

The Element module is closely modelled on React with Hooks. React Components are called `ElementComponent`s here (since `Component`
is already used by the ECS). There are a couple of concepts to keep track of here:

- `Entities` (React: DOM node) are just normal ECS entities, and they're kind of the "result" of all the work this module is doing.
- `Element` (React: `Element`). This is essentially a description of an `Entity` _or_ a `ElementComponent`. You can set normal ECS Components on it
  (for instance `.set(translation(), ..)`). An Element can have children, which will be translated to a `children()` component
  on the Entity.
- `ElementComponent` (React: `Component`) is basically a function which takes some user inputs + the hooks (local state of an instance) and produces
  an Element tree from this. `ElementComponent`s can wrap each other, so for instance an outer `ElementComponent` can return an Element which the inner `ElementComponent` only
  adds a Component to. In this case, there's only _one_ Element, even though there are two `ElementComponent`s. (i.e. there's not a 1:1 correspondence
  between `ElementComponent`s and `Element`s).
- `ElementInstance` (React: `Instance`) is an instance of an `Element` (remember, `Element`s are just descriptions). It has a reference
  to the Entity it "owns". When using Hooks, the state is stored on the `ElementInstance`.
- `ElementTree` is a tree of `ElementInstance`s.

More info:

- https://reactjs.org/blog/2015/12/18/react-components-elements-and-instances.html
