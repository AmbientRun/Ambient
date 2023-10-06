use ambient_native_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl};
use relative_path::RelativePathBuf;
#[cfg(feature = "russimp")]
use russimp::{
    material::{Material, PropertyTypeInfo},
    node::Node,
    scene::{PostProcess, Scene},
    texture::TextureType,
};

use crate::{model_crate::ModelCrate, TextureResolver};

#[allow(unused_variables)]
pub async fn import_url<'a>(
    assets: &'a AssetCache,
    url: &'a AbsAssetUrl,
    model_crate: &'a mut ModelCrate,
    resolve_texture: TextureResolver,
) -> anyhow::Result<RelativePathBuf> {
    #[cfg(feature = "russimp")]
    {
        let content = url.download_bytes(assets).await?;
        let extension = url.extension().unwrap_or_default();
        import(&content, model_crate, &extension, resolve_texture).await
    }
    #[cfg(not(feature = "russimp"))]
    panic!("This binary was built without assimp support");
}

#[cfg(feature = "russimp")]
pub async fn import<'a>(
    buffer: &'a [u8],
    model_crate: &'a mut ModelCrate,
    extension: &'a str,
    resolve_texture: TextureResolver,
) -> anyhow::Result<RelativePathBuf> {
    use crate::dotdot_path;
    use ambient_renderer::materials::pbr_material::PbrMaterialDesc;
    use std::collections::HashMap;

    let (path, materials) = import_sync(buffer, model_crate, extension)?;
    for (i, material) in materials.iter().enumerate() {
        let mut textures = HashMap::new();
        for (key, texs) in &material.textures {
            if let Some(tex) = texs.get(0) {
                let image = if let Some(_data) = &tex.data {
                    unimplemented!()
                } else {
                    let path = tex.path.replace("\\\\", "/").replace('\\', "/");
                    resolve_texture(path).await
                };
                if let Some(image) = image {
                    textures.insert(key.clone(), image);
                }
            }
            // for tex in texs {
            //     println!("{:?} {} {:?}", key, tex.data.is_some(), tex);
            // }
        }
        let mut out_material = PbrMaterialDesc {
            // source: todo!(),
            base_color: if let Some(tex) = textures
                .remove(&TextureType::BaseColor)
                .or_else(|| textures.remove(&TextureType::Diffuse))
            {
                Some(dotdot_path(model_crate.images.insert("base_color", tex).path).into())
            } else {
                None
            },
            opacity: textures
                .remove(&TextureType::Opacity)
                .map(|img| dotdot_path(model_crate.images.insert("opacity", img).path).into()),
            // base_color_factor: todo!(),
            // emissive_factor: todo!(),
            normalmap: textures
                .remove(&TextureType::Normals)
                .map(|img| dotdot_path(model_crate.images.insert("normals", img).path).into()),
            // transparent: todo!(),
            // alpha_cutoff: todo!(),
            // double_sided: todo!(),
            metallic_roughness: match (
                textures.remove(&TextureType::Metalness),
                textures.remove(&TextureType::Roughness),
            ) {
                (Some(mut metal), Some(rough)) => {
                    for (m, r) in metal.pixels_mut().zip(rough.pixels()) {
                        m[1] = r[0];
                    }
                    Some(
                        dotdot_path(model_crate.images.insert("metallic_roughness", metal).path)
                            .into(),
                    )
                }
                (Some(mut metal), None) => {
                    for p in metal.pixels_mut() {
                        p[1] = 255;
                    }
                    Some(
                        dotdot_path(model_crate.images.insert("metallic_roughness", metal).path)
                            .into(),
                    )
                }
                (None, Some(mut rough)) => {
                    for p in rough.pixels_mut() {
                        p[0] = 255;
                    }
                    Some(
                        dotdot_path(model_crate.images.insert("metallic_roughness", rough).path)
                            .into(),
                    )
                }
                (None, None) => None,
            },
            // metallic: todo!(),
            // roughness: todo!(),
            ..Default::default()
        };
        for prop in &material.properties {
            #[allow(clippy::single_match)]
            match &prop.key as &str {
                "?mat.name" => {
                    if let PropertyTypeInfo::String(value) = &prop.data {
                        out_material.name = Some(value.clone());
                    }
                }
                _ => {}
            }
            // println!("{} {:?} {:?} {}", prop.key, prop.data, prop.semantic, prop.index);
        }
        model_crate.materials.insert(i.to_string(), out_material);
    }
    Ok(path)
}

