struct Gizmo {
  model_matrix: mat4x4<f32>,
  color: vec3<f32>,
  corner: f32,
  scale: vec2<f32>,
  border_w: f32,
  corner_inner: f32,
};

struct GizmoBuffer {
  gizmos: array<Gizmo>,
};

@group(#GIZMOS_BIND_GROUP)
@binding(0)
var<storage> gizmo_buffer: GizmoBuffer;

@group(#GIZMOS_BIND_GROUP)
@binding(1)
var depth_buffer: texture_depth_2d;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) world_position: vec4<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) @interpolate(flat) inst: u32,
  @location(3) ndc: vec3<f32>,
};

@vertex
fn vs_main(@builtin(instance_index) instance_index: u32,
@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
  let local_pos = get_raw_mesh_position(vertex_index);
  let uv = get_raw_mesh_uv(vertex_index);

  let gizmo = gizmo_buffer.gizmos[instance_index];

  let pos = gizmo.model_matrix * vec4<f32>(local_pos, 1.);
  let clip_pos = (global_params.projection_view * pos);

  let ndc = clip_pos.xyz / clip_pos.w;
  return VertexOutput(clip_pos, pos, uv, instance_index, ndc);
}

fn corner(radius: f32, uv: vec2<f32>, stretch: vec2<f32>, scale: vec2<f32>) -> bool {
  let pos_scale = max(scale, vec2<f32>(0., 0.));
  // Position rel center of quad
  let unstretched_mid = vec2<f32>(uv.x - 0.5, uv.y - 0.5) * 2.0 / pos_scale;
  let mid = unstretched_mid / stretch;

  // Pos to the nearest corner on quad
  let corner = vec2<f32>(sign(mid.x), sign(mid.y)) /  stretch;
  // The middle of the circle rounding the corner
  // let max_len = min(stretch.x, stretch.y);
  let aspect = (stretch.y / stretch.x);
  let short_side = max(stretch.x, stretch.y);
  let r = radius / short_side;
  let anchor = corner - vec2<f32>(sign(mid.x), sign(mid.y)) * r ;

  let rel = mid - anchor;
  let to_corner = corner - mid;

  let ang = dot(normalize(mid - anchor), normalize(corner - anchor));

  return length(mid - anchor) > r &&
    (ang > 0.707 || abs(unstretched_mid.x) > 1.0 || abs(unstretched_mid.y) > 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let gizmo = gizmo_buffer.gizmos[in.inst];

  let scale = gizmo.scale.xy;


  let uv = vec2<f32>(in.ndc.x * 0.5 + 0.5, 1.0 - (in.ndc.y * 0.5 + 0.5));

  let depth = in.ndc.z;

  let covered = textureSampleCompare(depth_buffer, shadow_sampler, uv, depth);
  // let depth = textureSample(depth_buffer, default_sampler, uv);

  let size = min(scale.x, scale.y);
  if corner(gizmo.corner, in.uv, scale, vec2<f32>(1.)) || !corner(gizmo.corner_inner, in.uv, scale,  (vec2<f32>(1.) - gizmo.border_w / scale.yx)) {
    discard;
  }

  return vec4<f32>(gizmo.color, 0.2 * covered + (1.0 - covered));
}
