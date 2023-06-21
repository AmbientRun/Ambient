use ambient_ecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{is_default, is_false, is_true, materials::PipelinePbrMaterial, true_value};
use glam::Vec3;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModelsPipeline {
    /// The importer to use to process models.
    #[serde(default)]
    #[serde(skip_serializing_if = "ModelImporter::is_regular")]
    pub importer: ModelImporter,
    /// Use assimp as the importer.
    /// This will support more file formats, but is less well-integrated. Off by default.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub force_assimp: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Collider::is_none")]
    /// The physics collider to use for this mesh.
    pub collider: Collider,
    /// If a collider is present, this controls how it will interact with other colliders.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub collider_type: ColliderType,
    /// Whether or not this mesh should have its texture sizes capped.
    pub cap_texture_sizes: Option<ModelTextureSize>,
    /// Treats all assets in the pipeline as variations, and outputs a single asset which is a collection of all assets.
    /// Most useful for grass and other entities whose individual identity is not important.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub collection_of_variants: bool,
    /// Output prefabs that can be spawned. On by default.
    #[serde(default = "true_value")]
    #[serde(skip_serializing_if = "is_true")]
    pub output_prefabs: bool,
    /// Output the animations that belonged to this model.
    #[serde(default = "true_value")]
    #[serde(skip_serializing_if = "is_true")]
    pub output_animations: bool,
    /// If specified, these components will be added to the prefabs produced by `output_prefabs`.
    ///
    /// This is a great way to specify additional information about your prefab that can be used by gameplay logic.
    /// Note that these components should have static data (i.e. statistics), not dynamic state, as any such state could be
    /// replaced by this prefab being reloaded.
    #[serde(default)]
    #[serde(skip_serializing_if = "Entity::is_empty")]
    pub prefab_components: Entity,
    /// If specified, a list of overrides to use for the materials for the mesh.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub material_overrides: Vec<MaterialOverride>,
    /// If specified, a list of transformations to apply to this model. This can be used
    /// to correct coordinate space differences between your asset source and the runtime.
    ///
    /// These will be applied in sequence.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<ModelTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialOverride {
    /// The filter for this override (i.e. what it should apply to).
    pub filter: MaterialFilter,
    /// The material to use as the replacement.
    pub material: PipelinePbrMaterial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub enum ModelImporter {
    #[default]
    /// The default importer is sufficient for the majority of needs.
    Regular,
    /// Import Unity models.
    UnityModels {
        /// Whether or not the Unity prefabs should be converted to Ambient prefabs.
        use_prefabs: bool,
    },
    /// Import Quixel models.
    Quixel,
}

impl ModelImporter {
    /// Returns `true` if the model importer is [`Regular`].
    ///
    /// [`Regular`]: ModelImporter::Regular
    #[must_use]
    pub fn is_regular(&self) -> bool {
        matches!(self, Self::Regular)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Collider {
    #[default]
    /// No physics collider. The default.
    None,
    /// Extract the physics collider from the model.
    FromModel {
        /// Whether or not the normals should be flipped.
        #[serde(default)]
        #[serde(skip_serializing_if = "is_false")]
        flip_normals: bool,
        /// Whether or not the indices should be reversed for each triangle. On by default.
        #[serde(default = "true_value")]
        #[serde(skip_serializing_if = "is_true")]
        reverse_indices: bool,
    },
    /// Use a cylindrical character collider.
    Character {
        /// The radius of the collider.
        radius: Option<f32>,
        /// The height of the collider.
        height: Option<f32>,
    },
}

impl Collider {
    /// Returns `true` if the collider is [`None`].
    ///
    /// [`None`]: Collider::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum ColliderType {
    #[default]
    /// This object cannot move (e.g. a wall).
    Static,
    /// This object can move dynamically in the scene (e.g. a physics object).
    Dynamic,
    /// This object should only be present in the trigger-area scene.
    TriggerArea,
    /// This object should only be present in the picking scene.
    Picking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MaterialFilter {
    /// Replace all materials.
    All,
    /// Replace all materials that match this name exactly.
    ByName {
        /// The material name to replace. Must match exactly (i.e. is case-sensitive and does not ignore whitespace).
        name: String,
    },
}
impl MaterialFilter {
    pub fn by_name(name: impl Into<String>) -> Self {
        Self::ByName { name: name.into() }
    }
    pub fn is_all(&self) -> bool {
        matches!(self, MaterialFilter::All)
    }
}
impl Default for MaterialFilter {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTextureSize {
    /// Cap this model's textures to 128x128.
    X128,
    /// Cap this model's textures to 256x256.
    X256,
    /// Cap this model's textures to 512x512.
    X512,
    /// Cap this model's textures to 1024x1024.
    X1024,
    /// Cap this model's textures to 2048x2048.
    X2048,
    /// Cap this model's textures to 4096x4096.
    X4096,
    /// Cap this model's textures to SIZE x SIZE.
    /// It is strongly recommended that this is a power of two.
    Custom(u32),
}
impl ModelTextureSize {
    pub fn size(&self) -> u32 {
        match self {
            ModelTextureSize::X128 => 128,
            ModelTextureSize::X256 => 256,
            ModelTextureSize::X512 => 512,
            ModelTextureSize::X1024 => 1024,
            ModelTextureSize::X2048 => 2048,
            ModelTextureSize::X4096 => 4096,
            ModelTextureSize::Custom(size) => *size,
        }
    }
}
impl Default for ModelTextureSize {
    fn default() -> Self {
        Self::X512
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModelTransform {
    /// Rotate Y up to Z up.
    RotateYUpToZUp,
    /// Rotate X by `deg` degrees.
    RotateX {
        /// The degrees to rotate this model around the X axis.
        deg: f32,
    },
    /// Rotate Y by `deg` degrees.
    RotateY {
        /// The degrees to rotate this model around the Y axis.
        deg: f32,
    },
    /// Rotate Z by `deg` degrees.
    RotateZ {
        /// The degrees to rotate this model around the Z axis.
        deg: f32,
    },
    /// Scale this model.
    Scale {
        /// The factor to scale this model by.
        scale: f32,
    },
    /// Translate this model.
    Translate {
        /// The translation to apply to this model (i.e. this model will be moved by `translation` in the current coordinate space).
        translation: Vec3,
    },
    /// Scale this model's AABB.
    ScaleAABB {
        /// The factor to scale this model's AABB by.
        scale: f32,
    },
    /// Scale this model's animations (spatially, not in time).
    ScaleAnimations {
        /// The factor to scale this model's animations by.
        scale: f32,
    },
    /// Re-root this mesh.
    SetRoot {
        /// The name of the node to set as the new root for this mesh.
        name: String,
    },
    /// Re-center this mesh such that the root is located at the origin.
    Center,
}
