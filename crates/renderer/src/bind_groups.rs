use wgpu::BindGroup;

#[derive(Debug, Clone, Copy)]
pub struct BindGroups<'a> {
    pub globals: &'a BindGroup,
    pub entities: &'a BindGroup,
    // Subset of `mesh_data`
    pub mesh_meta: &'a BindGroup,
}
