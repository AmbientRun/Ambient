
struct Plane {
    normal: vec3<f32>,
    distance: f32,
};

fn plane_distance(plane: Plane, position: vec3<f32>) -> f32 {
    return dot(plane.normal, position) + plane.distance;
}

struct Camera {
    view: mat4x4<f32>,
    position: vec4<f32>,
    frustum_right: Plane,
    frustum_top: Plane,
    orthographic_size: vec2<f32>,
    frustum_near: f32,
    frustum_far: f32,
    cot_fov_2: f32,
};

struct Params {
    main_camera: Camera,
    shadow_cameras: array<Camera, #MAX_SHADOW_CASCADES>,
    lod_cutoff_scaling: f32,
};

@group(#LODDING_BIND_GROUP)
@binding(0)
var<uniform> params: Params;

struct CameraCullResult {
    fully_contained: bool,
    inside: bool,
};

fn cull_camera(camera: Camera, bounding_sphere: vec4<f32>) -> CameraCullResult {
    var res: CameraCullResult;

	let center = (camera.view * vec4<f32>(bounding_sphere.xyz, 1.)).xyz;
	let radius = bounding_sphere.w;

    let sphere_mirrored = vec3<f32>(abs(center.xy), center.z);

    let top_dist = plane_distance(camera.frustum_top, sphere_mirrored);
    let right_dist = plane_distance(camera.frustum_right, sphere_mirrored);

    res.inside = !(top_dist > radius) &&
        !(right_dist > radius) &&

        center.z + radius > camera.frustum_near &&
        center.z - radius < camera.frustum_far;

    res.fully_contained = !(top_dist > -radius) &&
        !(right_dist > -radius) &&

        center.z - radius > camera.frustum_near &&
        center.z + radius < camera.frustum_far;

    return res;
}

fn get_lod(entity_loc: vec2<u32>) -> u32 {

    let bounding_sphere = get_entity_world_bounding_sphere(entity_loc);
	let radius = bounding_sphere.w;

    var lod_cutoffs = get_entity_lod_cutoffs(entity_loc) ;

    let dist = length(params.main_camera.position.xyz - bounding_sphere.xyz);
    let clip_space_radius = radius * params.main_camera.cot_fov_2 / dist;
    for (var i=0u; i < 4u; i = i + 1u) {
        for (var j=0u; j < 4u; j = j + 1u) {
            if (clip_space_radius >= lod_cutoffs[i][j] * params.lod_cutoff_scaling) {
                return i * 4u + j;
            }
        }
    }

    return 0u;
}

fn update(entity_loc: vec2<u32>) {
    if (has_entity_gpu_lod(entity_loc)) {
        set_entity_gpu_lod(entity_loc, vec4<f32>(f32(get_lod(entity_loc)), 0.0, 0.0, 0.0));
    }
    var cameras: mat4x4<f32>;
    let bounding_sphere = get_entity_world_bounding_sphere(entity_loc);
    cameras[0][0] = f32(cull_camera(params.main_camera, bounding_sphere).inside);

    for (var i=1u; i <= #SHADOW_CASCADESu; i = i + 1u) {
        let a = i >> 2u;
        let b = i & 3u;

        cameras[a][b] = 0.0;
    }
    for (var i=0u; i < #SHADOW_CASCADESu; i = i + 1u) {
        let radius = bounding_sphere.w;
        let pixel_size = vec2<f32>(radius) * 2. / params.shadow_cameras[i].orthographic_size;
        let min_pixel_size = min(pixel_size.x, pixel_size.y);

        if (min_pixel_size < 0.01) {
            break;
        }

        let res = cull_camera(params.shadow_cameras[i], bounding_sphere);
        if (res.inside) {
            let a = (i + 1u) >> 2u;
            let b = (i + 1u) & 3u;
            cameras[a][b] = 1.0;
        }
        if (res.fully_contained) {
            break;
        }
    }
    set_entity_renderer_cameras_visible(entity_loc, cameras);
}
