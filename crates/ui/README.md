# Elements UI

The Elements UI is based on React for components, and Windows Forms for layout.
To read more about the react side of things, see the [elements documentation](../element/README.md).

## Layout

The layout is roughly based on [Windows Forms](https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout). There are two major layout components, Dock and Flow (which includes FlowColumn and FlowRow). Dock is top-down,
whereas Flow is bottom up; i.e. with a Dock you start from a given area (say the screen) and then divide it into smaller pieces with
each new element added to it. The Flow on the other hand auto-resizes itself to fit it's constituent components.

