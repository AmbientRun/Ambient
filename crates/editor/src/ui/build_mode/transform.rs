use std::{sync::Arc, time::Duration};

use ambient_core::{runtime, transform::get_world_transform, window::cursor_position, window::screen_to_clip_space};
use ambient_ecs::{EntityId, World};
use ambient_element::{element_component, Element, ElementComponent, ElementComponentExt, Group, Hooks};
use ambient_network::client::GameClient;
use ambient_std::{
    cb,
    shapes::{Plane, Ray, RayIntersectable},
    Cb,
};
use ambient_ui::{space_between_items, Button, FlowRow, HighjackMouse, Hotkey, Separator, STREET};
use ambient_window_types::MouseButton;
use ambient_window_types::{ModifiersState, VirtualKeyCode};
use anyhow::Context;
use glam::{vec3, Mat4, Quat, Vec2, Vec3, Vec3Swizzles};
use itertools::Itertools;
use parking_lot::Mutex;

use crate::{
    intents::{intent_place_ray, intent_set_transform, intent_translate, IntentPlaceRay, IntentTransform, IntentTranslate, TerrainOffset},
    rpc::AxisFlags,
    ui::{
        build_mode::{AxisGuide, EditorAction, GridGuide},
        EditorPrefs,
    },
};
const TRANSFORM_THROTTLE: Duration = Duration::from_millis(60);

fn get_world_transforms(world: &World, targets: &[EntityId]) -> anyhow::Result<Vec<Mat4>> {
    targets
        .iter()
        .map(|id| {
            let transform = get_world_transform(world, *id).context("No transform")?;
            Ok(transform)
        })
        .collect()
}

fn to_isometry(transform: Mat4) -> Mat4 {
    let (_, rot, pos) = transform.to_scale_rotation_translation();

    Mat4::from_scale_rotation_translation(Vec3::ONE, rot, pos)
}

#[derive(PartialEq, Copy, Debug, Clone)]
enum ConstraintSpace {
    Plane { normal: Vec3, point: Vec3 },
    Axis { axis: Vec3, point: Vec3 },
}

impl ConstraintSpace {
    pub fn constrain(&self, p: Vec3) -> Vec3 {
        match *self {
            ConstraintSpace::Plane { normal, point } => point + (p - point).reject_from_normalized(normal),
            ConstraintSpace::Axis { axis, point } => point + axis * (p - point).dot(axis),
        }
    }

    pub fn to_plane(self, view: Mat4) -> Plane {
        let forward = view.transform_vector3(Vec3::Z);

        match self {
            ConstraintSpace::Plane { normal, point } => {
                assert!(normal.is_normalized());
                Plane::from_normal_and_point(normal, point)
            }
            ConstraintSpace::Axis { axis, point } => {
                assert!(axis.is_normalized());
                let normal = axis.cross(forward).cross(axis).normalize_or_zero();
                Plane::from_normal_and_point(normal, point)
            }
        }
        .expect("Failed to map constraint plane")
    }

    pub fn intersect(&self, ray: Ray, view: Mat4) -> Option<Vec3> {
        let plane = self.to_plane(view);
        Some(ray.origin + ray.dir * plane.ray_intersect(ray)?)
    }
}

#[derive(Default, Debug, Clone)]
pub struct IntialState {
    transforms: Vec<Mat4>,
    midpoint: Vec3,
}

fn initial_transforms(hooks: &mut Hooks, game_client: &GameClient, targets: Arc<[EntityId]>) -> IntialState {
    hooks.use_memo_with(targets, |_, targets| {
        let state = game_client.game_state.lock();
        let transforms = match get_world_transforms(&state.world, targets) {
            Ok(v) => v,
            Err(err) => {
                log::error!("{err:?}");
                return Default::default();
            }
        };

        let midpoint: Vec3 =
            transforms.iter().map(|v| v.transform_point3(Vec3::ZERO)).fold(Vec3::ZERO, |acc, x| acc + x) / (targets.len().max(1)) as f32;

        IntialState { transforms, midpoint }
    })
}

