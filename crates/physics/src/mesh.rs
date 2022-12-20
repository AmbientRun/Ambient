use std::{sync::Arc};


use async_trait::async_trait;
use elements_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt}, download_asset::{AssetError, BytesFromUrlCachedPath, UrlString}
};

use itertools::Itertools;


use physxx::{
    PxConvexMesh, PxDefaultFileInputData, PxTriangleMesh
};
use serde::{Deserialize, Serialize};

use crate::{physx::PhysicsKey, rc_asset::PxRcAsset};

pub const PHYSX_TRIANGLE_MESH_EXTENSION: &str = ".pxtm";
pub const PHYSX_CONVEX_MESH_EXTENSION: &str = ".pxcm";

#[derive(Debug, Clone)]
pub enum PhysxGeometry {
    ConvexMesh(PxConvexMesh),
    TriangleMesh(PxTriangleMesh),
    // Heightfield
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxGeometryFromUrl(pub UrlString);
#[async_trait]
impl AsyncAssetKey<Result<Arc<PhysxGeometry>, AssetError>> for PhysxGeometryFromUrl {
    async fn load(self, assets: AssetCache) -> Result<Arc<PhysxGeometry>, AssetError> {
        if self.0.contains(PHYSX_TRIANGLE_MESH_EXTENSION) {
            Ok(Arc::new(PhysxGeometry::TriangleMesh(PhysxTriangleMeshFromUrl(self.0).get(&assets).await?.0)))
        } else {
            Ok(Arc::new(PhysxGeometry::ConvexMesh(PhysxConvexMeshFromUrl(self.0).get(&assets).await?.0)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysxTriangleMeshFromUrl(pub UrlString);
#[async_trait]
impl AsyncAssetKey<Result<PxRcAsset<PxTriangleMesh>, AssetError>> for PhysxTriangleMeshFromUrl {
    async fn load(self, assets: AssetCache) -> Result<PxRcAsset<PxTriangleMesh>, AssetError> {
        let file = BytesFromUrlCachedPath(self.0.clone()).get(&assets).await?;
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
pub struct PhysxConvexMeshFromUrl(pub UrlString);
#[async_trait]
impl AsyncAssetKey<Result<PxRcAsset<PxConvexMesh>, AssetError>> for PhysxConvexMeshFromUrl {
    async fn load(self, assets: AssetCache) -> Result<PxRcAsset<PxConvexMesh>, AssetError> {
        let file = BytesFromUrlCachedPath(self.0.clone()).get(&assets).await?;
        tokio::task::block_in_place(|| {
            let mem = PxDefaultFileInputData::new(&*file);
            let physics = PhysicsKey.get(&assets);
            let mesh = PxConvexMesh::new(physics.physics, &mem);
            Ok(PxRcAsset(mesh))
        })
    }
}