#[cfg(feature = "russimp")]
fn import_sync(
    buffer: &[u8],
    model_crate: &mut ModelCrate,
    extension: &str,
) -> anyhow::Result<(RelativePathBuf, Vec<Material>)> {
    use crate::animation_bind_id::{BindIdNodeFuncs, BindIdReg};
    use crate::dotdot_path;
    use ambient_animation::{AnimationClip, AnimationOutputs, AnimationTarget, AnimationTrack};
    use ambient_core::hierarchy::{children, parent};
    use ambient_core::transform::{local_to_parent, local_to_world, rotation, scale, translation};
    use ambient_ecs::generated::animation::components::bind_id;
    use ambient_ecs::{query, Entity, EntityId, World};
    use ambient_model::{
        model_skin_ix, model_skins, pbr_renderer_primitives_from_url, Model, ModelSkin,
        PbrRenderPrimitiveFromUrl,
    };
    use ambient_native_std::mesh::MeshBuilder;
    use glam::*;
    use itertools::Itertools;
    use russimp::animation::Quaternion;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::{cell::RefCell, rc::Rc};

    let mut bind_ids = BindIdReg::<String, Node>::new(BindIdNodeFuncs {
        node_to_id: |node| node.name.clone(),
        node_name: |node| Some(&node.name as &str),
    });

    let scene = Scene::from_buffer(
        buffer,
        vec![
            PostProcess::CalculateTangentSpace,
            PostProcess::JoinIdenticalVertices,
            PostProcess::Triangulate,
            PostProcess::EmbedTextures,
            PostProcess::FlipWindingOrder,
            PostProcess::GenerateUVCoords,
            PostProcess::FlipUVs,
            PostProcess::GenerateNormals,
        ],
        extension,
    )?;
    for (i, mesh) in scene.meshes.iter().enumerate() {
        let positions = mesh
            .vertices
            .iter()
            .map(|v| vec3(v.x, v.y, v.z))
            .collect_vec();
        let colors = if let Some(Some(colors)) = mesh.colors.get(0) {
            colors.iter().map(|c| vec4(c.r, c.g, c.b, c.a)).collect()
        } else {
            Vec::new()
        };
        let normals = mesh
            .normals
            .iter()
            .map(|v| vec3(v.x, v.y, v.z))
            .collect_vec();
        let tangents = mesh
            .tangents
            .iter()
            .map(|v| vec3(v.x, v.y, v.z))
            .collect_vec();
        let texcoords = mesh
            .texture_coords
            .iter()
            .filter_map(|tc| {
                tc.as_ref().map(|tc| {
                    tc.iter()
                        .map(|v| vec2(v.x.rem_euclid(1.), v.y.rem_euclid(1.)))
                        .collect_vec()
                })
            })
            .collect_vec();
        let indices = mesh.faces.iter().flat_map(|f| f.0.clone()).collect_vec();
        let mut out_mesh = MeshBuilder {
            positions,
            colors,
            normals,
            tangents,
            texcoords,
            indices,
            ..MeshBuilder::default()
        };
        if !mesh.bones.is_empty() {
            let mut joint_indices = mesh.vertices.iter().map(|_| Vec::new()).collect_vec();
            let mut joint_weights = mesh.vertices.iter().map(|_| Vec::new()).collect_vec();
            for (i, bone) in mesh.bones.iter().enumerate() {
                for weight in &bone.weights {
                    joint_indices[weight.vertex_id as usize].push(i as u32);
                    joint_weights[weight.vertex_id as usize].push(weight.weight);
                }
            }
            out_mesh.joint_indices = joint_indices
                .into_iter()
                .map(|v| {
                    uvec4(
                        v.get(0).map(|x| *x).unwrap_or_default(),
                        v.get(1).map(|x| *x).unwrap_or_default(),
                        v.get(2).map(|x| *x).unwrap_or_default(),
                        v.get(3).map(|x| *x).unwrap_or_default(),
                    )
                })
                .collect();
            out_mesh.joint_weights = joint_weights
                .into_iter()
                .map(|v| {
                    vec4(
                        v.get(0).map(|x| *x).unwrap_or_default(),
                        v.get(1).map(|x| *x).unwrap_or_default(),
                        v.get(2).map(|x| *x).unwrap_or_default(),
                        v.get(3).map(|x| *x).unwrap_or_default(),
                    )
                })
                .collect();
        }
        let out_mesh = out_mesh.build()?;
        model_crate.meshes.insert(i.to_string(), out_mesh);
    }

    let mut world = World::new("assimp", ambient_ecs::WorldContext::Prefab);

    fn assimp_matrix(t: &russimp::Matrix4x4) -> Mat4 {
        Mat4::from_cols_array(&[
            t.a1, t.a2, t.a3, t.a4, t.b1, t.b2, t.b3, t.b4, t.c1, t.c2, t.c3, t.c4, t.d1, t.d2,
            t.d3, t.d4,
        ])
        .transpose()
    }
    fn assimp_vec3(t: &russimp::Vector3D) -> Vec3 {
        vec3(t.x, t.y, t.z)
    }
    fn assimp_quat(t: &Quaternion) -> Quat {
        quat(t.x, t.y, t.z, t.w)
    }

    fn recursive_build_nodes(
        model_crate: &ModelCrate,
        scene: &Scene,
        world: &mut World,
        bind_ids: &mut BindIdReg<String, Node>,
        node: &Rc<RefCell<Node>>,
    ) -> EntityId {
        let node = node.borrow();

        let transform = assimp_matrix(&node.transformation);
        let (scl, rot, pos) = transform.to_scale_rotation_translation();
        let mut ed = Entity::new()
            .with(ambient_core::name(), node.name.clone())
            .with(translation(), pos)
            .with(rotation(), rot)
            .with(scale(), scl)
            .with(local_to_world(), Default::default())
            .with(bind_id(), bind_ids.get(&*node));
        if !node.meshes.is_empty() {
            ed.set(
                pbr_renderer_primitives_from_url(),
                node.meshes
                    .iter()
                    .flat_map(|mesh_i| {
                        scene
                            .meshes
                            .get(*mesh_i as usize)
                            .map(|mesh| PbrRenderPrimitiveFromUrl {
                                lod: 0,
                                material: Some(
                                    dotdot_path(
                                        model_crate
                                            .materials
                                            .loc
                                            .path(mesh.material_index.to_string()),
                                    )
                                    .into(),
                                ),
                                mesh: dotdot_path(model_crate.meshes.loc.path(mesh_i.to_string()))
                                    .into(),
                            })
                    })
                    .collect(),
            );
        }
        // TODO: This code won't work for multiple skins on a single node
        for mesh_i in &node.meshes {
            if let Some(mesh) = scene.meshes.get(*mesh_i as usize) {
                if !mesh.bones.is_empty() {
                    ed.set(model_skin_ix(), *mesh_i as usize);
                    break;
                }
            }
        }
        let id = ed.spawn(world);
        let childs = node
            .children
            .iter()
            .map(|c| recursive_build_nodes(model_crate, scene, world, bind_ids, c))
            .collect_vec();
        for c in &childs {
            world.add_component(*c, parent(), id).unwrap();
            world
                .add_component(*c, local_to_parent(), Default::default())
                .unwrap();
        }
        world.add_component(id, children(), childs).unwrap();
        id
    }
    if let Some(root) = &scene.root {
        let root = recursive_build_nodes(model_crate, &scene, &mut world, &mut bind_ids, root);
        world.add_resource(children(), vec![root]);
        // world.add_resource(name(), scene.name.to_string());
    }
    let mut skins = Vec::new();
    let bind_id_lookup = query(bind_id())
        .iter(&world, None)
        .map(|(id, b)| (b.clone(), id))
        .collect::<HashMap<_, _>>();
    for mesh in &scene.meshes {
        if !mesh.bones.is_empty() {
            skins.push(ModelSkin {
                inverse_bind_matrices: Arc::new(
                    mesh.bones
                        .iter()
                        .map(|b| assimp_matrix(&b.offset_matrix))
                        .collect(),
                ),
                joints: mesh
                    .bones
                    .iter()
                    .map(|b| {
                        let Some(id) = bind_ids.try_get_by_id(&b.name) else {
                            return EntityId::null();
                        };
                        bind_id_lookup
                            .get(id)
                            .map(|x| *x)
                            .unwrap_or(EntityId::null())
                    })
                    .collect(),
            });
        }
    }
    world.add_resource(model_skins(), skins);
    for animation in &scene.animations {
        let mut tracks = Vec::new();
        for channel in &animation.channels {
            let Some(target) = bind_ids.try_get_by_id(&channel.name) else {
                continue;
            };
            let target = AnimationTarget::BinderId(target.clone());
            if !channel.position_keys.is_empty() {
                tracks.push(AnimationTrack {
                    target: target.clone(),
                    inputs: channel
                        .position_keys
                        .iter()
                        .map(|k| k.time as f32 / animation.ticks_per_second as f32)
                        .collect(),
                    outputs: AnimationOutputs::Vec3 {
                        component: translation(),
                        data: channel
                            .position_keys
                            .iter()
                            .map(|k| assimp_vec3(&k.value))
                            .collect(),
                    },
                });
            }
            if !channel.rotation_keys.is_empty() {
                tracks.push(AnimationTrack {
                    target: target.clone(),
                    inputs: channel
                        .rotation_keys
                        .iter()
                        .map(|k| k.time as f32 / animation.ticks_per_second as f32)
                        .collect(),
                    outputs: AnimationOutputs::Quat {
                        component: rotation(),
                        data: channel
                            .rotation_keys
                            .iter()
                            .map(|k| assimp_quat(&k.value))
                            .collect(),
                    },
                });
            }
            if !channel.scaling_keys.is_empty() {
                tracks.push(AnimationTrack {
                    target: target.clone(),
                    inputs: channel
                        .scaling_keys
                        .iter()
                        .map(|k| k.time as f32 / animation.ticks_per_second as f32)
                        .collect(),
                    outputs: AnimationOutputs::Vec3 {
                        component: scale(),
                        data: channel
                            .scaling_keys
                            .iter()
                            .map(|k| assimp_vec3(&k.value))
                            .collect(),
                    },
                });
            }
        }
        let clip = AnimationClip {
            id: animation.name.clone(),
            tracks,
            start: 0.,
            end: animation.duration as f32 / animation.ticks_per_second as f32,
        };
        model_crate.animations.insert(&animation.name, clip);
    }
    // dump_world_hierarchy_to_tmp_file(&world);
    Ok((
        model_crate
            .models
            .insert(ModelCrate::MAIN, Model(world))
            .path,
        scene.materials.clone(),
    ))
}
