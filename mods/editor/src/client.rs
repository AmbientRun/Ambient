use ambient_api::{
    core::{
        app::components::{main_scene, name},
        messages::Frame,
        rect::components::{line_from, line_to, line_width, rect},
        rendering::components::{color, double_sided},
        transform::components::{rotation, translation},
    },
    input::CursorLockGuard,
    prelude::*,
};
use editor::{
    components::{editor_camera, in_editor, mouseover_entity, mouseover_position, selected_entity},
    messages::{Input, ToggleEditor},
};

#[main]
pub fn main() {
    let mut input_lock = None;
    let mut fixed_tick_last = game_time();

    let mut accumulated_aim_delta = Vec2::ZERO;
    let mut select_pressed = false;

    Frame::subscribe(move |_| {
        let fixed_tick_dt = game_time() - fixed_tick_last;

        if !entity::get_component(player::get_local(), in_editor()).unwrap_or_default() {
            return;
        }

        let Some(camera_id) = entity::get_component(player::get_local(), editor_camera()) else { return; };

        let (delta, input) = input::get_delta();

        if delta.mouse_buttons.contains(&MouseButton::Right) {
            input_lock = Some(CursorLockGuard::new());
        } else if delta.mouse_buttons_released.contains(&MouseButton::Right) {
            input_lock = None;
        }
        select_pressed |= delta.mouse_buttons.contains(&MouseButton::Left);

        let movement = [
            (KeyCode::W, -Vec2::Y),
            (KeyCode::S, Vec2::Y),
            (KeyCode::A, -Vec2::X),
            (KeyCode::D, Vec2::X),
        ]
        .iter()
        .filter(|(key, _)| input.keys.contains(key))
        .fold(Vec2::ZERO, |acc, (_, dir)| acc + *dir);

        if let Some(input_lock) = &mut input_lock {
            if input_lock.auto_unlock_on_escape(&input) {
                let speed = 4.0 * delta_time();
                accumulated_aim_delta += delta.mouse_position * speed;
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

    // hack: I'm lazy
    hooks.use_spawn(move |_| {
        if !in_editor {
            ToggleEditor {}.send_server_reliable();
        }

        |_| {}
    });

    if in_editor {
        Group::el([MenuBar::el(), MouseoverDisplay::el(), SelectedDisplay::el()])
    } else {
        Element::new()
    }
}

#[element_component]
fn MenuBar(hooks: &mut Hooks) -> Element {
    let (selected_entity, _) = hooks.use_entity_component(player::get_local(), selected_entity());

    WindowSized::el([with_rect(
        FlowRow::el([Text::el(format!(
            "Editor {} | Selected Entity: {}",
            env!("CARGO_PKG_VERSION"),
            selected_entity
                .map(display_entity)
                .unwrap_or_else(|| "none".to_string())
        ))])
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
        text += &format!("\n{}", display_entity(mouseover_entity));
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
            display_entity(selected_entity)
        ))
        .with(
            translation(),
            camera::world_to_screen(camera_id, position).extend(0.0),
        )
        .with(color(), Vec4::ONE),
        Gizmo::el(camera_id, selected_entity),
    ])
}

const GIZMO_LENGTH: f32 = 5.;
const GIZMO_WIDTH: f32 = 0.25;

#[element_component]
fn Gizmo(_hooks: &mut Hooks, camera_id: EntityId, entity: EntityId) -> Element {
    Group::el(
        GizmoLine::for_entity(entity)
            .into_iter()
            .map(|l| l.as_element(camera_id)),
    )
}

struct GizmoLine {
    origin: Vec3,
    direction: Vec3,
    color: Vec3,
}
impl GizmoLine {
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
        let our_color = if self.is_moused_over(camera_id, input::get().mouse_position) {
            self.color
        } else {
            self.color * 0.6
        }
        .extend(1.0);

        let head_length = 0.1;
        let line_end = self.origin + self.direction * (GIZMO_LENGTH * (1. - head_length));
        let head_end = line_end + self.direction * (GIZMO_LENGTH * head_length);

        Group::el([
            Element::new()
                .init_default(rect())
                .with_default(main_scene())
                .with(line_from(), self.origin)
                .with(line_to(), line_end)
                .with(line_width(), GIZMO_WIDTH)
                .with(double_sided(), true)
                .with(color(), our_color),
            Element::new()
                .init_default(rect())
                .with_default(main_scene())
                .with(line_from(), line_end)
                .with(line_to(), head_end)
                .with(line_width(), GIZMO_WIDTH * 3.)
                .with(double_sided(), true)
                .with(color(), our_color),
        ])
    }

    fn is_moused_over(&self, camera_id: EntityId, mouse_position: Vec2) -> bool {
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

fn display_entity(id: EntityId) -> String {
    entity::get_component(id, name()).unwrap_or_else(|| id.to_string())
}
