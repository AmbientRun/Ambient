pub use ambient_ecs as ecs;
use std::{future::Future, time::Duration};

pub mod components {
    pub mod app {
        pub use ambient_core::{
            name, ui_scene,
            window::{window_logical_size, window_physical_size, window_scale_factor},
        };
    }
    pub mod ecs {
        pub use ambient_core::hierarchy::{children, parent};
    }
    pub mod transform {
        pub use ambient_core::transform::{local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation};
    }
    pub mod rect {
        pub use ambient_rect::{background_color, border_color, border_radius, border_thickness, rect};
    }
    pub mod layout {
        pub use ambient_layout::{
            gpu_ui_size,
            guest_api::{
                align_horizontal_begin, align_horizontal_center, align_horizontal_end, align_vertical_begin, align_vertical_center,
                align_vertical_end, docking_bottom, docking_fill, docking_left, docking_right, docking_top, fit_horizontal_children,
                fit_horizontal_none, fit_horizontal_parent, fit_vertical_children, fit_vertical_none, fit_vertical_parent, layout_bookcase,
                layout_dock, layout_flow, layout_width_to_children, margin_bottom, margin_left, margin_right, margin_top,
                orientation_horizontal, orientation_vertical, padding_bottom, padding_left, padding_right, padding_top,
            },
            height, is_book_file, mesh_to_local_from_size, min_height, min_width, screen, space_between_items, width,
        };
    }
    pub mod text {
        pub use ambient_text::{font_family, font_size, font_style, text};
    }
    pub mod rendering {
        pub use ambient_renderer::color;
    }
    pub mod input {
        pub use ambient_input::{
            event_focus_change, event_keyboard_input, event_mouse_input, event_mouse_motion, event_mouse_wheel, event_mouse_wheel_pixels,
            event_received_character, keyboard_modifiers, keycode, mouse_button,
            picking::{mouse_over, mouse_pickable_max, mouse_pickable_min},
        };
    }
    pub mod player {
        pub use ambient_core::player::{local_user_id, player, user_id};
    }
}

pub fn run_async(world: &ecs::World, future: impl Future<Output = ()> + Send + 'static) {
    world.resource(ambient_core::runtime()).spawn(future);
}
pub async fn sleep(seconds: f32) {
    ambient_sys::time::sleep(Duration::from_secs_f32(seconds)).await;
}

pub mod window {
    use ambient_core::window::{window_ctl, WindowCtl};
    use ambient_ecs::World;
    use ambient_window_types::CursorIcon;

    pub fn set_cursor(world: &World, cursor: CursorIcon) {
        world.resource(window_ctl()).send(WindowCtl::SetCursorIcon(cursor.into())).ok();
    }
    pub fn get_clipboard() -> Option<String> {
        #[cfg(not(target_os = "unknown"))]
        {
            return arboard::Clipboard::new().ok().and_then(|mut x| x.get_text().ok());
        }
        #[cfg(target_os = "unknown")]
        {
            return None;
        }
    }
}
