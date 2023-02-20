use std::sync::Arc;

use ambient_gpu::shader_module::{Shader, ShaderModule};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey},
    include_file,
};

use super::{get_forward_module, MaterialShader, RendererShader};

pub struct StandardShaderKey {
    pub material_shader: Arc<MaterialShader>,
    pub lit: bool,
    pub shadow_cascades: u32,
}
impl std::fmt::Debug for StandardShaderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandardShaderKey").field("material_shader", &self.material_shader.id).field("lit", &self.lit).finish()
    }
}
impl SyncAssetKey<Arc<RendererShader>> for StandardShaderKey {
    fn load(&self, assets: AssetCache) -> Arc<RendererShader> {
        let id = format!("standard_shader_{}_{}", self.material_shader.id, self.lit);
        let shader = Shader::from_modules(
            &assets,
            id.clone(),
            [
                &get_forward_module(&assets, self.shadow_cascades),
                &self.material_shader.shader,
                &ShaderModule::new("StandardMaterial", include_file!("standard.wgsl"), vec![]),
            ]
            .into_iter(),
        );

        Arc::new(RendererShader {
            shader,
            id,
            vs_main: "vs_main".to_string(),
            fs_shadow_main: "fs_shadow_main".to_string(),
            fs_forward_main: if self.lit { "fs_forward_lit_main".to_string() } else { "fs_forward_unlit_main".to_string() },
            fs_outline_main: "fs_outlines_main".to_string(),
            transparent: false,
            double_sided: false,
            depth_write_enabled: true,
            transparency_group: 0,
        })
    }
}
