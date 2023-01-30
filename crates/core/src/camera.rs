use elements_ecs::{components, query, query_mut, Component, ECSError, EntityData, EntityId, Networked, Store, SystemGroup, World};
use elements_std::{
    math::Line, shapes::{BoundingBox, Plane, Ray, AABB}
};
use glam::{vec3, Mat4, Vec2, Vec3, Vec3Swizzles};
use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::{
    transform::{inv_local_to_world, local_to_world}, window_physical_size
};

#[derive(Clone, Copy, Debug)]
pub struct OrthographicRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

components!("camera", {
    orthographic: OrthographicRect,
    @[Networked, Store]
    perspective_infinite_reverse: (),
    @[Networked, Store]
    perspective: (),
    @[Networked, Store]
    near: f32,
    @[Networked, Store]
    far: f32,
    @[Networked, Store]
    fovy: f32,
    @[Networked, Store]
    aspect_ratio: f32,
    @[Networked, Store]
    aspect_ratio_from_window: (),
    @[Networked, Store]
    projection: glam::Mat4,
    @[Networked, Store]
    projection_view: glam::Mat4,
    @[Networked, Store]
    active_camera: f32, // Higher value means higher priority
    @[Networked, Store]
    fog: (),
    @[Networked, Store]
    shadows_far: f32,
});

pub fn camera_systems() -> SystemGroup {
    SystemGroup::new(
        "camera_systems",
        vec![
            query(aspect_ratio()).incl(aspect_ratio_from_window()).to_system(|q, world, qs, _| {
                let window_size = world.resource(window_physical_size());
                let aspect_ratio = window_size.x as f32 / window_size.y as f32;
                for (id, ratio) in q.collect_cloned(world, qs) {
                    if aspect_ratio != ratio {
                        world.set(id, self::aspect_ratio(), aspect_ratio).unwrap();
                    }
                }
            }),
            query_mut((projection(),), (near(), fovy(), aspect_ratio())).incl(perspective_infinite_reverse()).to_system(
                |q, world, qs, _| {
                    for (_, (projection,), (&near, &fovy, &aspect_ratio)) in q.iter(world, qs) {
                        *projection = glam::Mat4::perspective_infinite_reverse_lh(fovy, aspect_ratio, near);
                    }
                },
            ),
            query_mut((projection(),), (near(), far(), fovy(), aspect_ratio())).incl(perspective()).to_system(|q, world, qs, _| {
                for (_, (projection,), (&near, &far, &fovy, &aspect_ratio)) in q.iter(world, qs) {
                    *projection = perspective_reverse(fovy, aspect_ratio, near, far);
                }
            }),
            query_mut((projection(),), (near(), far(), orthographic())).to_system(|q, world, qs, _| {
                for (_, (projection,), (&near, &far, orth)) in q.iter(world, qs) {
                    *projection = orthographic_reverse(orth.left, orth.right, orth.bottom, orth.top, near, far);
                }
            }),
            query_mut((projection_view(),), (projection().changed(), inv_local_to_world().changed())).to_system(|q, world, qs, _| {
                for (_, (projection_view,), (projection, view)) in q.iter(world, qs) {
                    *projection_view = *projection * *view;
                }
            }),
        ],
    )
}

/// Elements uses a left handed reverse-z NDC. This function will produce a correct perspective matrix for that
pub fn perspective_reverse(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
    // far and near and swapped on purpose
    Mat4::perspective_lh(fov_y_radians, aspect_ratio, z_far, z_near)
}
/// Elements uses a left handed reverse-z NDC. This function will produce a correct orthographic matrix for that
pub fn orthographic_reverse(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    // far and near and swapped on purpose
    Mat4::orthographic_lh(left, right, bottom, top, far, near)
}

pub fn screen_ray(world: &World, camera: EntityId, mouse_origin: Vec2) -> Result<Ray, ECSError> {
    let camera_projection = world.get(camera, projection())?;
    let camera_view = world.get(camera, inv_local_to_world())?;
    let camera_pv = (camera_projection * camera_view).inverse();
    let camera_mouse_origin = camera_pv.project_point3(mouse_origin.extend(1.));
    let camera_mouse_end = camera_pv.project_point3(mouse_origin.extend(-1.));
    let camera_mouse_dir = (camera_mouse_end - camera_mouse_origin).normalize();
    Ok(Ray::new(camera_mouse_origin, camera_mouse_dir))
}

