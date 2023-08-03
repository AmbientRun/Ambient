use std::sync::Arc;

use ambient_animation::{AnimationClip, AnimationOutputs, AnimationTarget, AnimationTrack};
use ambient_core::{
    bounding::local_bounding_aabb,
    hierarchy::{children, parent},
    name,
    transform::{local_to_parent, local_to_world, rotation, scale, translation},
};
use ambient_ecs::{generated::components::core::animation::bind_id, Entity, World};
use ambient_gpu::sampler::SamplerKey;
use ambient_model::{
    model_skin_ix, model_skins, pbr_renderer_primitives_from_url, Model, ModelSkin,
    PbrRenderPrimitiveFromUrl,
};
use ambient_native_std::{
    asset_cache::AssetCache,
    asset_url::AbsAssetUrl,
    mesh::{flip_winding, generate_tangents, MeshBuilder},
    shapes::AABB,
};
use ambient_renderer::materials::pbr_material::PbrMaterialDesc;
use anyhow::Context;
use glam::{uvec4, Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
use gltf::animation::util::ReadOutputs;
use itertools::Itertools;
use relative_path::RelativePathBuf;

use self::gltf_import::GltfImport;
use crate::{
    animation_bind_id::{BindIdNodeFuncs, BindIdReg},
    dotdot_path,
    model_crate::ModelCrate,
};

mod gltf_import;

pub async fn import_url(
    assets: &AssetCache,
    url: &AbsAssetUrl,
    asset_crate: &mut ModelCrate,
) -> anyhow::Result<RelativePathBuf> {
    let content = url.download_bytes(assets).await?;
    let gltf = GltfImport::from_slice(url.to_string(), true, &content)?;
    import(&gltf, asset_crate).await
}

pub async fn import(
    import: &GltfImport,
    asset_crate: &mut ModelCrate,
) -> anyhow::Result<RelativePathBuf> {
    let name_ = |name: Option<&str>| {
        name.map(|x| format!("{}_", x.replace(['/', '\\'], "-")))
            .unwrap_or_default()
    };
    let mut bind_ids = BindIdReg::<usize, gltf::scene::Node<'_>>::new(BindIdNodeFuncs {
        node_to_id: |node| node.index(),
        node_name: |node| node.name(),
    });

    let mut meshes = import
        .document
        .meshes()
        .map(|mesh| {
            mesh.primitives()
                .map(|_| RelativePathBuf::new())
                .collect_vec()
        })
        .collect_vec();
    for (mesh_i, mesh) in import.document.meshes().enumerate() {
        for (prim_i, primitive) in mesh.primitives().enumerate() {
            let reader = primitive.reader(|buffer| Some(&import.buffers[buffer.index()]));

            let mut texcoords = Vec::new();
            while let Some(tc) = reader.read_tex_coords(texcoords.len() as u32) {
                texcoords.push(tc.into_f32().map(|x| x.into()).collect::<Vec<Vec2>>());
            }

            let positions = reader
                .read_positions()
                .context("GLTF mesh must contain vertex positions")?
                .map(Vec3::from)
                .collect::<Vec<Vec3>>();

            let mut indices = reader
                .read_indices()
                .context("GLTF mesh must contain an index buffer")?
                .into_u32()
                .collect_vec();
            flip_winding(&mut indices);

            let normals = if let Some(normals) = reader.read_normals() {
                normals.into_iter().map(Vec3::from).collect_vec()
            } else {
                Vec::new()
            };

            let mut tangents = if let Some(tangents) = reader.read_tangents() {
                tangents
                    .into_iter()
                    .map(|x| Vec4::from(x).xyz())
                    .collect_vec()
            } else {
                Vec::new()
            };
            if tangents.is_empty()
                && texcoords.get(0).map(|t| !t.is_empty()).unwrap_or_default()
                && !normals.is_empty()
            {
                tangents = generate_tangents(&positions, &texcoords[0], &normals, &indices);
            }

            let joint_indices = if let Some(joints) = reader.read_joints(0) {
                joints
                    .into_u16()
                    .map(|x| uvec4(x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32))
                    .collect_vec()
            } else {
                Vec::new()
            };

            let joint_weights = if let Some(weights) = reader.read_weights(0) {
                weights.into_f32().map(Vec4::from).collect_vec()
            } else {
                Vec::new()
            };

            let cpu_mesh = MeshBuilder {
                positions,
                normals,
                tangents,
                texcoords,
                indices,
                joint_indices,
                joint_weights,
                ..MeshBuilder::default()
            }
            .build()?;

            let path = asset_crate
                .meshes
                .insert(
                    &format!(
                        "{}{}_{}",
                        name_(mesh.name()),
                        mesh.index(),
                        primitive.index()
                    ),
                    cpu_mesh,
                )
                .path;
            meshes[mesh_i][prim_i] = path;
        }
    }

    for (index, animation) in import.document.animations().enumerate() {
        let tracks = animation
            .channels()
            .map(|channel| {
                let reader = channel.reader(|buffer| Some(&import.buffers[buffer.index()]));
                let target = AnimationTarget::BinderId(bind_ids.get(&channel.target().node()));
                let inputs = reader.read_inputs().unwrap().collect();
                match reader.read_outputs() {
                    Some(ReadOutputs::Translations(data)) => AnimationTrack {
                        target,
                        inputs,
                        outputs: AnimationOutputs::Vec3 {
                            component: translation(),
                            data: data.into_iter().map(|v| Vec3::from_slice(&v)).collect(),
                        },
                    },
                    Some(ReadOutputs::Scales(data)) => AnimationTrack {
                        target,
                        inputs,
                        outputs: AnimationOutputs::Vec3 {
                            component: scale(),
                            data: data.into_iter().map(|v| Vec3::from_slice(&v)).collect(),
                        },
                    },
                    Some(ReadOutputs::Rotations(data)) => AnimationTrack {
                        target,
                        inputs,
                        outputs: AnimationOutputs::Quat {
                            component: rotation(),
                            data: data.into_f32().map(|v| Quat::from_slice(&v)).collect(),
                        },
                    },
                    _ => unimplemented!(),
                }
            })
            .collect();
        let mut animation_clip = AnimationClip::from_tracks(tracks);
        animation_clip.id = animation.name().unwrap_or("").to_string();
        asset_crate.animations.insert(
            &format!("{}{}", name_(animation.name()), index),
            animation_clip,
        );
    }

    let mut images = Vec::new();
    for (index, image) in import.images.iter().enumerate() {
        let mut img = match image.format {
            gltf::image::Format::R8G8B8A8 => {
                image::RgbaImage::from_raw(image.width, image.height, image.pixels.clone()).unwrap()
            }
            gltf::image::Format::R8G8B8 => {
                let img =
                    image::RgbImage::from_raw(image.width, image.height, image.pixels.clone())
                        .unwrap();
                image::DynamicImage::ImageRgb8(img).into_rgba8()
            }
            gltf::image::Format::R16G16B16A16 => {
                let img = image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::from_raw(
                    image.width,
                    image.height,
                    bytemuck::cast_slice(&image.pixels).to_vec(),
                )
                .unwrap();
                image::DynamicImage::ImageRgba16(img).into_rgba8()
            }
            gltf::image::Format::R8 => {
                let img =
                    image::GrayImage::from_raw(image.width, image.height, image.pixels.clone())
                        .unwrap();
                image::DynamicImage::ImageLuma8(img).into_rgba8()
            }
            gltf::image::Format::R8G8 => {
                let img = image::GrayAlphaImage::from_raw(
                    image.width,
                    image.height,
                    image.pixels.clone(),
                )
                .unwrap();
                image::DynamicImage::ImageLumaA8(img).into_rgba8()
            }
            gltf::image::Format::R16 => unimplemented!(),
            gltf::image::Format::R16G16 => unimplemented!(),
            gltf::image::Format::R16G16B16 => {
                let img = image::ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(
                    image.width,
                    image.height,
                    bytemuck::cast_slice(&image.pixels).to_vec(),
                )
                .unwrap();
                image::DynamicImage::ImageRgb16(img).into_rgba8()
            }
            gltf::image::Format::R32G32B32FLOAT => unimplemented!(),
            gltf::image::Format::R32G32B32A32FLOAT => unimplemented!(),
        };
        let mut is_mr = false;
        for mat in import.document.materials() {
            if let Some(mr) = mat.pbr_metallic_roughness().metallic_roughness_texture() {
                if mr.texture().index() == index {
                    is_mr = true;
                    break;
                }
            }
        }
        if is_mr {
            for p in img.pixels_mut() {
                p[0] = p[2];
                p[2] = 0;
                p[3] = 255;
            }
        }
        let path = asset_crate.images.insert(&format!("{index}"), img).path;
        images.push(path);
    }

    let mut materials = Vec::new();
    for (index, mat) in import.document.materials().enumerate() {
        let pbr = mat.pbr_metallic_roughness();

        let mat_def = PbrMaterialDesc {
            name: mat.name().map(|x| x.to_string()),
            source: Some(import.name.clone()),
            base_color_factor: Some(glam::Vec4::from_slice(
                &mat.pbr_metallic_roughness().base_color_factor(),
            )),
            emissive_factor: Some(glam::Vec3::from_slice(&mat.emissive_factor()).extend(0.)),
            transparent: Some(mat.alpha_mode() == gltf::material::AlphaMode::Blend),
            alpha_cutoff: mat.alpha_cutoff(),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            base_color: pbr
                .base_color_texture()
                .and_then(|x| images.get(x.texture().index()))
                .map(|x| dotdot_path(x).into()),
            normalmap: mat
                .normal_texture()
                .and_then(|x| images.get(x.texture().index()))
                .map(|x| dotdot_path(x).into()),
            metallic_roughness: pbr
                .metallic_roughness_texture()
                .and_then(|x| images.get(x.texture().index()))
                .map(|x| dotdot_path(x).into()),
            double_sided: Some(mat.double_sided()),
            opacity: None,
            // TODO: Each GLTF texture knows its sampler modes, but Ambient's
            // current material model assumes a single sampler for all textures
            // in a material. Revisit once the renderer supports arbitrary
            // texture-sampler pairs.
            sampler: Some(SamplerKey::LINEAR_CLAMP_TO_EDGE),
        };
        materials.push(
            asset_crate
                .materials
                .insert(&format!("{}{}", name_(mat.name()), index), mat_def)
                .path,
        );
    }

    let mut world = World::new("gltf");
    let nodes = import
        .document
        .nodes()
        .map(|node| {
            let (trans, rot, scal) = node.transform().decomposed();

            let mut ed = Entity::new()
                .with(translation(), Vec3::from_slice(&trans))
                .with(rotation(), Quat::from_slice(&rot))
                .with(scale(), Vec3::from_slice(&scal))
                .with_default(local_to_world())
                .with(bind_id(), bind_ids.get(&node));

            if let Some(node_name) = node.name() {
                ed.set(name(), node_name.to_string());
            }

            if let Some(mesh_) = node.mesh() {
                let primitive_defs = mesh_
                    .primitives()
                    .map(|primitive| PbrRenderPrimitiveFromUrl {
                        mesh: dotdot_path(&meshes[mesh_.index()][primitive.index()]).into(),
                        material: primitive
                            .material()
                            .index()
                            .map(|material_index| dotdot_path(&materials[material_index]).into()),
                        lod: 0,
                    })
                    .collect_vec();
                ed.set(pbr_renderer_primitives_from_url(), primitive_defs);

                let aabbs = mesh_
                    .primitives()
                    .map(|primitive| {
                        let bb = primitive.bounding_box();
                        AABB {
                            min: bb.min.into(),
                            max: bb.max.into(),
                        }
                    })
                    .collect_vec();
                if let Some(aabb) = AABB::unions(&aabbs) {
                    ed.set(local_bounding_aabb(), aabb);
                }
            }

            if let Some(skin) = node.skin() {
                ed.set(model_skin_ix(), skin.index());
            }

            ed.spawn(&mut world)
        })
        .collect_vec();

    let mut skins = Vec::new();
    for skin in import.document.skins() {
        let r = skin.reader(|buffer| Some(&import.buffers[buffer.index()]));
        skins.push(ModelSkin {
            inverse_bind_matrices: Arc::new(if let Some(m) = r.read_inverse_bind_matrices() {
                m.map(|x| Mat4::from_cols_array_2d(&x)).collect()
            } else {
                Vec::new()
            }),
            joints: skin.joints().map(|j| nodes[j.index()]).collect(),
        });
    }
    world.add_resource(model_skins(), skins);

    for (id, node) in nodes.iter().zip(import.document.nodes()) {
        let childs = node.children().map(|x| nodes[x.index()]).collect_vec();
        if !childs.is_empty() {
            for child_id in &childs {
                world.add_component(*child_id, parent(), *id).unwrap();
                world
                    .add_component(*child_id, local_to_parent(), Default::default())
                    .unwrap();
            }
            world.add_component(*id, children(), childs).unwrap();
        }
    }
    let roots = import
        .document
        .scenes()
        .flat_map(|s| s.nodes().map(|x| nodes[x.index()]))
        .collect_vec();
    world.add_resource(children(), roots);
    world.add_resource(name(), import.name.to_string());

    Ok(asset_crate
        .models
        .insert(ModelCrate::MAIN, Model(world))
        .path)
}
