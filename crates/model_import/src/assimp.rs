use std::{cell::RefCell, collections::HashMap, rc::Rc};

use elements_core::{
    hierarchy::{children, dump_world_hierarchy_to_tmp_file, parent}, name, transform::{local_to_parent, local_to_world, rotation, scale, translation}
};
use elements_ecs::{EntityData, EntityId, World};
use elements_model::{pbr_renderer_primitives_from_url, Model, PbrRenderPrimitiveFromUrl};
use elements_renderer::materials::pbr_material::PbrMaterialFromUrl;
use elements_std::{asset_cache::AssetCache, download_asset::ContentLoc, mesh::Mesh};
use glam::{vec2, vec3, vec4, Mat4};

use itertools::Itertools;

use russimp::{
    material::{Material, PropertyTypeInfo}, node::Node, scene::{PostProcess, Scene}, texture::TextureType
};

use crate::{download_bytes, model_crate::ModelCrate, TextureResolver};

pub async fn import_url(
    assets: &AssetCache,
    url: &str,
    model_crate: &mut ModelCrate,
    resolve_texture: TextureResolver,
) -> anyhow::Result<String> {
    let content = download_bytes(assets, url).await?;
    let extension = match ContentLoc::parse(url) {
        Ok(ContentLoc::RelativePath(path)) => path.extension().map(|x| x.to_str().unwrap().to_string()),
        Ok(ContentLoc::Url(url)) => url.path().rsplit_once(".").map(|(_a, b)| b.to_string()),
        Err(_) => None,
    }
    .unwrap_or_default();
    import(&content, model_crate, &extension, resolve_texture).await
}

pub async fn import<'a>(
    buffer: &'a [u8],
    model_crate: &'a mut ModelCrate,
    extension: &'a str,
    resolve_texture: TextureResolver,
) -> anyhow::Result<String> {
    let (url, materials) = import_sync(buffer, model_crate, extension)?;
    for (i, material) in materials.iter().enumerate() {
        let mut textures = HashMap::new();
        for (key, texs) in &material.textures {
            if let Some(tex) = texs.get(0) {
                let image = if let Some(_data) = &tex.data {
                    todo!()
                } else {
                    let path = tex.path.replace("\\\\", "/").replace("\\", "/");
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
        let mut out_material = PbrMaterialFromUrl {
            // source: todo!(),
            base_color: if let Some(tex) = textures.remove(&TextureType::BaseColor).or_else(|| textures.remove(&TextureType::Diffuse)) {
                Some(model_crate.images.insert("base_color", tex).url.into())
            } else {
                None
            },
            opacity: textures.remove(&TextureType::Opacity).map(|img| model_crate.images.insert("opacity", img).url.into()),
            // base_color_factor: todo!(),
            // emissive_factor: todo!(),
            normalmap: textures.remove(&TextureType::Normals).map(|img| model_crate.images.insert("normals", img).url.into()),
            // transparent: todo!(),
            // alpha_cutoff: todo!(),
            // double_sided: todo!(),
            metallic_roughness: match (textures.remove(&TextureType::Metalness), textures.remove(&TextureType::Roughness)) {
                (Some(mut metal), Some(rough)) => {
                    for (m, r) in metal.pixels_mut().zip(rough.pixels()) {
                        m[1] = r[0];
                    }
                    Some(model_crate.images.insert("metallic_roughness", metal).url.into())
                }
                (Some(mut metal), None) => {
                    for p in metal.pixels_mut() {
                        p[1] = 255;
                    }
                    Some(model_crate.images.insert("metallic_roughness", metal).url.into())
                }
                (None, Some(mut rough)) => {
                    for p in rough.pixels_mut() {
                        p[0] = 255;
                    }
                    Some(model_crate.images.insert("metallic_roughness", rough).url.into())
                }
                (None, None) => None,
            },
            // metallic: todo!(),
            // roughness: todo!(),
            ..Default::default()
        };
        for prop in &material.properties {
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
    Ok(url)
}

fn import_sync(buffer: &[u8], model_crate: &mut ModelCrate, extension: &str) -> anyhow::Result<(String, Vec<Material>)> {
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
        ],
        extension,
    )?;
    for (i, mesh) in scene.meshes.iter().enumerate() {
        let out_mesh = Mesh {
            name: mesh.name.clone(),
            positions: Some(mesh.vertices.iter().map(|v| vec3(v.x, v.y, v.z)).collect()),
            colors: if let Some(Some(colors)) = mesh.colors.get(0) {
                Some(colors.iter().map(|c| vec4(c.r, c.g, c.b, c.a)).collect())
            } else {
                None
            },
            normals: Some(mesh.normals.iter().map(|v| vec3(v.x, v.y, v.z)).collect()),
            tangents: Some(mesh.tangents.iter().map(|v| vec3(v.x, v.y, v.z)).collect()),
            texcoords: mesh
                .texture_coords
                .iter()
                .map(|tc| tc.as_ref().map(|tc| tc.iter().map(|v| vec2(v.x, v.y)).collect()).unwrap_or_default())
                .collect(),
            // TODO(fred): Bones
            joint_indices: None,
            joint_weights: None,
            indices: Some(mesh.faces.iter().flat_map(|f| f.0.clone()).collect()),
        };
        model_crate.meshes.insert(i.to_string(), out_mesh);
    }

    let mut world = World::new("assimp");
    fn recursive_build_nodes(model_crate: &ModelCrate, scene: &Scene, world: &mut World, node: &Rc<RefCell<Node>>) -> EntityId {
        let node = node.borrow();

        let t = &node.transformation;
        let transform =
            Mat4::from_cols_array(&[t.a1, t.a2, t.a3, t.a4, t.b1, t.b2, t.b3, t.b4, t.c1, t.c2, t.c3, t.c4, t.d1, t.d2, t.d3, t.d4])
                .transpose();
        let (scl, rot, pos) = transform.to_scale_rotation_translation();
        let mut ed = EntityData::new()
            .set(name(), node.name.clone())
            .set(translation(), pos)
            .set(rotation(), rot)
            .set(scale(), scl)
            .set_default(local_to_world());
        if !node.meshes.is_empty() {
            ed.set_self(
                pbr_renderer_primitives_from_url(),
                node.meshes
                    .iter()
                    .flat_map(|mesh_i| {
                        scene.meshes.get(*mesh_i as usize).map(|mesh| PbrRenderPrimitiveFromUrl {
                            lod: 0,
                            material: Some(model_crate.materials.loc.url(mesh.material_index.to_string())),
                            mesh: model_crate.meshes.loc.url(mesh_i.to_string()),
                        })
                    })
                    .collect(),
            );
        }
        let id = ed.spawn(world);
        let childs = node.children.iter().map(|c| recursive_build_nodes(model_crate, scene, world, c)).collect_vec();
        for c in &childs {
            world.add_component(*c, parent(), id).unwrap();
            world.add_component(*c, local_to_parent(), Default::default()).unwrap();
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
    Ok((model_crate.models.insert(ModelCrate::MAIN, Model(world)).url, scene.materials.clone()))
}