pub fn get_active_camera(world: &World, scene: Component<()>) -> Option<EntityId> {
    query((scene, active_camera())).iter(world, None).max_by_key(|(_, (_, x))| OrderedFloat(**x)).map(|(id, _)| id)
}

#[derive(Clone, Debug)]
pub enum Projection {
    Orthographic { rect: OrthographicRect, near: f32, far: f32 },
    PerspectiveInfiniteReverse { fovy: f32, aspect_ratio: f32, near: f32 },
    Perspective { fovy: f32, aspect_ratio: f32, near: f32, far: f32 },
    Identity,
}
impl Projection {
    pub fn from_world(world: &World, entity: EntityId) -> Self {
        if let Ok(rect) = world.get(entity, orthographic()) {
            Self::Orthographic { rect, near: world.get(entity, near()).unwrap_or(-1.), far: world.get(entity, far()).unwrap_or(1.) }
        } else {
            let window_size = world.resource(window_physical_size());
            let aspect_ratio = window_size.x as f32 / window_size.y as f32;
            Self::PerspectiveInfiniteReverse {
                fovy: world.get(entity, fovy()).unwrap_or(1.),
                aspect_ratio,
                near: world.get(entity, near()).unwrap_or(0.1),
            }
        }
    }
    pub fn near(&self) -> f32 {
        match self {
            Projection::Orthographic { near, .. } => *near,
            Projection::PerspectiveInfiniteReverse { near, .. } => *near,
            Projection::Perspective { near, .. } => *near,
            Projection::Identity => -1.,
        }
    }
    pub fn far(&self) -> Option<f32> {
        match self {
            Projection::Orthographic { far, .. } => Some(*far),
            Projection::PerspectiveInfiniteReverse { .. } => None,
            Projection::Perspective { far, .. } => Some(*far),
            Projection::Identity => Some(1.),
        }
    }
    pub fn set_far(&mut self, new_far: f32) {
        *self = match self.clone() {
            Projection::Orthographic { rect, near, far: _ } => Projection::Orthographic { rect, near, far: new_far },
            Projection::PerspectiveInfiniteReverse { fovy, aspect_ratio, near } => {
                Projection::Perspective { fovy, aspect_ratio, near, far: new_far }
            }
            Projection::Perspective { fovy, aspect_ratio, near, far: _ } => {
                Projection::Perspective { fovy, aspect_ratio, near, far: new_far }
            }
            Projection::Identity => todo!(),
        }
    }
    pub fn fovy(&self) -> Option<f32> {
        match self {
            Projection::Orthographic { .. } => None,
            Projection::PerspectiveInfiniteReverse { fovy, .. } => Some(*fovy),
            Projection::Perspective { fovy, .. } => Some(*fovy),
            Projection::Identity => None,
        }
    }
    pub fn aspect(&self) -> Option<f32> {
        match self {
            Projection::Orthographic { .. } => None,
            Projection::PerspectiveInfiniteReverse { aspect_ratio, .. } => Some(*aspect_ratio),
            Projection::Perspective { aspect_ratio, .. } => Some(*aspect_ratio),
            Projection::Identity => None,
        }
    }
    pub fn is_infinite_reverse(&self) -> bool {
        matches!(self, Projection::PerspectiveInfiniteReverse { .. })
    }
    pub fn orthographic_size(&self) -> Option<Vec3> {
        match self {
            Projection::Orthographic { rect, near, far } => Some(vec3(rect.right - rect.left, rect.top - rect.bottom, far - near).abs()),
            Projection::PerspectiveInfiniteReverse { .. } => None,
            Projection::Perspective { .. } => None,
            Projection::Identity => None,
        }
    }
    pub fn matrix(&self) -> Mat4 {
        match self {
            Projection::Orthographic { rect, near, far } => orthographic_reverse(rect.left, rect.right, rect.bottom, rect.top, *near, *far),
            Projection::PerspectiveInfiniteReverse { fovy, aspect_ratio, near } => {
                Mat4::perspective_infinite_reverse_lh(*fovy, *aspect_ratio, *near)
            }
            Projection::Perspective { fovy, aspect_ratio, near, far } => perspective_reverse(*fovy, *aspect_ratio, *near, *far),
            Projection::Identity => Mat4::IDENTITY,
        }
    }
    pub fn to_entity_data(&self) -> EntityData {
        match self.clone() {
            Projection::Orthographic { rect, near, far } => {
                EntityData::new().set(orthographic(), rect).set(self::near(), near).set(self::far(), far)
            }
            Projection::PerspectiveInfiniteReverse { fovy, aspect_ratio, near } => EntityData::new()
                .set(perspective_infinite_reverse(), ())
                .set(self::near(), near)
                .set(self::fovy(), fovy)
                .set(self::aspect_ratio(), aspect_ratio),
            Projection::Perspective { fovy, aspect_ratio, near, far } => EntityData::new()
                .set(perspective(), ())
                .set(self::near(), near)
                .set(self::far(), far)
                .set(self::fovy(), fovy)
                .set(self::aspect_ratio(), aspect_ratio),
            Projection::Identity => todo!(),
        }
    }
    pub fn view_space_frustum(&self) -> CameraViewSpaceFrustum {
        let project_inv = self.matrix().inverse();
        let far = if self.is_infinite_reverse() { 0.9 } else { 0. };
        let near = 1.;
        let left_top_front = project_inv.project_point3(vec3(-1., 1., far));
        let right_top_front = project_inv.project_point3(vec3(1., 1., far));
        let right_top_back = project_inv.project_point3(vec3(1., 1., near));
        let right_bottom_back = project_inv.project_point3(vec3(1., -1., near));

        // assert!((left_top_front.x.abs() - right_top_front.x.abs()).abs() < 0.001);
        // assert!((right_top_back.y.abs() - right_bottom_back.y.abs()).abs() < 0.001);

        CameraViewSpaceFrustum {
            right: Plane::from_points(right_top_front, right_bottom_back, right_top_back).unwrap_or_else(Plane::zero),
            top: Plane::from_points(left_top_front, right_top_front, right_top_back).unwrap_or_else(Plane::zero),
        }
    }
}

