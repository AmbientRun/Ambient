use ambient_core::{
    gpu_components,
    gpu_ecs::{ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent},
    hierarchy::{children, parent},
    transform::{local_to_parent, mesh_to_local, translation},
};
use ambient_ecs::{
    components, ensure_has_component, query, query_mut, Debuggable, Description, DynSystem, EntityId, Name, Networked, Store, SystemGroup,
    World,
};
use ambient_input::picking::mouse_pickable;
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec4};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub mod guest_api;

components!("ui", {
    @[Debuggable, Networked, Store, Name["Layout"], Description["The layout to apply to this entity's children."]]
    layout: Layout,
    mesh_to_local_from_size: (),
    @[Debuggable, Networked, Store, Name["Width"], Description["The width of a UI element."]]
    width: f32,
    @[Debuggable, Networked, Store, Name["Height"], Description["The height of a UI element."]]
    height: f32,
    @[Debuggable, Networked, Store, Name["Minimum width"], Description["The minimum width of a UI element."]]
    min_width: f32,
    @[Debuggable, Networked, Store, Name["Minimum height"], Description["The minimum height of a UI element."]]
    min_height: f32,
    margin: Borders,
    padding: Borders,
    fit_vertical: Fit,
    fit_horizontal: Fit,
    docking: Docking,
    orientation: Orientation,
    align_horizontal: Align,
    align_vertical: Align,
    @[Debuggable, Networked, Store, Name["Space between items"], Description["Space between items in a layout."]]
    space_between_items: f32,
    @[Debuggable, Networked, Store, Name["Is book file"], Description["This is a file in a layout_bookcase."]]
    is_book_file: (),
    gpu_ui_size: Vec4,
});
gpu_components! {
    gpu_ui_size() => ui_size: GpuComponentFormat::Vec4,
}

pub fn init_all_components() {
    init_components();
    guest_api::init_components();
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Borders {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}
impl Borders {
    pub const ZERO: Borders = Borders { top: 0., bottom: 0., left: 0., right: 0. };
    pub fn top(top: f32) -> Self {
        Self { top, ..Default::default() }
    }
    pub fn bottom(bottom: f32) -> Self {
        Self { bottom, ..Default::default() }
    }
    pub fn left(left: f32) -> Self {
        Self { left, ..Default::default() }
    }
    pub fn right(right: f32) -> Self {
        Self { right, ..Default::default() }
    }
    pub fn horizontal(left_right: f32) -> Self {
        Self { left: left_right, right: left_right, ..Default::default() }
    }
    pub fn vertical(top_bottom: f32) -> Self {
        Self { top: top_bottom, bottom: top_bottom, ..Default::default() }
    }
    pub fn even(value: f32) -> Self {
        Self { top: value, bottom: value, left: value, right: value }
    }
    pub fn rect(top_bottom: f32, left_right: f32) -> Self {
        Self { top: top_bottom, bottom: top_bottom, left: left_right, right: left_right }
    }
    pub fn component_by_index(&self, index: usize) -> f32 {
        match index {
            0 => self.top,
            1 => self.bottom,
            2 => self.left,
            3 => self.right,
            _ => panic!("Index should be 0-3"),
        }
    }
    pub fn get_horizontal(&self) -> f32 {
        self.left + self.right
    }
    pub fn get_vertical(&self) -> f32 {
        self.top + self.bottom
    }
    pub fn set_top(mut self, top: f32) -> Self {
        self.top = top;
        self
    }
    pub fn set_bottom(mut self, bottom: f32) -> Self {
        self.bottom = bottom;
        self
    }
    pub fn set_left(mut self, left: f32) -> Self {
        self.left = left;
        self
    }
    pub fn set_right(mut self, right: f32) -> Self {
        self.right = right;
        self
    }
    pub fn offset(&self) -> Vec2 {
        vec2(self.left, self.top)
    }
    pub fn border_size(&self) -> Vec2 {
        vec2(self.get_horizontal(), self.get_vertical())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Begin,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    None,
    Parent,
    Children,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum Docking {
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Layout {
    Flow,
    Dock,
    Bookcase,
    /// Just copy the width of this component to it's children. Used for the ScrollArea
    WidthToChildren,
}

pub fn layout_systems() -> SystemGroup {
    SystemGroup::new(
        "layout",
        vec![
            Box::new(guest_api::systems()),
            // For all "normal" components, i.e. non-layout components
            query((width().changed(),)).excl(layout()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    invalidate_parent_layout(world, id, Orientation::Horizontal);
                }
            }),
            query((height().changed(),)).excl(layout()).to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    invalidate_parent_layout(world, id, Orientation::Vertical);
                }
            }),
            query((width().changed(), height().changed(), children().changed(), layout().changed())).optional_changed(parent()).to_system(
                |q, world, qs, _| {
                    let qs = qs.unwrap();
                    for _ in 0..100 {
                        let mut changed = false;
                        for (id, (_, _, children, layout)) in q.collect_cloned(world, Some(qs)) {
                            // dump_world_hierarchy_to_tmp_file(world);
                            changed = true;
                            match layout {
                                Layout::Dock => {
                                    dock_layout(world, id, children);
                                }
                                Layout::Flow => {
                                    flow_layout(world, id, children);
                                }
                                Layout::Bookcase => {
                                    bookcase_layout(world, id, children);
                                }
                                Layout::WidthToChildren => {
                                    width_to_children(world, id, children);
                                }
                            }
                        }
                        if !changed {
                            return;
                        }
                    }
                    log::warn!("Layout ran the full 100 iterations");
                },
            ),
            query_mut((mesh_to_local(),), (width().changed(), height().changed())).incl(mesh_to_local_from_size()).to_system(
                |q, world, qs, _| {
                    for (_, (mesh_to_local,), (&width, &height)) in q.iter(world, qs) {
                        *mesh_to_local = Mat4::from_scale(vec3(width, height, 1.));
                    }
                },
            ),
            node_clickable_system(),
            query_mut((gpu_ui_size(),), (width().changed(), height().changed())).to_system(|q, world, qs, _| {
                for (_, (size,), (width, height)) in q.iter(world, qs) {
                    *size = vec4(*width, *height, 0., 0.);
                }
            }),
        ],
    )
}

