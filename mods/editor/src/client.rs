use ambient_api::{
    core::{
        app::components::{main_scene, name},
        messages::Frame,
        physics::components::mass,
        rect::components::{line_from, line_to, line_width, rect},
        rendering::components::{color, double_sided},
        text::{components::font_style, types::FontStyle},
        transform::components::{rotation, scale, translation},
    },
    ecs::SupportedValue,
    input::CursorLockGuard,
    prelude::*,
};
use editor::{
    components::{editor_camera, mouseover_entity, mouseover_position, selected_entity},
    messages::{Input, ToggleEditor},
};
use editor_schema::components::in_editor;

#[main]
pub fn main() {
    let mut input_lock = None;
    let mut fixed_tick_last = game_time();

    let mut accumulated_aim_delta = Vec2::ZERO;

    let mut select_pressed = false;
    let mut freeze_pressed = false;

    let mut gizmo_active = None;
    let mut gizmo_accumulated_drag = 0.0;
    let mut gizmo_original_translation = None;

    let player_id = player::get_local();

    Frame::subscribe(move |_| {
        let fixed_tick_dt = game_time() - fixed_tick_last;

        if !entity::get_component(player_id, in_editor()).unwrap_or_default() {
            return;
        }

        let Some(camera_id) = entity::get_component(player_id, editor_camera()) else { return; };

        let (delta, input) = input::get_delta();

        if gizmo_active.is_none() && delta.mouse_buttons.contains(&MouseButton::Right) {
            input_lock = Some(CursorLockGuard::new());
        } else if delta.mouse_buttons_released.contains(&MouseButton::Right) {
            input_lock = None;
        }
        select_pressed |= delta.mouse_buttons.contains(&MouseButton::Left);
        freeze_pressed |= delta.keys_released.contains(&KeyCode::R);

        let movement = [
            (KeyCode::W, -Vec2::Y),
            (KeyCode::S, Vec2::Y),
            (KeyCode::A, -Vec2::X),
            (KeyCode::D, Vec2::X),
        ]
        .iter()
        .filter(|(key, _)| input.keys.contains(key))
        .fold(Vec2::ZERO, |acc, (_, dir)| acc + *dir);

        let mut aiming = false;
        if let Some(input_lock) = &mut input_lock {
            if input_lock.auto_unlock_on_escape(&input) {
                let speed = 4.0 * delta_time();
                accumulated_aim_delta += delta.mouse_position * speed;
                aiming = true;
            }
        }

        if !aiming {
            if let Some(selected_entity) = entity::get_component(player_id, selected_entity()) {
                let gizmos = Gizmo::for_entity(selected_entity);
                let gizmo = gizmos
                    .into_iter()
                    .find(|g| g.is_hovered(camera_id, input.mouse_position));

                if input.mouse_buttons.contains(&MouseButton::Left) {
                    if gizmo_active.is_none() {
                        if let Some(gizmo) = gizmo {
                            gizmo_active = Some(gizmo);
                            gizmo_original_translation =
                                entity::get_component(selected_entity, translation());
                            gizmo_accumulated_drag = 0.0;
                        }
                    }
                } else {
                    gizmo_active = None;
                    gizmo_original_translation = None;
                    gizmo_accumulated_drag = 0.0;
                }

                if let Some(gizmo) = &gizmo_active {
                    let gizmo_start_2d = camera::world_to_screen(camera_id, gizmo.origin);
                    let gizmo_end_2d =
                        camera::world_to_screen(camera_id, gizmo.origin + gizmo.direction);
                    let gizmo_dir_2d = (gizmo_end_2d - gizmo_start_2d).normalize();

                    let gizmo_mouse_alignment =
                        gizmo_dir_2d.dot(delta.mouse_position.normalize_or_zero());

                    let speed = 10.0 * delta_time();
                    gizmo_accumulated_drag += gizmo_mouse_alignment * speed;

                    select_pressed = false;
                }
            }
        }

        if fixed_tick_dt > Duration::from_millis(20) {
            let ray = camera::screen_position_to_world_ray(camera_id, input.mouse_position);

            let boost = input.keys.contains(&KeyCode::LShift);

            Input {
                aim_delta: accumulated_aim_delta,
                movement,
                boost,
                ray_origin: ray.origin,
                ray_direction: ray.dir,
                select: select_pressed,
                freeze: freeze_pressed,
                translate_to: gizmo_original_translation
                    .zip(gizmo_active.as_ref())
                    .map(|(t, g)| t + (g.direction * gizmo_accumulated_drag)),
            }
            .send_server_reliable();

            accumulated_aim_delta = Vec2::ZERO;
            select_pressed = false;

            fixed_tick_last = game_time();
        }
    });

    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    let in_editor = hooks
        .use_entity_component(player::get_local(), in_editor())
        .0
        .unwrap_or_default();

    hooks.use_keyboard_input(move |_, keycode, modifiers, pressed| {
        if modifiers == ModifiersState::empty() && keycode == Some(VirtualKeyCode::F5) && !pressed {
            ToggleEditor {}.send_server_reliable();
        }
    });

    FocusRoot::el([if in_editor {
        Group::el([MenuBar::el(), MouseoverDisplay::el(), SelectedDisplay::el()])
    } else {
        Element::new()
    }])
}

