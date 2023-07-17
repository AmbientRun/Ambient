use std::{cell::RefCell, rc::Rc};

use ambient_api::{components::core::app::window_logical_size, input::CursorLockGuard, prelude::*};

#[main]
pub async fn main() {
    let mut last_shot = game_time();
    let mut is_shooting = false;

    // TODO: fixed?
    let mut input_lock = InputLock::new();
    ambient_api::messages::Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();
        if !input_lock.update(&input) {
            return;
        }
        let mouse_delta = input.mouse_delta;
        let mut direction = Vec2::ZERO;
        let mut shoot = false;
        let mut walk = false;
        let mut jump = false;
        let mut duck = false;

        if input.keys.contains(&KeyCode::W) {
            direction.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            direction.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            direction.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            direction.x += 1.0;
        }

        if delta.keys.contains(&KeyCode::Space) {
            jump = true;
        }

        if input.keys.contains(&KeyCode::LControl) {
            duck = true;
        }

        if input.keys.contains(&KeyCode::LShift) {
            walk = true;
        }

        if input.mouse_buttons.contains(&MouseButton::Left) {
            if is_shooting {
                if game_time() - last_shot > Duration::from_millis(200) {
                    shoot = true;
                    last_shot = game_time();
                }
            } else {
                shoot = true;
                is_shooting = true;
                last_shot = game_time();
            }
        } else {
            is_shooting = false;
        }

        let toggle_zoom = delta.mouse_buttons.contains(&MouseButton::Right);

        let player_id = player::get_local();
        let cam = entity::get_component(player_id, components::player_cam_ref());
        if cam.is_none() {
            return;
        }

        let cam = cam.unwrap();
        let window_size =
            entity::get_component(entity::resources(), window_logical_size()).unwrap();
        let ray = camera::screen_position_to_world_ray(
            cam,
            vec2(window_size.x as f32 / 2., window_size.y as f32 / 2.),
        );

        messages::Input {
            direction,
            mouse_delta,
            shoot,
            toggle_zoom,
            is_shooting,
            walk,
            jump,
            duck,
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    });
}

struct InputLock {
    refcount: Rc<RefCell<u8>>,
    subscribers: Vec<message::Listener>,
    cursor_lock: Option<CursorLockGuard>,
}
impl InputLock {
    fn new() -> Self {
        // We keep the refcount local to this struct and not in the ECS
        // so that other embers can't mess with it.
        let refcount = Rc::new(RefCell::new(0));
        let subscribers = vec![
            messages::RequestInput::subscribe({
                let refcount = refcount.clone();
                move |_, _| {
                    *refcount.borrow_mut() += 1;
                }
            }),
            messages::ReleaseInput::subscribe({
                let refcount = refcount.clone();
                move |_, _| {
                    let mut refcount = refcount.borrow_mut();
                    *refcount = u8::saturating_sub(*refcount, 1);
                }
            }),
        ];

        Self {
            refcount,
            subscribers,
            cursor_lock: None,
        }
    }

    fn update(&mut self, input: &input::Input) -> bool {
        let refcount = *self.refcount.borrow();

        if refcount == 0 {
            if self.cursor_lock.is_none() {
                self.cursor_lock = Some(CursorLockGuard::new());
            }
        } else {
            self.cursor_lock = None;
        }

        match &mut self.cursor_lock {
            Some(lock) => lock.auto_unlock_on_escape(&input),
            _ => false,
        }
    }
}
impl Drop for InputLock {
    fn drop(&mut self) {
        for subscriber in self.subscribers.drain(..) {
            subscriber.stop();
        }
    }
}
