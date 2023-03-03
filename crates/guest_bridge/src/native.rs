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
        pub use ambient_layout::{height, min_height, min_width, width};
        pub use ambient_text::{font_size, text};
    }
    pub mod rendering {
        pub use ambient_renderer::color;
    }
}
