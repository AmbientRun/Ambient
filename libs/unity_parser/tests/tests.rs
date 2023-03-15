use ambient_unity_parser::{asset::Asset, mat::Material, prefab::PrefabFile};

#[test]
pub fn test_prefab() {
    PrefabFile::from_string(include_str!("Fir_01_Plant.prefab")).unwrap();
}

#[test]
pub fn test_asset() {
    Asset::from_string(include_str!("Fir_01_Plant_cross_s.asset")).unwrap();
}

#[test]
pub fn test_mat() {
    Material::from_string(include_str!("M_Fir_Bark_01.mat")).unwrap();
}
