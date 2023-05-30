pub mod capsule;
pub mod cube;
pub mod cuboid;
pub mod grid;
pub mod pyramid;
pub mod torus;
pub mod uvsphere;
use std::sync::Arc;

use ambient_gpu::mesh_buffer::GpuMesh;
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey},
    mesh::{generate_tangents, Mesh, MeshBuilder},
};
pub use capsule::*;
pub use cube::*;
use glam::*;
pub use grid::*;
pub use torus::*;
pub use uvsphere::*;

#[derive(Debug, Clone)]
pub struct QuadMeshKey;
impl SyncAssetKey<Arc<GpuMesh>> for QuadMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(&assets, &Mesh::from(QuadMesh::default()))
    }
}
/// Same as [QuadMeshKey], but unit-sized (e.g. length alongside axes is 1.0 at most)
#[derive(Debug, Clone)]
pub struct UnitQuadMeshKey;
impl SyncAssetKey<Arc<GpuMesh>> for UnitQuadMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(
            &assets,
            &Mesh::from(QuadMesh::from_position_size(-Vec2::ONE * 0.5, Vec2::ONE)),
        )
    }
}

#[derive(Debug, Clone)]
pub struct CubeMeshKey;
impl SyncAssetKey<Arc<GpuMesh>> for CubeMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(&assets, &Mesh::from(CubeMesh::default()))
    }
}
/// Same as [CubeMeshKey], but unit-sized (e.g. length alongside axes is 1.0 at most)
#[derive(Debug, Clone)]
pub struct UnitCubeMeshKey;
impl SyncAssetKey<Arc<GpuMesh>> for UnitCubeMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(
            &assets,
            &Mesh::from(CubeMesh {
                position: -Vec3::ONE * 0.5,
                size: Vec3::ONE,
                color: vec4(1.0, 1.0, 1.0, 1.0),
            }),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct SphereMeshKey(pub UVSphereMesh);
impl SyncAssetKey<Arc<GpuMesh>> for SphereMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(&assets, &Mesh::from(self.0))
    }
}

#[derive(Debug, Clone, Default)]
pub struct TorusMeshKey(pub TorusMesh);
impl SyncAssetKey<Arc<GpuMesh>> for TorusMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(&assets, &Mesh::from(self.0))
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapsuleMeshKey(pub CapsuleMesh);
impl SyncAssetKey<Arc<GpuMesh>> for CapsuleMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(&assets, &Mesh::from(self.0))
    }
}

#[derive(Debug, Clone)]
pub struct UIRectMeshKey;
impl SyncAssetKey<Arc<GpuMesh>> for UIRectMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        let mut quad = QuadMesh::from_position_size(Vec2::ZERO, Vec2::ONE);
        quad.flip_uvs = true;
        let mesh = Mesh::from(quad);
        GpuMesh::from_mesh(&assets, &mesh)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GridMeshKey(pub GridMesh);
impl SyncAssetKey<Arc<GpuMesh>> for GridMeshKey {
    fn load(&self, assets: AssetCache) -> Arc<GpuMesh> {
        GpuMesh::from_mesh(&assets, &Mesh::from(self.0.clone()))
    }
}

pub fn triangle() -> Mesh {
    MeshBuilder {
        positions: vec![
            vec3(0.0, 0.5, 0.0),
            vec3(-0.5, -0.5, 0.0),
            vec3(0.5, -0.5, 0.0),
        ],
        colors: vec![
            vec4(1.0, 0.0, 0.0, 1.0),
            vec4(0.0, 1.0, 0.0, 1.0),
            vec4(0.0, 0.0, 1.0, 1.0),
        ],
        indices: vec![0, 1, 2],
        ..MeshBuilder::default()
    }
    .build()
    .expect("Invalid triangle mesh")
}

pub fn pentagon() -> Mesh {
    MeshBuilder {
        positions: vec![
            vec3(-0.0868241, 0.49240386, 0.0),
            vec3(-0.49513406, 0.06958647, 0.0),
            vec3(-0.21918549, -0.44939706, 0.0),
            vec3(0.35966998, -0.3473291, 0.0),
            vec3(0.44147372, 0.2347359, 0.0),
        ],
        colors: vec![
            vec4(0.5, 0.0, 0.5, 1.0),
            vec4(0.5, 1.0, 0.5, 1.0),
            vec4(0.5, 0.0, 0.5, 1.0),
            vec4(0.5, 0.0, 0.5, 1.0),
            vec4(0.5, 0.0, 1.0, 1.0),
        ],
        indices: vec![0, 1, 4, 1, 2, 4, 2, 3, 4],
        ..MeshBuilder::default()
    }
    .build()
    .expect("Invalid pentagon mesh")
}

pub struct QuadMesh {
    pub corners: [Vec3; 4],
    pub flip_uvs: bool,
}
impl Default for QuadMesh {
    fn default() -> Self {
        Self::from_position_size(-Vec2::ONE, Vec2::ONE * 2.)
    }
}
impl QuadMesh {
    pub fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self {
            corners: [
                position.extend(0.),
                (position + vec2(size.x, 0.)).extend(0.),
                (position + vec2(0., size.y)).extend(0.),
                (position + size).extend(0.),
            ],
            flip_uvs: false,
        }
    }
}
impl From<QuadMesh> for Mesh {
    fn from(quad: QuadMesh) -> Self {
        let positions = quad.corners.into_iter().collect::<Vec<_>>();
        let normals = vec![
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
        ];
        let texcoords = if quad.flip_uvs {
            vec![
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
            ]
        } else {
            vec![
                vec2(0.0, 0.0),
                vec2(0.0, 1.0),
                vec2(1.0, 0.0),
                vec2(1.0, 1.0),
            ]
        };
        let indices = vec![0, 1, 2, 1, 3, 2];
        let tangents = generate_tangents(&positions, &texcoords, &normals, &indices);

        MeshBuilder {
            positions,
            normals,
            tangents,
            texcoords: vec![texcoords],
            indices,
            ..MeshBuilder::default()
        }
        .build()
        .expect("Invalid quad mesh")
    }
}
