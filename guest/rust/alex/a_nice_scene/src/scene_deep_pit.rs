pub const CONTENTS: &str = r#"
[gd_scene load_steps=9 format=3 uid="uid://eijjjl7u2gya"]

[ext_resource type="PackedScene" uid="uid://c2sgbrkrjwovj" path="res://03_scene_ExtraDeepPit/a_chunky_tile.glb" id="2_wks04"]
[ext_resource type="PackedScene" uid="uid://y8bchgv7j5tl" path="res://00_plane_roamer/roamer_1st_person.tscn" id="3_4llrd"]
[ext_resource type="PackedScene" uid="uid://dyqi25tykd1p7" path="res://03_scene_ExtraDeepPit/tall_noscale.glb" id="3_t778v"]

[sub_resource type="ProceduralSkyMaterial" id="ProceduralSkyMaterial_xdtef"]

[sub_resource type="Sky" id="Sky_4ox8a"]
sky_material = SubResource("ProceduralSkyMaterial_xdtef")

[sub_resource type="Environment" id="Environment_0ju7m"]
background_mode = 2
sky = SubResource("Sky_4ox8a")

[sub_resource type="BoxMesh" id="BoxMesh_nfbo8"]

[sub_resource type="WorldBoundaryShape3D" id="WorldBoundaryShape3D_mynke"]

[node name="ReallyDeepPit" type="Node3D"]

[node name="WorldEnvironment" type="WorldEnvironment" parent="."]
environment = SubResource("Environment_0ju7m")

[node name="sun" type="DirectionalLight3D" parent="."]
transform = Transform3D(0.447663, -0.642966, -0.621444, -0.331471, -0.764779, 0.552486, -0.830497, -0.041337, -0.555487, 0, 0, 0)
light_color = Color(1, 0.756863, 0.596078, 1)
shadow_enabled = true


[node name="cube1" type="MeshInstance3D" parent="."]
transform = Transform3D(18.8608, 0, 0, 0, 18.8608, 0, 0, 0, 18.8608, 0.0295095, -9.27181, 0.491428)
mesh = SubResource("BoxMesh_nfbo8")

[node name="a_chunky_tile" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 0.558637, -0.65875, 2.62275)

[node name="a_chunky_tile2" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 2.90053, -0.658749, 2.1842)

[node name="a_chunky_tile4" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 3.42905, -0.658749, -0.0815222)

[node name="a_chunky_tile5" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 3.42841, -0.843518, -2.28087)

[node name="a_chunky_tile6" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 4.44188, -0.714198, -3.80614)

[node name="a_chunky_tile7" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 5.48085, -0.44673, -1.52523)

[node name="a_chunky_tile3" parent="." instance=ExtResource("2_wks04")]
transform = Transform3D(0.984463, 0, -0.17559, 0, 1, 0, 0.17559, 0, 0.984463, 0.671613, -0.658749, 0.389035)

[node name="floor" type="StaticBody3D" parent="."]

[node name="CollisionShape3D" type="CollisionShape3D" parent="floor"]
shape = SubResource("WorldBoundaryShape3D_mynke")

[node name="tall_noscale" parent="." instance=ExtResource("3_t778v")]
transform = Transform3D(0.0307479, 0, 0.999527, 0, 1, 0, -0.999527, 0, 0.0307479, -4.21202, 0.0903187, -3.6696)

[node name="tall_noscale2" parent="." instance=ExtResource("3_t778v")]
transform = Transform3D(0.97108, 0, 0.238754, 0, 1, 0, -0.238754, 0, 0.97108, -0.259511, -1.90735e-06, -12.3719)

[node name="tall_noscale3" parent="." instance=ExtResource("3_t778v")]
transform = Transform3D(0.422697, 0, -0.906271, 0, 1, 0, 0.906271, 0, 0.422697, 3.64183, 2.83879, -10.4898)

[node name="tall_noscale4" parent="." instance=ExtResource("3_t778v")]
transform = Transform3D(0.019337, 0, -0.999813, 0, 1, 0, 0.999813, 0, 0.019337, 7.77865, 2.76937, -3.42511)

"#;
