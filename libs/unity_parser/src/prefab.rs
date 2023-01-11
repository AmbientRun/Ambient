use std::collections::HashMap;

use anyhow::Context;
use glam::{Mat4, Quat, Vec3};
use yaml_rust::Yaml;

use crate::{parse_unity_yaml, quat_from_yaml, vec3_from_yaml, UnityRef, YamlExt};

/// A unity .prefab file
#[derive(Debug)]
pub struct PrefabFile {
    pub objects: HashMap<i64, PrefabObject>,
}
impl PrefabFile {
    pub fn from_string(data: &str) -> anyhow::Result<Self> {
        Self::from_yaml(parse_unity_yaml(data)?)
    }
    pub fn from_yaml(docs: Vec<Yaml>) -> anyhow::Result<Self> {
        let objects = docs
            .iter()
            .map(|doc| {
                let (_, id) = doc["unity_object"].as_str().unwrap().split_once('&').unwrap();
                Ok((id.parse().unwrap(), PrefabObject::from_yaml(doc)?))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;
        Ok(Self { objects })
    }
    pub fn get_prefab(&self) -> Option<&Prefab> {
        self.objects.values().find_map(|x| if let PrefabObject::Prefab(x) = x { Some(x) } else { None })
    }
    pub fn get_root_game_object_ids(&self) -> Vec<i64> {
        if let Some(prefab) = self.get_prefab() {
            vec![prefab.root_game_object.file_id]
        } else {
            self.objects
                .values()
                .filter_map(|obj| {
                    if let PrefabObject::Transform(trans) = obj {
                        if trans.father.file_id == 0 {
                            return Some(trans.game_object.file_id);
                        }
                    }
                    None
                })
                .collect()
        }
    }
    pub fn get_root_game_objects(&self) -> Vec<&GameObject> {
        self.get_root_game_object_ids().iter().map(|id| self.objects.get(id).unwrap().as_object::<GameObject>().unwrap()).collect()
    }
    pub fn dump(&self) -> String {
        let root_ids = self.get_root_game_object_ids();
        let yml = Yaml::Array(
            root_ids
                .iter()
                .map(|id| {
                    let mut obj = self.objects.get(id).unwrap().dump(self, false);
                    obj.insert(Yaml::String("id".to_string()), Yaml::Integer(*id));
                    Yaml::Hash(obj)
                })
                .collect(),
        );
        let mut out_str = String::new();
        let mut emitter = yaml_rust::YamlEmitter::new(&mut out_str);
        emitter.multiline_strings(false);
        emitter.dump(&yml).unwrap();
        out_str
    }
}

#[derive(Debug)]
pub enum PrefabObject {
    Prefab(Prefab),
    GameObject(GameObject),
    LODGroup(LODGroup),
    MeshRenderer(MeshRenderer),
    MeshFilter(MeshFilter),
    Transform(Transform),
    Unimplemented,
}
impl PrefabObject {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        if yaml["Prefab"].as_hash().is_some() {
            Ok(Self::Prefab(Prefab::from_yaml(&yaml["Prefab"])?))
        } else if yaml["GameObject"].as_hash().is_some() {
            Ok(Self::GameObject(GameObject::from_yaml(&yaml["GameObject"])?))
        } else if yaml["LODGroup"].as_hash().is_some() {
            Ok(Self::LODGroup(LODGroup::from_yaml(&yaml["LODGroup"])?))
        } else if yaml["MeshRenderer"].as_hash().is_some() {
            Ok(Self::MeshRenderer(MeshRenderer::from_yaml(&yaml["MeshRenderer"])?))
        } else if yaml["MeshFilter"].as_hash().is_some() {
            Ok(Self::MeshFilter(MeshFilter::from_yaml(&yaml["MeshFilter"])?))
        } else if yaml["Transform"].as_hash().is_some() {
            Ok(Self::Transform(Transform::from_yaml(&yaml["Transform"])?))
        } else {
            Ok(Self::Unimplemented)
            // bail!("Unimplemented: {:?}", yaml);
        }
    }
    pub fn as_object<T: GetObject>(&self) -> Option<&T> {
        T::get_object(self)
    }
    pub fn dump(&self, prefab: &PrefabFile, dump_game_obj: bool) -> yaml_rust::yaml::Hash {
        match self {
            PrefabObject::Prefab(o) => o.dump(prefab),
            PrefabObject::GameObject(o) => o.dump(prefab),
            PrefabObject::LODGroup(o) => o.dump(prefab),
            PrefabObject::MeshRenderer(o) => o.dump(prefab),
            PrefabObject::MeshFilter(o) => o.dump(prefab),
            PrefabObject::Transform(o) => o.dump(prefab, dump_game_obj),
            PrefabObject::Unimplemented => {
                let mut out = yaml_rust::yaml::Hash::new();
                out.insert(Yaml::String("not_implemented".to_string()), Yaml::String("yet".to_string()));
                out
            }
        }
    }
}

pub trait GetObject {
    fn get_object(obj: &PrefabObject) -> Option<&Self>;
}

#[derive(Debug)]
pub struct LODGroup {
    pub lods: Vec<UnityLod>,
}
impl LODGroup {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self {
            lods: yaml["m_LODs"]
                .as_vec()
                .context("m_LODs not a vec")?
                .iter()
                .map(|lod| UnityLod::from_yaml(lod))
                .collect::<anyhow::Result<Vec<UnityLod>>>()?,
        })
    }
    pub fn dump(&self, prefab: &PrefabFile) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("LODGroup".to_string()));
        out.insert(Yaml::String("lods".to_string()), Yaml::Array(self.lods.iter().map(|lod| Yaml::Hash(lod.dump(prefab))).collect()));
        out
    }
}
impl GetObject for LODGroup {
    fn get_object(obj: &PrefabObject) -> Option<&Self> {
        if let PrefabObject::LODGroup(obj) = obj {
            Some(obj)
        } else {
            None
        }
    }
}
#[derive(Debug)]
pub struct UnityLod {
    renderer: UnityRef,
    pub screen_relative_height: f32,
}
impl UnityLod {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self {
            renderer: UnityRef::from_yaml(&yaml["renderers"][0]["renderer"])?,
            screen_relative_height: yaml["screenRelativeHeight"].as_float().context("screenRelativeHeight not a float")? as f32,
        })
    }
    pub fn get_renderer<'a>(&self, prefab: &'a PrefabFile) -> Option<&'a MeshRenderer> {
        prefab.objects.get(&self.renderer.file_id).and_then(|o| o.as_object::<MeshRenderer>())
    }
    pub fn dump(&self, _prefab: &PrefabFile) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("UnityLod".to_string()));
        out.insert(Yaml::String("renderer".to_string()), Yaml::Integer(self.renderer.file_id));
        out
    }
}

