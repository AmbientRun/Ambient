use std::{collections::HashMap, io::Cursor, path::PathBuf, sync::Arc};

use ambient_animation::{animation_bind_id_from_name, AnimationClip};
use ambient_core::{
    bounding::local_bounding_aabb,
    hierarchy::children,
    name,
    transform::{local_to_parent, local_to_world, mesh_to_local, TransformSystem},
};
use ambient_ecs::{query, query_mut, Component, ComponentValue, Entity, EntityId, FrameEvent, System, World};
use ambient_model::{
    animation_bind_id, model_from_url, model_skin_ix, model_skins, pbr_renderer_primitives_from_url, Model, PbrRenderPrimitiveFromUrl,
};
use ambient_physics::{
    collider::{character_controller_height, character_controller_radius, collider, ColliderDef, ColliderFromUrls},
    mesh::PhysxGeometryFromUrl,
    physx::PhysicsKey,
};
use ambient_renderer::{
    double_sided,
    lod::{gpu_lod, lod_cutoffs, LodCutoffs},
    materials::pbr_material::PbrMaterialDesc,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    download_asset::AssetsCacheDir,
    mesh::Mesh,
    shapes::AABB,
};
use anyhow::Context;
use futures::FutureExt;
use glam::{Mat4, Vec3};
use image::{ImageOutputFormat, RgbaImage};
use itertools::Itertools;
use ordered_float::Float;
use physxx::{PxConvexFlag, PxConvexMeshDesc, PxDefaultMemoryOutputStream, PxMeshFlag, PxTriangleMeshDesc};
use relative_path::RelativePathBuf;

use crate::{dotdot_path, MaterialFilter, TextureResolver};

#[derive(Debug, Clone)]
pub struct AssetLoc {
    pub id: String,
    pub path: RelativePathBuf,
}

pub struct AssetMapLoc {
    store: String,
    extension: String,
}
impl AssetMapLoc {
    pub fn path(&self, id: impl Into<String>) -> RelativePathBuf {
        format!("{}/{}.{}", self.store, id.into(), self.extension).into()
    }
    pub fn id_from_path(&self, path: impl Into<RelativePathBuf>) -> Option<String> {
        let path: RelativePathBuf = path.into();
        let path = path.to_string();
        if let Some((_, file)) = path.split_once(&format!("{}/", self.store)) {
            if let Some((id, extension)) = file.rsplit_once('.') {
                if extension == self.extension {
                    return Some(id.to_string());
                }
            }
        }
        None
    }
}

pub struct AssetMap<T> {
    pub loc: AssetMapLoc,
    pub content: HashMap<String, T>,
    pub serialize: fn(&T) -> Vec<u8>,
}
impl<T: Send + 'static> AssetMap<T> {
    fn new(store: &str, extension: &str, serialize: fn(&T) -> Vec<u8>) -> Self {
        Self { loc: AssetMapLoc { store: store.to_string(), extension: extension.into() }, content: Default::default(), serialize }
    }

    pub fn get_by_path(&self, path: impl Into<RelativePathBuf>) -> Option<&T> {
        self.content.get(&self.loc.id_from_path(path)?)
    }
    pub fn insert(&mut self, id: impl Into<String>, content: T) -> AssetLoc {
        let id: String = id.into();
        self.content.insert(id.clone(), content);
        AssetLoc { path: self.loc.path(&id), id }
    }
    pub fn to_items(&self) -> Vec<AssetItem> {
        self.content.iter().map(|(id, content)| AssetItem { path: self.loc.path(id), data: Arc::new((self.serialize)(content)) }).collect()
    }
}

