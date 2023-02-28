use std::sync::Arc;

use ambient_animation::{animation_bind_id_from_name, AnimationClip, AnimationOutputs, AnimationTarget, AnimationTrack};
use ambient_core::{
    bounding::local_bounding_aabb,
    hierarchy::{children, parent},
    name,
    transform::{local_to_parent, local_to_world, rotation, scale, translation},
};
use ambient_ecs::{Entity, World};
use ambient_model::{model_skin_ix, model_skins, pbr_renderer_primitives_from_url, Model, ModelSkin, PbrRenderPrimitiveFromUrl};
use ambient_renderer::materials::pbr_material::PbrMaterialFromUrl;
use ambient_std::{asset_cache::AssetCache, asset_url::AbsAssetUrl, mesh::Mesh, shapes::AABB};
use glam::{uvec4, Mat4, Quat, UVec4, Vec2, Vec3, Vec4, Vec4Swizzles};
use gltf::animation::util::ReadOutputs;
use itertools::Itertools;
use relative_path::RelativePathBuf;

use self::gltf_import::GltfImport;
use crate::{dotdot_path, model_crate::ModelCrate};

mod gltf_import;

pub async fn import_url(assets: &AssetCache, url: &AbsAssetUrl, asset_crate: &mut ModelCrate) -> anyhow::Result<RelativePathBuf> {
    let content = url.download_bytes(assets).await?;
    let gltf = GltfImport::from_slice(url.to_string(), true, &content)?;
    import(&gltf, asset_crate).await
}

