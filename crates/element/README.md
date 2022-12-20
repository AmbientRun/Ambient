# Elements Element

The Element module is closely modelled on React with Hooks. React Components are called Parts here (since "Component"
is already used in the ECS). There are a couple of concepts to keep track of here:

- "Entities" (React: DOM node) are just normal ECS entities, and they're kind of the "result" of all the work this module is doing.
- "Element" (React: Element). This is essentially a description of an Entity _or_ a Part. You can set normal ECS Components on it
  (for instance .set(translation(), ..)). An Element can have children, which will be translated to a children() component
  on the Entity.
- "Part" (React: Component) is basically a function which takes some user inputs + the hooks (local state of an instance) and produces
  an Element tree from this. Parts can wrap each other, so for instance an outer Part can return an Element which the inner Part only
  adds a Component to. In this case, there's only _one_ Element, even though there are two Parts. (i.e. there's not a 1:1 correspondence
  between Parts and Elements).
- "ElementInstance" (React: Instance) is an instance of an Element (remember, Elements are just descriptions). It has a reference
  to the Entity it "owns". When using Hooks, the state is stored on the ElementInstance.
- "ElementTree" is a tree of ElementInstances.

More info:
 - https://reactjs.org/blog/2015/12/18/react-components-elements-and-instances.html
