
@group(#GLOBALS_BIND_GROUP)
@binding(0)
var default_sampler: sampler;

struct ForwardGlobalParams {
    projection_view: mat4x4<f32>,
    inv_projection_view: mat4x4<f32>,
    camera_position: vec4<f32>,
    camera_forward: vec3<f32>,
    camera_far: f32,
    sun_direction: vec4<f32>,
    sun_diffuse: vec4<f32>,
    sun_ambient: vec4<f32>,
    fog_color: vec4<f32>,
    forward_camera_position: vec4<f32>,
    fog: i32,
    time: f32,
    fog_height_falloff: f32,
    fog_density: f32,

    debug_metallic_roughness: f32,
    debug_normals: f32,
    debug_shading: f32,
};

struct ShadowCamera {
    viewproj: mat4x4<f32>,
    far: f32,
    near: f32,
};

@group(#GLOBALS_BIND_GROUP)
@binding(1)
var<uniform> global_params: ForwardGlobalParams;

struct ShadowCameras {
    cameras: array<ShadowCamera>,
};

@group(#GLOBALS_BIND_GROUP)
@binding(2)
var<storage> shadow_cameras: ShadowCameras;

@group(#GLOBALS_BIND_GROUP)
@binding(3)
var shadow_sampler: sampler_comparison;
@group(#GLOBALS_BIND_GROUP)
@binding(4)
var shadow_texture: texture_depth_2d_array;

@group(#GLOBALS_BIND_GROUP)
@binding(5)
var solids_screen_color: texture_2d<f32>;

@group(#GLOBALS_BIND_GROUP)
@binding(6)
var solids_screen_depth: texture_depth_2d;

@group(#GLOBALS_BIND_GROUP)
@binding(7)
var solids_screen_normal: texture_2d<f32>;

fn inside(v: vec3<f32>) -> bool {
    return v.x > -1. && v.x < 1. && v.y > -1. && v.y < 1. && v.z > 0. && v.z < 1.;
}

fn fetch_shadow_cascade(cascade: i32, homogeneous_coords: vec3<f32>) -> f32 {
    let light_local = homogeneous_coords.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return textureSampleCompareLevel(shadow_texture, shadow_sampler, light_local, cascade, homogeneous_coords.z + 0.0001);
}

fn get_shadow_cascade(world_position: vec4<f32>) -> i32 {
    for (var i: i32=0; i < #SHADOW_CASCADES; i = i + 1) {
        let cam = shadow_cameras.cameras[i];
        let p = cam.viewproj * world_position;
        if (inside(p.xyz / p.w)) {
            return i;
        }
    }
    return 0;
}

fn fetch_shadow(light_angle: f32, world_position: vec4<f32>) -> f32 {
    for (var i: i32=0; i < #SHADOW_CASCADES; i = i + 1) {
        let cam = shadow_cameras.cameras[i];

        // The texel size is in world coordinates, transform to depth buffer by
        // dividing by the depth of the camera
        let p = cam.viewproj * world_position;
        let p = p.xyz / p.w;
        if (inside(p)) {
            return fetch_shadow_cascade(i, p);
        }
    }
    return 1.;
}

fn screen_pixel_to_uv(pixel_position: vec2<f32>, screen_size: vec2<f32>) -> vec2<f32> {
    return pixel_position / screen_size;
}
fn screen_uv_to_pixel(uv: vec2<f32>, screen_size: vec2<f32>) -> vec2<f32> {
    return uv * screen_size;
}
fn screen_uv_to_ndc(uv: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(uv.x * 2. - 1., -(uv.y * 2. - 1.), 0.);
}
fn screen_ndc_to_uv(ndc: vec3<f32>) -> vec2<f32> {
    return vec2<f32>((ndc.x + 1.) / 2., (-ndc.y + 1.) / 2.);
}
fn screen_pixel_to_ndc(pixel_position: vec2<f32>, screen_size: vec2<f32>) -> vec3<f32> {
    return screen_uv_to_ndc(screen_pixel_to_uv(pixel_position, screen_size));
}
fn screen_ndc_to_pixel(ndc: vec3<f32>, screen_size: vec2<f32>) -> vec2<f32> {
    return screen_uv_to_pixel(screen_ndc_to_uv(ndc), screen_size);
}
fn project_point(transform: mat4x4<f32>, position: vec3<f32>) -> vec3<f32> {
    let p = transform * vec4<f32>(position, 1.);
    return p.xyz / p.w;
}

fn get_solids_screen_depth(screen_ndc: vec3<f32>) -> f32 {
    let screen_tc = screen_ndc_to_uv(screen_ndc);
    return textureSampleLevel(solids_screen_depth, default_sampler, screen_tc, 0.);
}
fn get_solids_screen_color(screen_ndc: vec3<f32>) -> vec3<f32> {
    let screen_tc = screen_ndc_to_uv(screen_ndc);
    return textureSample(solids_screen_color, default_sampler, screen_tc).rgb;
}
fn get_solids_screen_normal(screen_ndc: vec3<f32>) -> vec3<f32> {
    let screen_tc = screen_ndc_to_uv(screen_ndc);
    return textureSample(solids_screen_normal, default_sampler, screen_tc).rgb;
}

struct MaterialInput {
    position: vec4<f32>,
    texcoord: vec2<f32>,
    world_position: vec3<f32>,
    normal: vec3<f32>,
    normal_matrix: mat3x3<f32>,
    instance_index: u32,
    entity_loc: vec2<u32>,
    local_position: vec3<f32>,
};

struct MaterialOutput {
    base_color: vec3<f32>,
    emissive_factor: vec3<f32>,
    opacity: f32,
    alpha_cutoff: f32,
    shading: f32,
    normal: vec3<f32>,
    metallic: f32,
    roughness: f32,
};

struct MainFsOut {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
}

fn apply_fog(color: vec3<f32>, camera_pos: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    // From https://developer.amd.com/wordpress/media/2012/10/Wenzel-Real-time_Atmospheric_Effects_in_Games.pdf
    let camera_to_world_pos = world_pos - camera_pos;
    let vol_fog_height_density_at_viewer = exp( -global_params.fog_height_falloff * camera_pos.z );

    var fog_int = length(camera_to_world_pos) * vol_fog_height_density_at_viewer;
    let slope_threashold = 0.01;
    if (abs(camera_to_world_pos.z) > slope_threashold) {
        let t = global_params.fog_height_falloff * camera_to_world_pos.z;
        fog_int = fog_int * ( 1.0 - exp( -t ) ) / t;
    }
    let fog_amount = 1. - exp( -global_params.fog_density * fog_int );
    return mix(color, global_params.fog_color.rgb, clamp(fog_amount, 0., 1.));
}

fn fresnel(ndoth: f32, f0: vec3<f32>) -> vec3<f32> {
    let v = clamp(1.0 - ndoth, 0.0, 1.0);
    return f0 + (1.0 - f0) * pow(v, 5.0);
}

// Section: PBR

fn distribution_ggx(normal: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    // A squared roughness looks more correct based on observation by Disney and
    // Unreal
    let a = roughness * roughness;
    let a2 = a * a;

    let ndoth = max(dot(normal, h), 0.0);
    let ndoth2 = ndoth * ndoth;

    let numerator =a2;
    let denom = ndoth2 * (a2 - 1.0) + 1.0;

    let denom = PI * denom * denom;
    return numerator / denom;
}

fn geometry_schlick_ggx(ndotv: f32, k: f32) -> f32 {
    let numerator = ndotv;
    let denom = ndotv * (1.0 - k) + k;

    return numerator / denom;
}

fn geometry_smith(normal: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32)
-> f32 {
    // See prior comment about roughness squaring
    let a = (roughness * roughness) + 1.0;

    // Direct mapping
    // (r+1)^2 / 8.0
    let k = (a * a) / 8.0;

    let ndotv = max(dot(normal, v), 0.0);
    let ndotl = max(dot(normal, l), 0.0);

    return
          geometry_schlick_ggx(ndotv, k)
        * geometry_schlick_ggx(ndotl, k);
}

fn shading(material: MaterialOutput, world_position: vec4<f32>) -> vec4<f32> {
    if (global_params.debug_shading > 0.0) {
      return vec4(material.base_color.rgb, material.opacity);
    }

    let v = normalize(global_params.camera_position.xyz - world_position.xyz);

    let l = normalize(global_params.sun_direction.xyz);
    let h = normalize(v + l);

    let albedo = material.base_color.rgb;

    let metallic = material.metallic;
    let roughness = material.roughness;
    let normal = material.normal;

    // Interpolate the normal incidence.
    //
    // I.e; the reflected light rays when viewed straight ahead.
    //
    // For dielectric materials, such as wood, ceramic, plastics, etc, the
    // reflected light is not tinted and reflected by an average of (0.04, 0.04,
    // .0.4) of the
    // incoming ray.
    //
    // This value approaches (1,1,1) at a perpendicular incidence.
    //
    // Metallic materials tint the reflected light, and this is perceived as the
    // metals gloss.
    //
    // This tint is approximated to the albedo when the metallic factor is high
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let f = fresnel(max(dot(h,v), 0.0), f0);

    // Approximate microfacet alignment againt the halfway view direction
    let ndf = distribution_ggx(normal, h, roughness);
    // Approximate geometry occlusion and shadowing caused by microfacet induced
    // roughness
    let g = geometry_smith(normal, v, l, roughness);

    let ndotl = max(dot(normal, l), 0.0);
    let ndotv = max(dot(normal, v), 0.0);

    /// dgf / (4 n*v n*l)
    let numerator = ndf * g * f;
    let denom =
        max(4.0
        * ndotv
        * ndotl, 0.001);


    // Use fresnell scattering as a reflection coefficient
    let ks = f;

    // The diffuse/refracted coefficient is the opposite of the reflected
    // factor.
    //
    // If the material is metallic, the reflacted light is completely absorbed
    // and does not exit the material.
    //
    // This causes metallic objects to have no diffuse light
    let kd = (vec3<f32>(1.0) - ks) * (1.0 - metallic);

    let lambert = kd * albedo / PI;
    // Cook-torrance specular reflection
    let specular = ks * (ndf * g * f) / denom;

    let radiance = global_params.sun_diffuse.rgb;

    let in_shadow = fetch_shadow(ndotl, world_position);

    let direct = (lambert + specular) * radiance * ndotl * in_shadow;

    let indirect = albedo * global_params.sun_ambient.rgb;

    let lum = direct + indirect;

    var color = mix(material.base_color.rgb, lum, material.shading) + material.emissive_factor;

    color = mix(color, vec3(metallic, roughness, 0.0), global_params.debug_metallic_roughness);
    color = mix(color, normal, global_params.debug_normals);

    if (global_params.fog != 0) {
        color = apply_fog(color, global_params.camera_position.xyz, world_position.xyz);
    }

    // let color = vec3<f32>(roughness, metallic, 0.0);
    // color = color + u32_to_color(u32(get_shadow_cascade(world_position))) * 0.2;

    // let color = vec3<f32>(max(dot(material.normal, l), 0.0), 0.0, 0.0);

    return vec4<f32>(color,  material.opacity);

}

struct FSOutput {
    @location(0) color: vec4<f32>,
    @location(1) outline: vec4<f32>,
};
