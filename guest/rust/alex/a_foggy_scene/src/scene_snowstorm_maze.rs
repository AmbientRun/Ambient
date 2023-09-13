pub const CONTENTS: &str = r#"

[gd_scene load_steps=8 format=3 uid="uid://uwm0tnj0o4aa"]

[ext_resource type="PackedScene" uid="uid://doqqrij317vb3" path="res://01_dans_bldgs/models/MSH_Building_001.glb" id="1_vcgap"]
[ext_resource type="PackedScene" uid="uid://cvp3slow4hlnv" path="res://00_plane_roamer/roamer_3rd_person.tscn" id="2_ct5dw"]
[ext_resource type="PackedScene" uid="uid://cw3tyyl2sgyt2" path="res://08_scene_BeachWithLighthouse/lighthouse.glb" id="3_slvbw"]
[ext_resource type="PackedScene" uid="uid://dvhj23mo2mcml" path="res://03_scene_ExtraDeepPit/really_lumpy_wall.glb" id="4_un2hq"]

[sub_resource type="Environment" id="Environment_tc6ps"]
background_mode = 1
background_color = Color(0.129412, 0.513726, 0.666667, 1)
fog_light_color = Color(0.129412, 0.513726, 0.666667, 1)
fog_light_energy = 2.0
fog_density = 0.03

[sub_resource type="PlaneMesh" id="PlaneMesh_8weks"]
size = Vector2(999, 999)

[sub_resource type="WorldBoundaryShape3D" id="WorldBoundaryShape3D_kvtac"]

[node name="snowstorm_maze" type="Node3D"]

[node name="WorldEnvironment" type="WorldEnvironment" parent="."]
environment = SubResource("Environment_tc6ps")

[node name="plane" type="MeshInstance3D" parent="."]
mesh = SubResource("PlaneMesh_8weks")

[node name="MSH_Building_001" parent="." instance=ExtResource("1_vcgap")]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, -10.9305, 0, 28.4423)


[node name="flor" type="StaticBody3D" parent="."]

[node name="CollisionShape3D" type="CollisionShape3D" parent="flor"]
shape = SubResource("WorldBoundaryShape3D_kvtac")

[node name="DirectionalLight3D" type="DirectionalLight3D" parent="."]
transform = Transform3D(1, 0, 0, 0, 0.796815, 0.604224, 0, -0.604224, 0.796815, 0, 0.97549, 0)

[node name="lighthouse" parent="." instance=ExtResource("3_slvbw")]
transform = Transform3D(16.3895, 0, 5.43649, 0, 17.2676, 0, -5.43649, 0, 16.3895, 50.0283, 7.15256e-06, -24.5468)

