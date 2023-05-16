use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek},
    sync::Arc,
};

use ambient_core::{
    hierarchy::{children, parent},
    transform::local_to_parent,
};
use ambient_ecs::World;
use ambient_model::{model_skins, Model, ModelSkin};
use ambient_renderer::skinning;
use ambient_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use anyhow::Context;
use fbxcel::tree::{
    any::AnyTree,
    v7400::{NodeHandle, Tree},
};
use futures::future::join_all;
use glam::{Mat4, Vec3};
use indexmap::IndexMap;
use itertools::Itertools;
use relative_path::RelativePathBuf;

use self::{
    animation::{FbxAnimationCurve, FbxAnimationCurveNode, FbxAnimationLayer, FbxAnimationStack},
    material::{FbxMaterial, FbxTexture, FbxVideo},
    mesh::{FbxCluster, FbxGeometry, FbxSkin},
    model::FbxModel,
};
use crate::{model_crate::ModelCrate, TextureResolver};

mod animation;
mod material;
mod mesh;
mod model;

pub async fn import_url(
    assets: &AssetCache,
    url: &AbsAssetUrl,
    asset_crate: &mut ModelCrate,
    texture_resolver: TextureResolver,
) -> anyhow::Result<RelativePathBuf> {
    let content = url.download_bytes(assets).await?;
    let cursor = Cursor::new(&*content);
    import_from_fbx_reader(asset_crate, url.to_string(), true, cursor, texture_resolver).await
}

pub async fn import_from_fbx_reader(
    asset_crate: &mut ModelCrate,
    name: String,
    load_images: bool,
    reader: impl Read + Seek,
    texture_resolver: TextureResolver,
) -> anyhow::Result<RelativePathBuf> {
    match AnyTree::from_seekable_reader(reader).context("Failed to load tree")? {
        AnyTree::V7400(_, tree, _) => {
            let mut doc = FbxDoc::from_tree(tree);

            let mut n_meshes = HashMap::new();

            for (id, geo) in doc.geometries.iter() {
                let meshes = geo.to_cpu_meshes(&doc.skins, &doc.clusters);
                n_meshes.insert(*id, meshes.len());
                for (index, mesh) in meshes.into_iter().enumerate() {
                    asset_crate.meshes.insert(format!("{id}_{index}"), mesh);
                }
            }
            let animations = animation::get_animations(&doc);
            for (id, clip) in animations {
                asset_crate.animations.insert(id, clip);
            }

            let images = if load_images {
                join_all(doc.videos.iter().map(|(id, video)| async {
                    (*id, video.to_image(texture_resolver.clone()).await)
                }))
                .await
                .into_iter()
                .filter_map(|(id, image)| {
                    image.map(|img| (id, asset_crate.images.insert(id.to_string(), img)))
                })
                .collect::<HashMap<_, _>>()
            } else {
                Default::default()
            };
            for material in doc.materials.values() {
                let mat = material.to_model_material(
                    name.to_string(),
                    &doc.textures,
                    &doc.videos,
                    &images,
                    asset_crate,
                );
                asset_crate.materials.insert(material.id.to_string(), mat);
            }

            let mut world = World::new("fbx_reader");
            let mut entities = HashMap::new();
            for model in doc.models.values() {
                model.create_model_nodes(&doc, &mut world, &mut entities, asset_crate, &n_meshes);
            }
            for model in doc.models.values() {
                let id = *entities.get(&model.id).unwrap();
                world
                    .set(
                        id,
                        children(),
                        model
                            .children
                            .iter()
                            .map(|i| *entities.get(i).unwrap())
                            .collect(),
                    )
                    .unwrap();
                if let Some(pi) = model.parent {
                    world
                        .add_component(id, parent(), *entities.get(&pi).unwrap())
                        .unwrap();
                }
                if let Ok(joints) = world.get_ref(id, skinning::joints_by_fbx_id()).cloned() {
                    world
                        .remove_component(id, skinning::joints_by_fbx_id())
                        .unwrap();
                    world
                        .add_component(
                            id,
                            skinning::joints(),
                            joints
                                .into_iter()
                                .map(|id| *entities.get(&id).unwrap())
                                .collect(),
                        )
                        .unwrap();
                }
            }
            let mut skins = Vec::new();
            for skin in doc.skins.values() {
                skins.push(ModelSkin {
                    inverse_bind_matrices: Arc::new(skin.inverse_bind_matrices(&doc.clusters)),
                    joints: skin
                        .joints(&doc.clusters)
                        .into_iter()
                        .map(|id| *entities.get(&id).unwrap())
                        .collect(),
                });
            }
            world.add_resource(model_skins(), skins);
            world.add_resource(ambient_core::name(), name);

            let roots = doc
                .models
                .values_mut()
                .filter_map(|model| if model.is_root { Some(model.id) } else { None })
                .collect_vec();

            world.add_resource(
                children(),
                roots.iter().map(|id| *entities.get(id).unwrap()).collect(),
            );

            world.add_resource(
                local_to_parent(),
                Mat4::from_scale(Vec3::ONE * doc.global_settings.unit_scale_factor),
            );

            Ok(asset_crate
                .models
                .insert(ModelCrate::MAIN, Model(world))
                .path)
        }
        _ => Err(anyhow::anyhow!("Got FBX tree of unsupported version")),
    }
}

