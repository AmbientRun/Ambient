use std::collections::HashMap;

use ambient_core::{
    hierarchy::children,
    name,
    transform::{
        fbx_complex_transform, fbx_post_rotation, fbx_pre_rotation, fbx_rotation_offset,
        fbx_rotation_pivot, fbx_scaling_offset, fbx_scaling_pivot, local_to_parent, local_to_world,
        mesh_to_local, rotation, scale, translation,
    },
};
use ambient_ecs::{Entity, EntityId, World};
use ambient_model::{model_skin_ix, pbr_renderer_primitives_from_url, PbrRenderPrimitiveFromUrl};
use ambient_renderer::double_sided;
use fbxcel::tree::v7400::NodeHandle;
use glam::{vec3, EulerRot, Mat4, Quat, Vec3};
use itertools::Itertools;

use super::FbxDoc;
use crate::{dotdot_path, model_crate::ModelCrate};

#[derive(Debug)]
pub struct FbxModel {
    pub id: i64,
    pub node_name: String,

    translation: Option<Vec3>,
    rotation_offset: Option<Vec3>,
    rotation_pivot: Option<Vec3>,
    pre_rotation: Option<Vec3>,
    rotation: Option<Vec3>,
    post_rotation: Option<Vec3>,
    scaling_offset: Option<Vec3>,
    scaling_pivot: Option<Vec3>,
    scaling: Option<Vec3>,
    geometric_translation: Option<Vec3>,
    geometric_rotation: Option<Vec3>,
    geometric_scale: Option<Vec3>,

    double_sided: bool,

    pub geometries: Vec<i64>,
    pub materials: Vec<i64>,
    pub parent: Option<i64>,
    pub children: Vec<i64>,
    pub is_root: bool,

    pub local_to_parent: Mat4,
    pub local_to_model: Mat4,
    pub geometry_to_model: Mat4,
}
impl FbxModel {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let node_name = node.attributes()[1]
            .get_string()
            .unwrap()
            .split('\u{0}')
            .next()
            .unwrap()
            .to_string();

        let double_sided = node
            .children()
            .find_map(|node| {
                if node.name() == "Culling" {
                    Some(node.attributes()[0].get_string().unwrap() == "CullingOff")
                } else {
                    None
                }
            })
            .unwrap_or(false);

