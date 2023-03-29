use std::sync::Arc;

use ambient_core::{
    gpu_components,
    gpu_ecs::{
        ComponentToGpuSystem, GpuComponentFormat, GpuWorld, GpuWorldShaderModuleKey, GpuWorldSyncEvent, GpuWorldUpdater,
        ENTITIES_BIND_GROUP,
    },
};
use ambient_ecs::{components, ArchetypeFilter, Component, Entity, EntityId, System, SystemGroup, World};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    gpu_run::GpuRun,
};
use ambient_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use glam::{vec4, Vec4};
use parking_lot::Mutex;
use tokio::runtime::Runtime;

components!("gpu", {
    cpu_banana: Vec4,
    carrot: Vec4,
    tomato: Vec4,
});

gpu_components! {
    carrot() => carrot: GpuComponentFormat::Vec4,
    tomato() => tomato: GpuComponentFormat::Vec4,
}

struct TestCommon {
    assets: AssetCache,
    world: World,
    gpu_world: Arc<Mutex<GpuWorld>>,
    sync: SystemGroup<GpuWorldSyncEvent>,
}

impl TestCommon {
    async fn new() -> Self {
        ambient_core::init_all_components();
        init_components();
        init_gpu_components();

        let gpu = Arc::new(Gpu::new(None).await);
        let mut world = World::new("TestCommon");

        let assets = AssetCache::new(tokio::runtime::Handle::current());
        GpuKey.insert(&assets, gpu.clone());
        let gpu_world = Arc::new(Mutex::new(GpuWorld::new(assets.clone())));
        let sync = SystemGroup::new(
            "sync",
            vec![
                Box::new(ComponentToGpuSystem::new(GpuComponentFormat::Vec4, carrot(), gpu_components::carrot())),
                Box::new(ComponentToGpuSystem::new(GpuComponentFormat::Vec4, tomato(), gpu_components::tomato())),
            ],
        );
        world.add_component(world.resource_entity(), ambient_core::gpu_ecs::gpu_world(), gpu_world.clone()).unwrap();
        world.add_component(world.resource_entity(), ambient_core::gpu(), gpu).unwrap();

        Self { world, gpu_world, sync, assets }
    }
    fn update(&mut self) {
        self.gpu_world.lock().update(&self.world);
        self.sync.run(&mut self.world, &GpuWorldSyncEvent);
    }
    async fn get_gpu_component(&self, id: EntityId, component: Component<Vec4>) -> Vec4 {
        let loc = self.world.entity_loc(id).unwrap();
        let loc = vec4(loc.archetype as f32, loc.index as f32, 0., 0.);

        let module = GpuWorldShaderModuleKey { read_only: true }.get(&self.assets);
        let bind_group = self.gpu_world.lock().create_bind_group(true);
        GpuRun::new("gpu_ecs", format!("return get_entity_{}(vec2<u32>(u32(input.x), u32(input.y)));", component.path_last()))
            .add_module(module)
            .add_bind_group(ENTITIES_BIND_GROUP, bind_group)
            .run(&self.assets, loc)
            .await
    }
    async fn set_gpu_component(&self, id: EntityId, component: Component<Vec4>, value: f32) {
        let loc = self.world.entity_loc(id).unwrap();

        let input = vec4(loc.archetype as f32, loc.index as f32, value, 0.);
        let module = GpuWorldShaderModuleKey { read_only: false }.get(&self.assets);
        let bind_group = self.gpu_world.lock().create_bind_group(false);
        let _res: Vec4 = GpuRun::new(
            "gpu_ecs",
            format!(
                "set_entity_{}(vec2<u32>(u32(input.x), u32(input.y)), vec4<f32>(input.z)); return vec4<f32>(0.);",
                component.path_last()
            ),
        )
        .add_module(module)
        .add_bind_group(ENTITIES_BIND_GROUP, bind_group)
        .run(&self.assets, input)
        .await;
    }

    async fn assert_gpu_cpu_components_eq(&self, id: EntityId, component: Component<Vec4>) {
        let gpu = self.get_gpu_component(id, component).await;
        let cpu = self.world.get(id, component).unwrap();
        eprintln!("{gpu} {cpu}");
        assert_eq!(gpu, cpu);
    }
}

static SERIAL_TEST: Mutex<()> = Mutex::new(());

#[test]
fn two_entities() {
    let _guard = SERIAL_TEST.lock();
    tracing_subscriber::fmt::init();
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut test = TestCommon::new().await;

        let a = Entity::new().with(carrot(), vec4(7., 7., 3., 7.)).spawn(&mut test.world);
        test.update();

        let b = Entity::new().with(tomato(), vec4(1., 1., 3., 1.)).spawn(&mut test.world);
        test.update();
        test.assert_gpu_cpu_components_eq(a, carrot()).await;
        test.assert_gpu_cpu_components_eq(b, tomato()).await;
    })
}

#[tokio::test]
async fn gpu_ecs() {
    let _guard = SERIAL_TEST.lock();
    tracing_subscriber::fmt::init();
    let mut test = TestCommon::new().await;

    let _ignored = Entity::new().with(cpu_banana(), vec4(7., 7., 3., 7.)).spawn(&mut test.world);

    let a = Entity::new().with(carrot(), vec4(7., 7., 3., 7.)).spawn(&mut test.world);

    test.update();
    test.assert_gpu_cpu_components_eq(a, carrot()).await;

    test.world.set(a, carrot(), vec4(3., 9., 2., 1.)).unwrap();
    test.update();
    test.assert_gpu_cpu_components_eq(a, carrot()).await;

    test.world.add_component(a, cpu_banana(), vec4(0., 1., 2., 3.)).unwrap();
    test.update();
    test.assert_gpu_cpu_components_eq(a, carrot()).await;

    let b = Entity::new().with(tomato(), vec4(1., 1., 3., 1.)).spawn(&mut test.world);
    test.update();
    test.assert_gpu_cpu_components_eq(a, carrot()).await;
    test.assert_gpu_cpu_components_eq(b, tomato()).await;

    test.world.despawn(a);
    test.update();
    test.assert_gpu_cpu_components_eq(b, tomato()).await;
}

#[tokio::test]
async fn gpu_update_with_gpu_run() {
    let _guard = SERIAL_TEST.lock();
    tracing_subscriber::fmt::init();
    let mut test = TestCommon::new().await;

    let a = Entity::new().with(carrot(), vec4(7., 7., 3., 7.)).spawn(&mut test.world);
    test.update();
    assert_eq!(test.get_gpu_component(a, carrot()).await, vec4(7., 7., 3., 7.));
    test.set_gpu_component(a, carrot(), 1.).await;
    assert_eq!(test.get_gpu_component(a, carrot()).await, vec4(1., 1., 1., 1.));
}

#[tokio::test]
async fn gpu_update_with_gpu_ecs_update() {
    let _guard = SERIAL_TEST.lock();
    tracing_subscriber::fmt::init();
    let mut test = TestCommon::new().await;

    let a = Entity::new().with(carrot(), vec4(7., 7., 3., 7.)).spawn(&mut test.world);
    test.update();
    assert_eq!(test.get_gpu_component(a, carrot()).await, vec4(7., 7., 3., 7.));
    let mut update = GpuWorldUpdater::new(
        test.assets.clone(),
        "test".to_string(),
        ArchetypeFilter::new().incl(carrot()),
        vec![],
        &[],
        "set_entity_carrot(entity_loc, vec4<f32>(1.));".to_string(),
    );
    update.run(&test.world, &[]);
    assert_eq!(test.get_gpu_component(a, carrot()).await, vec4(1., 1., 1., 1.));
}