#[element_component]
fn MenuBar(_hooks: &mut Hooks) -> Element {
    WindowSized::el([with_rect(
        FlowRow::el([Text::el(format!("Editor {}", env!("CARGO_PKG_VERSION")))])
            .with_padding_even(4.0),
    )
    .with(fit_horizontal(), Fit::Parent)
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))])
}

#[element_component]
fn MouseoverDisplay(hooks: &mut Hooks) -> Element {
    let player_id = player::get_local();
    let (mouseover_position, _) = hooks.use_entity_component(player_id, mouseover_position());
    let (mouseover_entity, _) = hooks.use_entity_component(player_id, mouseover_entity());
    let (camera_id, _) = hooks.use_entity_component(player_id, editor_camera());

    let Some(mouseover_position) = mouseover_position else { return Element::new(); };
    let Some(camera_id) = camera_id else { return Element::new(); };

    let mut text = format!("{:.02?}", mouseover_position.to_array());
    if let Some(mouseover_entity) = mouseover_entity {
        text += &format!("\n{}", entity_name(mouseover_entity));
    }

    let mouseover_position_2d = camera::world_to_screen(camera_id, mouseover_position).extend(0.0);
    Text::el(text)
        .with(translation(), mouseover_position_2d)
        .with(color(), Vec4::ONE)
}

#[element_component]
fn SelectedDisplay(hooks: &mut Hooks) -> Element {
    let player_id = player::get_local();
    let (selected_entity, _) = hooks.use_entity_component(player_id, selected_entity());
    let (camera_id, _) = hooks.use_entity_component(player_id, editor_camera());

    // TODO: is there a better way to force this element to re-render every frame?
    let rerender = hooks.use_rerender_signal();
    hooks.use_frame(move |_| {
        rerender();
    });

    let Some(selected_entity) = selected_entity else { return Element::new(); };
    let Some(camera_id) = camera_id else { return Element::new(); };

    let position = entity::get_component(selected_entity, translation()).unwrap_or_default();
    Group::el([
        Text::el(format!(
            "{:.02?}\n{}",
            position.to_array(),
            entity_name(selected_entity)
        ))
        .with(
            translation(),
            camera::world_to_screen(camera_id, position).extend(0.0),
        )
        .with(color(), Vec4::ONE),
        EntityView::el(selected_entity),
        GizmoDisplay::el(camera_id, selected_entity),
    ])
}

const GIZMO_LENGTH: f32 = 5.;
const GIZMO_WIDTH: f32 = 0.25;

#[element_component]
fn EntityView(_hooks: &mut Hooks, entity: EntityId) -> Element {
    struct Displays(EntityId, Vec<Element>);
    impl Displays {
        fn add<T: Editor + SupportedValue>(&mut self, name: &str, component: Component<T>) {
            let Some(value) = entity::get_component(self.0, component) else {return;};

            self.1.push(
                FlowColumn::el([
                    Text::el(name).with(font_style(), FontStyle::Bold),
                    value.editor(cb(|_| {}), EditorOpts::default()),
                ])
                .with(space_between_items(), 4.0),
            )
        }
    }

    let mut displays = Displays(entity, vec![]);
    displays.add("Name", name());
    displays.add("Translation", translation());
    displays.add("Scale", scale());
    displays.add("Mass", mass());

    WindowSized::el([with_rect(
        FlowColumn::el(
            std::iter::once(Text::el(entity_name(entity)).section_style())
                .chain(displays.1.into_iter()),
        )
        .with_padding_even(4.0)
        .with(space_between_items(), 4.0),
    )
    .with(docking(), Docking::Right)
    .with(margin(), vec4(STREET * 4., STREET, STREET, STREET))
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))])
}

