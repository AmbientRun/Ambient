use convert_case::{Case, Casing};
use std::{collections::HashMap, fmt::Display};

pub(crate) struct BindIdNodeFuncs<Id, Node> {
    pub node_to_id: fn(node: &Node) -> Id,
    pub node_name: fn(node: &Node) -> Option<&str>,
}

pub(crate) struct BindIdReg<Id, Node> {
    node_to_bind_id: HashMap<Id, String>,
    bind_id_to_node: HashMap<String, Id>,
    funcs: BindIdNodeFuncs<Id, Node>,
}
impl<Id: Display + std::hash::Hash + PartialEq + Eq + Clone, Node> BindIdReg<Id, Node> {
    pub fn new(funcs: BindIdNodeFuncs<Id, Node>) -> Self {
        Self {
            node_to_bind_id: Default::default(),
            bind_id_to_node: Default::default(),
            funcs,
        }
    }
    pub fn get(&mut self, node: &Node) -> String {
        let node_id = (self.funcs.node_to_id)(node);
        if let Some(bind_id) = self.node_to_bind_id.get(&node_id) {
            return bind_id.to_string();
        }
        let mut i = 0;
        let name_base = animation_bind_id_from_name(
            &(self.funcs.node_name)(node)
                .map(|x| x.to_string())
                .unwrap_or_else(|| format!("{}", node_id)),
        );
        let mut name = name_base.clone();
        loop {
            if !self.bind_id_to_node.contains_key(&name) {
                self.node_to_bind_id.insert(node_id.clone(), name.clone());
                self.bind_id_to_node.insert(name.clone(), node_id);
                return name;
            }
            i += 1;
            name = format!("{}_{}", name_base, i);
        }
    }
}

/// This will normalize a node name into a bind id, which is useful for retargeting
/// animations from one character to another.
fn animation_bind_id_from_name(name: &str) -> String {
    let name = if let Some((_a, b)) = name.split_once(':') {
        b.to_string()
    } else {
        name.to_string()
    };
    fn normalize_name(value: &str) -> String {
        if let Some(index) = value.strip_prefix("Thumb") {
            return format!("HandThumb{index}");
        } else if let Some(index) = value.strip_prefix("Index") {
            return format!("HandIndex{index}");
        } else if let Some(index) = value.strip_prefix("Middle") {
            return format!("HandMiddle{index}");
        } else if let Some(index) = value.strip_prefix("Ring") {
            return format!("HandRing{index}");
        } else if let Some(index) = value.strip_prefix("Pinky") {
            return format!("HandPinky{index}");
        }
        match value {
            "Knee" => "Leg".to_string(),
            _ => value.to_string(),
        }
    }
    if let Some(sub) = name.strip_prefix("L_") {
        format!("Left{}", normalize_name(&sub.to_case(Case::Pascal)))
    } else if let Some(sub) = name.strip_prefix("R_") {
        format!("Right{}", normalize_name(&sub.to_case(Case::Pascal)))
    } else {
        let name = name.to_case(Case::Pascal);
        if name.contains("Armature") {
            "Armature".to_string()
        } else {
            name
        }
    }
}