pub fn gpu_world_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "ui/layout/gpu_world",
        vec![Box::new(ComponentToGpuSystem::new(GpuComponentFormat::Vec4, gpu_ui_size(), gpu_components::ui_size()))],
    )
}

const Z_DELTA: f32 = -0.00001;

fn dock_layout(world: &mut World, id: EntityId, children: Vec<EntityId>) {
    let padding = world.get(id, padding()).unwrap_or(Borders::ZERO);
    let orientation = world.get(id, orientation()).unwrap_or(Orientation::Vertical);
    let default_dock = match orientation {
        Orientation::Vertical => Docking::Top,
        Orientation::Horizontal => Docking::Left,
    };
    let mut remaining_size = vec2(world.get(id, width()).unwrap_or(0.), world.get(id, height()).unwrap_or(0.)) - padding.border_size();
    let mut remaining_offset = padding.offset();
    for (i, &c) in children.iter().enumerate() {
        let dock = world.get(c, docking()).unwrap_or(if i == children.len() - 1 { Docking::Fill } else { default_dock });
        let child_fit_horizontal = world.get(c, fit_horizontal()).unwrap_or(Fit::Parent);
        let child_fit_vertical = world.get(c, fit_vertical()).unwrap_or(Fit::Parent);
        let child_margin = world.get(c, margin()).unwrap_or(Borders::ZERO);
        match dock {
            Docking::Top => {
                world.set_if_changed(c, translation(), (remaining_offset + child_margin.offset()).extend(Z_DELTA)).ok();
                if child_fit_horizontal != Fit::Children {
                    world.set_if_changed(c, width(), remaining_size.x - child_margin.get_horizontal()).ok();
                }
                let height = world.get(c, height()).unwrap_or(0.) + child_margin.get_vertical();
                remaining_offset.y += height;
                remaining_size.y -= height;
            }
            Docking::Bottom => {
                let height = world.get(c, height()).unwrap_or(0.);
                world
                    .set_if_changed(
                        c,
                        translation(),
                        vec3(
                            remaining_offset.x + child_margin.left,
                            remaining_offset.y + remaining_size.y - height - child_margin.top,
                            Z_DELTA,
                        ),
                    )
                    .ok();
                if child_fit_horizontal != Fit::Children {
                    world.set_if_changed(c, width(), remaining_size.x - child_margin.get_horizontal()).ok();
                }
                remaining_size.y -= height + child_margin.get_vertical();
            }
            Docking::Left => {
                world.set_if_changed(c, translation(), (remaining_offset + child_margin.offset()).extend(Z_DELTA)).ok();
                if child_fit_vertical != Fit::Children {
                    world.set_if_changed(c, height(), remaining_size.y - child_margin.get_vertical()).ok();
                }
                let width = world.get(c, width()).unwrap_or(0.) + child_margin.get_horizontal();
                remaining_offset.x += width;
                remaining_size.x -= width;
            }
            Docking::Right => {
                let width = world.get(c, width()).unwrap_or(0.);
                world
                    .set_if_changed(
                        c,
                        translation(),
                        vec3(
                            remaining_offset.x + remaining_size.x - width - child_margin.left,
                            remaining_offset.y + child_margin.top,
                            Z_DELTA,
                        ),
                    )
                    .ok();
                if child_fit_vertical != Fit::Children {
                    world.set_if_changed(c, height(), remaining_size.y - child_margin.get_vertical()).ok();
                }
                remaining_size.x -= width + child_margin.get_horizontal();
            }
            Docking::Fill => {
                world.set_if_changed(c, translation(), (remaining_offset + child_margin.offset()).extend(Z_DELTA)).ok();
                if child_fit_horizontal != Fit::Children {
                    world.set_if_changed(c, width(), remaining_size.x - child_margin.get_horizontal()).ok();
                }
                if child_fit_vertical != Fit::Children {
                    world.set_if_changed(c, height(), remaining_size.y - child_margin.get_vertical()).ok();
                }
                remaining_offset.x += remaining_size.x;
                remaining_offset.y += remaining_size.y;
                remaining_size.x = 0.;
                remaining_size.y = 0.;
            }
        }
    }
}

