//! Defines a scroll area.
use ambient_element::{to_owned, element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        app::{window_scale_factor},
        rect::{background_color, border_radius},
        ecs::{children, parent},
        layout::{fit_horizontal_children, fit_horizontal_parent, layout_width_to_children, width, height},
        transform::{translation, local_to_world, local_to_parent},
        rendering::{scissors_recursive}
    },
    messages,
};
use glam::{vec3, vec2, Vec3, Vec2, vec4, uvec4, Vec4, Mat4};

use crate::{
    layout::{Flow, MeasureSize},
    UIBase, Rectangle,
};
use ambient_cb::cb;

/// Sizing config of a scroll area
#[derive(Debug, Clone)]
pub enum ScrollAreaSizing {
    /// Resizes the scroll area to fit the width of its children
    FitChildrenWidth,
    /// Assumes the width from the parent and propagates it to the children
    FitParentWidth,
}

/// A scroll area that can be used to scroll its child.
#[element_component]
pub fn ScrollArea(
    hooks: &mut Hooks,
    /// The scroll area sizing
    sizing: ScrollAreaSizing,
    /// The child element
    inner: Element,
) -> Element {
    let (scroll, set_scroll) = hooks.use_state(0.);
    let (inner_size, set_inner_size) = hooks.use_state(Vec2::ZERO);
    hooks.use_runtime_message::<messages::WindowMouseWheel>(move |_world, event| {
        let delta = event.delta;
        set_scroll(scroll + if event.pixels { delta.y } else { delta.y * 20. });
    });
    match sizing {
        ScrollAreaSizing::FitChildrenWidth => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    MeasureSize::el(
                        Flow(vec![inner]).el().with_default(fit_horizontal_children()).with(translation(), vec3(0., scroll, 0.)),
                        cb(move |size| {
                            set_inner_size(size);
                        }),
                    ),
                ])
                .with(width(), inner_size.x)
        }
        ScrollAreaSizing::FitParentWidth => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    Flow(vec![inner]).el().with_default(fit_horizontal_parent()).with(translation(), vec3(0., scroll, 0.)),
                ])
                .with_default(layout_width_to_children())
        }
    }
}


/// A scroll box view that can be used to scroll its child with a scroll bar.
#[element_component]
pub fn ScrollBoxView(
    hooks: &mut Hooks,
    /// Width of the scroll area
    min_width: f32,
    /// Height of the scroll area
    min_height: f32,
    /// Distance of scrolling in y axis
    scroll_height: f32,
    /// The child element
    inner: Element,
) -> Element {
    let (scroll, set_scroll) = hooks.use_state(0.);
    let (inner_size, set_inner_size) = hooks.use_state(Vec2::ZERO);
    let (ratio, set_ratio) = hooks.use_state_with(|world| {
        let r = world.resource(window_scale_factor()).clone();
        r as f32
    });

    hooks.use_runtime_message::<messages::WindowMouseWheel>(move |world, event| {
        let delta = event.delta;
        let mouse_pos = scroll + if event.pixels { delta.y } else { delta.y * 20. };
        set_scroll(mouse_pos.clamp(-scroll_height, 0.));
    });

    let bar_height = min_height / (min_height + scroll_height) * min_height;
    let offset = scroll / scroll_height * (min_height - bar_height);

    let id = hooks.use_ref_with(|_| None);
    // let container_id = hooks.use_ref_with(|_| None);
    // let pos = hooks.use_ref_with(|_| None);
    let (f, set_f) = hooks.use_state(0_u32);
    let (canvas_offset, set_canvas_offset) = hooks.use_state(Vec2::ZERO);

    hooks.use_frame({
        to_owned![id];
        move |world| {
            // TODO: for some reason, the first frame is not correct for translation
            // when the canvas is in another flow
            if f >= 2 {
                return;
            }
            if let Some(id) = *id.lock() {
                // *pos.lock() = Some(world.get(id, translation()).unwrap());
                // println!("pos: {:?}", pos);
                let pos = world.get(id, translation()).unwrap();
                set_canvas_offset(vec2( pos.x, -pos.y));
                // let sci = (vec4(0., 0., 150., 150.) * ratio).as_uvec4() +
                // uvec4(0, (*pos.lock()).unwrap().y as u32, 0, 0);
                // if let Some(container_id) = *container_id.lock() {
                //     println!("sci added: {:?}", sci);
                //     world.add_component(container_id, scissors_recursive(), sci).unwrap();
                // }
            }
            set_f(f + 1);
        }
    });


    let canvas = Rectangle
        .el()
        .with(width(), min_width)
        .with(height(), min_height)
        .with(background_color(), vec4(0.1, 0.4, 0.2, 0.4))
        // .with_default(local_to_parent())
        // .with_default(local_to_world())
        .on_spawned({
            to_owned![id];
            move |world, new_id, _| {
            *id.lock() = Some(new_id);
            // let pos = world.get(new_id, translation()).unwrap();
            // let sci = (vec4(0., 0., 150., 150.) * ratio).as_uvec4() +
            // uvec4(0, pos.y as u32, 0, 0);
            // if let Some(container_id) = *container_id.lock() {
            //     println!("sci added: {:?}", sci);
            //     world.add_component(container_id, scissors_recursive(), sci).unwrap();
            // }
            // world.add_component(new_id, scissors_recursive(), sci);
        }})
        .init_default(children())
        .children(vec![
            Flow(vec![inner]).el()
                .with_default(fit_horizontal_children())
                .with(scissors_recursive(), {
                    println!("canvas offset: {:?}", canvas_offset);
                    (vec4(
                        canvas_offset.x,
                        -canvas_offset.y,
                        min_width,
                        min_height,
                    ) * ratio).as_uvec4()
                })
                .with(translation(), vec3(0., scroll, 0.)),

            Rectangle::el()
            .with(width(), 5.)
            .with(height(), bar_height)
            .with(border_radius(), Vec4::ONE * 4.0)
            .with(background_color(), vec4(0.6, 0.6, 0.6, 1.0))
            .with_default(local_to_parent())
            .with_default(local_to_world())
            .with(translation(), vec3(min_width+200.0, -offset, 0.)),

        ]);
    canvas
}