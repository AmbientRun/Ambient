

@group(MATERIAL_BIND_GROUP)
@binding(0)
var normals_texture: texture_2d<f32>;

fn to_spherical_coordinates(v: vec3<f32>) -> vec3<f32> {
    let radius = length(v);
    let theta = acos(v.z / radius);
    let phi = atan2(v.y, v.x);
    return vec3<f32>(radius, theta, phi);
}

fn screen_space_reflections(world_position: vec3<f32>, screen_ray_dir: vec3<f32>, normal: vec3<f32>, screen_size: vec2<f32>, screen_position: vec2<f32>) -> vec3<f32> {
    var reflected_dir = reflect(screen_ray_dir, normal);
    reflected_dir.z = max(0., reflected_dir.z);

    // Figure out how big a 20 pixel step is in the world
    let reflected_p1 = world_position + reflected_dir;
    let reflected_p1_ndc = project_point(global_params.projection_view, reflected_p1);
    let reflected_p1_pixel = screen_ndc_to_pixel(vec3<f32>(reflected_p1_ndc.xy, 0.), screen_size);
    let step_size = 20.;
    let reflected_dir_pixel = reflected_p1_pixel - screen_position.xy;
    let reflected_step_pixel = normalize(reflected_dir_pixel) * step_size;
    let dir_scaling = length(reflected_step_pixel) / length(reflected_dir_pixel);

    var step = reflected_dir * dir_scaling;
    var pos = world_position + step;
    for (var i = 0; i < 5; i = i + 1) {
        let pos_ndc = project_point(global_params.projection_view, pos);
        let screen_tc = screen_ndc_to_uv(pos_ndc);
        if screen_tc.x < 0. || screen_tc.x >= 1. || screen_tc.y < 0. || screen_tc.y >= 1. {
            continue;
        }
        let screen_depth = textureSampleLevel(solids_screen_depth, default_sampler, screen_tc, 0.);
        if pos_ndc.z >= screen_depth && pos_ndc.z < screen_depth * 1.001 && screen_depth < 0.9999 {
            return textureSampleLevel(solids_screen_color, default_sampler, screen_tc, 0.).rgb;
        }
        step = step * 2.;
        pos = pos + step;
    }
    let sc = to_spherical_coordinates(reflected_dir);
    let tc = vec2<f32>(sc.z / (PI * 2.), sc.y / (PI * 1.));
    let sky = get_sky_color(global_params.camera_far, world_position, reflected_dir);
    return sky;
}

fn get_material(in: MaterialInput) -> MaterialOutput {
    var out: MaterialOutput;

    let screen_size = vec2<f32>(textureDimensions(solids_screen_depth));

    let normal_t1 = textureSample(normals_texture, default_sampler, in.world_position.xy * 0.05 + vec2<f32>(global_params.time * 0.01, 0.)).xyz;
    let normal_t2 = textureSample(normals_texture, default_sampler, in.world_position.xy * 0.1 + vec2<f32>(0., global_params.time * 0.02)).xyz;
    let normal_t = (normal_t1 + normal_t2) / 2.;
    let normal = normalize(normal_t * 2. - 1.);

    let screen_ray_dir = normalize(in.world_position.xyz - global_params.camera_position.xyz);
    let reflection_color = screen_space_reflections(in.world_position.xyz, screen_ray_dir, normal, screen_size, in.position.xy);

    let screen_tc = screen_pixel_to_uv(in.position.xy, screen_size);
    let screen_ndc = screen_uv_to_ndc(screen_tc);
    let screen_depth = get_solids_screen_depth(screen_ndc);
    let screen_depth_pos = project_point(global_params.inv_projection_view, vec3<f32>(screen_ndc.x, screen_ndc.y, screen_depth));

    let screen_color = get_solids_screen_color(screen_ndc);

    let water_depth = distance(in.world_position.xyz, screen_depth_pos);
    let water_fog_color = from_srgb_to_linear(vec3<f32>(22., 81., 102.) / 255.);

    let reflectiveness = pow(1. - dot(-screen_ray_dir, normal), 3.);

    out.opacity = 1.;
    out.alpha_cutoff = 0.;
    out.base_color = mix(interpolate_clamped_1_3(water_depth, 0., 4., screen_color, water_fog_color), reflection_color, reflectiveness);
    out.emissive_factor = vec3<f32>(0., 0., 0.);
    out.shading = 0.1;
    out.normal = in.normal;
    out.roughness = 0.1;
    out.metallic = 0.0; // Water acts like a mirror
    return out;
}
