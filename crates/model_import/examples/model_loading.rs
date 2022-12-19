use elements_app::App;
use elements_core::{
    asset_cache, camera::{active_camera, far}, main_scene, transform::*
};
use elements_ecs::{EntityData, World};
use elements_element::ElementComponentExt;
use elements_model::{model_def, ModelDef};
use elements_model_import::{MaterialFilter, ModelImportPipeline, ModelImportTransform, ModelTransform};
use elements_primitives::{Cube, Quad};
use elements_renderer::{color, materials::pbr_material::PbrMaterialFromUrl};
use elements_std::{asset_cache::AsyncAssetKeyExt, math::SphericalCoords};
use glam::*;

async fn init(world: &mut World) {
    let assets = world.resource(asset_cache()).clone();

    Quad.el().set(scale(), Vec3::ONE * 20.).spawn_static(world);

    let asset_pipelines = vec![
        {
            let fir_base =
                "https://dims-content.fra1.digitaloceanspaces.com/assets/models/Unity/Dynamic%20Nature%20-%20Mountain%20Tree%20Pack/";

            ModelImportPipeline::new()
                .add_step(ModelImportTransform::MergeUnityMeshLods { url: format!("{fir_base}Fir_02_Small.FBX"), lod_cutoffs: None })
                .add_step(ModelImportTransform::OverrideMaterial {
                    filter: MaterialFilter::by_name("M_leaves_Fir"),
                    material: Box::new(PbrMaterialFromUrl {
                        base_color: Some(format!("{fir_base}Textures/T_Fir_leaves_BC_T.TGA").into()),
                        ..Default::default()
                    }),
                })
        },
        {
            let grass_base = "https://dims-content.fra1.digitaloceanspaces.com/assets/models/Quixel/Grass_vlkhcbxia_2K_3dplant_ms/";
            let grass_atlas = PbrMaterialFromUrl {
                base_color: Some(format!("{grass_base}Textures/Atlas/vlkhcbxia_2K_Albedo.jpg").into()),
                opacity: Some(format!("{grass_base}Textures/Atlas/vlkhcbxia_2K_Opacity.jpg").into()),
                double_sided: Some(true),
                ..Default::default()
            };
            let grass_billboard = PbrMaterialFromUrl {
                base_color: Some(format!("{grass_base}Textures/Billboard/Billboard_2K_Albedo.jpg").into()),
                opacity: Some(format!("{grass_base}Textures/Billboard/Billboard_2K_Opacity.jpg").into()),
                alpha_cutoff: Some(0.1),
                double_sided: Some(true),
                ..Default::default()
            };
            ModelImportPipeline::new()
                .add_step(ModelImportTransform::MergeMeshLods {
                    lods: vec![
                        ModelImportPipeline::model(format!("{grass_base}Var11/Var11_LOD0.fbx")).add_step(
                            ModelImportTransform::OverrideMaterial { filter: MaterialFilter::All, material: Box::new(grass_atlas.clone()) },
                        ),
                        ModelImportPipeline::model(format!("{grass_base}Var11/Var11_LOD1.fbx")).add_step(
                            ModelImportTransform::OverrideMaterial { filter: MaterialFilter::All, material: Box::new(grass_atlas.clone()) },
                        ),
                        ModelImportPipeline::model(format!("{grass_base}Var11/Var11_LOD2.fbx")).add_step(
                            ModelImportTransform::OverrideMaterial { filter: MaterialFilter::All, material: Box::new(grass_atlas.clone()) },
                        ),
                        ModelImportPipeline::model(format!("{grass_base}Var11/Var11_LOD3.fbx")).add_step(
                            ModelImportTransform::OverrideMaterial { filter: MaterialFilter::All, material: Box::new(grass_atlas) },
                        ),
                        ModelImportPipeline::model(format!("{grass_base}Var11/Var11_LOD4.fbx")).add_step(
                            ModelImportTransform::OverrideMaterial { filter: MaterialFilter::All, material: Box::new(grass_billboard) },
                        ),
                    ],
                    lod_cutoffs: None,
                })
                .add_step(ModelImportTransform::Transform(ModelTransform::Scale { scale: 5. }))
        },
        ModelImportPipeline::model("https://dims-content.fra1.digitaloceanspaces.com/assets/models/Misc/Soldier.glb"),
        ModelImportPipeline::model_raw("https://dims-content.fra1.digitaloceanspaces.com/assets/models/PolyHaven/Barrel_01_4k.glb"),
        ModelImportPipeline::model("https://dims-content.fra1.digitaloceanspaces.com/assets/models/PolyHaven/Barrel_01_4k.glb"),
    ];
    let mut model_defs = Vec::new();
    for pipeline in asset_pipelines.iter() {
        let model_url = pipeline.produce_local_model_url(&assets).await.unwrap();
        model_defs.push(ModelDef(model_url));
    }

    // "Regular" spawning
    for (i, model_def) in model_defs.iter().enumerate() {
        let xy = vec2(i as f32 * 3., 0.);
        Cube.el().set(translation(), xy.extend(-0.9)).set(color(), vec4(0.3, 0.3, 0.3, 1.)).spawn_static(world);
        let model = model_def.get(&assets).await.unwrap();
        let entity = model.spawn(world, &Default::default());
        world.add_component(entity, translation(), xy.extend(0.1)).unwrap();
    }

    // Attaching
    for (i, mod_def) in model_defs.iter().enumerate() {
        let xy = vec2(i as f32 * 3., 3.);
        Cube.el().set(translation(), xy.extend(-0.9)).set(color(), vec4(0.3, 0.3, 0.3, 1.)).spawn_static(world);
        EntityData::new().set(model_def(), mod_def.clone()).set(translation(), xy.extend(0.1)).spawn(world);
    }

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .set(far(), 2000.)
        .spawn(world);
}

fn main() {
    env_logger::init();
    App::run_debug_app_with_config(false, true, true, |app, runtime| {
        runtime.block_on(async { init(&mut app.world).await });
    });
}