#[derive(Debug)]
pub struct FbxDoc {
    pub global_settings: FbxGlobalSettings,

    pub models: HashMap<i64, FbxModel>,
    pub materials: HashMap<i64, FbxMaterial>,
    pub textures: HashMap<i64, FbxTexture>,
    pub videos: HashMap<i64, FbxVideo>,

    pub geometries: HashMap<i64, FbxGeometry>,
    pub skins: IndexMap<i64, FbxSkin>,
    pub clusters: HashMap<i64, FbxCluster>,

    pub animation_stacks: HashMap<i64, FbxAnimationStack>,
    pub animation_layers: HashMap<i64, FbxAnimationLayer>,
    pub animation_curve_nodes: HashMap<i64, FbxAnimationCurveNode>,
    pub animation_curves: HashMap<i64, FbxAnimationCurve>,

    pub poses: HashMap<i64, FbxPose>,
}
impl FbxDoc {
    pub async fn from_url(assets: &AssetCache, url: &AbsAssetUrl) -> anyhow::Result<Self> {
        let content = url.download_bytes(assets).await?;
        let cursor = Cursor::new(&*content);
        match AnyTree::from_seekable_reader(cursor).context("Failed to load tree")? {
            AnyTree::V7400(_, tree, _) => Ok(Self::from_tree(tree)),
            _ => Err(anyhow::anyhow!("Unsupported fbx format (not 7.4)")),
        }
    }
    fn from_tree(tree: Tree) -> Self {
        let mut doc = Self {
            global_settings: FbxGlobalSettings::new(tree.root()),

            models: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            videos: HashMap::new(),

            geometries: HashMap::new(),
            skins: IndexMap::new(),
            clusters: HashMap::new(),

            animation_stacks: HashMap::new(),
            animation_layers: HashMap::new(),
            animation_curve_nodes: HashMap::new(),
            animation_curves: HashMap::new(),

            poses: HashMap::new(),
        };
        let objects = tree
            .root()
            .children()
            .find(|node| node.name() == "Objects")
            .unwrap();
        let connections_node = tree
            .root()
            .children()
            .find(|node| node.name() == "Connections")
            .unwrap();
        let connections = connections_node
            .children()
            .map(FbxConnection::from_node)
            .collect_vec();

        for node in objects.children() {
            match node.name() {
                "Model" => {
                    let model = FbxModel::from_node(node);
                    doc.models.insert(model.id, model);
                }
                "Material" => {
                    let material = FbxMaterial::from_node(node);
                    doc.materials.insert(material.id, material);
                }
                "Texture" => {
                    let texture = FbxTexture::from_node(node);
                    doc.textures.insert(texture.id, texture);
                }
                "Video" => {
                    let video = FbxVideo::from_node(node);
                    doc.videos.insert(video.id, video);
                }

                "Geometry" => {
                    let geo = FbxGeometry::from_node(node, &doc.global_settings);
                    doc.geometries.insert(geo.id, geo);
                }
                "Deformer" => match node.attributes()[2].get_string().unwrap() {
                    "Skin" => {
                        let skin = FbxSkin::from_node(node);
                        doc.skins.insert(skin.id, skin);
                    }
                    "Cluster" => {
                        let cluster = FbxCluster::from_node(node);
                        doc.clusters.insert(cluster.id, cluster);
                    }
                    _ => panic!(
                        "Unrecognized type: {}",
                        node.attributes()[2].get_string().unwrap()
                    ),
                },

                "AnimationStack" => {
                    let stack = FbxAnimationStack::from_node(node);
                    doc.animation_stacks.insert(stack.id, stack);
                }
                "AnimationLayer" => {
                    let id = node.attributes()[0].get_i64().unwrap();
                    doc.animation_layers.insert(
                        id,
                        FbxAnimationLayer {
                            curve_nodes: Vec::new(),
                        },
                    );
                }
                "AnimationCurveNode" => {
                    let curve_node = FbxAnimationCurveNode::from_node(node);
                    doc.animation_curve_nodes.insert(curve_node.id, curve_node);
                }
                "AnimationCurve" => {
                    let curve = FbxAnimationCurve::from_node(node);
                    doc.animation_curves.insert(curve.id, curve);
                }

                "Pose" => {
                    let pose = FbxPose::from_node(node);
                    doc.poses.insert(pose.id, pose);
                }
                _ => {}
            }
        }

        let object_types: HashMap<i64, String> = objects
            .children()
            .map(|node| {
                let id = node.attributes()[0].get_i64().unwrap();
                (
                    id,
                    match node.name() {
                        "Deformer" => node.attributes()[2].get_string().unwrap().to_string(),
                        _ => node.name().to_string(),
                    },
                )
            })
            .collect();

        for FbxConnection {
            to, from, property, ..
        } in connections
        {
            if let (Some(to_type), Some(from_type)) =
                (object_types.get(&to), object_types.get(&from))
            {
                match (to_type as &str, from_type as &str) {
                    ("Geometry", "Model") => doc.models.get_mut(&from).unwrap().geometries.push(to),
                    ("Material", "Model") => doc.models.get_mut(&from).unwrap().materials.push(to),
                    ("Texture", "Material") => {
                        if let Some(key) = property.as_ref().map(|x| x as &str) {
                            doc.materials
                                .get_mut(&from)
                                .unwrap()
                                .textures
                                .insert(key.to_string(), to);
                        }
                    }
                    ("Video", "Texture") => doc.textures.get_mut(&from).unwrap().video = Some(to),
                    ("Model", "Model") => {
                        doc.models.get_mut(&from).unwrap().children.push(to);
                        let to_model = doc.models.get_mut(&to).unwrap();
                        to_model.is_root = false;
                        to_model.parent = Some(from);
                    }

                    ("Cluster", "Skin") => doc.skins.get_mut(&from).unwrap().clusters.push(to),
                    ("Skin", "Geometry") => doc.geometries.get_mut(&from).unwrap().skin = Some(to),
                    ("Model", "Cluster") => doc.clusters.get_mut(&from).unwrap().bone_id = Some(to),

                    ("AnimationLayer", "AnimationStack") => {
                        doc.animation_stacks.get_mut(&from).unwrap().layers.push(to)
                    }
                    ("AnimationCurveNode", "AnimationLayer") => doc
                        .animation_layers
                        .get_mut(&from)
                        .unwrap()
                        .curve_nodes
                        .push(to),
                    ("AnimationCurve", "AnimationCurveNode") => {
                        doc.animation_curve_nodes
                            .get_mut(&from)
                            .unwrap()
                            .curves
                            .insert(property.as_ref().unwrap().to_string(), to);
                    }
                    ("AnimationCurveNode", "Model") => {
                        doc.animation_curve_nodes
                            .get_mut(&to)
                            .unwrap()
                            .outputs
                            .push((from, property.as_ref().unwrap().to_string()));
                    }
                    _ => {}
                }
            }
        }
        doc
    }
}

