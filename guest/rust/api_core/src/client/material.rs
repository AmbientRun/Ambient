use crate::internal::wit;

#[derive(Clone, Copy)]
pub struct Descriptor<'a> {
    pub base_color_map: &'a str,
    pub normal_map: &'a str,
    pub metallic_roughness_map: &'a str,
    pub sampler: &'a str,
}

pub fn create(desc: &Descriptor) -> String {
    let desc = wit::client_material::Descriptor {
        base_color_map: desc.base_color_map,
        normal_map: desc.normal_map,
        metallic_roughness_map: desc.metallic_roughness_map,
        sampler: desc.sampler,
    };
    wit::client_material::create(desc)
}
