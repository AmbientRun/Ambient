use ambient_std::mesh::{generate_tangents, Mesh, MeshBuilder};
use glam::{vec2, vec3, Vec3, Vec4};

pub struct CuboidMesh {
    /// Order: Bottom [ Left (Back, Front), Right (Back, Front) ] - Top [ Left (Back, Front), Right (Back, Front) ]
    pub positions: [Vec3; 8],
    pub color: Option<Vec4>,
    pub normals: bool,
    pub texcoords: bool,
    pub tangents: bool,
}

impl<'a> From<&'a CuboidMesh> for Mesh {
    fn from(cuboid: &'a CuboidMesh) -> Mesh {
        let back_left_bottom = 0; // back left bottom
        let front_left_bottom = 1;
        let back_right_bottom = 2;
        let front_right_bottom = 3;

        let back_left_top = 4;
        let front_left_top = 5;
        let back_right_top = 6;
        let front_right_top = 7;

        let positions = vec![
            //-Z
            cuboid.positions[back_left_bottom],
            cuboid.positions[front_left_bottom],
            cuboid.positions[back_right_bottom],
            cuboid.positions[front_right_bottom],
            //+Z
            cuboid.positions[back_left_top],
            cuboid.positions[back_right_top],
            cuboid.positions[front_left_top],
            cuboid.positions[front_right_top],
            //-X
            cuboid.positions[back_left_bottom],
            cuboid.positions[back_right_bottom],
            cuboid.positions[back_left_top],
            cuboid.positions[back_right_top],
            //+X
            cuboid.positions[front_right_bottom],
            cuboid.positions[front_left_bottom],
            cuboid.positions[front_right_top],
            cuboid.positions[front_left_top],
            //-Y
            cuboid.positions[front_left_bottom],
            cuboid.positions[back_left_bottom],
            cuboid.positions[front_left_top],
            cuboid.positions[back_left_top],
            //+Y
            cuboid.positions[back_right_bottom],
            cuboid.positions[front_right_bottom],
            cuboid.positions[back_right_top],
            cuboid.positions[front_right_top],
        ];

        let texcoords = if cuboid.texcoords {
            vec![vec![
                //-Z
                vec2(0.0, 0.0),
                vec2(0.0, 1.0),
                vec2(1.0, 0.0),
                vec2(1.0, 1.0),
                //+Z
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                //-X
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                //+X
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                //-Y
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
                //+Y
                vec2(0.0, 1.0),
                vec2(1.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 0.0),
            ]]
        } else {
            Vec::new()
        };

        let normals = if cuboid.normals {
            vec![
                //-Z
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 0.0, -1.0),
                //+Z
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 0.0, 1.0),
                //-X
                vec3(-1.0, 0.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
                //+X
                vec3(1.0, 0.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                vec3(1.0, 0.0, 0.0),
                //-Y
                vec3(0.0, -1.0, 0.0),
                vec3(0.0, -1.0, 0.0),
                vec3(0.0, -1.0, 0.0),
                vec3(0.0, -1.0, 0.0),
                //+Y
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 1.0, 0.0),
            ]
        } else {
            Vec::new()
        };

        let colors = if let Some(color) = cuboid.color {
            vec![color; 24]
        } else {
            Vec::new()
        };

        let mut indices = Vec::new();

        for i in 0..6 {
            indices.push(i * 4);
            indices.push(i * 4 + 2);
            indices.push(i * 4 + 1);

            indices.push(i * 4 + 1);
            indices.push(i * 4 + 2);
            indices.push(i * 4 + 3);
        }

        let tangents = if cuboid.tangents {
            generate_tangents(&positions, &texcoords[0], &indices)
        } else {
            Vec::new()
        };

        let mesh_builder = MeshBuilder {
            positions,
            colors,
            normals,
            tangents,
            texcoords,
            indices,
            ..MeshBuilder::default()
        };
        mesh_builder.build().expect("Invalid cuboid mesh")
    }
}