#[derive(Debug)]
pub struct Prefab {
    pub root_game_object: UnityRef,
}
impl Prefab {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self { root_game_object: UnityRef::from_yaml(&yaml["m_RootGameObject"])? })
    }
    pub fn dump(&self, _prefab: &PrefabFile) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("Prefab".to_string()));
        out
    }
}
impl GetObject for Prefab {
    fn get_object(obj: &PrefabObject) -> Option<&Self> {
        if let PrefabObject::Prefab(obj) = obj {
            Some(obj)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct GameObject {
    components: Vec<UnityRef>,
    pub name: String,
}
impl GameObject {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self {
            components: yaml["m_Component"]
                .as_vec()
                .context("m_Component not a vec")?
                .iter()
                .map(|comp| UnityRef::from_yaml(&comp["component"]))
                .collect::<anyhow::Result<Vec<UnityRef>>>()?,
            name: yaml["m_Name"].as_str().context("GameObject missing m_Name")?.to_string(),
        })
    }
    pub fn get_component<'a, T: GetObject>(&self, prefab: &'a PrefabFile) -> Option<&'a T> {
        for c in &self.components {
            if let Some(obj) = prefab.objects.get(&c.file_id) {
                if let Some(obj) = obj.as_object::<T>() {
                    return Some(obj);
                }
            }
        }
        None
    }
    pub fn dump(&self, prefab: &PrefabFile) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("GameObject".to_string()));
        out.insert(Yaml::String("name".to_string()), Yaml::String(self.name.clone()));
        out.insert(
            Yaml::String("components".to_string()),
            Yaml::Array(self.components.iter().map(|c| Yaml::Hash(c.dump(prefab, false))).collect()),
        );
        if let Some(transform) = self.get_component::<Transform>(prefab) {
            out.insert(
                Yaml::String("children".to_string()),
                Yaml::Array(
                    transform
                        .children
                        .iter()
                        .map(|c| {
                            let obj = &prefab.objects.get(&c.file_id).unwrap().as_object::<Transform>().unwrap().game_object;
                            let obj = prefab.objects.get(&obj.file_id).unwrap().as_object::<GameObject>().unwrap();
                            Yaml::Hash(obj.dump(prefab))
                        })
                        .collect(),
                ),
            );
        }
        out
    }
}
impl GetObject for GameObject {
    fn get_object(obj: &PrefabObject) -> Option<&Self> {
        if let PrefabObject::GameObject(obj) = obj {
            Some(obj)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct MeshRenderer {
    game_object: UnityRef,
    pub materials: Vec<UnityRef>,
}
impl MeshRenderer {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self {
            game_object: UnityRef::from_yaml(&yaml["m_GameObject"])?,
            materials: yaml["m_Materials"]
                .as_vec()
                .context("m_Materials not a vec")?
                .iter()
                .map(|mat| UnityRef::from_yaml(mat))
                .collect::<anyhow::Result<Vec<UnityRef>>>()?,
        })
    }
    pub fn get_game_object<'a>(&self, prefab: &'a PrefabFile) -> Option<&'a GameObject> {
        prefab.objects.get(&self.game_object.file_id).and_then(|x| x.as_object::<GameObject>())
    }
    pub fn dump(&self, _prefab: &PrefabFile) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("MeshRenderer".to_string()));
        out
    }
}
impl GetObject for MeshRenderer {
    fn get_object(obj: &PrefabObject) -> Option<&Self> {
        if let PrefabObject::MeshRenderer(obj) = obj {
            Some(obj)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct MeshFilter {
    pub mesh: UnityRef,
}
impl MeshFilter {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self { mesh: UnityRef::from_yaml(&yaml["m_Mesh"])? })
    }
    pub fn dump(&self, _prefab: &PrefabFile) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("MeshFilter".to_string()));
        out
    }
}
impl GetObject for MeshFilter {
    fn get_object(obj: &PrefabObject) -> Option<&Self> {
        if let PrefabObject::MeshFilter(obj) = obj {
            Some(obj)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Transform {
    pub local_rotation: Quat,
    pub local_position: Vec3,
    pub local_scale: Vec3,
    pub children: Vec<UnityRef>,
    pub father: UnityRef,
    pub game_object: UnityRef,
}
impl Transform {
    fn from_yaml(yaml: &Yaml) -> anyhow::Result<Self> {
        Ok(Self {
            local_rotation: quat_from_yaml(&yaml["m_LocalRotation"])?,
            local_position: vec3_from_yaml(&yaml["m_LocalPosition"])?,
            local_scale: vec3_from_yaml(&yaml["m_LocalScale"])?,
            children: yaml["m_Children"]
                .as_vec()
                .context("Not a vec")?
                .iter()
                .map(|child| UnityRef::from_yaml(child))
                .collect::<anyhow::Result<Vec<_>>>()?,
            father: UnityRef::from_yaml(&yaml["m_Father"])?,
            game_object: UnityRef::from_yaml(&yaml["m_GameObject"])?,
        })
    }
    pub fn absolute_transform(&self, prefab: &PrefabFile) -> Mat4 {
        let mat = Mat4::from_scale_rotation_translation(self.local_scale, self.local_rotation, self.local_position);
        if self.father.file_id == 0 {
            mat
        } else {
            let parent = prefab.objects.get(&self.father.file_id).unwrap().as_object::<Transform>().unwrap().absolute_transform(prefab);
            parent * mat
        }
    }
    pub fn dump(&self, prefab: &PrefabFile, dump_game_obj: bool) -> yaml_rust::yaml::Hash {
        let mut out = yaml_rust::yaml::Hash::new();
        out.insert(Yaml::String("type".to_string()), Yaml::String("Transform".to_string()));
        out.insert(Yaml::String("local_rotation".to_string()), Yaml::String(format!("{}", self.local_rotation)));
        out.insert(Yaml::String("local_position".to_string()), Yaml::String(format!("{}", self.local_position)));
        out.insert(Yaml::String("local_scale".to_string()), Yaml::String(format!("{}", self.local_scale)));
        if dump_game_obj {
            out.insert(Yaml::String("game_object".to_string()), Yaml::Hash(self.game_object.dump(prefab, true)));
        }
        out
    }
}
impl GetObject for Transform {
    fn get_object(obj: &PrefabObject) -> Option<&Self> {
        if let PrefabObject::Transform(obj) = obj {
            Some(obj)
        } else {
            None
        }
    }
}
