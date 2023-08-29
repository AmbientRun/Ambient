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
    use crate::dotdot_path;
    use ambient_core::hierarchy::{children, dump_world_hierarchy_to_tmp_file, parent};
    use ambient_core::transform::{local_to_parent, local_to_world, rotation, scale, translation};
    use ambient_ecs::{Entity, EntityId, World};
    use ambient_model::{pbr_renderer_primitives_from_url, Model, PbrRenderPrimitiveFromUrl};
    use ambient_native_std::mesh::MeshBuilder;
    use glam::*;
    use itertools::Itertools;
    use std::{cell::RefCell, rc::Rc};

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
        let out_mesh = MeshBuilder {
            positions,
            colors,
            normals,
            tangents,
            texcoords,
            indices,
            ..MeshBuilder::default()
        }
        .build()?;
        model_crate.meshes.insert(i.to_string(), out_mesh);
    }

    let mut world = World::new("assimp", ambient_ecs::WorldContext::Prefab);
    fn recursive_build_nodes(
        model_crate: &ModelCrate,
        scene: &Scene,
        world: &mut World,
        node: &Rc<RefCell<Node>>,
    ) -> EntityId {
        let node = node.borrow();

        let t = &node.transformation;
        let transform = Mat4::from_cols_array(&[
            t.a1, t.a2, t.a3, t.a4, t.b1, t.b2, t.b3, t.b4, t.c1, t.c2, t.c3, t.c4, t.d1, t.d2,
            t.d3, t.d4,
        ])
        .transpose();
        let (scl, rot, pos) = transform.to_scale_rotation_translation();
        let mut ed = Entity::new()
            .with(ambient_core::name(), node.name.clone())
            .with(translation(), pos)
            .with(rotation(), rot)
            .with(scale(), scl)
            .with(local_to_world(), Default::default());
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
        let id = ed.spawn(world);
        let childs = node
            .children
            .iter()
            .map(|c| recursive_build_nodes(model_crate, scene, world, c))
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
        let root = recursive_build_nodes(model_crate, &scene, &mut world, root);
        world.add_resource(children(), vec![root]);
        // world.add_resource(name(), scene.name.to_string());
    }
    dump_world_hierarchy_to_tmp_file(&world);
    Ok((
        model_crate
            .models
            .insert(ModelCrate::MAIN, Model(world))
            .path,
        scene.materials.clone(),
    ))
}