pub struct ModelCrate {
    pub models: AssetMap<Model>,
    pub prefabs: AssetMap<World>,
    pub meshes: AssetMap<Mesh>,
    pub animations: AssetMap<AnimationClip>,
    pub images: AssetMap<image::RgbaImage>,
    pub materials: AssetMap<PbrMaterialDesc>,
    pub px_triangle_meshes: AssetMap<Vec<u8>>,
    pub px_convex_meshes: AssetMap<Vec<u8>>,
    pub colliders: AssetMap<ColliderFromUrls>,
}
impl ModelCrate {
    pub fn new() -> Self {
        Self {
            models: AssetMap::new("models", "json", |v| serde_json::to_vec(v).unwrap()),
            prefabs: AssetMap::new("prefabs", "json", |v| serde_json::to_vec(v).unwrap()),
            meshes: AssetMap::new("meshes", "mesh", |v| bincode::serialize(v).unwrap()),
            animations: AssetMap::new("animations", "anim", |v| bincode::serialize(v).unwrap()),
            images: AssetMap::new("images", "png", |v| {
                let mut data = Cursor::new(Vec::new());
                v.write_to(&mut data, ImageOutputFormat::Png).unwrap();
                data.into_inner()
            }),
            materials: AssetMap::new("materials", "json", |v| serde_json::to_vec(v).unwrap()),
            px_triangle_meshes: AssetMap::new("px_triangle_meshes", "pxtm", |v| v.clone()),
            px_convex_meshes: AssetMap::new("px_convex_meshes", "pxcm", |v| v.clone()),
            colliders: AssetMap::new("colliders", "json", |v| serde_json::to_vec(v).unwrap()),
        }
    }
    pub async fn local_import(assets: &AssetCache, url: &AbsAssetUrl, normalize: bool, force_assimp: bool) -> anyhow::Result<Model> {
        let cache_path = AssetsCacheDir.get(assets).join("pipelines").join(url.relative_cache_path());
        let mut model = Self::new();
        model
            .import(
                assets,
                url,
                normalize,
                force_assimp,
                Arc::new(|path| {
                    async move {
                        let path: PathBuf = path.into();
                        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                        log::info!("XXX {filename:?}");
                        None
                    }
                    .boxed()
                }),
            )
            .await?;
        model.update_node_primitive_aabbs_from_cpu_meshes();
        model.model_mut().update_model_aabb();
        model.produce_local_model(assets, cache_path).await
    }
    pub async fn write_to_fs(&self, path: &PathBuf) {
        for item in self.to_items() {
            let item_path = item.path.to_path(path);
            std::fs::create_dir_all(item_path.parent().unwrap())
                .context(format!("Failed to create dir: {:?}", item_path.parent().unwrap()))
                .unwrap();
            tokio::fs::write(&item_path, &*item.data).await.context(format!("Failed to write file: {item_path:?}")).unwrap();
        }
    }
    pub fn to_items(&self) -> Vec<AssetItem> {
        [
            self.models.to_items().into_iter(),
            self.prefabs.to_items().into_iter(),
            self.meshes.to_items().into_iter(),
            self.animations.to_items().into_iter(),
            self.images.to_items().into_iter(),
            self.materials.to_items().into_iter(),
            self.px_triangle_meshes.to_items().into_iter(),
            self.px_convex_meshes.to_items().into_iter(),
            self.colliders.to_items().into_iter(),
        ]
        .into_iter()
        .flatten()
        .collect_vec()
    }
    pub const MAIN: &str = "main";
    pub fn model(&self) -> &Model {
        self.models.content.get(Self::MAIN).unwrap()
    }
    pub fn model_mut(&mut self) -> &mut Model {
        self.models.content.get_mut(Self::MAIN).unwrap()
    }
    pub fn model_world(&self) -> &World {
        self.models.content.get(Self::MAIN).map(|x| &x.0).unwrap()
    }
    pub fn model_world_mut(&mut self) -> &mut World {
        self.models.content.get_mut(Self::MAIN).map(|x| &mut x.0).unwrap()
    }
    pub fn prefab_world(&self) -> &World {
        self.prefabs.content.get(Self::MAIN).unwrap()
    }
    pub fn prefab_world_mut(&mut self) -> &mut World {
        self.prefabs.content.get_mut(Self::MAIN).unwrap()
    }

    pub async fn produce_local_model_url(&self, path: PathBuf) -> anyhow::Result<PathBuf> {
        self.write_to_fs(&path).await;
        let model_id = self.models.content.keys().next().unwrap();
        Ok(self.models.loc.path(model_id).to_path(path))
    }
    pub async fn produce_local_model(&self, assets: &AssetCache, path: PathBuf) -> anyhow::Result<Model> {
        let url = self.produce_local_model_url(path).await?;
        let mut model = Model::from_file(&url).await?;
        model.load(assets, &url.into()).await?;
        Ok(model)
    }

