use std::sync::Arc;

use ambient_core::asset_cache;
use ambient_ecs::{
    components, query, Debuggable, Description, DynSystem, Entity, EntityId, FnSystem, Name, Networked, Resource, Store, SystemGroup, World,
};
use ambient_network::server::{ForkingEvent, ShutdownEvent};
use ambient_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use collider::{collider_shapes, collider_shapes_convex};
use glam::{vec3, Mat4, Vec3};
use helpers::release_px_scene;
use parking_lot::Mutex;
use physx::{
    actor_aggregate, articulation_cache, articulation_link, articulation_reduce_coordinate, character_controller, fixed_joint,
    physics_shape, revolute_joint, rigid_actor, rigid_dynamic, rigid_static,
};
use physxx::{
    AsPxActor, PxContactPairHeader, PxControllerManagerRef, PxMaterial, PxPvdSceneFlag, PxRigidActor, PxRigidActorRef, PxSceneDesc,
    PxSceneFlags, PxSceneRef, PxSimulationEventCallback, PxUserData,
};
use serde::{Deserialize, Serialize};

use crate::physx::PhysicsKey;

pub mod collider;
pub mod helpers;
pub mod intersection;
pub mod mesh;
pub mod physx;
pub mod rc_asset;
pub mod visualization;

components!("physics", {
    @[Resource]
    main_physics_scene: PxSceneRef,
    @[Resource]
    picking_scene: PxSceneRef,
    @[Resource]
    trigger_areas_scene: PxSceneRef,
    @[Resource]
    main_controller_manager: PxControllerManagerRef,
    @[Resource]
    wood_physics_material: PxMaterial,
    @[Debuggable, Resource]
    collisions: Arc<Mutex<Vec<(PxRigidActorRef, PxRigidActorRef)>>>,

    @[
        Debuggable, Networked, Store,
        Name["Unit velocity"],
        Description["The velocity of a character/unit."]
    ]
    unit_velocity: Vec3,
    @[
        Debuggable, Networked, Store,
        Name["Unit mass"],
        Description["The mass of a character/unit."]
    ]
    unit_mass: f32,
    @[
        Debuggable, Networked, Store,
        Name["Unit yaw"],
        Description["The yaw of a character/unit."]
    ]
    unit_yaw: f32,
    @[
        Debuggable, Networked, Store, Resource,
        Name["Collider loads"],
        Description["Contains all colliders that were loaded in this physics tick."]
    ]
    collider_loads: Vec<EntityId>,
    @[
        Debuggable, Networked, Store, Resource,
        Name["Make physics static"],
        Description["All physics objects will be made static when loaded."]
    ]
    make_physics_static: bool,
});
pub fn init_all_components() {
    init_components();
    physx::init_components();
    collider::init_components();
    visualization::init_components();
}

pub const GRAVITY: f32 = 9.82;
pub fn create_server_resources(assets: &AssetCache, server_resources: &mut Entity) {
    let physics = PhysicsKey.get(assets);
    server_resources.set(crate::physx::physics(), physics.clone());

    let mut main_scene_desc = PxSceneDesc::new(physics.physics);
    main_scene_desc.set_cpu_dispatcher(&physics.dispatcher);
    main_scene_desc.set_gravity(vec3(0., 0., -GRAVITY));
    main_scene_desc.update_flags(|flags| flags | PxSceneFlags::ENABLE_CCD);
    main_scene_desc.set_filter_shader(main_physx_scene_filter_shader, true);
    let collisions = Arc::new(Mutex::new(Vec::new()));
    {
        let collisions = collisions.clone();
        main_scene_desc.set_simulation_event_callbacks(PxSimulationEventCallback {
            collision_callback: Some(Box::new(move |header: &PxContactPairHeader| {
                if let (Some(a), Some(b)) = (header.actors[0], header.actors[1]) {
                    collisions.lock().push((a, b));
                }
            })),
        });
    }
    let main_scene = PxSceneRef::new(&physics.physics, &main_scene_desc);
    server_resources.set(self::collisions(), collisions);
    server_resources.set(self::collider_loads(), vec![]);

    main_scene.get_scene_pvd_client().set_scene_pvd_flags(
        PxPvdSceneFlag::TRANSMIT_CONSTRAINTS | PxPvdSceneFlag::TRANSMIT_SCENEQUERIES | PxPvdSceneFlag::TRANSMIT_CONTACTS,
    );

    let main_controller_manager = PxControllerManagerRef::new(&main_scene, true);

    let mut picking_scene_desc = PxSceneDesc::new(physics.physics);
    picking_scene_desc.set_cpu_dispatcher(&physics.dispatcher);
    let picking_scene = PxSceneRef::new(&physics.physics, &picking_scene_desc);

    let mut trigger_areas_desc = PxSceneDesc::new(physics.physics);
    trigger_areas_desc.set_cpu_dispatcher(&physics.dispatcher);
    let trigger_areas = PxSceneRef::new(&physics.physics, &trigger_areas_desc);

    server_resources.set(self::main_physics_scene(), main_scene);
    server_resources.set(crate::picking_scene(), picking_scene);
    server_resources.set(crate::trigger_areas_scene(), trigger_areas);
    server_resources.set(self::main_controller_manager(), main_controller_manager);
    server_resources.set(self::wood_physics_material(), PxMaterial::new(physics.physics, 0.5, 0.5, 0.6));
}

