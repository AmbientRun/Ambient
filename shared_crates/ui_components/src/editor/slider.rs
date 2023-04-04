use std::f32::consts::E;

use ambient_cb::{cb, Cb};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        app::cursor_position,
        input::{event_mouse_input, event_mouse_motion},
        layout::{height, space_between_items, width},
        rect::{background_color, border_radius},
        transform::{local_to_world, translation},
    },
    ecs::EntityId,
    window::set_cursor,
};
use ambient_math::{interpolate, interpolate_clamped};
use ambient_shared_types::events::{WINDOW_MOUSE_INPUT, WINDOW_MOUSE_MOTION};
use ambient_window_types::CursorIcon;
use glam::{vec3, Vec4};

use super::Editor;
use crate::{
    default_theme::{primary_color, STREET},
    editor::EditorOpts,
    layout::FlowRow,
    text::Text,
    Rectangle, UIBase, UIExt,
};

#[derive(Clone, Debug)]
pub struct Slider {
    pub value: f32,
    pub on_change: Option<Cb<dyn Fn(f32) + Sync + Send>>,
    pub min: f32,
    pub max: f32,
    pub width: f32,
    pub logarithmic: bool,
    pub round: Option<u32>,
    pub suffix: Option<&'static str>,
}
impl Slider {
    pub fn new(value: f32, on_change: impl Fn(f32) + Sync + Send + 'static) -> Self {
        Self { value, on_change: Some(cb(on_change)), min: 0., max: 1., width: 100., logarithmic: false, round: None, suffix: None }
    }
    #[cfg(feature = "guest")]
    pub fn new_for_entity_component(hooks: &mut Hooks, entity: EntityId, component: ambient_guest_bridge::ecs::Component<f32>) -> Self {
        use ambient_guest_bridge::api::entity;
        let rerender = hooks.use_rerender_signal();
        Self::new(entity::get_component(entity, component).unwrap_or_default(), move |value| {
            entity::set_component(entity, component, value);
            rerender();
        })
    }
}

impl ElementComponent for Slider {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Slider { value, min, max, width: slider_width, logarithmic, round, suffix, .. } = *self;
        const THUMB_WIDTH: f32 = 12.;
        const SLIDER_HEIGHT: f32 = 12.;

        fn cleanup_value(value: f32, min: f32, max: f32, round: Option<u32>) -> f32 {
            let mut processed = value.clamp(min, max);

            if let Some(decimal_precision) = round {
                let exponent = 10.0f32.powi(decimal_precision as i32);
                processed = (processed * exponent).round() / exponent;
            }

            processed
        }

        let value = cleanup_value(value, min, max, round);
        hooks.use_spawn({
            let on_change = self.on_change.clone();
            let old_value = self.value;
            move |_| {
                if old_value != value {
                    if let Some(on_change) = on_change {
                        on_change(value);
                    }
                }

                Box::new(|_| {})
            }
        });
        let block_id = hooks.use_ref_with(|_| EntityId::null());
        let is_moveable = self.on_change.is_some();
        // Sets the value with some sanitization
        let on_change_raw =
            self.on_change.map(|f| -> Cb<dyn Fn(f32) + Sync + Send> { cb(move |value: f32| f(cleanup_value(value, min, max, round))) });
        // Sets the value after converting from [0-1] to the value range
        let on_change_factor =
            on_change_raw.clone().map(|f| cb(move |p: f32| f(if logarithmic { p.powf(E) } else { p } * (max - min) + min)));

        // f(x) = p ^ e
        // f'(f(x)) = x
        // f'(y) = y ^ (1/e)
        // (p ^ e) ^ (1/e) = p ^ (e / e) = p ^ 1 = p
        let p = interpolate(value, min, max, 0., 1.);
        let block_left_offset = if logarithmic { p.powf(1. / E) } else { p } * (slider_width - THUMB_WIDTH);
        let block_left_offset = if block_left_offset.is_nan() || block_left_offset.is_infinite() { 0. } else { block_left_offset };

        let dragging = hooks.use_ref_with(|_| false);
        hooks.use_event(WINDOW_MOUSE_INPUT, {
            let dragging = dragging.clone();
            move |_, event| {
                if let Some(pressed) = event.get(event_mouse_input()) {
                    if !pressed {
                        *dragging.lock() = false;
                    }
                }
            }
        });

        hooks.use_event(WINDOW_MOUSE_MOTION, {
            let dragging = dragging.clone();
            let block_id = block_id.clone();
            move |world, event| {
                if let Some(on_change_factor) = &on_change_factor {
                    if *dragging.lock() && event.get_ref(event_mouse_motion()).is_some() {
                        let block_id = *block_id.lock();
                        let (_, _, block_position) = world.get(block_id, local_to_world()).unwrap().to_scale_rotation_translation();
                        let block_width = world.get(block_id, width()).unwrap_or_default();
                        let position = world.resource(cursor_position());
                        on_change_factor(interpolate_clamped(position.x, block_position.x, block_position.x + block_width, 0., 1.));
                    }
                }
            }
        });

        let rectangle = Rectangle
            .el()
            .with(width(), slider_width)
            .with(height(), 2.)
            .with(translation(), vec3(0., (SLIDER_HEIGHT - 2.) / 2., 0.))
            .with(background_color(), primary_color().into())
            .on_spawned(move |_, id, _| *block_id.lock() = id);

        let thumb = {
            let thumb = UIBase
                .el()
                .with(width(), THUMB_WIDTH)
                .with(height(), SLIDER_HEIGHT)
                .with_background(primary_color().into())
                .with(border_radius(), Vec4::ONE * THUMB_WIDTH / 2.)
                .with(translation(), vec3(block_left_offset, 0., -0.01))
                .with_clickarea()
                .on_mouse_enter(|world, _| {
                    set_cursor(world, CursorIcon::Hand);
                })
                .on_mouse_leave(|world, _| {
                    set_cursor(world, CursorIcon::Default);
                });

            if is_moveable {
                thumb
                    .on_mouse_down(move |_world, _id, _| {
                        *dragging.lock() = true;
                    })
                    .el()
            } else {
                thumb.el()
            }
        };

        FlowRow::el([
            UIBase.el().with(width(), slider_width).with(height(), SLIDER_HEIGHT).children(vec![rectangle, thumb]),
            FlowRow::el([f32::edit_or_view(value, on_change_raw, EditorOpts::default()), suffix.map(Text::el).unwrap_or_default()]),
        ])
        .with(space_between_items(), STREET)
    }
}

#[derive(Clone, Debug)]
pub struct IntegerSlider {
    pub value: i32,
    pub on_change: Option<Cb<dyn Fn(i32) + Sync + Send>>,
    pub min: i32,
    pub max: i32,
    pub width: f32,
    pub logarithmic: bool,
    pub suffix: Option<&'static str>,
}
impl ElementComponent for IntegerSlider {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self { value, on_change, min, max, width, logarithmic, suffix } = *self;
        Slider {
            value: value as f32,
            on_change: on_change.map(|on_change| -> Cb<dyn Fn(f32) + Sync + Send> { cb(move |value: f32| on_change(value as i32)) }),
            min: min as f32,
            max: max as f32,
            width,
            logarithmic,
            round: None,
            suffix,
        }
        .el()
    }
}