pub async fn import(import: &GltfImport, asset_crate: &mut ModelCrate) -> anyhow::Result<RelativePathBuf> {
    let name_ = |name: Option<&str>| name.map(|x| format!("{x}_")).unwrap_or_default();

    let mut meshes = import.document.meshes().map(|mesh| mesh.primitives().map(|_| RelativePathBuf::new()).collect_vec()).collect_vec();
    for (mesh_i, mesh) in import.document.meshes().enumerate() {
        for (prim_i, primitive) in mesh.primitives().enumerate() {
            let reader = primitive.reader(|buffer| Some(&import.buffers[buffer.index()]));

            let mut texcoords = Vec::new();
            while let Some(tc) = reader.read_tex_coords(texcoords.len() as u32) {
                texcoords.push(tc.into_f32().map(|x| x.into()).collect::<Vec<Vec2>>());
            }

            let flip_indices = true;
            let mut cpu_mesh = Mesh {
                name: format!("{}:{}:{}", import.name, mesh.index(), primitive.index()),
                positions: reader.read_positions().map(|v| v.map(|x| x.into()).collect::<Vec<Vec3>>()),
                normals: reader.read_normals().map(|v| v.map(|x| x.into()).collect::<Vec<Vec3>>()),
                tangents: reader.read_tangents().map(|v| v.map(|x| Vec4::from(x).xyz()).collect::<Vec<Vec3>>()),
                texcoords,
                colors: None,
                joint_indices: reader
                    .read_joints(0)
                    .map(|v| v.into_u16().map(|v| uvec4(v[0] as u32, v[1] as u32, v[2] as u32, v[3] as u32)).collect::<Vec<UVec4>>()),
                joint_weights: reader.read_weights(0).map(|v| v.into_f32().map(|x| x.into()).collect::<Vec<Vec4>>()),
                indices: reader.read_indices().map(|v| {
                    if flip_indices {
                        v.into_u32()
                            .chunks(3)
                            .into_iter()
                            .flat_map(|chunk| {
                                let mut chunk = chunk.collect_vec();
                                chunk.swap(1, 2);
                                chunk
                            })
                            .collect::<Vec<u32>>()
                    } else {
                        v.into_u32().collect::<Vec<u32>>()
                    }
                }),
            };
            cpu_mesh.try_ensure_tangents();
            let path = asset_crate.meshes.insert(&format!("{}{}_{}", name_(mesh.name()), mesh.index(), primitive.index()), cpu_mesh).path;
            meshes[mesh_i][prim_i] = path;
        }
    }

    for (index, animation) in import.document.animations().into_iter().enumerate() {
        let tracks = animation
            .channels()
            .into_iter()
            .map(|channel| {
                let reader = channel.reader(|buffer| Some(&import.buffers[buffer.index()]));
                let target = AnimationTarget::BinderId(animation_bind_id_from_name(channel.target().node().name().unwrap_or("")));
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
                            data: data.into_f32().into_iter().map(|v| Quat::from_slice(&v)).collect(),
                        },
                    },
                    _ => unimplemented!(),
                }
            })
            .collect();
        let mut animation_clip = AnimationClip::from_tracks(tracks);
        animation_clip.id = animation.name().unwrap_or("").to_string();
        asset_crate.animations.insert(&format!("{}{}", name_(animation.name()), index), animation_clip);
    }

    let mut images = Vec::new();
    for (index, image) in import.images.iter().enumerate() {
        let mut img = match image.format {
            gltf::image::Format::R8G8B8A8 => image::RgbaImage::from_raw(image.width, image.height, image.pixels.clone()).unwrap(),
            gltf::image::Format::R8G8B8 => {
                let img = image::RgbImage::from_raw(image.width, image.height, image.pixels.clone()).unwrap();
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
                let img = image::GrayImage::from_raw(image.width, image.height, image.pixels.clone()).unwrap();
                image::DynamicImage::ImageLuma8(img).into_rgba8()
            }
            gltf::image::Format::R8G8 => {
                let img = image::GrayAlphaImage::from_raw(image.width, image.height, image.pixels.clone()).unwrap();
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

        let mat_def = PbrMaterialFromUrl {
            name: mat.name().map(|x| x.to_string()),
            source: Some(import.name.clone()),
            base_color_factor: Some(glam::Vec4::from_slice(&mat.pbr_metallic_roughness().base_color_factor())),
            emissive_factor: Some(glam::Vec3::from_slice(&mat.emissive_factor()).extend(0.)),
            transparent: Some(mat.alpha_mode() == gltf::material::AlphaMode::Blend),
            alpha_cutoff: mat.alpha_cutoff(),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            base_color: pbr.base_color_texture().and_then(|x| images.get(x.texture().index())).map(|x| dotdot_path(x).into()),
            normalmap: mat.normal_texture().and_then(|x| images.get(x.texture().index())).map(|x| dotdot_path(x).into()),
            metallic_roughness: pbr
                .metallic_roughness_texture()
                .and_then(|x| images.get(x.texture().index()))
                .map(|x| dotdot_path(x).into()),
            double_sided: Some(mat.double_sided()),
            opacity: None,
        };
        materials.push(asset_crate.materials.insert(&format!("{}{}", name_(mat.name()), index), mat_def).path);
    }

    let mut world = World::new("gltf");
    let nodes = import
        .document
        .nodes()
        .map(|node| {
            let (trans, rot, scal) = node.transform().decomposed();

            let mut ed = Entity::new()
                .set(translation(), Vec3::from_slice(&trans))
                .set(rotation(), Quat::from_slice(&rot))
                .set(scale(), Vec3::from_slice(&scal))
                .set_default(local_to_world());

            if let Some(node_name) = node.name() {
                ed.set_self(name(), node_name.to_string());
            }

            if let Some(mesh_) = node.mesh() {
                let primitive_defs = mesh_
                    .primitives()
                    .map(|primitive| PbrRenderPrimitiveFromUrl {
                        mesh: dotdot_path(&meshes[mesh_.index()][primitive.index()]).into(),
                        material: primitive.material().index().map(|material_index| dotdot_path(&materials[material_index]).into()),
                        lod: 0,
                    })
                    .collect_vec();
                ed.set_self(pbr_renderer_primitives_from_url(), primitive_defs);

                let aabbs = mesh_
                    .primitives()
                    .map(|primitive| {
                        let bb = primitive.bounding_box();
                        AABB { min: bb.min.into(), max: bb.max.into() }
                    })
                    .collect_vec();
                if let Some(aabb) = AABB::unions(&aabbs) {
                    ed.set_self(local_bounding_aabb(), aabb);
                }
            }

            if let Some(skin) = node.skin() {
                ed.set_self(model_skin_ix(), skin.index());
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
                world.add_component(*child_id, local_to_parent(), Default::default()).unwrap();
            }
            world.add_component(*id, children(), childs).unwrap();
        }
    }
    let roots = import.document.scenes().flat_map(|s| s.nodes().map(|x| nodes[x.index()])).collect_vec();
    world.add_resource(children(), roots);
    world.add_resource(name(), import.name.to_string());

    Ok(asset_crate.models.insert(ModelCrate::MAIN, Model(world)).path)
}
