use std::f32::consts::TAU;

use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::runtime;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_std::{time::Clock, IntoDuration};
use ambient_ui_native::{
    self,
    graph::{Graph, GraphScaleKind, GraphStyle},
    *,
};
use fixed_vec_deque::FixedVecDeque;
use glam::{vec2, vec4, Vec2};
use itertools::Itertools;
use rand::{prelude::StdRng, Rng, SeedableRng};

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (k, set_k) = hooks.use_state(1.0);
        let max = 256;

        let (history, set_history) = hooks.use_state(FixedVecDeque::<[Vec2; 128]>::new());

        let clock = Clock::new();
        let points: (Vec<_>, Vec<_>, Vec<_>) = (0..max)
            .map(|v| {
                let v = v as f32 / max as f32;
                let x1 = v * 8.0 - 5.5;
                let y1 = (k * 0.1 + x1 * TAU).sin() * (k * x1).cos();

                let x2 = v * 32.0 - 5.0;
                let y2 = k * x2 * x2 * ((k + x2).sin());

                let x3 = v * 2.0;
                let y3 = (k * x3).exp();

                (vec2(x1, y1), vec2(x2, y2), vec2(x3, y3))
            })
            .multiunzip();

        let runtime = hooks.world.resource(runtime()).clone();
        {
            let mut history = history.clone();
            hooks.use_memo_with((), move |_, _| {
                runtime.spawn(async move {
                    log::info!("Spawning task");
                    let mut interval = tokio::time::interval(50.ms());

                    let mut upd = tokio::time::interval(250.ms());
                    let mut prev = 0.0;
                    let mut rng = StdRng::from_entropy();

                    loop {
                        tokio::select! {
                            _ = interval.tick() => {
                                let k = (clock.elapsed().as_secs_f32() * TAU / 4.0).sin() * 5.0;
                                set_k(k)

                            }
                            _ = upd.tick() => {
                                let t = clock.elapsed().as_secs_f32();
                                let v = prev + rng.gen_range(-1.0..1.0);

                                prev = v;

                                *history.push_front() = vec2(t, v);
                                set_history(history.clone());
                            }
                        }
                    }
                });
            });
        }

        ScrollArea(
            FlowColumn(vec![
                Graph {
                    points: points.0,
                    width: 800.0,
                    height: 200.0,
                    style: GraphStyle { color: vec4(1.0, 0.0, 0.0, 1.0), ..Default::default() },
                    x_scale: GraphScaleKind::Fixed { count: 8 },
                    y_bounds: Some((-2.0, 2.0)),
                    ..Default::default()
                }
                .el(),
                Graph {
                    points: points.1,
                    width: 600.0,
                    height: 200.0,
                    style: GraphStyle { color: vec4(1.0, 1.0, 0.0, 1.0), ..Default::default() },
                    ..Default::default()
                }
                .el(),
                Graph {
                    points: points.2,
                    width: 400.0,
                    height: 200.0,
                    style: GraphStyle { color: vec4(0.5, 0.0, 1.0, 1.0), ..Default::default() },
                    ..Default::default()
                }
                .el(),
                Graph {
                    points: history.iter().copied().collect_vec(),
                    width: 800.0,
                    height: 300.0,
                    x_scale: GraphScaleKind::Dynamic { spacing: 64.0, snap: false },
                    ..Default::default()
                }
                .el(),
            ])
            .el()
            .with(padding(), Borders::even(32.0))
            .with(space_between_items(), 128.0),
        )
        .el()
    }
}

async fn init(app: &mut App) {
    let world = &mut app.world;
    Example.el().spawn_interactive(world);
    UICamera.el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