#[element_component]
pub(super) fn PlaceController(
    hooks: &mut Hooks,
    targets: Arc<[EntityId]>,
    on_click: Cb<dyn Fn(ambient_window_types::MouseButton) + Sync + Send>,
) -> Element {
    assert_ne!(targets.len(), 0);
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let (prefs, _) = hooks.consume_context::<EditorPrefs>().unwrap();

    // Use a memo, that way the intent is reverted when the axis changes
    let action = hooks.use_memo_with(prefs, |world, _| {
        Arc::new(Mutex::new(EditorAction::new(
            world.resource(runtime()).clone(),
            game_client.clone(),
            intent_place_ray(),
            TRANSFORM_THROTTLE,
        )))
    });

    let action = Arc::downgrade(&action);
    Group(vec![HighjackMouse {
        on_click: {
            let action = action.clone();
            cb(move |button| {
                if button != ambient_window_types::MouseButton::Left {
                    return;
                }
                if let Some(action) = action.upgrade() {
                    action.lock().confirm();
                }
                on_click(button)
            })
        },
        on_mouse_move: cb(move |world, _, _| {
            let state = game_client.game_state.lock();
            let mouse_clip_pos = screen_to_clip_space(world, *world.resource(cursor_position()));

            let targets = targets.clone();

            let ray = state.screen_ray(mouse_clip_pos);

            let intent = IntentPlaceRay { targets: targets.to_vec(), ray, snap: prefs.snap };

            if let Some(action) = action.upgrade() {
                action.lock().push_intent(intent);
            }
        }),
        hide_mouse: false,
    }
    .el()])
    .el()
}

/// The TranslationController is created at the start of a translation action and lives for the duration of
/// it. The controller is removed once it's completed (a user "commits" with a click or cancels with escape
/// for instance).
#[derive(Debug, Clone)]
pub(super) struct TranslationController {
    pub targets: Arc<[EntityId]>,
    pub on_click: Cb<dyn Fn(MouseButton) + Sync + Send>,
}

impl ElementComponent for TranslationController {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { targets, on_click } = *self;

        let (axis, set_axis) = hooks.use_state(AxisFlags::all());

        assert_ne!(targets.len(), 0);
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        let (prefs, _) = hooks.consume_context::<EditorPrefs>().unwrap();

        // Freeze to_relative to the position when moving was started
        let initial_state = initial_transforms(hooks, &game_client, targets.clone());

        let game_state = game_client.game_state.lock();

        let to_target_local = to_isometry(initial_state.transforms.last().unwrap().inverse());
        let to_view_local = to_isometry(game_state.view().unwrap());

        // Use a memo, that way the intent is reverted when the axis changes
        let action = hooks.use_memo_with((axis, prefs), |world, _| {
            Arc::new(Mutex::new(EditorAction::new(
                world.resource(runtime()).clone(),
                game_client.clone(),
                intent_translate(),
                TRANSFORM_THROTTLE,
            )))
        });

        let action = Arc::downgrade(&action);

        let (initial_cursor_offset, _) = hooks.use_state_with(|world| {
            let mouse_clip_pos = screen_to_clip_space(world, *world.resource(cursor_position()));
            let clip_pos = game_state.proj_view().unwrap().project_point3(initial_state.midpoint).xy();
            mouse_clip_pos - clip_pos
        });

        let bits = axis.bits();
        let axis_vec = vec3(
            axis.contains(AxisFlags::X) as i32 as f32,
            axis.contains(AxisFlags::Y) as i32 as f32,
            axis.contains(AxisFlags::Z) as i32 as f32,
        );

        let (to_relative, constraints) = match bits.count_ones() {
            1 => {
                // Line
                let to_relative = if prefs.use_global_coordinates { Default::default() } else { to_target_local };
                let point = to_relative.transform_point3(initial_state.midpoint);
                let point = prefs.snap(point);

                (to_relative, ConstraintSpace::Axis { axis: axis_vec, point })
            }
            2 => {
                let to_relative = if prefs.use_global_coordinates { Default::default() } else { to_target_local };
                let point = to_relative.transform_point3(initial_state.midpoint);
                let point = prefs.snap(point);

                (to_relative, ConstraintSpace::Plane { normal: 1.0 - axis_vec, point })
            }
            // Do stuff in view space
            0 | 3 => {
                (to_view_local, ConstraintSpace::Plane { normal: Vec3::Z, point: to_view_local.transform_point3(initial_state.midpoint) })
            }
            _ => unreachable!(),
        };

        let from_relative = to_relative.inverse();

        let guide = {
            // Update the guide according to the constraint space
            match constraints {
                ConstraintSpace::Plane { normal, point } => {
                    // let point = prefs.snap(point);

                    // Convert it into world space
                    GridGuide {
                        rotation: Quat::from_mat4(&from_relative) * Quat::from_rotation_arc(Vec3::Z, normal),
                        // Transform into world space
                        point: from_relative.transform_point3(point),
                    }
                    .el()
                }
                ConstraintSpace::Axis { axis, point } => {
                    // let point = prefs.snap(point);

                    let point = from_relative.transform_point3(point);
                    let axis = from_relative.transform_vector3(axis).normalize();

                    AxisGuide { axis, point }.el()
                }
            }
        };

        drop(game_state);

