use itertools::Itertools;
use kiwi_core::{asset_cache, async_ecs::async_run, runtime, session_start};
use kiwi_ecs::{components, query, SystemGroup};
use kiwi_intent::{intent_applied, intent_reverted, intent_timestamp, use_old_state, IntentRegistry};
use kiwi_std::asset_cache::AsyncAssetKeyExt;

use crate::brushes::{TerrainBrushKey, TerrainBrushStroke};

components!("terrain", {
    intent_terrain_stroke: TerrainBrushStroke,
    intent_terrain_stroke_state: (),
    stroke_client_applied: (),
});

pub fn register_intents(reg: &mut IntentRegistry) {
    reg.register(
        intent_terrain_stroke(),
        intent_terrain_stroke_state(),
        |ctx, stroke| {
            stroke.ensure_cells_exist(ctx.world);
            Ok(())
        },
        |_, _| Ok(()),
        use_old_state,
    )
}
pub fn terrain_intent_client_system() -> SystemGroup {
    SystemGroup::new(
        "dims/terrain/intent/client",
        vec![query((intent_terrain_stroke(), intent_timestamp()))
            .incl(intent_applied())
            .excl(intent_reverted())
            .excl(stroke_client_applied())
            .spawned()
            .to_system(|q, world, qs, _| {
                let mut strokes =
                    q.collect_cloned(world, qs).into_iter().filter(|(_, (stroke, _ts))| stroke.cells_exist(world)).collect_vec();
                if !strokes.is_empty() {
                    for (id, _) in &strokes {
                        world.add_component(*id, stroke_client_applied(), ()).unwrap();
                    }
                    let session_start = *world.resource(session_start());
                    strokes.retain(|(_, (_, ts))| *ts > session_start);
                    let async_run = world.resource(async_run()).clone();
                    let assets = world.resource(asset_cache()).clone();
                    world.resource(runtime()).spawn(async move {
                        let brush = TerrainBrushKey.get(&assets).await;
                        async_run.run(move |world| {
                            for (_, (stroke, _)) in strokes {
                                brush.apply(world, stroke);
                            }
                        });
                    });
                }
            })],
    )
}