        let mut model = Self {
            id,
            node_name,

            translation: None,
            rotation_offset: None,
            rotation_pivot: None,
            pre_rotation: None,
            rotation: None,
            post_rotation: None,
            scaling_offset: None,
            scaling_pivot: None,
            scaling: None,
            geometric_translation: None,
            geometric_rotation: None,
            geometric_scale: None,

            double_sided,

            geometries: Default::default(),
            materials: Default::default(),
            parent: None,
            children: Default::default(),
            is_root: true,

            local_to_parent: Mat4::IDENTITY,
            local_to_model: Mat4::IDENTITY,
            geometry_to_model: Mat4::IDENTITY,
        };
        let props = node
            .children()
            .find(|node| node.name() == "Properties70")
            .unwrap();
        for prop in props.children() {
            let prop_type = prop.attributes()[0].get_string().unwrap();
            match prop_type {
                "Lcl Translation" => model.translation = Some(prop_vec3(prop)),
                "RotationOffset" => model.rotation_offset = Some(prop_vec3(prop)),
                "RotationPivot" => model.rotation_pivot = Some(prop_vec3(prop)),
                "PreRotation" => model.pre_rotation = Some(prop_rotation(prop)),
                "Lcl Rotation" => model.rotation = Some(prop_rotation(prop)),
                "PostRotation" => model.post_rotation = Some(prop_rotation(prop)),
                "ScalingOffset" => model.scaling_offset = Some(prop_vec3(prop)),
                "ScalingPivot" => model.scaling_pivot = Some(prop_vec3(prop)),
                "Lcl Scaling" => model.scaling = Some(prop_vec3(prop)),
                "GeometricTranslation" => model.geometric_translation = Some(prop_vec3(prop)),
                "GeometricRotation" => model.geometric_rotation = Some(prop_rotation(prop)),
                "GeometricScaling" => model.geometric_scale = Some(prop_vec3(prop)),
                _ => {}
            }
        }
        model
    }
    fn is_complex_transform(&self) -> bool {
        self.rotation_offset.is_some()
            || self.rotation_pivot.is_some()
            || self.pre_rotation.is_some()
            || self.post_rotation.is_some()
            || self.scaling_offset.is_some()
            || self.scaling_pivot.is_some()
    }
    pub fn create_model_nodes(
        &self,
        doc: &FbxDoc,
        world: &mut World,
        entities: &mut HashMap<i64, EntityId>,
        asset_crate: &ModelCrate,
        n_meshes: &HashMap<i64, usize>,
    ) {
        let mut out_node = Entity::new()
            .with(name(), self.node_name.to_string())
            .with_default(children());
        if self.double_sided {
            out_node.set(double_sided(), true);
        }
        if self.is_complex_transform() {
            out_node.set(fbx_complex_transform(), ());
            out_node.set(fbx_rotation_offset(), Vec3::ZERO);
            out_node.set(fbx_rotation_pivot(), Vec3::ZERO);
            out_node.set(fbx_pre_rotation(), Quat::IDENTITY);
            out_node.set(fbx_post_rotation(), Quat::IDENTITY);
            out_node.set(fbx_scaling_offset(), Vec3::ZERO);
            out_node.set(fbx_scaling_pivot(), Vec3::ZERO);
            out_node.set(translation(), Vec3::ZERO);
            out_node.set(rotation(), Quat::IDENTITY);
            out_node.set(scale(), Vec3::ONE);
        }

        if let Some(value) = self.rotation_offset {
            out_node.set(fbx_rotation_offset(), value);
        }
        if let Some(value) = self.rotation_pivot {
            out_node.set(fbx_rotation_pivot(), value);
        }
        if let Some(value) = self
            .pre_rotation
            .map(|r| Quat::from_euler(EulerRot::ZYX, r.z, r.y, r.x))
        {
            out_node.set(fbx_pre_rotation(), value);
        }
        if let Some(value) = self
            .post_rotation
            .map(|r| Quat::from_euler(EulerRot::ZYX, r.z, r.y, r.x))
        {
            out_node.set(fbx_post_rotation(), value);
        }
        if let Some(value) = self.scaling_offset {
            out_node.set(fbx_scaling_offset(), value);
        }
        if let Some(value) = self.scaling_pivot {
            out_node.set(fbx_scaling_pivot(), value);
        }

        // Need to give these values since they might be animated
        out_node.set(translation(), self.translation.unwrap_or(Vec3::ZERO));
        out_node.set(
            rotation(),
            self.rotation
                .or(Some(Vec3::ZERO))
                .map(|r| Quat::from_euler(EulerRot::ZYX, r.z, r.y, r.x))
                .unwrap(),
        );
        out_node.set(scale(), self.scaling.unwrap_or(Vec3::ONE));
        out_node.set(local_to_world(), Default::default());
        if self.parent.is_some() {
            out_node.set(local_to_parent(), Default::default());
        }

        if !self.geometries.is_empty() {
            assert_eq!(
                self.geometries.len(),
                1,
                "FbxNodes are currently expected to only have one FbxGeometry"
            );
            let geo = self.geometries[0];
            if let Some(n_meshes) = n_meshes.get(&geo) {
                let prims = (0..*n_meshes)
                    .map(|mi| PbrRenderPrimitiveFromUrl {
                        mesh: dotdot_path(asset_crate.meshes.loc.path(format!("{geo}_{mi}")))
                            .into(),
                        material: self.materials.get(mi).map(|x| {
                            dotdot_path(asset_crate.materials.loc.path(x.to_string())).into()
                        }),
                        lod: 0,
                    })
                    .collect_vec();
                out_node.set(pbr_renderer_primitives_from_url(), prims);
            }
            if let Some(skin) = doc
                .geometries
                .get(&geo)
                .and_then(|geo| geo.skin)
                .and_then(|id| doc.skins.get_index_of(&id))
            {
                out_node.set(model_skin_ix(), skin);
            }
            if self.geometric_translation.is_some()
                || self.geometric_rotation.is_some()
                || self.geometric_scale.is_some()
            {
                out_node.set(
                    mesh_to_local(),
                    Mat4::from_scale_rotation_translation(
                        self.geometric_scale.unwrap_or(Vec3::ONE),
                        self.geometric_rotation
                            .map(|rot| Quat::from_euler(EulerRot::XYZ, rot.x, rot.y, rot.z))
                            .unwrap_or(Quat::IDENTITY),
                        self.geometric_translation.unwrap_or(Vec3::ZERO),
                    ),
                );
            }
        }

        entities.insert(self.id, out_node.spawn(world));
    }
}
fn prop_vec3(prop: NodeHandle) -> Vec3 {
    vec3(
        prop.attributes()[4].get_f64().unwrap() as f32,
        prop.attributes()[5].get_f64().unwrap() as f32,
        prop.attributes()[6].get_f64().unwrap() as f32,
    )
}
fn prop_rotation(prop: NodeHandle) -> Vec3 {
    vec3(
        (prop.attributes()[4].get_f64().unwrap() as f32).to_radians(),
        (prop.attributes()[5].get_f64().unwrap() as f32).to_radians(),
        (prop.attributes()[6].get_f64().unwrap() as f32).to_radians(),
    )
}
