use std::path::{Path, PathBuf};

use ambient_pipeline_types::materials::{MaterialsImporter, MaterialsPipeline};
use ambient_project::Manifest;
use anyhow::Context;
use futures::{future::ready, stream, StreamExt, TryStreamExt};
use itertools::Itertools;

use crate::{
    get_asset_files,
    migrate::toml::json_pipeline::PipelineOneOrMany,
    pipelines::{
        audio::AudioPipeline,
        models::{Collider, ModelsPipeline},
        Pipeline, PipelineProcessor, PipelineSchema,
    },
    register_from_manifest,
};

pub async fn process(manifest: &Manifest, path: PathBuf) -> anyhow::Result<()> {
    register_from_manifest(manifest);
    let assets_path = path.join("assets");

    stream::iter(get_asset_files(&assets_path))
        .filter(|path| ready(path.ends_with("pipeline.json")))
        .then(|path: PathBuf| async move {
            migrate_pipeline(&path)
                .await
                .with_context(|| format!("Error migrating pipeline {path:?}"))?;

            Ok(())
        })
        .try_collect::<()>()
        .await
}

async fn migrate_pipeline(path: &Path) -> anyhow::Result<()> {
    tracing::info!(?path, "Processing pipeline");

    let str = tokio::fs::read_to_string(path)
        .await
        .context("Error reading json pipeline file")?;

    tracing::info!("Read string: {str}");

    let de = &mut serde_json::de::Deserializer::from_str(&str);

    let value: PipelineOneOrMany = serde_path_to_error::deserialize(de)
        .with_context(|| format!("Error deserializing json pipeline file {:?}", path))?;

    tracing::info!("Deserialized json pipeline file: {value:#?}");

    let value = PipelineSchema {
        pipelines: {
            match value {
                PipelineOneOrMany::Many(v) => v.into_iter().map_into().collect(),
                PipelineOneOrMany::One(p) => vec![p.into()],
            }
        },
    };

    let mut toml = String::new();
    let serializer = toml::ser::Serializer::new(&mut toml);
    serde_path_to_error::serialize(&value, serializer).context("Error serializing json to toml")?;

    tracing::info!("Serialized to toml: {toml}");

    let toml_path = path.with_extension("toml");
    tokio::fs::write(&toml_path, toml)
        .await
        .context("Error writing toml pipeline file")?;

    eprintln!("Wrote toml pipeline file: {:?}", toml_path);

    Ok(())
}

mod json_pipeline {
    use ambient_ecs::Entity;
    use ambient_model_import::{ModelTextureSize, ModelTransform};
    use ambient_physics::collider::ColliderType;
    use ambient_pipeline_types::materials::PipelinePbrMaterial;
    use serde::{Deserialize, Serialize};

    use crate::pipelines::models::{MaterialOverride, ModelImporter};