        AxisButtons { axis, set_axis }.el().children(vec![Group(vec![
            guide,
            HighjackMouse {
                on_click: {
                    let action = action.clone();
                    cb(move |button| {
                        if button != ambient_window_types::MouseButton::Left {
                            return;
                        }

                        if let Some(action) = action.upgrade() {
                            action.lock().confirm();
                        }

                        on_click(button)
                    })
                },
                on_mouse_move: cb(move |world, _, _| {
                    let game_state = game_client.game_state.lock();
                    let mouse_clip_pos = screen_to_clip_space(world, *world.resource(cursor_position())) - initial_cursor_offset;

                    assert!(!axis.is_empty());

                    let targets = targets.clone();

                    let mut ray = game_state.screen_ray(mouse_clip_pos);

                    // Transform the picking ray to relative space
                    ray.dir = to_relative.transform_vector3(ray.dir);
                    ray.origin = to_relative.transform_point3(ray.origin);

                    let position = match constraints.intersect(ray, to_view_local) {
                        Some(v) => v,
                        None => {
                            tracing::warn!("No intersect");
                            return;
                        }
                    };

                    let position = prefs.snap(position);
                    let position = constraints.constrain(position);

                    // Convert back into world space
                    let position = from_relative.transform_point3(position);

                    let intent = IntentTranslate { targets: targets.to_vec(), position };
                    tracing::debug!("Translating: {intent:#?}");

                    if let Some(action) = action.upgrade() {
                        action.lock().push_intent(intent);
                    }
                }),
                hide_mouse: false,
            }
            .el(),
        ])
        .el()])
    }
}

#[derive(Debug, Clone)]
pub(super) struct ScaleController {
    pub targets: Arc<[EntityId]>,
    pub on_click: Cb<dyn Fn(MouseButton) + Sync + Send>,
}
impl ElementComponent for ScaleController {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { on_click, targets } = *self;
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        let runtime = hooks.world.resource(runtime()).clone();
        let (axis, set_axis) = hooks.use_state(AxisFlags::all());

        let action = hooks.use_memo_with(axis, |_, _| {
            Arc::new(Mutex::new(EditorAction::new(runtime, game_client.clone(), intent_set_transform(), TRANSFORM_THROTTLE)))
        });

        let action = Arc::downgrade(&action);

        // Freeze to_relative to the position when moving was started
        let state = initial_transforms(hooks, &game_client, targets.clone());

        let update = {
            let action = action.clone();
            Arc::new(move |pos: Vec2| {
                let delta = pos.x * 0.01;

                let mut new_scale = Vec3::ONE;
                if axis.contains(AxisFlags::X) {
                    new_scale.x = 1. + delta;
                }
                if axis.contains(AxisFlags::Y) {
                    new_scale.y = 1. + delta;
                }
                if axis.contains(AxisFlags::Z) {
                    new_scale.z = 1. + delta;
                }

                let to_local = Mat4::from_translation(-state.midpoint);
                let to_scaled_world = Mat4::from_translation(state.midpoint) * Mat4::from_scale(new_scale);

                let new_transforms = state.transforms.iter().map(|&transform| to_scaled_world * (to_local * transform)).collect_vec();

                if let Some(action) = action.upgrade() {
                    action.lock().push_intent(IntentTransform {
                        entities: targets.to_vec(),
                        transforms: new_transforms,
                        terrain_offset: TerrainOffset::Update,
                    });
                }
            })
        };

        AxisButtons { axis, set_axis }.el().children(vec![Group(vec![HighjackMouse {
            on_mouse_move: cb(move |_, pos, _| update(pos)),
            on_click: cb(move |button| {
                if button != MouseButton::Left {
                    return;
                }
                if let Some(action) = action.upgrade() {
                    action.lock().confirm();
                }
                on_click(button);
            }),
            hide_mouse: false,
        }
        .el()])
        .el()])
    }
}

#[derive(Debug, Clone)]
pub(super) struct RotateController {
    pub targets: Arc<[EntityId]>,
    pub on_click: Cb<dyn Fn(MouseButton) + Sync + Send>,
}

impl ElementComponent for RotateController {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { on_click, targets } = *self;
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        let runtime = hooks.world.resource(runtime()).clone();
        let (axis, set_axis) = hooks.use_state(AxisFlags::all());

        let (prefs, _) = hooks.consume_context::<EditorPrefs>().unwrap();

        let action = hooks.use_memo_with(axis, |_, _| {
            Arc::new(Mutex::new(EditorAction::new(runtime, game_client.clone(), intent_set_transform(), TRANSFORM_THROTTLE)))
        });

        let action = Arc::downgrade(&action);

        // Freeze to_relative to the position when moving was started
        let state = initial_transforms(hooks, &game_client, targets.clone());

        let to_relative = {
            if prefs.use_global_coordinates {
                Mat4::IDENTITY
            } else if let Some(transform) = state.transforms.last() {
                transform.inverse()
            } else {
                Mat4::IDENTITY
            }
        };

