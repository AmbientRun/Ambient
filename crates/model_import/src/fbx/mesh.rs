use std::collections::HashMap;

use elements_std::mesh::Mesh;
use fbxcel::tree::v7400::NodeHandle;
use glam::{uvec4, vec2, vec3, vec4, Mat4, Vec2, Vec3};
use indexmap::IndexMap;
use itertools::Itertools;

use super::{read_matrix, FbxGlobalSettings};

#[derive(PartialEq, Eq, Debug)]
enum FbxMappingInformationType {
    ByPolygonVertex,
    ByVertex,
    ByPolygon,
    AllSame,
}
impl FbxMappingInformationType {
    fn from_node(container_node: NodeHandle) -> Self {
        match container_node.children().find(|node| node.name() == "MappingInformationType").unwrap().attributes()[0].get_string().unwrap()
        {
            "ByPolygonVertex" => Self::ByPolygonVertex,
            "ByVertice" => Self::ByVertex,
            "ByVertex" => Self::ByVertex,
            "ByPolygon" => Self::ByPolygon,
            "AllSame" => Self::AllSame,
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
enum FbxReferenceInformationType {
    Direct,
    IndexToDirect,
}
impl FbxReferenceInformationType {
    fn from_node(container_node: NodeHandle) -> Self {
        match container_node.children().find(|node| node.name() == "ReferenceInformationType").unwrap().attributes()[0]
            .get_string()
            .unwrap()
        {
            "Direct" => Self::Direct,
            "IndexToDirect" => Self::IndexToDirect,
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct FbxLayerElementNormal {
    normals: Vec<Vec3>,
    info_type: FbxMappingInformationType,
    _ref_type: FbxReferenceInformationType,
}
impl FbxLayerElementNormal {
    pub fn from_node(geometry_node: NodeHandle) -> Option<Self> {
        let normals_container_node = geometry_node.children().find(|node| node.name() == "LayerElementNormal")?;
        let normals_node = normals_container_node.children().find(|node| node.name() == "Normals").unwrap();
        let normals = normals_node.attributes()[0].get_arr_f64().unwrap().chunks(3).map(read_vec3).collect_vec();
        Some(Self {
            normals,
            info_type: FbxMappingInformationType::from_node(normals_container_node),
            _ref_type: FbxReferenceInformationType::from_node(normals_container_node),
        })
    }
}

#[derive(Debug)]
pub struct FbxLayerElementTangent {
    tangents: Vec<Vec3>,
    info_type: FbxMappingInformationType,
    _ref_type: FbxReferenceInformationType,
}
impl FbxLayerElementTangent {
    pub fn from_node(geometry_node: NodeHandle) -> Option<Self> {
        let tangents_container_node = geometry_node.children().find(|node| node.name() == "LayerElementTangent")?;
        let tangents_node = tangents_container_node.children().find(|node| node.name() == "Tangents").unwrap();
        let tangents = tangents_node.attributes()[0].get_arr_f64().unwrap().chunks(3).map(read_vec3).collect_vec();
        Some(Self {
            tangents,
            info_type: FbxMappingInformationType::from_node(tangents_container_node),
            _ref_type: FbxReferenceInformationType::from_node(tangents_container_node),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct TrianglePoint {
    vertex_index: usize,
    polygon_vertex_index: usize,
}

#[derive(Debug)]
pub struct FbxGeometry {
    pub id: i64,
    name: String,
    vertices: Vec<Vec3>,
    polygon_vertex_indices: Vec<i32>,
    polygons: Vec<Vec<TrianglePoint>>,
    normals: Option<FbxLayerElementNormal>,
    tangents: Option<FbxLayerElementTangent>,
    uvs: Vec<FbxLayerElementUV>,
    materials: Option<FbxLayerElementMaterial>,
    pub skin: Option<i64>,
}
impl FbxGeometry {
    pub fn from_node(node: NodeHandle, _: &FbxGlobalSettings) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let name = node.attributes()[1].get_string().unwrap().split('\u{0}').next().unwrap();

        let vertices_node = node.children().find(|node| node.name() == "Vertices").unwrap();
        let vertices = vertices_node.attributes()[0].get_arr_f64().unwrap().chunks(3).map(read_vec3).collect_vec();

        let polygon_vertex_indices_node = node.children().find(|node| node.name() == "PolygonVertexIndex").unwrap();
        let polygon_vertex_indices = polygon_vertex_indices_node.attributes()[0].get_arr_i32().unwrap();
        let mut polygons = Vec::new();
        let mut polygon = Vec::new();
        for (polygon_vertex_index, &vertex_index) in polygon_vertex_indices.iter().enumerate() {
            if vertex_index >= 0 {
                polygon.push(TrianglePoint { polygon_vertex_index, vertex_index: vertex_index as usize });
            } else {
                let index = -vertex_index - 1; // https://stackoverflow.com/questions/7736845/can-anyone-explain-the-fbx-format-for-me
                polygon.push(TrianglePoint { polygon_vertex_index, vertex_index: index as usize });
                polygons.push(polygon.clone());
                polygon = Vec::new();
            }
        }

        let materials_container_node = node.children().find(|node| node.name() == "LayerElementMaterial");

        Self {
            id,
            name: name.to_string(),
            vertices,
            polygon_vertex_indices: polygon_vertex_indices.to_vec(),
            polygons,
            normals: FbxLayerElementNormal::from_node(node),
            tangents: FbxLayerElementTangent::from_node(node),
            uvs: node.children().filter_map(FbxLayerElementUV::from_node).sorted_by_key(|x| x.channel).collect(),
            materials: materials_container_node.map(FbxLayerElementMaterial::from_node),
            skin: None,
        }
    }
    pub fn to_cpu_meshes(&self, skins: &IndexMap<i64, FbxSkin>, clusters: &HashMap<i64, FbxCluster>) -> Vec<Mesh> {
        // FBX is a bit complicated; there is a "merged" list of vertices in the self.vertices field (positions),
        // but other properties (such as normals) may require them to be unmerged, since one corner can have multiple
        // normals. This code handles both cases; when a vertex can be shared by multiple faces it will be, and when
        // it can't it will be split up into multiple vertices.

        let mut vertex_joint_indices = self.vertices.iter().map(|_| Vec::new()).collect_vec();
        let mut vertex_joint_weights = self.vertices.iter().map(|_| Vec::new()).collect_vec();

        if let Some(skin_id) = self.skin {
            if let Some(skin) = skins.get(&skin_id) {
                for (joint_index, cluster_id) in skin.clusters.iter().enumerate() {
                    let cluster = clusters.get(cluster_id).unwrap();
                    for (vertex_index, weight) in cluster.indexes.iter().zip(cluster.weights.iter()) {
                        vertex_joint_indices[*vertex_index as usize].push(joint_index as u32);
                        vertex_joint_weights[*vertex_index as usize].push(*weight as f32);
                    }
                }
            }
        }

        // The polygon_vertices represent all vertices for all polygons, so they may be reduntant. For instance, a triangluated
        // quad will have 3+3=6 polygon vertices, but at a later step they may get merged into just 4 vertices
        let polygon_vertices = self
            .polygon_vertex_indices
            .iter()
            .enumerate()
            .map(|(polygon_vertex_index, &vertex_index)| {
                let vertex_index = if vertex_index >= 0 { vertex_index } else { -vertex_index - 1 } as usize;
                IntermediateVertex {
                    position: self.vertices[vertex_index],
                    normal: self.normals.as_ref().map(|normals| match normals.info_type {
                        FbxMappingInformationType::ByPolygonVertex => normals.normals[polygon_vertex_index],
                        FbxMappingInformationType::ByVertex => normals.normals[vertex_index],
                        _ => todo!(),
                    }),
                    tangent: self.tangents.as_ref().map(|tangents| match tangents.info_type {
                        FbxMappingInformationType::ByPolygonVertex => tangents.tangents[polygon_vertex_index],
                        FbxMappingInformationType::ByVertex => tangents.tangents[vertex_index],
                        _ => todo!(),
                    }),
                    uvs: self
                        .uvs
                        .iter()
                        .map(|uv| match uv.mapping_info_type {
                            FbxMappingInformationType::ByPolygonVertex => {
                                let index = match uv.mapping_ref_type {
                                    FbxReferenceInformationType::Direct => polygon_vertex_index as i32,
                                    FbxReferenceInformationType::IndexToDirect => uv.uv_indices.as_ref().unwrap()[polygon_vertex_index],
                                };
                                if index >= 0 {
                                    uv.uvs[index as usize]
                                } else {
                                    Default::default()
                                }
                            }
                            FbxMappingInformationType::ByVertex => {
                                uv.uvs[match uv.mapping_ref_type {
                                    FbxReferenceInformationType::Direct => vertex_index,
                                    FbxReferenceInformationType::IndexToDirect => uv.uv_indices.as_ref().unwrap()[vertex_index] as usize,
                                }]
                            }
                            _ => todo!(),
                        })
                        .collect(),
                    joint_indices: vertex_joint_indices[vertex_index].clone(),
                    joint_weights: vertex_joint_weights[vertex_index].clone(),
                }
            })
            .collect_vec();
        let polygon_materials = if let Some(materials) = &self.materials {
            if materials.mapping_info_type == FbxMappingInformationType::AllSame {
                vec![self.polygons.clone()]
            } else {
                let mut res = Vec::new();
                for (poly, mat) in self.polygons.iter().zip(materials.materials.iter()) {
                    if res.len() < *mat as usize + 1 {
                        res.resize(*mat as usize + 1, Vec::new());
                    }
                    res[*mat as usize].push(poly.clone());
                }
                res
            }
        } else {
            vec![self.polygons.clone()]
        };
        polygon_materials
            .into_iter()
            .map(|polygons| {
                if polygons.is_empty() {
                    return Mesh {
                        name: self.name.clone(),
                        texcoords: (0..self.uvs.len()).map(|_i| Vec::new()).collect(),
                        ..Default::default()
                    };
                }
                let mut vertices = Vec::<Vec<(IntermediateVertex, u32)>>::new();
                vertices.resize(self.vertices.len(), Vec::new());

                let mut final_vertices = Vec::new();
                let mut indices = Vec::new();

                let mut triangles = Vec::new();
                for polygon in polygons {
                    for i in 0..(polygon.len() - 2) {
                        triangles.push([polygon[0], polygon[2 + i], polygon[1 + i]]);
                    }
                }

                for triangle in triangles {
                    for i in 0..3 {
                        let vertex = &polygon_vertices[triangle[i].polygon_vertex_index];
                        let variants = &mut vertices[triangle[i].vertex_index];
                        let index = if let Some((_, index)) = variants.iter().find(|(v, _)| v == vertex) {
                            *index
                        } else {
                            let index = final_vertices.len() as u32;
                            final_vertices.push(vertex.clone());
                            variants.push((vertex.clone(), index));
                            index
                        };
                        indices.push(index);
                    }
                }
                let mut mesh = Mesh {
                    name: self.name.clone(),
                    positions: Some(final_vertices.iter().map(|v| v.position).collect()),
                    colors: None,
                    normals: if final_vertices[0].normal.is_some() {
                        Some(final_vertices.iter().map(|v| v.normal.unwrap()).collect())
                    } else {
                        None
                    },
                    tangents: if final_vertices[0].tangent.is_some() {
                        Some(final_vertices.iter().map(|v| v.tangent.unwrap()).collect())
                    } else {
                        None
                    },
                    texcoords: (0..self.uvs.len()).map(|i| final_vertices.iter().map(|v| v.uvs[i]).collect()).collect(),
                    joint_indices: if self.skin.is_some() {
                        Some(
                            final_vertices
                                .iter()
                                .map(|v| {
                                    uvec4(
                                        v.joint_indices.first().copied().unwrap_or(0),
                                        v.joint_indices.get(1).copied().unwrap_or(0),
                                        v.joint_indices.get(2).copied().unwrap_or(0),
                                        v.joint_indices.get(3).copied().unwrap_or(0),
                                    )
                                })
                                .collect(),
                        )
                    } else {
                        None
                    },
                    joint_weights: if self.skin.is_some() {
                        Some(
                            final_vertices
                                .iter()
                                .map(|v| {
                                    vec4(
                                        v.joint_weights.first().copied().unwrap_or(0.),
                                        v.joint_weights.get(1).copied().unwrap_or(0.),
                                        v.joint_weights.get(2).copied().unwrap_or(0.),
                                        v.joint_weights.get(3).copied().unwrap_or(0.),
                                    )
                                })
                                .collect(),
                        )
                    } else {
                        None
                    },
                    indices: Some(indices),
                };
                mesh.try_ensure_tangents();
                mesh
            })
            .collect_vec()
    }
}

fn read_vec3(p: &[f64]) -> Vec3 {
    vec3(p[0] as f32, p[1] as f32, p[2] as f32)
}

#[derive(PartialEq, Clone, Default, Debug)]
struct IntermediateVertex {
    position: Vec3,
    normal: Option<Vec3>,
    tangent: Option<Vec3>,
    uvs: Vec<Vec2>,
    joint_indices: Vec<u32>,
    joint_weights: Vec<f32>,
}

#[derive(Debug)]
struct FbxLayerElementUV {
    channel: i32,
    mapping_info_type: FbxMappingInformationType,
    mapping_ref_type: FbxReferenceInformationType,
    uvs: Vec<Vec2>,
    uv_indices: Option<Vec<i32>>,
}
impl FbxLayerElementUV {
    fn from_node(uv_container_node: NodeHandle) -> Option<Self> {
        if uv_container_node.name() == "LayerElementUV" {
            let uv_node = uv_container_node.children().find(|node| node.name() == "UV").unwrap();
            let uvs = uv_node.attributes()[0].get_arr_f64().unwrap().chunks(2).map(|p| vec2(p[0] as f32, 1. - p[1] as f32)).collect_vec();
            let uv_index_node = uv_container_node.children().find(|node| node.name() == "UVIndex");
            let uv_indices = uv_index_node.map(|node| node.attributes()[0].get_arr_i32().unwrap().to_vec());
            Some(Self {
                channel: uv_container_node.attributes()[0].get_i32().unwrap(),
                mapping_info_type: FbxMappingInformationType::from_node(uv_container_node),
                mapping_ref_type: FbxReferenceInformationType::from_node(uv_container_node),
                uvs,
                uv_indices,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct FbxLayerElementMaterial {
    mapping_info_type: FbxMappingInformationType,
    _mapping_ref_type: FbxReferenceInformationType,
    materials: Vec<i32>,
}
impl FbxLayerElementMaterial {
    fn from_node(materials_container_node: NodeHandle) -> Self {
        let materials_node = materials_container_node.children().find(|node| node.name() == "Materials").unwrap();
        let materials = materials_node.attributes()[0].get_arr_i32().unwrap().to_vec();
        Self {
            mapping_info_type: FbxMappingInformationType::from_node(materials_container_node),
            _mapping_ref_type: FbxReferenceInformationType::from_node(materials_container_node),
            materials,
        }
    }
}

#[derive(Debug)]
pub struct FbxSkin {
    pub id: i64,
    pub clusters: Vec<i64>,
}
impl FbxSkin {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        Self { id, clusters: Vec::new() }
    }
    pub fn inverse_bind_matrices(&self, clusters: &HashMap<i64, FbxCluster>) -> Vec<Mat4> {
        self.clusters
            .iter()
            .map(|cluster_id| {
                let cluster = clusters.get(cluster_id).unwrap();

                cluster.transform
            })
            .collect()
    }
    pub fn joints(&self, clusters: &HashMap<i64, FbxCluster>) -> Vec<i64> {
        self.clusters
            .iter()
            .map(|cluster_id| {
                let cluster = clusters.get(cluster_id).unwrap();
                cluster.bone_id.unwrap()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct FbxCluster {
    pub id: i64,
    pub indexes: Vec<i32>,
    pub weights: Vec<f64>,
    pub transform: Mat4,
    pub transform_link: Mat4,
    pub bone_id: Option<i64>,
}
impl FbxCluster {
    pub fn from_node(node: NodeHandle) -> Self {
        let id = node.attributes()[0].get_i64().unwrap();
        let indexes = node.children().find(|node| node.name() == "Indexes");
        let weights = node.children().find(|node| node.name() == "Weights");
        Self {
            id,
            indexes: indexes.map(|indexes| indexes.attributes()[0].get_arr_i32().unwrap().to_vec()).unwrap_or_default(),
            weights: weights.map(|weights| weights.attributes()[0].get_arr_f64().unwrap().to_vec()).unwrap_or_default(),
            transform: read_matrix(node.children().find(|node| node.name() == "Transform").unwrap()),
            transform_link: read_matrix(node.children().find(|node| node.name() == "TransformLink").unwrap()),
            bone_id: None,
        }
    }
}