#[derive(Clone)]
pub struct Camera {
    pub projection: Projection,
    pub view: Mat4,
    pub shadows_far: f32,
}
impl Camera {
    pub fn from_world(world: &World, entity: EntityId) -> Option<Self> {
        Some(Self {
            view: world.get(entity, inv_local_to_world()).ok()?,
            projection: Projection::from_world(world, entity),
            shadows_far: world.get(entity, shadows_far()).unwrap_or(2_000.0),
        })
    }
    pub fn get_active(world: &World, scene: Component<()>) -> Option<Self> {
        if let Some(cam) = get_active_camera(world, scene) {
            Self::from_world(world, cam)
        } else {
            None
        }
    }
    pub fn world_space_frustum_points(&self) -> Vec<Vec3> {
        let proj_view_inv = self.projection_view().inverse();
        let s = 1.;
        vec![
            proj_view_inv.project_point3(vec3(-s, -s, 0.)),
            proj_view_inv.project_point3(vec3(-s, -s, s)),
            proj_view_inv.project_point3(vec3(-s, s, 0.)),
            proj_view_inv.project_point3(vec3(-s, s, s)),
            proj_view_inv.project_point3(vec3(s, -s, 0.)),
            proj_view_inv.project_point3(vec3(s, -s, s)),
            proj_view_inv.project_point3(vec3(s, s, 0.)),
            proj_view_inv.project_point3(vec3(s, s, s)),
        ]
    }
    pub fn world_space_frustum_points_for_shadow_cascade(&self, cascade_index: u32, n_cascades: u32) -> Vec<Vec3> {
        // From: http://developer.download.nvidia.com/SDK/10.5/opengl/src/cascaded_shadow_maps/doc/cascaded_shadow_maps.pdf
        fn split_z(linear_factor: f32, near: f32, far: f32, i: u32, n: u32) -> f32 {
            let p = (i as f32) / (n as f32);
            (1. - linear_factor) * near * (far / near).powf(p) + linear_factor * (near + p * (far - near))
        }

        let near = 1.;
        let linear_factor = 0.0;
        let main_projection = self.projection.matrix();
        let main_projection_view_inv = self.projection_view().inverse();
        let far = self.projection.far().expect("Shadow camera can't be infinite. Use set_far(shadow_far) to get a shadow camera");
        let p0 = split_z(linear_factor, near, far, cascade_index, n_cascades);
        let p1 = split_z(linear_factor, near, far, cascade_index + 1, n_cascades);
        let z0 = main_projection.project_point3(vec3(0., 0., p0)).z;
        let z1 = main_projection.project_point3(vec3(0., 0., p1)).z;
        let frustum = vec![
            Vec3::new(-1.0, -1.0, z0),
            Vec3::new(-1.0, 1.0, z0),
            Vec3::new(1.0, -1.0, z0),
            Vec3::new(1.0, 1.0, z0),
            Vec3::new(-1.0, -1.0, z1),
            Vec3::new(-1.0, 1.0, z1),
            Vec3::new(1.0, -1.0, z1),
            Vec3::new(1.0, 1.0, z1),
        ];
        frustum.iter().map(|x| main_projection_view_inv.project_point3(*x)).collect()
    }
    pub fn world_space_frustum_lines(&self) -> Vec<Line> {
        let points = self.world_space_frustum_points();
        vec![
            Line(points[0], points[1]),
            Line(points[0], points[2]),
            Line(points[2], points[3]),
            Line(points[1], points[3]),
            Line(points[4], points[4 + 1]),
            Line(points[4], points[4 + 2]),
            Line(points[4 + 2], points[4 + 3]),
            Line(points[4 + 1], points[4 + 3]),
            Line(points[0], points[4]),
            Line(points[1], points[5]),
            Line(points[2], points[6]),
            Line(points[3], points[7]),
        ]
    }
    pub fn projection_view(&self) -> Mat4 {
        self.projection.matrix() * self.view
    }
    pub fn position(&self) -> Vec3 {
        self.view.inverse().transform_point3(Vec3::ZERO)
    }
    pub fn forward(&self) -> Vec3 {
        self.view.inverse().transform_vector3(Vec3::Z)
    }
    /// This is the same camera but with shadow_far applied
    pub fn to_shadows_far_bound(&self) -> Self {
        let mut cam = self.clone();
        cam.projection.set_far(self.shadows_far);
        cam
    }
    /// This will create shadow map camera for the given cascade, which will snap to pixels in the
    /// shadow map.
    pub fn create_snapping_shadow_camera(
        &self,
        light_direction: Vec3,
        cascade_index: u32,
        n_cascades: u32,
        shadow_map_resolution: u32,
    ) -> Self {
        let main_camera = self.to_shadows_far_bound();
        let frustum_world = main_camera.world_space_frustum_points_for_shadow_cascade(cascade_index, n_cascades);
        let frustum_perspective = frustum_world.iter().map(|x| main_camera.view.project_point3(*x)).collect_vec();
        let frustum_size = AABB::from_points(&frustum_perspective);
        let mut shadow_view = if light_direction != Vec3::Z {
            Mat4::look_at_lh(light_direction, Vec3::ZERO, Vec3::Z)
        } else {
            Mat4::look_at_lh(light_direction, Vec3::ZERO, (Vec3::Z + Vec3::X * 0.001).normalize())
        };
        assert!(!shadow_view.is_nan());
        let frustum_shadow = frustum_world.iter().map(|x| shadow_view.project_point3(*x)).collect_vec();

        // find min and max in shadow space
        let frustum_shadow_aabb = AABB::from_points(&frustum_shadow);

        // Find size of a shadow texel
        let size = (frustum_size.max - frustum_size.min).max_element();

        let texel_size = size / shadow_map_resolution as f32;
        let center_xy = (frustum_shadow_aabb.min.xy() + frustum_shadow_aabb.max.xy()) / 2.;
        let center_xy_snapped = (center_xy / (texel_size * 2.)).floor() * texel_size * 2.;

        // Center the view, so that the projection is symmetrical (culling relies on symetrical projection)
        shadow_view = Mat4::from_translation(-center_xy_snapped.extend(0.)) * shadow_view;

        let left = -Vec2::splat(size);
        let right = Vec2::splat(size);

        let near = frustum_shadow_aabb.min.z - 300.0;
        let far = frustum_shadow_aabb.max.z + 300.0;

        // assert_eq!(-ortho_min.x, ortho_max.x);
        // assert_eq!(-ortho_min.y, ortho_max.y);
        Self {
            view: shadow_view,
            projection: Projection::Orthographic {
                rect: OrthographicRect { left: left.x, right: right.x, bottom: left.y, top: right.y },
                near,
                far,
            },
            shadows_far: far,
        }
    }
    pub fn fitted_ortographic(eye: Vec3, lookat: Vec3, up: Vec3, fit: BoundingBox, aspect: f32) -> Self {
        let view = Mat4::look_at_lh(eye, lookat, up);
        let bounding = fit.transform(&view).to_aabb();
        let size = bounding.size();
        let bounding_aspect = size.x / size.y;
        let ortho = if bounding_aspect < aspect {
            OrthographicRect {
                left: bounding.center().x - size.y * aspect * 0.5,
                right: bounding.center().x + size.y * aspect * 0.5,
                top: bounding.max.y,
                bottom: bounding.min.y,
            }
        } else {
            OrthographicRect {
                left: bounding.min.x,
                right: bounding.max.x,
                top: bounding.center().y + size.x * (1. / aspect) * 0.5,
                bottom: bounding.center().y - size.x * (1. / aspect) * 0.5,
            }
        };
        Self { projection: Projection::Orthographic { rect: ortho, near: bounding.min.z, far: bounding.max.z }, view, shadows_far: 100. }
    }
    pub fn to_entity_data(&self) -> EntityData {
        self.projection
            .to_entity_data()
            .set(local_to_world(), self.view.inverse())
            .set(inv_local_to_world(), self.view)
            .set(projection_view(), self.projection_view())
    }
}