    pub async fn import(
        &mut self,
        assets: &AssetCache,
        url: &AbsAssetUrl,
        normalize: bool,
        force_assimp: bool,
        resolve_texture: TextureResolver,
    ) -> anyhow::Result<()> {
        let is_fbx = url.extension().unwrap_or_default() == "fbx";
        let is_glb = url.extension().unwrap_or_default() == "glb";
        if force_assimp {
            crate::assimp::import_url(assets, url, self, resolve_texture).await?;
        } else if is_fbx {
            if let Err(err) = crate::fbx::import_url(assets, url, self, resolve_texture.clone()).await {
                match err.downcast::<fbxcel::tree::any::Error>() {
                    Ok(err) => {
                        if let fbxcel::tree::any::Error::ParserCreation(fbxcel::pull_parser::any::Error::Header(
                            fbxcel::low::HeaderError::MagicNotDetected,
                        )) = &err
                        {
                            crate::assimp::import_url(assets, url, self, resolve_texture).await?;
                        } else {
                            return Err(err.into());
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
        } else if is_glb {
            crate::gltf::import_url(assets, url, self).await?;
        } else {
            crate::assimp::import_url(assets, url, self, resolve_texture).await?;
        }
        if normalize {
            self.model_mut().rotate_yup_to_zup();
            if is_fbx {
                self.model_mut().transform(Mat4::from_scale(Vec3::ONE / 100.));
            }
        }
        Ok(())
    }
    pub fn merge_mesh_lods(&mut self, cutoffs: Option<Vec<f32>>, lods: Vec<ModelNodeRef>) {
        let default_min_screen_size = 0.04; // i.e. 4%
        let lod_step = (1. / default_min_screen_size).powf(1. / (lods.len() - 1) as f32);
        let mut cutoffs = cutoffs.unwrap_or_else(|| (0..lods.len()).map(|i| 1. / lod_step.powi(i as i32)).collect_vec());

        let lod_0_node = lods[0].get_node_id();
        let lod_0_world = lods[0].world();

        let mut world = World::new("model mesh lods");
        world.add_resource(name(), format!("{}_merged_lods", lod_0_world.resource_opt(name()).map(|x| x as &str).unwrap_or("unknown")));
        if let Some(aabb) = lod_0_world.resource_opt(local_bounding_aabb()) {
            world.add_resource(local_bounding_aabb(), *aabb);
        }
        if let Some(ltp) = lod_0_world.resource_opt(local_to_parent()) {
            world.add_resource(local_to_parent(), *ltp);
        }

        let mut root = Entity::new()
            .with(name(), "root".to_string())
            .with(lod_cutoffs(), LodCutoffs::new(&cutoffs))
            .with_default(gpu_lod())
            .with(
                mesh_to_local(),
                lod_0_world.get(lod_0_node, local_to_world()).unwrap_or_default()
                    * lod_0_world.get(lod_0_node, mesh_to_local()).unwrap_or(Mat4::IDENTITY),
            )
            .with(double_sided(), lod_0_world.get(lod_0_node, double_sided()).unwrap_or_default())
            .with(local_bounding_aabb(), lod_0_world.get(lod_0_node, local_bounding_aabb()).unwrap_or_default())
            .with_default(local_to_world())
            .with_default(pbr_renderer_primitives_from_url());

        for (i, lod) in lods.iter().enumerate() {
            let lod_world = lod.world();
            let lod_id = lod.get_node_id();
            for primitive in lod_world.get_ref(lod_id, pbr_renderer_primitives_from_url()).cloned().unwrap_or_default() {
                let mesh_id = format!("{}_{}", i, lod.model.meshes.loc.id_from_path(primitive.mesh.path()).unwrap());
                let mesh_path =
                    self.meshes.insert(mesh_id.clone(), lod.model.meshes.get_by_path(primitive.mesh.path()).unwrap().clone()).path;
                let material = primitive.material.as_ref().and_then(|mat_url| {
                    let mat_id = lod.model.materials.loc.id_from_path(mat_url.path())?;
                    let lod_mat = lod.model.materials.content.get(&mat_id)?;
                    Some(self.materials.insert(i.to_string(), lod_mat.clone()).path)
                });
                root.get_mut(pbr_renderer_primitives_from_url()).unwrap().push(PbrRenderPrimitiveFromUrl {
                    mesh: dotdot_path(mesh_path).into(),
                    material: material.map(|x| dotdot_path(x).into()),
                    lod: i,
                });
            }
        }
        let root = root.spawn(&mut world);
        world.add_resource(children(), vec![root]);
        self.models.insert(ModelCrate::MAIN, Model(world));
    }
    pub fn merge_unity_style_mesh_lods(&mut self, source: &ModelCrate, cutoffs: Option<Vec<f32>>) {
        let mut lods = source.model_world().resource(children()).clone();
        lods.sort_by_key(|id| {
            let name = source.model_world().get_ref(*id, name()).unwrap();
            &name[(name.len() - 2)..]
        });
        self.merge_mesh_lods(cutoffs, lods.into_iter().map(|id| ModelNodeRef { model: source, root: Some(id) }).collect())
    }
    pub fn set_all_material(&mut self, material: PbrMaterialDesc) {
        self.materials.content.clear();
        let mat_path = dotdot_path(self.materials.insert("main".to_string(), material).path);
        for (_, primitives, _) in query_mut(pbr_renderer_primitives_from_url(), ()).iter(self.model_world_mut(), None) {
            for primitive in primitives.iter_mut() {
                primitive.material = Some(mat_path.clone().into());
            }
        }
    }
    pub fn update_node_primitive_aabbs_from_cpu_meshes(&mut self) {
        let world = &mut self.models.content.get_mut(ModelCrate::MAIN).unwrap().0;
        let joint_matrices = world.resource_opt(model_skins()).map(|skins| {
            skins
                .iter()
                .map(|skin| {
                    skin.joints
                        .iter()
                        .zip(skin.inverse_bind_matrices.iter())
                        .map(|(joint, inv_bind_mat)| world.get(*joint, local_to_world()).unwrap_or_default() * *inv_bind_mat)
                        .collect_vec()
                })
                .collect::<Vec<_>>()
        });
        for (node, primitives) in query(pbr_renderer_primitives_from_url()).collect_cloned(world, None) {
            let aabbs = primitives
                .iter()
                .filter_map(|p| {
                    if let Some(mesh) = self.meshes.get_by_path(&RelativePathBuf::from("materials").join(p.mesh.path())) {
                        if let Ok(skin_id) = world.get(node, model_skin_ix()) {
                            if let Some(joint_matrices) = joint_matrices.as_ref().unwrap().get(skin_id) {
                                let mut mesh = mesh.clone();
                                let joint_matrices = joint_matrices
                                    .iter()
                                    .map(|mat| {
                                        (world.get(node, local_to_world()).unwrap_or_default()
                                            * world.get(node, mesh_to_local()).unwrap_or_default())
                                        .inverse()
                                            * *mat
                                    })
                                    .collect_vec();
                                mesh.apply_skin(&joint_matrices);
                                return mesh.aabb();
                            }
                        } else {
                            return mesh.aabb();
                        }
                    }
                    None
                })
                .collect_vec();
            if let Some(aabb) = AABB::unions(&aabbs) {
                world.add_component(node, local_bounding_aabb(), aabb).unwrap();
            }
        }
    }
    pub fn make_new_root(&mut self, node_id: EntityId) {
        let world = self.model_world_mut();
        *world.resource_mut(children()) = vec![node_id];
        // TODO(fred): Do the below; clear out unused materials etc.
        // let mut model = Self {
        //     name: format!("{}#{}", self.name, node_id),
        //     source_url: self.source_url.clone(),
        //     roots: vec![node_id.to_string()],
        //     ..Default::default()
        // };
        // self.add_nodes_to_model_recursive(&mut model, node_id);
        // model.update_local_to_models();
        // model.update_model_aabb();
        // model
    }
    // fn add_nodes_to_model_recursive(&self, target: &mut Self, id: &str) {
    //     if let Some(node) = self.nodes.get(id) {
    //         target.nodes.insert(id.to_string(), node.clone());
    //         for primitive in &node.primitives {
    //             if !target.cpu_meshes.contains_key(&primitive.mesh) {
    //                 if let Some(mesh) = self.cpu_meshes.get(&primitive.mesh) {
    //                     target.cpu_meshes.insert(primitive.mesh.clone(), mesh.clone());
    //                 }
    //             }
    //             if !target.gpu_meshes.contains_key(&primitive.mesh) {
    //                 if let Some(mesh) = self.gpu_meshes.get(&primitive.mesh) {
    //                     target.gpu_meshes.insert(primitive.mesh.clone(), mesh.clone());
    //                 }
    //             }
    //             if let Some(material_id) = &primitive.material {
    //                 if !target.gpu_materials.contains_key(material_id) {
    //                     if let Some(material) = self.gpu_materials.get(material_id) {
    //                         target.gpu_materials.insert(material_id.clone(), material.clone());
    //                     }
    //                 }
    //             }
    //         }
    //         if let Some(skin_id) = &node.skin {
    //             if !target.skins.contains_key(skin_id) {
    //                 if let Some(skin) = self.skins.get(skin_id) {
    //                     target.skins.insert(skin_id.clone(), skin.clone());
    //                 }
    //             }
    //         }
    //         for c in &node.children {
    //             self.add_nodes_to_model_recursive(target, c);
    //         }
    //     }
    // }
    pub fn override_material(&mut self, filter: &MaterialFilter, material: PbrMaterialDesc) {
        if filter.is_all() {
            self.set_all_material(material);
        } else {
            for old_mat in self.materials.content.values_mut() {
                if filter.matches(&*old_mat) {
                    *old_mat = material.clone();
                }
            }
        }
    }
    pub fn cap_texture_sizes(&mut self, max_size: u32) {
        for image in self.images.content.values_mut() {
            cap_texture_size(image, max_size);
        }
    }
    pub fn update_transforms(&mut self) {
        TransformSystem::new().run(self.model_world_mut(), &FrameEvent);
    }
    pub fn create_animation_bind_ids(&mut self) {
        let world = self.model_world_mut();
        for (id, name) in query(name()).collect_cloned(world, None) {
            world.add_component(id, animation_bind_id(), animation_bind_id_from_name(&name)).unwrap();
        }
    }
    pub fn finalize_model(&mut self) {
        self.update_transforms();
        self.update_node_primitive_aabbs_from_cpu_meshes();
        self.model_mut().update_model_aabb();
        self.create_animation_bind_ids();
        self.model_mut().remove_non_storage_matrices();
    }

    pub fn create_prefab_from_model(&mut self) {
        self.create_prefab(Entity::new().with(model_from_url(), dotdot_path(self.models.loc.path(ModelCrate::MAIN)).into()))
    }

    pub fn create_prefab(&mut self, data: Entity) {
        let mut prefab = World::new("prefab_asset");
        let o = data.spawn(&mut prefab);
        prefab.add_resource(children(), vec![o]);
        self.prefabs.insert(ModelCrate::MAIN, prefab);
    }
    pub fn add_component_to_prefab<T: ComponentValue>(&mut self, component: Component<T>, value: T) {
        let world = self.prefab_world_mut();
        let object = world.resource(children())[0];
        world.add_component(object, component, value).unwrap();
    }
    pub fn create_character_collider(&mut self, radius: Option<f32>, height: Option<f32>) {
        let world = self.prefab_world_mut();
        let object = world.resource(children())[0];
        world.add_component(object, character_controller_radius(), radius.unwrap_or(0.5)).unwrap();
        world.add_component(object, character_controller_height(), height.unwrap_or(2.0)).unwrap();
    }
    pub fn create_collider_from_model(&mut self, assets: &AssetCache, flip_normals: bool, reverse_indices: bool) -> anyhow::Result<()> {
        self.update_transforms();
        let physics = PhysicsKey.get(assets);
        let create_triangle_mesh = |asset_crate: &mut ModelCrate, id: &str| -> bool {
            if asset_crate.px_triangle_meshes.content.contains_key(id) {
                return true;
            }
            let mesh = asset_crate.meshes.content.get(id).unwrap();
            if let Some(desc) = physx_triangle_mesh_desc_from_mesh(mesh, flip_normals, reverse_indices) {
                let stream = PxDefaultMemoryOutputStream::new();
                let mut res = physxx::PxTriangleMeshCookingResult::Success;
                if !physics.cooking.cook_triangle_mesh(&desc, &stream, &mut res) {
                    log::error!("Failed to cook triangle mesh: {:?}", res);
                    return false;
                }
                asset_crate.px_triangle_meshes.content.insert(id.to_string(), stream.get_data());
                true
            } else {
                false
            }
        };
        let create_convex_mesh = |asset_crate: &mut ModelCrate, id: &str, scale_signum: Vec3| -> Option<RelativePathBuf> {
            // Physx doesn't support negative scaling on Convex meshes, so we need to generate a mesh with the right
            // scale signum first, and then scale that with the absolute scale
            let to_sign = |v| if v >= 0. { "p" } else { "n" }.to_string();
            let full_id = format!("{id}_{}{}{}", to_sign(scale_signum.x), to_sign(scale_signum.y), to_sign(scale_signum.z));
            if asset_crate.px_convex_meshes.content.contains_key(&full_id) {
                return Some(asset_crate.px_convex_meshes.loc.path(&full_id));
            }
            let mesh = asset_crate.meshes.content.get(id).unwrap();

            let desc = PxConvexMeshDesc {
                // Apply the correct mirroring according to the base scale
                points: mesh.positions.as_ref().unwrap().iter().map(|&p| p * scale_signum).collect_vec(),
                indices: mesh.indices.clone(),
                vertex_limit: None,
                flags: Some(PxConvexFlag::COMPUTE_CONVEX),
            };
            let stream = PxDefaultMemoryOutputStream::new();
            let mut res = physxx::PxConvexMeshCookingResult::Success;
            if !physics.cooking.cook_convex_mesh(&desc, &stream, &mut res) {
                log::error!("Failed to cook convex mesh: {:?}", res);
                return None;
            }
            Some(asset_crate.px_convex_meshes.insert(full_id, stream.get_data()).path)
        };
        let mut convex = Vec::new();
        let mut triangle = Vec::new();
        let world_transform = self.model().get_transform().unwrap_or_default();
        let entities = {
            let world = self.model_world();
            query(pbr_renderer_primitives_from_url()).collect_cloned(world, None)
        };
        for (id, prims) in entities {
            let ltw = self.model_world().get(id, local_to_world()).unwrap_or_default();
            if let Some(max_lod) = prims.iter().map(|x| x.lod).max() {
                let mtl = self.model_world().get(id, mesh_to_local()).unwrap_or_default();
                // Only use the "max" lod for colliders
                for primitive in prims.into_iter().filter(|x| x.lod == max_lod) {
                    let transform = world_transform * ltw * mtl;
                    let (scale, rot, pos) = transform.to_scale_rotation_translation();
                    let mesh_id = self.meshes.loc.id_from_path(primitive.mesh.path()).unwrap();
                    if create_triangle_mesh(self, &mesh_id) {
                        if let Some(convex_path) = create_convex_mesh(self, &mesh_id, scale.signum()) {
                            let convex_path = dotdot_path(convex_path);
                            let triangle_path = dotdot_path(self.px_triangle_meshes.loc.path(mesh_id));
                            convex.push((
                                Mat4::from_scale_rotation_translation(scale.abs(), rot, pos),
                                PhysxGeometryFromUrl(convex_path.into()),
                            ));
                            triangle.push((transform, PhysxGeometryFromUrl(triangle_path.into())));
                        }
                    }
                }
            }
        }
        let obj_collider = self.colliders.insert(ModelCrate::MAIN.to_string(), ColliderFromUrls { convex, concave: triangle });
        let prefab = self.prefab_world_mut();
        prefab
            .add_component(
                prefab.resource(children())[0],
                collider(),
                ColliderDef::Asset { collider: dotdot_path(obj_collider.path).into() },
            )
            .unwrap();
        Ok(())
    }
}
pub struct AssetItem {
    pub path: RelativePathBuf,
    pub data: Arc<Vec<u8>>,
}

pub fn cap_texture_size(image: &mut RgbaImage, max_size: u32) {
    if image.width() > max_size || image.height() > max_size {
        let (width, height) = if image.width() >= image.height() {
            (max_size, (max_size as f32 * image.height() as f32 / image.width() as f32) as u32)
        } else {
            ((max_size as f32 * image.width() as f32 / image.height() as f32) as u32, max_size)
        };
        *image = image::imageops::resize(&*image as &image::RgbaImage, width, height, image::imageops::FilterType::CatmullRom);
    }
}

pub struct ModelNodeRef<'a> {
    pub model: &'a ModelCrate,
    pub root: Option<EntityId>,
}
impl<'a> ModelNodeRef<'a> {
    fn world(&self) -> &World {
        self.model.model_world()
    }
    fn get_node_id(&self) -> EntityId {
        self.root.unwrap_or(self.world().resource(children())[0])
    }
}

pub fn physx_triangle_mesh_desc_from_mesh(mesh: &Mesh, flip_normals: bool, reverse_indices: bool) -> Option<PxTriangleMeshDesc> {
    let mut desc = PxTriangleMeshDesc {
        points: mesh.positions.clone()?,
        indices: mesh.indices.clone()?,
        flags: if flip_normals { Some(PxMeshFlag::FLIPNORMALS) } else { None },
    };
    if desc.points.is_empty() || desc.indices.is_empty() {
        return None;
    }

    if reverse_indices {
        for i in 0..(desc.indices.len() / 3) {
            desc.indices.swap(i * 3 + 1, i * 3 + 2);
        }
    }
    Some(desc)
}