fn flow_layout(world: &mut World, id: EntityId, children: Vec<EntityId>) {
    let orientation = world.get(id, orientation()).unwrap_or(Orientation::Horizontal);
    let space_between_items = world.get(id, space_between_items()).unwrap_or(0.);
    let self_padding = world.get(id, padding()).unwrap_or(Borders::ZERO);
    let self_size = vec2(world.get(id, width()).unwrap_or(0.), world.get(id, height()).unwrap_or(0.));
    let mut offset = Vec2::ZERO;
    let self_fit_horizontal = world.get(id, fit_horizontal()).unwrap_or(Fit::None);
    let self_fit_vertical = world.get(id, fit_vertical()).unwrap_or(Fit::None);
    let self_min_width = world.get(id, min_width()).unwrap_or(0.);
    let self_min_height = world.get(id, min_height()).unwrap_or(0.);
    let self_max_width = if self_fit_horizontal == Fit::Children { f32::INFINITY } else { self_size.x };
    let self_max_height = if self_fit_vertical == Fit::Children { f32::INFINITY } else { self_size.y };
    let mut children_width: f32 = 0.;
    let mut children_height: f32 = 0.;
    let mut line_width = 0.;
    let mut line_height = 0.;
    let children = children.iter().filter(|id| world.has_component(**id, local_to_parent())).copied().collect_vec();
    let items = children
        .iter()
        .map(|&c| {
            let child_margin = world.get(c, margin()).unwrap_or(Borders::ZERO);

            let child_fit_horizontal = world.get(c, fit_horizontal()).unwrap_or(Fit::None);
            let child_fit_vertical = world.get(c, fit_vertical()).unwrap_or(Fit::None);

            let child_size = vec2(
                if child_fit_horizontal == Fit::Parent {
                    0.
                } else {
                    world.get(c, width()).unwrap_or(0.) + child_margin.left + child_margin.right
                },
                if child_fit_vertical == Fit::Parent {
                    0.
                } else {
                    world.get(c, height()).unwrap_or(0.) + child_margin.top + child_margin.bottom
                },
            );
            let break_line = match orientation {
                Orientation::Horizontal => offset.x + child_size.x >= self_max_width,
                Orientation::Vertical => offset.y + child_size.y >= self_max_height,
            };
            if break_line {
                match orientation {
                    Orientation::Horizontal => {
                        offset.x = 0.;
                        offset.y += line_height;
                        line_height = 0.;
                    }
                    Orientation::Vertical => {
                        offset.y = 0.;
                        offset.x += line_width;
                        line_width = 0.;
                    }
                }
            }
            children_width = children_width.max(offset.x + child_size.x);
            children_height = children_height.max(offset.y + child_size.y);
            let child_position = vec3(child_margin.left, child_margin.top, 0.) + offset.floor().extend(Z_DELTA);
            match orientation {
                Orientation::Horizontal => offset.x += child_size.x + space_between_items,
                Orientation::Vertical => offset.y += child_size.y + space_between_items,
            }
            line_width = line_width.max(child_size.x);
            line_height = line_height.max(child_size.y);

            child_position
        })
        .collect_vec();

    let inner_width = children_width.max(self_min_width - self_padding.get_horizontal()) + self_padding.get_horizontal();
    let inner_height = children_height.max(self_min_height - self_padding.get_vertical()) + self_padding.get_vertical();

    let new_self_width = if self_fit_horizontal == Fit::Children { inner_width } else { self_size.x };
    let new_self_height = if self_fit_vertical == Fit::Children { inner_height } else { self_size.y };

    let align_horizontal = world.get(id, align_horizontal()).unwrap_or(Align::Begin);
    let align_vertical = world.get(id, align_vertical()).unwrap_or(Align::Begin);
    let align_left = match align_horizontal {
        Align::Begin => self_padding.left,
        Align::Center => (new_self_width - children_width) / 2.,
        Align::End => new_self_width - children_width - self_padding.left,
    };

    let align_top = match align_vertical {
        Align::Begin => self_padding.top,
        Align::Center => (new_self_height - children_height) / 2.,
        Align::End => new_self_height - children_height - self_padding.top,
    };

    for (&c, pos) in children.iter().zip(items.into_iter()) {
        let child_margin = world.get(c, margin()).unwrap_or(Borders::ZERO);
        let child_base_position = vec3(align_left, align_top, 0.) + pos;
        let child_fit_horizontal = world.get(c, fit_horizontal()).unwrap_or(Fit::None);
        let child_fit_vertical = world.get(c, fit_vertical()).unwrap_or(Fit::None);
        let child_width = if child_fit_horizontal == Fit::Parent {
            let child_new_width = new_self_width - child_base_position.x - child_margin.right - self_padding.right;
            world.set_if_changed(c, width(), child_new_width).ok();
            child_new_width
        } else {
            world.get(c, width()).unwrap_or(0.)
        };
        let child_height = if child_fit_vertical == Fit::Parent {
            let child_new_height = new_self_height - child_base_position.y - child_margin.bottom - self_padding.bottom;
            world.set_if_changed(c, height(), child_new_height).ok();
            child_new_height
        } else {
            world.get(c, height()).unwrap_or(0.)
        };
        let mut child_position = child_base_position;
        match orientation {
            Orientation::Horizontal => match align_vertical {
                Align::Begin => {}
                Align::Center => {
                    child_position.y += (children_height - child_height) / 2.;
                }
                Align::End => {
                    child_position.y += children_height - child_height;
                }
            },
            Orientation::Vertical => match align_horizontal {
                Align::Begin => {}
                Align::Center => {
                    child_position.x += (children_width - child_width) / 2.;
                }
                Align::End => {
                    child_position.x += children_width - child_width;
                }
            },
        }
        world.set_if_changed(c, translation(), child_position).ok();
    }
    if self_fit_horizontal == Fit::Children && self_size.x != new_self_width {
        world.set(id, width(), new_self_width).ok();
        invalidate_parent_layout(world, id, Orientation::Horizontal);
    }
    if self_fit_vertical == Fit::Children && self_size.y != new_self_height {
        world.set(id, height(), new_self_height).ok();
        invalidate_parent_layout(world, id, Orientation::Vertical);
    }
}

