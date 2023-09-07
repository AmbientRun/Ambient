pub mod audio;
pub mod materials;
pub mod models;
pub use audio::AudioPipeline;
pub use materials::MaterialsPipeline;
pub use models::{Collider, ModelImporter, ModelsPipeline};
use serde::{Deserialize, Serialize};
use std::path::Path;

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

/// The outermost structure of the pipeline.toml file.
///
/// Is a struct of arrays of pipelines as toml does not support top-level arrays
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelinesFile {
    pub pipelines: Vec<Pipeline>,
}
impl PipelinesFile {
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap()
    }
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        std::fs::write(path, self.to_toml())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
/// Desrcibes how the pipeline assets should be processed.
pub enum PipelineProcessor {
    /// The models asset pipeline.
    /// Will import models (including constituent materials and animations) and generate prefabs for them by default.
    Models(ModelsPipeline),
    /// The materials asset pipeline.
    /// Will import specific materials without needing to be part of a model.
    Materials(MaterialsPipeline),
    /// The audio asset pipeline.
    /// Will import supported audio file formats and produce Ogg Vorbis or WAV files to be used by the runtime.
    Audio(AudioPipeline),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Describes a single complete pipeline such as which processor to use, input file filtering, and output tagging.
pub struct Pipeline {
    /// The type of pipeline to use.
    ///
    /// This enum is flattened, simply use:
    /// ```toml
    /// type = "Models"
    /// ```
    ///
    /// or
    ///
    /// ```toml
    /// type = "Materials"
    /// importer = "Unity"
    /// ```
    #[serde(flatten)]
    pub processor: PipelineProcessor,
    /// Filter the sources used to feed this pipeline.
    /// This is a list of glob patterns for accepted files.
    /// All files are accepted if this is empty.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    /// Tags to apply to the output resources.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Categories to apply to the output resources.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Vec<String>>,
}
