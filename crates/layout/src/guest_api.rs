use crate::{
    align_horizontal, align_vertical, docking, fit_horizontal, fit_vertical, layout, margin, orientation, padding, Borders, Docking, Layout,
};
use ambient_ecs::{components, ensure_has_component, query, Debuggable, Description, Name, Networked, Store, SystemGroup};

// This file only exists because Enums aren't available in the wasm layer yet; once that exists this can be removed
components!("layout", {
    @[Debuggable, Networked, Store, Name["Flow layout"], Description["Bottom-up flow layout."]]
    layout_flow: (),
    @[Debuggable, Networked, Store, Name["Dock layout"], Description["Top-down dock layout."]]
    layout_dock: (),
    @[Debuggable, Networked, Store, Name["Bookcase layout"], Description["Min-max bookcase layout."]]
    layout_bookcase: (),
    @[Debuggable, Networked, Store, Name["Layout width to children"], Description["Width to children."]]
    layout_width_to_children: (),

    @[Debuggable, Networked, Store, Name["Orientation horizontal"], Description["Layout orientation: horizontal."]]
    orientation_horizontal: (),
    @[Debuggable, Networked, Store, Name["Orientation vertical"], Description["Layout orientation: vertical."]]
    orientation_vertical: (),

    @[Debuggable, Networked, Store, Name["Align horizontal begin"], Description["Layout alignment: horizontal begin."]]
    align_horizontal_begin: (),
    @[Debuggable, Networked, Store, Name["Align horizontal center"], Description["Layout alignment: horizontal center."]]
    align_horizontal_center: (),
    @[Debuggable, Networked, Store, Name["Align horizontal end"], Description["Layout alignment: horizontal end."]]
    align_horizontal_end: (),
    @[Debuggable, Networked, Store, Name["Align vertical begin"], Description["Layout alignment: vertical begin."]]
    align_vertical_begin: (),
    @[Debuggable, Networked, Store, Name["Align vertical center"], Description["Layout alignment: vertical center."]]
    align_vertical_center: (),
    @[Debuggable, Networked, Store, Name["Align vertical end"], Description["Layout alignment: vertical end."]]
    align_vertical_end: (),

    @[Debuggable, Networked, Store, Name["Fit vertical none"], Description["Layout fit: vertical none."]]
    fit_vertical_none: (),
    @[Debuggable, Networked, Store, Name["Fit vertical parent"], Description["Layout fit: vertical parent."]]
    fit_vertical_parent: (),
    @[Debuggable, Networked, Store, Name["Fit vertical children"], Description["Layout fit: vertical children."]]
    fit_vertical_children: (),
    @[Debuggable, Networked, Store, Name["Fit horizontal none"], Description["Layout fit: horizontal none."]]
    fit_horizontal_none: (),
    @[Debuggable, Networked, Store, Name["Fit horizontal parent"], Description["Layout fit: horizontal parent."]]
    fit_horizontal_parent: (),
    @[Debuggable, Networked, Store, Name["Fit horizontal children"], Description["Layout fit: horizontal children."]]
    fit_horizontal_children: (),

    @[Debuggable, Networked, Store, Name["Margin left"], Description["Layout margin: left."]]
    margin_left: f32,
    @[Debuggable, Networked, Store, Name["Margin right"], Description["Layout margin: right."]]
    margin_right: f32,
    @[Debuggable, Networked, Store, Name["Margin top"], Description["Layout margin: top."]]
    margin_top: f32,
    @[Debuggable, Networked, Store, Name["Margin bottom"], Description["Layout margin: bottom."]]
    margin_bottom: f32,

    @[Debuggable, Networked, Store, Name["Padding left"], Description["Layout padding: left."]]
    padding_left: f32,
    @[Debuggable, Networked, Store, Name["Padding right"], Description["Layout padding: right."]]
    padding_right: f32,
    @[Debuggable, Networked, Store, Name["Padding top"], Description["Layout padding: top."]]
    padding_top: f32,
    @[Debuggable, Networked, Store, Name["Padding bottom"], Description["Layout padding: bottom."]]
    padding_bottom: f32,

    @[Debuggable, Networked, Store, Name["Docking top"], Description["Layout docking: top."]]
    docking_top: (),
    @[Debuggable, Networked, Store, Name["Docking bottom"], Description["Layout docking: bottom."]]
    docking_bottom: (),
    @[Debuggable, Networked, Store, Name["Docking left"], Description["Layout docking: left."]]
    docking_left: (),
    @[Debuggable, Networked, Store, Name["Docking right"], Description["Layout docking: right."]]
    docking_right: (),
    @[Debuggable, Networked, Store, Name["Docking fill"], Description["Layout docking: fill."]]
    docking_fill: (),
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
            ensure_has_component(docking_top(), docking(), Docking::Top),
            ensure_has_component(docking_bottom(), docking(), Docking::Bottom),
            ensure_has_component(docking_left(), docking(), Docking::Left),
            ensure_has_component(docking_right(), docking(), Docking::Right),
            ensure_has_component(docking_fill(), docking(), Docking::Fill),
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