fn bookcase_layout(world: &mut World, id: EntityId, files: Vec<EntityId>) {
    let orientation = world.get(id, orientation()).unwrap_or(Orientation::Horizontal);
    let self_size = vec2(world.get(id, width()).unwrap_or(0.), world.get(id, height()).unwrap_or(0.));
    let mut children_size = Vec2::ZERO;
    let mut offset = Vec2::ZERO;
    let to_update = files
        .iter()
        .map(|&file| {
            assert!(world.has_component(file, is_book_file()), "Each child of a Bookcase should be a BookFile");
            let file_childs = world.get_ref(file, children()).expect("BookFile must contain children");
            let container = file_childs[0];
            let book = file_childs[1];
            let book_size = vec2(world.get(book, width()).unwrap_or(0.), world.get(book, height()).unwrap_or(0.));
            children_size.x = children_size.x.max(book_size.x);
            children_size.y = children_size.y.max(book_size.y);
            world.set_if_changed(file, translation(), offset.extend(Z_DELTA)).ok();
            world.set_if_changed(container, translation(), Vec2::ZERO.extend(Z_DELTA)).ok();
            world.set_if_changed(book, translation(), Vec2::ZERO.extend(Z_DELTA * 10.)).ok();
            if orientation == Orientation::Vertical {
                offset.y += book_size.y;
            } else {
                offset.x += book_size.x;
            }
            (container, book_size)
        })
        .collect_vec();
    for (container, book_size) in to_update {
        if orientation == Orientation::Vertical {
            world.set_if_changed(container, width(), children_size.x).ok();
            world.set_if_changed(container, height(), book_size.y).ok();
        } else {
            world.set_if_changed(container, width(), book_size.x).ok();
            world.set_if_changed(container, height(), children_size.y).ok();
        }
    }
    let new_size = if orientation == Orientation::Vertical { vec2(children_size.x, offset.y) } else { vec2(offset.x, children_size.x) };
    if new_size.x != self_size.x {
        world.set(id, width(), new_size.x).ok();
        invalidate_parent_layout(world, id, Orientation::Horizontal);
    }
    if new_size.y != self_size.y {
        world.set(id, height(), new_size.y).ok();
        invalidate_parent_layout(world, id, Orientation::Vertical);
    }
}

