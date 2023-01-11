use elements_std::mesh::Mesh;
use glam::{vec2, vec3};
use itertools::Itertools;
use yaml_rust::Yaml;

use crate::parse_unity_yaml;

/// A unity .asset file
pub struct Asset {
    pub mesh: Mesh,
}
impl Asset {
    pub fn from_string(data: &str) -> anyhow::Result<Self> {
        Ok(Self::from_yaml(parse_unity_yaml(data)?.pop().unwrap()))
    }
    pub fn from_yaml(asset: Yaml) -> Self {
        // From: https://docs.unity3d.com/ScriptReference/Rendering.VertexAttribute.html
        #[repr(usize)]
        #[allow(dead_code)]
        enum UnityVertexAttribs {
            Position,
            Normal,
            Tangent,
            Color,
            TexCoord0,
            TexCoord1,
            TexCoord2,
            TexCoord3,
            TexCoord4,
            TexCoord5,
            TexCoord6,
            TexCoord7,
            BlendWeight,
            BlendIndices,
        }
        let vertex_count = asset["Mesh"]["m_VertexData"]["m_VertexCount"].as_i64().unwrap() as usize;
        let vertex_data_size = asset["Mesh"]["m_VertexData"]["m_DataSize"].as_i64().unwrap() as usize;
        let vertex_size = vertex_data_size / vertex_count;
        struct Channel {
            offset: usize,
            dimension: usize,
        }
        let channels = asset["Mesh"]["m_VertexData"]["m_Channels"]
            .as_vec()
            .unwrap()
            .iter()
            .map(|channel| Channel {
                offset: channel["offset"].as_i64().unwrap() as usize,
                dimension: channel["dimension"].as_i64().unwrap() as usize,
            })
            .collect_vec();
        let vertex_data = asset["Mesh"]["m_VertexData"]["_typelessdata"].as_str().unwrap();
        let vertex_data = parse_unity_typeless_data(vertex_data);

        #[allow(clippy::identity_op)]
        let positions = if channels[UnityVertexAttribs::Position as usize].dimension > 0 {
            Some(
                (0..vertex_count)
                    .map(|i| {
                        let offset = (i * vertex_size + channels[UnityVertexAttribs::Position as usize].offset) / 4;
                        vec3(vertex_data[offset + 0], vertex_data[offset + 1], vertex_data[offset + 2])
                    })
                    .collect_vec(),
            )
        } else {
            None
        };
        #[allow(clippy::identity_op)]
        let normals = if channels[UnityVertexAttribs::Normal as usize].dimension > 0 {
            Some(
                (0..vertex_count)
                    .map(|i| {
                        let offset = (i * vertex_size + channels[UnityVertexAttribs::Normal as usize].offset) / 4;
                        vec3(vertex_data[offset + 0], vertex_data[offset + 1], vertex_data[offset + 2])
                    })
                    .collect_vec(),
            )
        } else {
            None
        };
        #[allow(clippy::identity_op)]
        let tangents = if channels[UnityVertexAttribs::Tangent as usize].dimension > 0 {
            Some(
                (0..vertex_count)
                    .map(|i| {
                        let offset = (i * vertex_size + channels[UnityVertexAttribs::Tangent as usize].offset) / 4;
                        vec3(vertex_data[offset + 0], vertex_data[offset + 1], vertex_data[offset + 2])
                    })
                    .collect_vec(),
            )
        } else {
            None
        };
        let texcoords = (0..8)
            .filter_map(|texcoord| {
                #[allow(clippy::identity_op)]
                if channels[UnityVertexAttribs::TexCoord0 as usize + texcoord].dimension > 0 {
                    Some(
                        (0..vertex_count)
                            .map(|i| {
                                let offset = (i * vertex_size + channels[UnityVertexAttribs::TexCoord0 as usize + texcoord].offset) / 4;
                                vec2(vertex_data[offset + 0], 1. - vertex_data[offset + 1])
                            })
                            .collect_vec(),
                    )
                } else {
                    None
                }
            })
            .collect_vec();

        let index_data = asset["Mesh"]["m_IndexBuffer"].as_str().unwrap();
        let indices = parse_unity_index_data(index_data);
        // for i in 0..(indices.len() / 3) {
        //     let x = indices[i * 3 + 1];
        //     indices[i * 3 + 1] = indices[i * 3 + 2];
        //     indices[i * 3 + 2] = x;
        // }

        Asset {
            mesh: Mesh {
                name: "unity_mesh".to_string(),
                positions,
                colors: None,
                normals,
                tangents,
                texcoords,
                joint_indices: None,
                joint_weights: None,
                indices: Some(indices),
            },
        }
    }
}