    fn true_value() -> bool {
        true
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    #[serde(tag = "type")]
    pub enum Collider {
        #[default]
        /// No physics collider. The default.
        None,
        /// Extract the physics collider from the model.
        FromModel {
            /// Whether or not the normals should be flipped.
            #[serde(default)]
            flip_normals: bool,
            /// Whether or not the indices should be reversed for each triangle. On by default.
            #[serde(default = "true_value")]
            reverse_indices: bool,
        },
        /// Use a cylindrical character collider.
        Character {
            /// The radius of the collider.
            radius: Option<f32>,
            /// The height of the collider.
            height: Option<f32>,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum PipelineConfig {
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
    pub struct Pipeline {
        /// The type of pipeline to use.
        pub pipeline: PipelineConfig,
        /// Filter the sources used to feed this pipeline.
        /// This is a list of glob patterns for accepted files.
        /// All files are accepted if this is empty.
        #[serde(default)]
        pub sources: Vec<String>,
        /// Tags to apply to the output resources.
        #[serde(default)]
        pub tags: Vec<String>,
        /// Categories to apply to the output resources.
        #[serde(default)]
        pub categories: Vec<Vec<String>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ModelsPipeline {
        /// The importer to use to process models.
        #[serde(default)]
        pub importer: ModelImporter,
        /// Use assimp as the importer.
        /// This will support more file formats, but is less well-integrated. Off by default.
        #[serde(default)]
        pub force_assimp: bool,
        #[serde(default)]
        /// The physics collider to use for this mesh.
        pub collider: Collider,
        /// If a collider is present, this controls how it will interact with other colliders.
        #[serde(default)]
        pub collider_type: ColliderType,
        /// Whether or not this mesh should have its texture sizes capped.
        pub cap_texture_sizes: Option<ModelTextureSize>,
        /// Treats all assets in the pipeline as variations, and outputs a single asset which is a collection of all assets.
        /// Most useful for grass and other entities whose individual identity is not important.
        #[serde(default)]
        pub collection_of_variants: bool,
        /// Output prefabs that can be spawned. On by default.
        #[serde(default = "true_value")]
        pub output_prefabs: bool,
        /// Output the animations that belonged to this model.
        #[serde(default = "true_value")]
        pub output_animations: bool,
        /// If specified, these components will be added to the prefabs produced by `output_prefabs`.
        ///
        /// This is a great way to specify additional information about your prefab that can be used by gameplay logic.
        /// Note that these components should have static data (i.e. statistics), not dynamic state, as any such state could be
        /// replaced by this prefab being reloaded.
        #[serde(default)]
        pub prefab_components: Entity,
        /// If specified, a list of overrides to use for the materials for the mesh.
        #[serde(default)]
        pub material_overrides: Vec<MaterialOverride>,
        /// If specified, a list of transformations to apply to this model. This can be used
        /// to correct coordinate space differences between your asset source and the runtime.
        ///
        /// These will be applied in sequence.
        #[serde(default)]
        pub transforms: Vec<ModelTransform>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct AudioPipeline {
        /// Whether or not the audio should be converted to Ogg Vorbis.
        #[serde(default)]
        pub convert: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type")]
    #[allow(clippy::large_enum_variant)]
    #[serde(deny_unknown_fields)]
    pub enum MaterialsImporter {
        /// Import a single material, as specified.
        /// All of its dependent assets (URLs, etc) will be resolved during the build process.
        Single(PipelinePbrMaterial),
        /// Import Quixel materials.
        Quixel,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MaterialsPipeline {
        /// The importer to use for materials.
        pub importer: Box<MaterialsImporter>,
        /// Whether or not decal prefabs should be created for each of these materials.
        #[serde(default)]
        pub output_decals: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
    pub(super) enum PipelineOneOrMany {
        Many(Vec<Pipeline>),
        One(Pipeline),
    }
}

impl From<json_pipeline::MaterialsImporter> for MaterialsImporter {
    fn from(value: json_pipeline::MaterialsImporter) -> Self {
        match value {
            json_pipeline::MaterialsImporter::Single(v) => Self::Single(v),
            json_pipeline::MaterialsImporter::Quixel => Self::Quixel,
        }
    }
}

impl From<json_pipeline::Collider> for Collider {
    fn from(value: json_pipeline::Collider) -> Self {
        match value {
            json_pipeline::Collider::None => Self::None,
            json_pipeline::Collider::FromModel {
                flip_normals,
                reverse_indices,
            } => Self::FromModel {
                flip_normals,
                reverse_indices,
            },
            json_pipeline::Collider::Character { radius, height } => {
                Self::Character { radius, height }
            }
        }
    }
}

impl From<json_pipeline::MaterialsPipeline> for MaterialsPipeline {
    fn from(value: json_pipeline::MaterialsPipeline) -> Self {
        Self {
            importer: Box::new(MaterialsImporter::from(*value.importer)),
            output_decals: value.output_decals,
        }
    }
}

impl From<json_pipeline::AudioPipeline> for AudioPipeline {
    fn from(value: json_pipeline::AudioPipeline) -> Self {
        Self {
            convert: value.convert,
        }
    }
}

impl From<json_pipeline::ModelsPipeline> for ModelsPipeline {
    fn from(value: json_pipeline::ModelsPipeline) -> Self {
        Self {
            importer: value.importer,
            force_assimp: value.force_assimp,
            collider: value.collider.into(),
            collider_type: value.collider_type,
            cap_texture_sizes: value.cap_texture_sizes,
            collection_of_variants: value.collection_of_variants,
            output_prefabs: value.output_prefabs,
            output_animations: value.output_animations,
            prefab_components: value.prefab_components,
            material_overrides: value.material_overrides,
            transforms: value.transforms,
        }
    }
}

impl From<json_pipeline::PipelineConfig> for PipelineProcessor {
    fn from(value: json_pipeline::PipelineConfig) -> Self {
        match value {
            json_pipeline::PipelineConfig::Models(v) => Self::Models(v.into()),
            json_pipeline::PipelineConfig::Materials(v) => Self::Materials(v.into()),
            json_pipeline::PipelineConfig::Audio(v) => Self::Audio(v.into()),
        }
    }
}

impl From<json_pipeline::Pipeline> for Pipeline {
    fn from(value: json_pipeline::Pipeline) -> Self {
        Self {
            processor: value.pipeline.into(),
            sources: value.sources,
            tags: value.tags,
            categories: value.categories,
        }
    }
}