fn width_to_children(world: &mut World, id: EntityId, children: Vec<EntityId>) {
    let self_width = world.get(id, width()).unwrap_or(0.);
    for c in children {
        world.set(c, width(), self_width).ok();
    }
}

fn invalidate_parent_layout(world: &mut World, id: EntityId, orientation: Orientation) {
    let self_is_parent_fit = match orientation {
        Orientation::Horizontal => world.get(id, fit_horizontal()).unwrap_or(Fit::None) == Fit::Parent,
        Orientation::Vertical => world.get(id, fit_vertical()).unwrap_or(Fit::None) == Fit::Parent,
    };
    if self_is_parent_fit {
        return;
    }
    if let Ok(parent) = world.get(id, parent()) {
        let comp = match orientation {
            Orientation::Horizontal => width(),
            Orientation::Vertical => height(),
        };
        world.get_mut(parent, comp).ok();
        if world.has_component(parent, is_book_file()) {
            invalidate_parent_layout(world, parent, orientation);
        }
    }
}

fn node_clickable_system() -> DynSystem {
    query_mut((mouse_pickable(),), (width().changed(), height().changed())).to_system(|q, world, qs, _| {
        for (_, (pickable,), (&width, &height)) in q.iter(world, qs) {
            pickable.max = vec3(width, height, 0.0001);
        }
    })
}
