use serde::{Deserialize, Serialize};

use crate::{audio::AudioPipeline, materials::MaterialsPipeline, models::ModelsPipeline};

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