#[element_component]
fn GizmoDisplay(_hooks: &mut Hooks, camera_id: EntityId, entity: EntityId) -> Element {
    Group::el(
        Gizmo::for_entity(entity)
            .into_iter()
            .map(|l| l.as_element(camera_id)),
    )
}

struct Gizmo {
    origin: Vec3,
    direction: Vec3,
    color: Vec3,
}
impl Gizmo {
    fn new(origin: Vec3, direction: Vec3, color: Vec3) -> Self {
        Self {
            origin,
            direction,
            color,
        }
    }

    fn for_entity(id: EntityId) -> [Self; 3] {
        let origin = entity::get_component(id, translation()).unwrap_or_default();
        let rotation = entity::get_component(id, rotation()).unwrap_or_default();

        [
            Self::new(origin, rotation * Vec3::X, vec3(1., 0., 0.)),
            Self::new(origin, rotation * Vec3::Y, vec3(0., 1., 0.)),
            Self::new(origin, rotation * Vec3::Z, vec3(0., 0., 1.)),
        ]
    }

    fn as_element(&self, camera_id: EntityId) -> Element {
        let our_color = if self.is_hovered(camera_id, input::get().mouse_position) {
            self.color
        } else {
            self.color * 0.6
        }
        .extend(1.0);

        let head_length_pct = 0.1;
        let head_segments = 4;
        let head_segment_length = GIZMO_LENGTH * (head_length_pct / head_segments as f32);
        let line_end = self.origin + self.direction * (GIZMO_LENGTH * (1. - head_length_pct));

        Group::el(
            std::iter::once(
                Element::new()
                    .init_default(rect())
                    .with_default(main_scene())
                    .with(line_from(), self.origin)
                    .with(line_to(), line_end)
                    .with(line_width(), GIZMO_WIDTH)
                    .with(double_sided(), true)
                    .with(color(), our_color),
            )
            .chain((0..head_segments).map(|i| {
                let width = GIZMO_WIDTH * (2. - 1.5 * (i as f32 / head_segments as f32));

                Element::new()
                    .init_default(rect())
                    .with_default(main_scene())
                    .with(
                        line_from(),
                        line_end + (i as f32 * head_segment_length) * self.direction,
                    )
                    .with(
                        line_to(),
                        line_end + ((i + 1) as f32 * head_segment_length) * self.direction,
                    )
                    .with(line_width(), width)
                    .with(double_sided(), true)
                    .with(color(), our_color)
            })),
        )
    }

    fn is_hovered(&self, camera_id: EntityId, mouse_position: Vec2) -> bool {
        is_mouse_in_cylinder(
            self.origin,
            self.direction,
            // such a bodge
            GIZMO_WIDTH * 10.,
            GIZMO_LENGTH * 1.5,
            camera_id,
            mouse_position,
        )
    }
}

fn is_mouse_in_cylinder(
    position: Vec3,
    direction: Vec3,
    radius: f32,
    length: f32,
    camera_id: EntityId,
    mouse_position: Vec2,
) -> bool {
    let screen_pos = camera::world_to_screen(camera_id, position);
    let screen_dir = camera::world_to_screen(camera_id, position + direction * length) - screen_pos;
    let screen_mouse_pos = mouse_position - screen_pos;

    let screen_mouse_pos_on_cylinder_axis =
        screen_mouse_pos.dot(screen_dir) / screen_dir.dot(screen_dir) * screen_dir;
    let screen_mouse_pos_on_cylinder = screen_mouse_pos - screen_mouse_pos_on_cylinder_axis;
    let distance_to_cylinder = screen_mouse_pos_on_cylinder.length();

    distance_to_cylinder <= radius
}

// todo: move to API
pub fn fixed_rate_tick(dt: Duration, mut callback: impl FnMut(Duration) + 'static) {
    let mut last_tick = game_time();
    Frame::subscribe(move |_| {
        let delta = game_time() - last_tick;
        if delta < dt {
            return;
        }

        callback(delta);

        last_tick = game_time();
    });
}

fn entity_name(id: EntityId) -> String {
    entity::get_component(id, name()).unwrap_or_else(|| id.to_string())
}