[node name="really_lumpy_wall" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(1.29344, 0, -2.61318, 0, 2.91577, 0, 2.61318, 0, 1.29344, 18.0552, 0, 31.4974)

[node name="really_lumpy_wall2" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(1.29344, 0, -2.61318, 0, 2.91577, 0, 2.61318, 0, 1.29344, 27.8501, -1.57305, 19.7813)

[node name="really_lumpy_wall3" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.84309, 0, -0.646977, 0, 4.19639, 0, 0.646977, 0, -2.84309, 27.8501, -1.57305, -0.116438)

[node name="really_lumpy_wall4" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-1.60193, 0, -0.359371, 0, 3.36146, 0, 0.221802, 0, -2.59551, 21.0642, -0.439059, 12.1418)

[node name="really_lumpy_wall7" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(2.88178, -0.188764, -0.401778, 0.148037, 2.89661, -0.299083, 0.4185, 0.275198, 2.87243, -22.4898, -0.654373, 7.98182)

[node name="really_lumpy_wall11" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.91162, 0.1521, 0.0319177, 0.148037, 2.89661, -0.299083, -0.0473092, -0.297037, -2.90021, -22.944, -0.65438, 35.9634)

[node name="really_lumpy_wall12" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-0.394595, -0.276753, -2.87566, 0.148037, 2.89661, -0.299083, 2.88515, -0.186476, -0.377951, 1.54267, -0.654388, 41.1597)

[node name="really_lumpy_wall13" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(0.760952, -0.327543, -2.7956, 0.148037, 2.89661, -0.299083, 2.81083, -0.0638816, 0.772581, -18.9267, -0.654388, -35.5473)

[node name="really_lumpy_wall14" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.67753, 0.25313, 1.12626, 0.148037, 2.89661, -0.299083, -1.14482, -0.217465, -2.67279, -40.1965, -0.654388, -19.4587)

[node name="really_lumpy_wall15" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.89776, 0.116946, -0.301687, 0.148037, 2.89661, -0.299083, 0.287709, -0.312553, -2.88466, -35.469, -1.25317, 1.46419)

[node name="really_lumpy_wall16" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(0.266042, 0.284763, 2.88961, 0.148037, 2.89661, -0.299083, -2.89983, 0.173998, 0.249837, -16.9677, -1.25317, -42.1598)

[node name="really_lumpy_wall17" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(2.59713, 0.00411778, 1.32538, 0.148037, 2.89661, -0.299083, -1.31709, 0.333689, 2.57985, 9.06544, -1.65282, -43.0768)

[node name="really_lumpy_wall28" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.0879, -0.179272, -1.96531, 0.152519, 4.61273, -0.280787, 2.15448, -0.500275, -1.8847, 4.75438, -1.20728, -18.1707)

[node name="really_lumpy_wall18" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(0.3753, 0.277993, 2.87812, 0.148037, 2.89661, -0.299083, -2.88772, 0.184621, 0.35872, 16.4483, -1.25317, -34.9276)

[node name="really_lumpy_wall19" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(2.91025, -0.157569, -0.0855644, 0.148037, 2.89661, -0.299083, 0.101164, 0.294173, 2.89913, 36.3785, -1.25317, -31.2022)

[node name="really_lumpy_wall20" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(0.361044, -0.315416, -2.87609, 0.148037, 2.89661, -0.299083, 2.88954, -0.108988, 0.374685, 41.4283, -1.25317, -8.26677)

[node name="really_lumpy_wall27" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(2.91049, -0.137498, 0.108935, 0.148037, 2.89661, -0.299083, -0.0941161, 0.304072, 2.89834, 41.4283, -1.25317, -18.1081)

[node name="really_lumpy_wall21" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(1.955, 0.123087, 2.15976, 0.148037, 2.89661, -0.299083, -2.15819, 0.310185, 1.9359, 37.9256, -1.25317, -3.43519)

[node name="really_lumpy_wall22" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(2.24251, -0.30444, -1.83852, 0.148037, 2.89661, -0.299083, 1.85767, 0.13668, 2.24323, 51.8494, -1.25317, 11.1629)

[node name="really_lumpy_wall23" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.76317, 0.234244, 0.900964, 0.148037, 2.89661, -0.299083, -0.91907, -0.237687, -2.7569, 34.701, -1.25317, 34.7411)

[node name="really_lumpy_wall24" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.912, 0.148021, -0.00776815, 0.148037, 2.89661, -0.299083, -0.00746512, -0.299091, -2.90038, 8.58969, -1.25317, 51.9275)

[node name="really_lumpy_wall25" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-0.574278, -0.264548, -2.84639, 0.148037, 2.89661, -0.299083, 2.85482, -0.203421, -0.557074, 30.2618, -1.25316, 57.785)

[node name="really_lumpy_wall26" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(1.277, -0.333713, -2.59993, 0.148037, 2.89661, -0.299083, 2.61707, -0.00101347, 1.28555, 45.09, -1.25315, 46.9641)

[node name="really_lumpy_wall8" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(1.28494, -0.333715, -2.59601, 0.148037, 2.89661, -0.299083, 2.61318, 0, 1.29344, -13.3577, -1.09604, -1.90327)

[node name="really_lumpy_wall9" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.49413, -0.317926, -1.47647, -0.325396, 2.89661, -0.0740475, 1.47484, 0.101433, -2.51322, -10.7059, -1.09604, -19.9764)

[node name="really_lumpy_wall10" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.8244, -0.333715, -0.642725, -0.325396, 2.89661, -0.0740476, 0.646976, 0, -2.84309, -20.099, -1.87269, -9.5427)

[node name="really_lumpy_wall5" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.65815, 0, 1.1983, 0, 2.91577, 0, -1.1983, 0, -2.65815, 7.80377, -2.32198, -16.017)

[node name="really_lumpy_wall6" parent="." instance=ExtResource("4_un2hq")]
transform = Transform3D(-2.90769, 0.132769, 0.171511, 0.178374, 2.77551, 0.875474, -0.123395, 0.88354, -2.77594, 1.91383, -1.57304, 34.9487)



"#;