#[derive(Debug, Clone)]
pub struct PxShapeUserData {
    pub entity: EntityId,
    pub density: f32,
    /// The local pose of the shape before additional scaling is applied
    pub base_pose: Mat4,
}

impl Default for PxShapeUserData {
    fn default() -> Self {
        Self { entity: EntityId::null(), density: 1.0, base_pose: Mat4::IDENTITY }
    }
}

#[derive(Debug, Clone)]
pub struct PxActorUserData {
    pub serialize: bool,
}
impl Default for PxActorUserData {
    fn default() -> Self {
        Self { serialize: true }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum ColliderScene {
    Physics,
    TriggerArea,
    Picking,
}

impl ColliderScene {
    pub fn get_scene(&self, world: &World) -> PxSceneRef {
        match self {
            ColliderScene::Physics => *world.resource(main_physics_scene()),
            ColliderScene::TriggerArea => *world.resource(trigger_areas_scene()),
            ColliderScene::Picking => *world.resource(picking_scene()),
        }
    }
    pub fn from_usize(v: usize) -> Self {
        match v {
            0 => Self::Physics,
            1 => Self::TriggerArea,
            2 => Self::Picking,
            _ => panic!("Invalid value: {v}"),
        }
    }
}

#[derive(Debug)]
pub struct PxWoodMaterialKey;
impl SyncAssetKey<PxMaterial> for PxWoodMaterialKey {
    fn load(&self, assets: AssetCache) -> PxMaterial {
        let physics = PhysicsKey.get(&assets);
        PxMaterial::new(physics.physics, 0.5, 0.5, 0.6)
    }
}

unsafe extern "C" fn main_physx_scene_filter_shader(mut info: *mut physxx::sys::FilterShaderCallbackInfo) -> u16 {
    (*(*info).pairFlags).mBits |= (physxx::sys::PxPairFlag::eSOLVE_CONTACT
        | physxx::sys::PxPairFlag::eDETECT_DISCRETE_CONTACT
        | physxx::sys::PxPairFlag::eDETECT_CCD_CONTACT
        | physxx::sys::PxPairFlag::eCONTACT_DEFAULT
        | physxx::sys::PxPairFlag::eNOTIFY_TOUCH_FOUND) as u16;
    (physxx::sys::PxFilterFlag::eDEFAULT) as u16
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "physics",
        vec![
            query((physics_shape(),)).despawned().to_system(|q, world, qs, _| {
                for (id, (shape,)) in q.iter(world, qs) {
                    if let Some(actor) = shape.get_actor() {
                        actor.detach_shape(shape, true);
                        for shape in actor.get_shapes() {
                            let ud = shape.get_user_data::<PxShapeUserData>().unwrap();
                            if ud.entity == id {
                                actor.detach_shape(&shape, true);
                            }
                        }
                        if actor.get_nb_shapes() == 0 {
                            actor.as_actor().remove_user_data::<PxActorUserData>();
                            actor.release();
                        }
                    }
                }
            }),
            query((character_controller(),)).despawned().to_system(|q, world, qs, _| {
                for (_, (controller,)) in q.iter(world, qs) {
                    controller.release();
                }
            }),
            Box::new(collider::server_systems()),
            Box::new(visualization::server_systems()),
        ],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new("physics", vec![Box::new(visualization::client_systems())])
}

/// Starts the physx simulation step concurrently.
///
/// Results will be available after [`fetch_simulation_system`]
pub fn run_simulation_system() -> DynSystem {
    Box::new(FnSystem::new(|world, _| {
        profiling::scope!("run_simulation_system");
        let scene = world.resource(main_physics_scene());
        scene.simulate(1. / 60.);
    }))
}

/// Ensures the physx simulation data is available.
///
/// Must only be called once per [`run_simulation_system`]
pub fn fetch_simulation_system() -> DynSystem {
    Box::new(FnSystem::new(|world, _| {
        profiling::scope!("fetch_simulation_system");

        world.resource(collisions()).lock().clear();
        world.resource_mut(collider_loads()).clear();
        let scene = world.resource(main_physics_scene());
        // Ensure the previous simulation has completed
        scene.fetch_results(true);
    }))
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "physics/on_forking_systems",
        vec![Box::new(FnSystem::new(|world, _| {
            let mut ed = Entity::new();
            create_server_resources(world.resource(asset_cache()), &mut ed);
            world.add_components(world.resource_entity(), ed).unwrap();

            for (id, _) in query(()).incl(actor_aggregate()).collect_cloned(world, None) {
                world.remove_component(id, actor_aggregate()).unwrap();
            }
            for (id, _) in query(()).incl(rigid_actor()).collect_cloned(world, None) {
                world.remove_component(id, rigid_actor()).unwrap();
            }
            for (id, _) in query(()).incl(rigid_dynamic()).collect_cloned(world, None) {
                world.remove_component(id, rigid_dynamic()).unwrap();
            }
            for (id, _) in query(()).incl(rigid_static()).collect_cloned(world, None) {
                world.remove_component(id, rigid_static()).unwrap();
            }
            for (id, _) in query(()).incl(physics_shape()).collect_cloned(world, None) {
                world.remove_component(id, physics_shape()).unwrap();
            }
            for (id, _) in query(()).incl(fixed_joint()).collect_cloned(world, None) {
                world.remove_component(id, fixed_joint()).unwrap();
            }
            for (id, _) in query(()).incl(revolute_joint()).collect_cloned(world, None) {
                world.remove_component(id, revolute_joint()).unwrap();
            }
            for (id, _) in query(()).incl(articulation_reduce_coordinate()).collect_cloned(world, None) {
                world.remove_component(id, articulation_reduce_coordinate()).unwrap();
            }
            for (id, _) in query(()).incl(articulation_link()).collect_cloned(world, None) {
                world.remove_component(id, articulation_link()).unwrap();
            }
            for (id, _) in query(()).incl(articulation_cache()).collect_cloned(world, None) {
                world.remove_component(id, articulation_cache()).unwrap();
            }
            for (id, _) in query(()).incl(character_controller()).collect_cloned(world, None) {
                world.remove_component(id, character_controller()).unwrap();
            }
            for (id, _) in query(()).incl(collider_shapes()).collect_cloned(world, None) {
                world.remove_component(id, collider_shapes()).unwrap();
            }
            for (id, _) in query(()).incl(collider_shapes_convex()).collect_cloned(world, None) {
                world.remove_component(id, collider_shapes_convex()).unwrap();
            }
        }))],
    )
}
pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "physics/on_shutdown_systems",
        vec![Box::new(FnSystem::new(|world, _| {
            world.resource(main_physics_scene()).fetch_results(true);
            release_px_scene(*world.resource(main_physics_scene()));
            release_px_scene(*world.resource(picking_scene()));
            release_px_scene(*world.resource(trigger_areas_scene()));
        }))],
    )
}