impl Default for Camera {
    fn default() -> Self {
        let view = Mat4::from_translation(-Vec3::X * 10.);
        Self { projection: Projection::Perspective { near: 0.1, far: 100., fovy: 1., aspect_ratio: 1. }, view, shadows_far: 100. }
    }
}

#[derive(Debug)]
pub struct CameraViewSpaceFrustum {
    pub right: Plane,
    pub top: Plane,
}

pub fn shadow_cameras_from_world(
    world: &World,
    shadow_cascades: u32,
    shadow_map_resolution: u32,
    light_direction: Vec3,
    scene: Component<()>,
) -> Vec<Camera> {
    let camera = Camera::get_active(world, scene).unwrap();
    (0..shadow_cascades)
        .map(|cascade| camera.create_snapping_shadow_camera(light_direction, cascade, shadow_cascades, shadow_map_resolution))
        .collect()
}

#[test]
fn test_frustum() {
    let projection = Projection::Orthographic { rect: OrthographicRect { left: -5., right: 5., bottom: -5., top: 5. }, near: -5., far: 5. };
    let frustum = projection.view_space_frustum();
    assert_eq!(frustum.right.distance(Vec3::X * 6.), 1.);
    assert_eq!(frustum.top.distance(Vec3::Y * 6.), 1.);
}

#[test]
fn test_frustum_reverse_z() {
    let projection = Projection::PerspectiveInfiniteReverse { fovy: 1., aspect_ratio: 1., near: 1. };

    for z in [1., 10., 100.] {
        let near = projection.matrix().project_point3(Vec3::Z * z + vec3(1., 1., 0.));
        println!("point {z} = {near}");
    }

    let inv_proj = projection.matrix().inverse();

    for z in [1., 0.9, 0.5, 0.1, 0.] {
        let near = inv_proj.project_point3(Vec3::Z * z + vec3(1., 1., 0.));
        println!("inv point {z} = {near}");
    }

    let frustum = projection.view_space_frustum();
    println!("{frustum:?}");
    assert!(frustum.right.distance(Vec3::X * 6.) > 0.);
    assert!(frustum.top.distance(Vec3::Y * 6.) > 0.);

    assert!(frustum.right.distance(Vec3::Z * 100.) < 0.);
    assert!(frustum.top.distance(Vec3::Z * 100.) < 0.);
}
