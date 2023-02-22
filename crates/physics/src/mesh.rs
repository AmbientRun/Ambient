use std::sync::Arc;

use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::{AbsAssetUrl, AssetUrl},
    download_asset::{AssetError, AssetsCacheOnDisk, BytesFromUrl, BytesFromUrlCachedPath},
};
use async_trait::async_trait;
use physxx::{PxConvexMesh, PxDefaultFileInputData, PxDefaultMemoryInputData, PxPhysicsRef, PxTriangleMesh};
use serde::{Deserialize, Serialize};

use crate::rc_asset::PxRcAsset;

pub const PHYSX_TRIANGLE_MESH_EXTENSION: &str = "pxtm";
pub const PHYSX_CONVEX_MESH_EXTENSION: &str = "pxcm";

#[derive(Debug, Clone)]
pub enum PhysxGeometry {
    ConvexMesh(PxConvexMesh),
    TriangleMesh(PxTriangleMesh),
    // Heightfield
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxGeometryFromUrl(pub AssetUrl);
impl PhysxGeometryFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<Self> {
        Ok(Self(self.0.resolve(base_url)?.into()))
    }
}

#[async_trait]
impl AsyncAssetKey<Result<Arc<PhysxGeometry>, AssetError>> for PhysxGeometryFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<PhysxGeometry>, AssetError> {
        if self.0.extension().unwrap_or_default().to_lowercase() == PHYSX_TRIANGLE_MESH_EXTENSION {
            Ok(Arc::new(PhysxGeometry::TriangleMesh(PhysxTriangleMeshFromUrl(self.0.unwrap_abs().into()).get(&assets).await?.0)))
        } else {
            Ok(Arc::new(PhysxGeometry::ConvexMesh(PhysxConvexMeshFromUrl(self.0.unwrap_abs().into()).get(&assets).await?.0)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxTriangleMeshFromUrl(pub AssetUrl);
impl PhysxTriangleMeshFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<Self> {
        Ok(Self(self.0.resolve(base_url)?.into()))
    }
}

#[async_trait]
impl AsyncAssetKey<Result<PxRcAsset<PxTriangleMesh>, AssetError>> for PhysxTriangleMeshFromUrl {
    async fn load(self, assets: AssetCache) -> Result<PxRcAsset<PxTriangleMesh>, AssetError> {
        if AssetsCacheOnDisk.get(&assets) {
            let file = BytesFromUrlCachedPath { url: self.0.unwrap_abs() }.get(&assets).await?;
            tokio::task::block_in_place(|| {
                let mem = PxDefaultFileInputData::new(&*file);
                let mesh = PxTriangleMesh::new(PxPhysicsRef::get(), &mem);
                Ok(PxRcAsset(mesh))
            })
        } else {
            let data = BytesFromUrl { url: self.0.unwrap_abs(), cache_on_disk: false }.get(&assets).await?;
            tokio::task::block_in_place(|| {
                let mem = PxDefaultMemoryInputData::new((*data).clone());
                let mesh = PxTriangleMesh::new(PxPhysicsRef::get(), &mem);
                Ok(PxRcAsset(mesh))
            })
        }
    }
    fn cpu_size(&self, value: &Result<PxRcAsset<PxTriangleMesh>, AssetError>) -> Option<u64> {
        value.as_ref().ok().map(|mesh| {
            mesh.get_nb_triangles() as u64 * 3 * 4 + // TODO; need to get triangle mesh flags to see if it's 16 or 32 bit indices
            mesh.get_nb_vertices() as u64 * 3 * 4
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxConvexMeshFromUrl(pub AssetUrl);
impl PhysxConvexMeshFromUrl {
    pub fn resolve(&self, base_url: &AbsAssetUrl) -> anyhow::Result<Self> {
        Ok(Self(self.0.resolve(base_url)?.into()))
    }
}

#[async_trait]
impl AsyncAssetKey<Result<PxRcAsset<PxConvexMesh>, AssetError>> for PhysxConvexMeshFromUrl {
    async fn load(self, assets: AssetCache) -> Result<PxRcAsset<PxConvexMesh>, AssetError> {
        if AssetsCacheOnDisk.get(&assets) {
            let file = BytesFromUrlCachedPath { url: self.0.unwrap_abs() }.get(&assets).await?;
            tokio::task::block_in_place(|| {
                let mem = PxDefaultFileInputData::new(&*file);
                let mesh = PxConvexMesh::new(PxPhysicsRef::get(), &mem);
                Ok(PxRcAsset(mesh))
            })
        } else {
            let data = BytesFromUrl { url: self.0.unwrap_abs(), cache_on_disk: false }.get(&assets).await?;
            tokio::task::block_in_place(|| {
                let mem = PxDefaultMemoryInputData::new((*data).clone());
                let mesh = PxConvexMesh::new(PxPhysicsRef::get(), &mem);
                Ok(PxRcAsset(mesh))
            })
        }
    }
}
