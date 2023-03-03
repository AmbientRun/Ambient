use crate::{align_horizontal, align_vertical, fit_horizontal, fit_vertical, layout, margin, orientation, padding, Borders, Layout};
use ambient_ecs::{components, ensure_has_component, query, Debuggable, Description, Name, Networked, Store, SystemGroup};

// This file only exists because Enums aren't available in the wasm layer yet; once that exists this can be removed
components!("ui", {
    @[Debuggable, Networked, Store, Name["Flow layout"], Description["Bottom-up flow layout."]]
    layout_flow: (),
    @[Debuggable, Networked, Store, Name["Dock layout"], Description["Top-down dock layout."]]
    layout_dock: (),
    @[Debuggable, Networked, Store, Name["Bookcase layout"], Description["Min-max bookcase layout."]]
    layout_bookcase: (),
    @[Debuggable, Networked, Store, Name["Layout width to children"], Description["Width to children."]]
    layout_width_to_children: (),

    @[Debuggable, Networked, Store, Name["Orientation horizontal"], Description["Layout orientation."]]
    orientation_horizontal: (),
    @[Debuggable, Networked, Store, Name["Orientation vertical"], Description["Layout orientation."]]
    orientation_vertical: (),

    @[Debuggable, Networked, Store, Name["Align horizontal begin"], Description["Layout component."]]
    align_horizontal_begin: (),
    @[Debuggable, Networked, Store, Name["Align horizontal center"], Description["Layout component."]]
    align_horizontal_center: (),
    @[Debuggable, Networked, Store, Name["Align horizontal end"], Description["Layout component."]]
    align_horizontal_end: (),
    @[Debuggable, Networked, Store, Name["Align vertical begin"], Description["Layout component."]]
    align_vertical_begin: (),
    @[Debuggable, Networked, Store, Name["Align vertical center"], Description["Layout component."]]
    align_vertical_center: (),
    @[Debuggable, Networked, Store, Name["Align vertical end"], Description["Layout component."]]
    align_vertical_end: (),

    @[Debuggable, Networked, Store, Name["Fit vertical none"], Description["Layout component."]]
    fit_vertical_none: (),
    @[Debuggable, Networked, Store, Name["Fit vertical parent"], Description["Layout component."]]
    fit_vertical_parent: (),
    @[Debuggable, Networked, Store, Name["Fit vertical children"], Description["Layout component."]]
    fit_vertical_children: (),
    @[Debuggable, Networked, Store, Name["Fit horizontal none"], Description["Layout component."]]
    fit_horizontal_none: (),
    @[Debuggable, Networked, Store, Name["Fit horizontal parent"], Description["Layout component."]]
    fit_horizontal_parent: (),
    @[Debuggable, Networked, Store, Name["Fit horizontal children"], Description["Layout component."]]
    fit_horizontal_children: (),

    @[Debuggable, Networked, Store, Name["Margin left"], Description["Layout component."]]
    margin_left: f32,
    @[Debuggable, Networked, Store, Name["Margin right"], Description["Layout component."]]
    margin_right: f32,
    @[Debuggable, Networked, Store, Name["Margin top"], Description["Layout component."]]
    margin_top: f32,
    @[Debuggable, Networked, Store, Name["Margin bottom"], Description["Layout component."]]
    margin_bottom: f32,

    @[Debuggable, Networked, Store, Name["Padding left"], Description["Layout component."]]
    padding_left: f32,
    @[Debuggable, Networked, Store, Name["Padding right"], Description["Layout component."]]
    padding_right: f32,
    @[Debuggable, Networked, Store, Name["Padding top"], Description["Layout component."]]
    padding_top: f32,
    @[Debuggable, Networked, Store, Name["Padding bottom"], Description["Layout component."]]
    padding_bottom: f32,
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "layout",
        vec![
            ensure_has_component(layout_flow(), layout(), Layout::Flow),
            ensure_has_component(layout_dock(), layout(), Layout::Dock),
            ensure_has_component(layout_bookcase(), layout(), Layout::Bookcase),
            ensure_has_component(layout_width_to_children(), layout(), Layout::WidthToChildren),
            ensure_has_component(orientation_horizontal(), orientation(), crate::Orientation::Horizontal),
            ensure_has_component(orientation_vertical(), orientation(), crate::Orientation::Vertical),
            ensure_has_component(align_horizontal_begin(), align_horizontal(), crate::Align::Begin),
            ensure_has_component(align_horizontal_center(), align_horizontal(), crate::Align::Center),
            ensure_has_component(align_horizontal_end(), align_horizontal(), crate::Align::End),
            ensure_has_component(align_vertical_begin(), align_vertical(), crate::Align::Begin),
            ensure_has_component(align_vertical_center(), align_vertical(), crate::Align::Center),
            ensure_has_component(align_vertical_end(), align_vertical(), crate::Align::End),
            ensure_has_component(fit_vertical_none(), fit_vertical(), crate::Fit::None),
            ensure_has_component(fit_vertical_parent(), fit_vertical(), crate::Fit::Parent),
            ensure_has_component(fit_vertical_children(), fit_vertical(), crate::Fit::Children),
            ensure_has_component(fit_horizontal_none(), fit_horizontal(), crate::Fit::None),
            ensure_has_component(fit_horizontal_parent(), fit_horizontal(), crate::Fit::Parent),
            ensure_has_component(fit_horizontal_children(), fit_horizontal(), crate::Fit::Children),
            query((margin_left().changed(), margin_right().changed(), margin_top().changed(), margin_bottom().changed())).to_system(
                |q, world, qs, _| {
                    for (id, (left, right, top, bottom)) in q.collect_cloned(world, qs) {
                        world.add_component(id, margin(), Borders { left, right, top, bottom }).unwrap();
                    }
                },
            ),
            query((padding_left().changed(), padding_right().changed(), padding_top().changed(), padding_bottom().changed())).to_system(
                |q, world, qs, _| {
                    for (id, (left, right, top, bottom)) in q.collect_cloned(world, qs) {
                        world.add_component(id, padding(), Borders { left, right, top, bottom }).unwrap();
                    }
                },
            ),
        ],
    )
}