pub enum FbxConnectionType {
    ObjectObject,
    ObjectProperty,
}
pub struct FbxConnection {
    pub connection_type: FbxConnectionType,
    pub to: i64,
    pub from: i64,
    pub property: Option<String>,
}
impl FbxConnection {
    fn from_node(node: NodeHandle) -> Self {
        let t = node.attributes()[0].get_string().unwrap();
        let t = match t {
            "OO" => FbxConnectionType::ObjectObject,
            "OP" => FbxConnectionType::ObjectProperty,
            _ => panic!("Unexpected: {t}"),
        };
        let to = node.attributes()[1].get_i64().unwrap();
        let from = node.attributes()[2].get_i64().unwrap();
        Self {
            from,
            to,
            property: match t {
                FbxConnectionType::ObjectProperty => {
                    Some(node.attributes()[3].get_string().unwrap().to_string())
                }
                _ => None,
            },
            connection_type: t,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FbxGlobalSettings {
    up_axis: usize,
    up_axis_sign: f32,
    front_axis: usize,
    front_axis_sign: f32,
    coord_axis: usize,
    coord_axis_sign: f32,
    unit_scale_factor: f32,
}
impl Default for FbxGlobalSettings {
    fn default() -> Self {
        Self {
            up_axis: 1,
            up_axis_sign: 1.,
            front_axis: 2,
            front_axis_sign: 1.,
            coord_axis: 0,
            coord_axis_sign: 1.,
            unit_scale_factor: 1.,
        }
    }
}
impl FbxGlobalSettings {
    fn new(root: NodeHandle) -> Self {
        let global_settings = root
            .children()
            .find(|node| node.name() == "GlobalSettings")
            .unwrap();
        let properties = global_settings
            .children()
            .find(|node| node.name() == "Properties70")
            .unwrap();
        let mut settings = Self::default();
        for prop in properties.children() {
            let name = prop.attributes()[0].get_string().unwrap();
            match name {
                "UpAxis" => settings.up_axis = prop.attributes()[4].get_i32().unwrap() as usize,
                "UpAxisSign" => {
                    settings.up_axis_sign = prop.attributes()[4].get_i32().unwrap() as f32
                }
                "FrontAxis" => {
                    settings.front_axis = prop.attributes()[4].get_i32().unwrap() as usize
                }
                "FrontAxisSign" => {
                    settings.front_axis_sign = prop.attributes()[4].get_i32().unwrap() as f32
                }
                "CoordAxis" => {
                    settings.coord_axis = prop.attributes()[4].get_i32().unwrap() as usize
                }
                "CoordAxisSign" => {
                    settings.coord_axis_sign = prop.attributes()[4].get_i32().unwrap() as f32
                }
                "UnitScaleFactor" => {
                    settings.unit_scale_factor = prop.attributes()[4].get_f64().unwrap() as f32
                }
                _ => {}
            }
        }
        settings
    }
}

#[derive(Debug)]
pub struct FbxPose {
    id: i64,
    _nodes: Vec<(i64, Mat4)>,
}
impl FbxPose {
    fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        Self {
            id,
            _nodes: node
                .children()
                .filter_map(|child| {
                    if child.name() == "PoseNode" {
                        let n = child.children().find(|node| node.name() == "Node").unwrap();
                        let matrix = child
                            .children()
                            .find(|node| node.name() == "Matrix")
                            .unwrap();
                        Some((n.attributes()[0].get_i64().unwrap(), read_matrix(matrix)))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

pub fn read_matrix(node: NodeHandle) -> Mat4 {
    let mat = node.attributes()[0]
        .get_arr_f64()
        .unwrap()
        .iter()
        .map(|v| *v as f32)
        .collect_vec();
    Mat4::from_cols_slice(&mat)
}
