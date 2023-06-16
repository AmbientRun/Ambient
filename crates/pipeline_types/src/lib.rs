pub mod audio;
pub mod materials;
pub mod models;
pub mod pipeline;

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_true(value: &bool) -> bool {
    *value
}

fn true_value() -> bool {
    true
}

fn is_default<T: PartialEq + Default>(value: &T) -> bool {
    *value == Default::default()
}

pub use audio::AudioPipeline;
pub use materials::MaterialsPipeline;
pub use models::{Collider, ModelImporter, ModelsPipeline};
pub use pipeline::{Pipeline, PipelineProcessor};
