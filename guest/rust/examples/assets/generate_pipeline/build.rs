use ambient_pipeline_types::{
    models::{ModelTextureSize, ModelTransform},
    ModelImporter, ModelsPipeline, Pipeline, PipelineProcessor, PipelinesFile,
};

fn main() {
    PipelinesFile {
        pipelines: vec![Pipeline {
            processor: PipelineProcessor::Models(ModelsPipeline {
                importer: ModelImporter::Regular,
                cap_texture_sizes: Some(ModelTextureSize::Custom(2)),
                transforms: vec![ModelTransform::RotateZ { deg: 90. }],
                ..Default::default()
            }),
            sources: vec!["*".to_string()],
            tags: vec![],
            categories: vec![],
        }],
    }
    .save_to_file("assets/pipeline.toml")
    .unwrap();
}
