pub use ambient_ecs as ecs;

pub mod components {
    pub mod app {
        pub use ambient_core::{name, ui_scene, window_logical_size, window_physical_size, window_scale_factor};
    }
    pub mod ecs {
        pub use ambient_core::hierarchy::{children, parent};
    }
    pub mod transform {
        pub use ambient_core::transform::{local_to_parent, local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation};
    }
    pub mod ui {
        pub use ambient_layout::{
            gpu_ui_size,
            guest_api::{
                align_horizontal_begin, align_horizontal_center, align_horizontal_end, align_vertical_begin, align_vertical_center,
                align_vertical_end, docking_bottom, docking_fill, docking_left, docking_right, docking_top, fit_horizontal_children,
                fit_horizontal_none, fit_horizontal_parent, fit_vertical_children, fit_vertical_none, fit_vertical_parent, layout_bookcase,
                layout_dock, layout_flow, layout_width_to_children, margin_bottom, margin_left, margin_right, margin_top,
                orientation_horizontal, orientation_vertical, padding_bottom, padding_left, padding_right, padding_top,
            },
            height, is_book_file, mesh_to_local_from_size, min_height, min_width, space_between_items, width,
        };
        pub use ambient_rect::{background_color, border_color, border_radius, border_thickness, rect};
        pub use ambient_text::{font_size, text};
    }
    pub mod rendering {
        pub use ambient_renderer::color;
    }
    pub mod input {
        pub use ambient_input::{
            event_mouse_input, event_mouse_motion, event_mouse_wheel, event_mouse_wheel_pixels, mouse_button,
            picking::{mouse_over, mouse_pickable_max, mouse_pickable_min},
        };
    }
}