        let midpoint: Vec3 =
            state.transforms.iter().map(|v| v.transform_point3(Vec3::ZERO)).fold(Vec3::ZERO, |acc, x| acc + x) / targets.len() as f32;

        let axis = if axis.is_all() {
            AxisFlags::Z
        } else if axis.bits().count_ones() > 1 {
            axis.complement()
        } else {
            axis
        };

        let from_relative = to_relative.inverse();
        let up = from_relative.transform_vector3(Vec3::Z).normalize();
        let right = from_relative.transform_vector3(Vec3::X).normalize();
        let forward = from_relative.transform_vector3(Vec3::Y).normalize();

        let update = {
            let action = action.clone();
            Arc::new(move |pos: Vec2| {
                let axis = axis.as_vec3();

                let mov = pos.x * 0.01;

                let yaw = axis.z * mov;
                let pitch = axis.x * mov;
                let roll = axis.y * (1.0 - axis.x) * mov;

                let rot = Quat::from_axis_angle(up, yaw) * Quat::from_axis_angle(right, pitch) * Quat::from_axis_angle(forward, roll);

                let to_local = Mat4::from_translation(-midpoint);
                let to_rotated_world = Mat4::from_translation(midpoint) * Mat4::from_quat(rot);

                let new_transforms = state.transforms.iter().map(|&transform| to_rotated_world * to_local * transform).collect_vec();

                if let Some(action) = action.upgrade() {
                    action.lock().push_intent(IntentTransform {
                        entities: targets.to_vec(),
                        transforms: new_transforms,
                        terrain_offset: TerrainOffset::Update,
                    });
                }
            })
        };

        let mut items = Vec::new();
        if axis.contains(AxisFlags::X) {
            items.push(AxisGuide { axis: right, point: midpoint }.el())
        }
        if axis.contains(AxisFlags::Y) {
            items.push(AxisGuide { axis: forward, point: midpoint }.el())
        }
        if axis.contains(AxisFlags::Z) {
            items.push(AxisGuide { axis: up, point: midpoint }.el())
        }

        items.push(
            HighjackMouse {
                on_mouse_move: cb(move |_, pos, _| {
                    update(pos);
                }),
                on_click: cb(move |button| {
                    if button != MouseButton::Left {
                        return;
                    }
                    if let Some(action) = action.upgrade() {
                        action.lock().confirm();
                    }
                    on_click(button)
                }),
                hide_mouse: false,
            }
            .el(),
        );

        AxisButtons { axis, set_axis }.el().children(vec![Group(items).el()])
    }
}

#[element_component]
pub fn AxisButtons(_: &mut Hooks, axis: AxisFlags, set_axis: Cb<dyn Fn(AxisFlags) + Send + Sync>) -> Element {
    let toggle_axis = move |new: AxisFlags| {
        if axis == new {
            set_axis(AxisFlags::all())
        } else {
            set_axis(new)
        }
    };

    FlowRow(vec![
        Separator { vertical: true }.el(),
        Button::new(
            "X",
            closure!(clone toggle_axis, |_| {
                toggle_axis(AxisFlags::X);
            }),
        )
        .hotkey(VirtualKeyCode::X)
        .toggled(axis.contains(AxisFlags::X))
        .el(),
        Button::new(
            "Y",
            closure!(clone toggle_axis, |_| {
                toggle_axis(AxisFlags::Y);
            }),
        )
        .hotkey(VirtualKeyCode::Y)
        .toggled(axis.contains(AxisFlags::Y))
        .el(),
        Button::new(
            "Z",
            closure!(clone toggle_axis, |_| {
                toggle_axis(AxisFlags::Z);
            }),
        )
        .hotkey(VirtualKeyCode::Z)
        .toggled(axis.contains(AxisFlags::Z))
        .el(),
        Hotkey::new(
            VirtualKeyCode::Z,
            closure!(clone toggle_axis, |_| {
                toggle_axis(!AxisFlags::Z);
            }),
            Element::new(),
        )
        .hotkey_modifier(ModifiersState::SHIFT)
        .el(),
        Hotkey::new(
            VirtualKeyCode::X,
            closure!(clone toggle_axis, |_| {
                toggle_axis(!AxisFlags::X);
            }),
            Element::new(),
        )
        .hotkey_modifier(ModifiersState::SHIFT)
        .el(),
        Hotkey::new(
            VirtualKeyCode::Y,
            closure!(clone toggle_axis, |_| {
                toggle_axis(!AxisFlags::Y);
            }),
            Element::new(),
        )
        .hotkey_modifier(ModifiersState::SHIFT)
        .el(),
    ])
    .el()
    .set(space_between_items(), STREET)
}
