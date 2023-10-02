pub struct GodotNode {
    pub name: String,
    pub pos: Option<Vec3>,
    pub rot: Option<Quat>,
    pub siz: Option<Vec3>,
    pub path: Option<String>,
}

#[allow(dead_code)]
pub fn debug_nodes(nodes: &HashMap<String, GodotNode>) {
    for node in nodes.values() {
        println!(
            "node\n\t~{}\n\t~po{:?}\n\t~rt{:?}\n\t~sz{:?}\n\t~{:?}",
            node.name, node.pos, node.rot, node.siz, node.path
        );
    }
}

pub fn scene_contents_to_nodes(scene_contents: &str) -> HashMap<String, GodotNode> {
    // let text_contents = std::fs::read_to_string("cubesSceneTest.tscn")
    //     .expect("Failed to read the file");

    // println!("main.tscn contents = \nscene_raw}");
    let mut ext_paths: HashMap<String, String> = HashMap::new();
    let mut nodes: HashMap<String, GodotNode> = HashMap::new();
    let re_positioned_nodes =
        Regex::new(r#"\[node name=\"([^\"]*)\"[^\[]*parent=\"([^\"]*)\"[^\[]*Transform3D\((.*)\)"#)
            .unwrap();
    for cap in re_positioned_nodes.captures_iter(scene_contents) {
        let nodepath: String = format!("{}/{}", &cap[2], &cap[1]);
        let t: Vec<f32> = cap[3]
            .split(", ")
            .filter_map(|s| s.parse::<f32>().ok())
            .collect::<Vec<f32>>();

        let (myscale, myrotation, _mytranslation) = Mat4::from_mat3(Mat3 {
            x_axis: Vec3 {
                x: t[0],
                z: t[1],
                y: -t[2],
            },
            z_axis: Vec3 {
                x: t[3],
                z: t[4],
                y: -t[5],
            },
            y_axis: -Vec3 {
                x: t[6],
                z: t[7],
                y: -t[8],
            },
        })
        .to_scale_rotation_translation();

        nodes.insert(
            nodepath,
            GodotNode {
                name: cap[1].to_string(),
                pos: Some(Vec3 {
                    x: t[9],
                    z: t[10],
                    y: t[11],
                }),
                rot: Some(myrotation),
                siz: Some(myscale),
                path: None,
            },
        );
        // println!("Node header: {} @ t{}", &cap[1], &cap[3]);
    }

    let re_unpositioned_nodes =
        Regex::new(r#"\[node name=\"([^\"]*)\"[^\[]*parent=\"([^\"]*)\""#).unwrap();
    for cap in re_unpositioned_nodes.captures_iter(scene_contents) {
        let nodepath: String = format!("{}/{}", &cap[2], &cap[1]);
        if !nodes.contains_key(&nodepath) {
            nodes.insert(
                cap[1].to_string(),
                GodotNode {
                    name: cap[1].to_string(),
                    pos: None,
                    rot: None,
                    siz: None,
                    path: None,
                },
            );
        }
    }

    let re_ext_resources = Regex::new(r#"path=\"res:.*\/(.*\.[^\"]*).*id=\"([^\"]*)"#).unwrap();
    for cap in re_ext_resources.captures_iter(scene_contents) {
        ext_paths.insert(cap[2].to_string(), cap[1].to_string());
    }

    let re_ext_resource_nodes =
        Regex::new(r#"\[node name=\"([^\"]*)\".*parent=\"([^\"]*)\".*ExtResource\(\"([^\"]*)\"\)"#)
            .unwrap();
    for cap in re_ext_resource_nodes.captures_iter(scene_contents) {
        let nodepath: String = format!("{}/{}", &cap[2], &cap[1]);
        if nodes.contains_key(&nodepath) && ext_paths.contains_key(&cap[3]) {
            if let Some(node) = nodes.get_mut(&nodepath) {
                node.path = Some(ext_paths[&cap[3]].to_string());
            };
        } else {
            println!(
                "ERR: could not find node {:?} or path {:?}",
                &nodepath, &cap[3]
            );
        }
    }

    nodes
}

use std::collections::HashMap;

use regex::Regex;

use glam::f32::{Mat3, Mat4, Quat, Vec3};
