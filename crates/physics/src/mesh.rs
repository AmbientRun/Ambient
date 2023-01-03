use std::sync::Arc;

use async_trait::async_trait;
use elements_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt}, asset_url::{AbsAssetUrl, AbsAssetUrlOrRelativePath}, download_asset::{AssetError, BytesFromUrlCachedPath}
};
use itertools::Itertools;
use physxx::{PxConvexMesh, PxDefaultFileInputData, PxTriangleMesh};
use serde::{Deserialize, Serialize};

use crate::{physx::PhysicsKey, rc_asset::PxRcAsset};

pub const PHYSX_TRIANGLE_MESH_EXTENSION: &str = "pxtm";
pub const PHYSX_CONVEX_MESH_EXTENSION: &str = "pxcm";

#[derive(Debug, Clone)]
pub enum PhysxGeometry {
    ConvexMesh(PxConvexMesh),
    TriangleMesh(PxTriangleMesh),
    // Heightfield
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxGeometryFromUrl(pub AbsAssetUrlOrRelativePath);
impl PhysxGeometryFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<PhysxGeometryFromResolvedUrl> {
        Ok(PhysxGeometryFromResolvedUrl(self.0.resolve(base_url)?))
    }
}

#[derive(Debug, Clone)]
pub struct PhysxGeometryFromResolvedUrl(pub AbsAssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<Arc<PhysxGeometry>, AssetError>> for PhysxGeometryFromResolvedUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<PhysxGeometry>, AssetError> {
        if self.0.extension().unwrap_or_default().to_lowercase() == PHYSX_TRIANGLE_MESH_EXTENSION {
            Ok(Arc::new(PhysxGeometry::TriangleMesh(PhysxTriangleMeshFromResolvedUrl(self.0).get(&assets).await?.0)))
        } else {
            Ok(Arc::new(PhysxGeometry::ConvexMesh(PhysxConvexMeshFromResolvedUrl(self.0).get(&assets).await?.0)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxTriangleMeshFromUrl(pub AbsAssetUrlOrRelativePath);
impl PhysxTriangleMeshFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<PhysxTriangleMeshFromResolvedUrl> {
        Ok(PhysxTriangleMeshFromResolvedUrl(self.0.resolve(base_url)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxTriangleMeshFromResolvedUrl(pub AbsAssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<PxRcAsset<PxTriangleMesh>, AssetError>> for PhysxTriangleMeshFromResolvedUrl {
    async fn load(self, assets: AssetCache) -> Result<PxRcAsset<PxTriangleMesh>, AssetError> {
        let file = BytesFromUrlCachedPath { url: self.0.clone() }.get(&assets).await?;
        tokio::task::block_in_place(|| {
            let mem = PxDefaultFileInputData::new(&*file);
            let physics = PhysicsKey.get(&assets);
            let mesh = PxTriangleMesh::new(physics.physics, &mem);
            Ok(PxRcAsset(mesh))
        })
    }
    fn cpu_size(&self, value: &Result<PxRcAsset<PxTriangleMesh>, AssetError>) -> Option<usize> {
        value.as_ref().ok().map(|mesh| {
            (mesh.get_nb_triangles() * 3 * 4 + // TODO; need to get triangle mesh flags to see if it's 16 or 32 bit indices
            mesh.get_nb_vertices() * 3 * 4) as usize
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxConvexMeshFromUrl(pub AbsAssetUrlOrRelativePath);
impl PhysxConvexMeshFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<PhysxConvexMeshFromResolvedUrl> {
        Ok(PhysxConvexMeshFromResolvedUrl(self.0.resolve(base_url)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxConvexMeshFromResolvedUrl(pub AbsAssetUrl);
#[async_trait]
impl AsyncAssetKey<Result<PxRcAsset<PxConvexMesh>, AssetError>> for PhysxConvexMeshFromResolvedUrl {
    async fn load(self, assets: AssetCache) -> Result<PxRcAsset<PxConvexMesh>, AssetError> {
        let file = BytesFromUrlCachedPath { url: self.0.clone() }.get(&assets).await?;
        tokio::task::block_in_place(|| {
            let mem = PxDefaultFileInputData::new(&*file);
            let physics = PhysicsKey.get(&assets);
            let mesh = PxConvexMesh::new(physics.physics, &mem);
            Ok(PxRcAsset(mesh))
        })
    }
}