fn parse_unity_typeless_data(input: &str) -> Vec<f32> {
    // From: https://gist.github.com/maierfelix/1802f9523dee4090e16dc44a4ac70176
    fn hex2float(num: i32) -> f32 {
        let sign = if num < 0 { -1. } else { 1. };
        let exponent = ((num >> 23) & 0xff) - 127;
        let mantissa = 1. + ((num & 0x7fffff) as f32 / 0x7fffff as f32);
        sign * mantissa * (2f32).powi(exponent)
    }
    fn swap32(val: i64) -> i32 {
        (((val << 24) & 0xff000000) | ((val << 8) & 0xff0000) | ((val >> 8) & 0xff00) | ((val >> 24) & 0xff)) as i32
    }
    (0..(input.len() / 8))
        .map(|i| {
            let res = i64::from_str_radix(&input[i * 8..(i + 1) * 8], 16).unwrap();
            hex2float(swap32(res))
        })
        .collect_vec()
}
#[test]
fn test_parse_unity_typeless_data() {
    let input = "8388d6bf9bb924407c09bfbd00000000000000000000803f000080bff2cba53100000000000080bf7cb2e63eb987f03e7cb2e63eb987f03efa45f5bfaca2b03f502827bd00000000000000000000803f000080bf596511b300000000000080bff73dec3eb345823ef73dec3eb345823e181b34bd80cecdbd0000000000000000000000000000803f000080bfd22fc53100000000000080bfe603c13ef7824ebce603c13ef7824ebc8911ac3f76d42e4018facabd00000000000000000000803f000080bfb431043300000000000080bf69f6a03ea11cff3e69f6a03ea11cff3e9eb590bf0a2079407859dabd00000000000000000000803f000080bf23d4883300000000000080bf3c1ada3e3529353f3c1ada3e3529353f0bf0583fc72c8f403ca2c7bd00000000000000000000803f000080bfce890b3400000000000080bf3f6fac3ee404503f3f6fac3ee404503fa099213d2467ae401828bbbd00000000000000000000803f000080bff8ae303400000000000080bfce16bf3efb147d3fce16bf3efb147d3ffabeda3f9478933f542827bd00000000000000000000803f000080bfb887863200000000000080bffc8a983e9d755a3efc8a983e9d755a3e181b34bd80cecdbd000000000000008000000080000080bf0000803fd4c4203300000000000080bf0d7e5f3ff7824ebc0d7e5f3ff7824ebcfa45f5bfaca2b03f502827bd0000008000000080000080bf0000803f5965113300000000000080bf04e1493fb345823e04e1493fb345823e8388d6bf9bb924407c09bfbd0000008000000080000080bf0000803f4858f13200000000000080bfc2a64c3fb987f03ec2a64c3fb987f03e9eb590bf0a2079407859dabd0000008000000080000080bf0000803f24d488b300000000000080bfe2f2523f3529353fe2f2523f3529353f8911ac3f76d42e4018facabd0000008000000080000080bf0000803fad9e09ad00000000000080bfcc846f3fa11cff3ecc846f3fa11cff3e0bf0583fc72c8f403ca2c7bd0000008000000080000080bf0000803fce890bb400000000000080bf61c8693fe404503f61c8693fe404503fa099213d2467ae401828bbbd0000008000000080000080bf0000803ff8ae30b400000000000080bf9974603ffb147d3f9974603ffb147d3ffabeda3f9478933f542827bd0000008000000080000080bf0000803fb88786b200000000000080bf82ba733f9d755a3e82ba733f9d755a3e70a4aabda9112e40033897bf000080bf000000800000008000000000ce7c0533000080bf000080bf69a32d3f8603fe3e69a32d3f8603fe3ec87b76bd34ec9e3f4f9103c0000080bf000000800000008000000000c7d75b32000080bf000080bf63bb373f08fc6a3e63bb373f08fc6a3e189050bd40dfe8bd803f1bbd000080bf000000800000008000000000b90cb632000080bf000080bf0470203f599175bc0470203f599175bc000000008b488840167a403f000080bf000000800000008000000000fb090133000080bf000080bf0152173f0d13463f0152173f0d13463f00000000784c8e40590a32bf000080bf00000080000000800000000088dec132000080bf000080bf5707283f35c14e3f5707283f35c14e3f8829a1bd1cfc2f402fc8ac3f000080bf000000800000008000000000c269e032000080bf000080bfbb6a103fa163003fbb6a103fa163003f00000000c6d0ae4000000000000080bf000000800000008000000000263b3232000080bf000080bf0000203f6aad7d3f0000203f6aad7d3fe88563bd32a79b3f655b0140000080bf000000800000008000000000696d85b2000080bf000080bfb1aa083f3444663eb1aa083f3444663e189050bd40dfe8bd803f1bbd0000803f000000000000000000000000c55bdab10000803f000080bfe47ffc3d599175bce47ffc3d599175bcc87b76bd34ec9e3f4f9103c00000803f000000000000000000000000ffbcd5330000803f000080bfd449043d08fc6a3ed449043d08fc6a3e70a4aabda9112e40033897bf0000803f00000000000000000000000041b9b3b10000803f000080bfbce4923d8603fe3ebce4923d8603fe3e00000000784c8e40590a32bf0000803f00000000000000000000000088dec1b20000803f000080bf49c5bf3d35c14e3f49c5bf3d35c14e3f000000008b488840167a403f0000803f000000000000000000000000fb0901b30000803f000080bffdb7223e0d13463ffdb7223e0d13463f8829a1bd1cfc2f402fc8ac3f0000803f000000000000000000000000a3cd22b30000803f000080bf13553e3ea163003f13553e3ea163003f00000000c6d0ae40000000000000803f000000000000000000000000263b32b20000803f000080bf0000003e6aad7d3f0000003e6aad7d3fe88563bd32a79b3f655b01400000803f000000000000000000000000fc3a26b30000803f000080bf3a555d3e3444663e3a555d3e3444663e";

    let res = parse_unity_typeless_data(input);
    let start_expected = vec![
        -1.676, 2.5738, -0.0933, 0., 0., 1., -1., 0., 0., -1., 0.4506, 0.4698, 0.4506, 0.4698, -1.9162, 1.38, -0.0408, 0., 0., 1., -1.,
        -0., 0., -1., 0.4614, 0.2544, 0.4614, 0.2544,
    ];
    for i in 0..start_expected.len() {
        assert!((start_expected[i] - res[i]).abs() < 0.01);
    }
}
fn parse_unity_index_data(input: &str) -> Vec<u32> {
    fn swap16(val: i16) -> i16 {
        ((val & 0xFF) << 8) | ((val >> 8) & 0xFF)
    }
    (0..(input.len() / 4))
        .map(|i| {
            let p = i16::from_str_radix(&input[i * 4..(i + 1) * 4], 16).unwrap();
            swap16(p) as u32
        })
        .collect()
}
#[test]
fn test_parse_unity_index_data() {
    let input = "000001000200000003000400030000000200030005000400050006000400070003000200080009000a000b000c000a0008000a000c000b000d000c000b000e000d0008000c000f00100011001200100013001400150010001200150013001000130016001400170015001200180019001a001b001c001a0018001a001d001a001c001d001b001e001c0018001d001f00";
    let res = parse_unity_index_data(input);
    let expected = vec![
        0, 1, 2, 0, 3, 4, 3, 0, 2, 3, 5, 4, 5, 6, 4, 7, 3, 2, 8, 9, 10, 11, 12, 10, 8, 10, 12, 11, 13, 12, 11, 14, 13, 8, 12, 15, 16, 17,
        18, 16, 19, 20, 21, 16, 18, 21, 19, 16, 19, 22, 20, 23, 21, 18, 24, 25, 26, 27, 28, 26, 24, 26, 29, 26, 28, 29, 27, 30, 28, 24, 29,
        31,
    ];
    assert_eq!(res, expected);
}
