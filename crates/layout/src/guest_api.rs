//! This file only exists because Enums aren't available in the wasm layer yet; once that exists this can be removed
use crate::{
    align_horizontal, align_vertical, docking, fit_horizontal, fit_vertical, layout, margin, orientation, padding, Borders, Docking, Layout,
};
pub use ambient_ecs::generated::components::core::layout::*;
use ambient_ecs::{ensure_has_component, query, SystemGroup};

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "layout/guest_api",
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
            query((margin_left().changed(), margin_right().changed(), margin_top().changed(), margin_bottom().changed()))
                .to_system_with_name("layout/guest_api/margin", |q, world, qs, _| {
                    for (id, (left, right, top, bottom)) in q.collect_cloned(world, qs) {
                        world.add_component(id, margin(), Borders { left, right, top, bottom }).unwrap();
                    }
                }),
            query((padding_left().changed(), padding_right().changed(), padding_top().changed(), padding_bottom().changed()))
                .to_system_with_name("layout/guest_api/padding", |q, world, qs, _| {
                    for (id, (left, right, top, bottom)) in q.collect_cloned(world, qs) {
                        world.add_component(id, padding(), Borders { left, right, top, bottom }).unwrap();
                    }
                }),
        ],
    )
}
