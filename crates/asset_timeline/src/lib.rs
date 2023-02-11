use std::{collections::HashMap, sync::Arc};

use glam::{vec3, vec4, Vec4};
use itertools::Itertools;
use kiwi_core::{asset_cache, transform::translation};
use kiwi_ecs::World;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_renderer::color;
use kiwi_std::{
    asset_cache::{AssetKey, AssetLifetime, AssetTimeline, AssetsTimeline},
    color::Color,
    pretty_duration, to_byte_unit, Cb,
};
use kiwi_ui::{
    docking, fit_horizontal, height, margin, use_interval, width, Borders, Button, ButtonStyle, Dock, Docking, Editor, Fit, FlowColumn,
    FlowRow, Rectangle, StylesExt, Text, Tooltip, UIBase, UIExt, STREET,
};

#[derive(Debug, Clone)]
pub struct AssetTimelineVisualizer {
    pub timeline: AssetsTimeline,
}
impl ElementComponent for AssetTimelineVisualizer {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let total_count = self.timeline.assets.len();
        let (limit, set_limit) = hooks.use_state(Some(100));
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Sorting {
            CpuSize,
            GpuSize,
            Name,
        }
        let (sorting, set_sorting) = hooks.use_state(Sorting::CpuSize);
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Filter {
            Loading,
            Alive,
            Aborted,
            All,
        }
        let (filter, set_filter) = hooks.use_state(Filter::Loading);
        let mut roots: HashMap<AssetKey, (AssetTimeline, usize)> = self
            .timeline
            .assets
            .iter()
            .filter(|x| x.1.stack.is_empty())
            .map(|(k, a)| (k.clone(), (a.clone(), a.gpu_size.unwrap_or_default())))
            .collect();
        for asset in self.timeline.assets.values() {
            if !asset.stack.is_empty() {
                if let Some(node) = roots.get_mut(&asset.stack[0]) {
                    node.1 += asset.gpu_size.unwrap_or_default();
                }
            }
        }
        let total_roots_gpu_size = roots.values().map(|x| x.1).sum::<usize>();
        let timeline = Arc::new(self.timeline);
        let mut children = roots
            .into_iter()
            .filter(|(_, (value, _))| match filter {
                Filter::Loading => value.is_loading(),
                Filter::Aborted => value.is_aborted(),
                Filter::Alive => value.is_alive,
                Filter::All => true,
            })
            .sorted_by(|x, y| match sorting {
                Sorting::CpuSize => {
                    let res = x.1 .0.cpu_size.unwrap_or(0).cmp(&y.1 .0.cpu_size.unwrap_or(0)).reverse();
                    if res.is_eq() {
                        x.0.cmp(&y.0)
                    } else {
                        res
                    }
                }
                Sorting::GpuSize => {
                    let res = x.1 .1.cmp(&y.1 .1).reverse();
                    if res.is_eq() {
                        x.0.cmp(&y.0)
                    } else {
                        res
                    }
                }
                Sorting::Name => x.0.cmp(&y.0),
            })
            .take(limit.unwrap_or(total_count))
            .map(|(key, (value, total_gpu_size))| {
                AssetTimelineRow { key, value, timeline: timeline.clone(), padding: 0., total_gpu_size: Some(total_gpu_size) }.el()
            })
            .collect_vec();
        children.insert(
            0,
            FlowRow::el([
                FlowRow::el([
                    Button::new("Loading", {
                        let set_filter = set_filter.clone();
                        move |_| set_filter(Filter::Loading)
                    })
                    .toggled(filter == Filter::Loading)
                    .style(ButtonStyle::Flat)
                    .el(),
                    Button::new("Alive", {
                        let set_filter = set_filter.clone();
                        move |_| set_filter(Filter::Alive)
                    })
                    .toggled(filter == Filter::Alive)
                    .style(ButtonStyle::Flat)
                    .el(),
                    Button::new("Aborted", {
                        let set_filter = set_filter.clone();
                        move |_| set_filter(Filter::Aborted)
                    })
                    .toggled(filter == Filter::Aborted)
                    .style(ButtonStyle::Flat)
                    .el(),
                    Button::new(format!("All ({total_count})"), {
                        let set_filter = set_filter.clone();
                        move |_| set_filter(Filter::All)
                    })
                    .toggled(filter == Filter::All)
                    .style(ButtonStyle::Flat)
                    .el(),
                ]),
                FlowRow::el([
                    Button::new("Cpu size", {
                        let set_sorting = set_sorting.clone();
                        move |_| set_sorting(Sorting::CpuSize)
                    })
                    .toggled(sorting == Sorting::CpuSize)
                    .style(ButtonStyle::Flat)
                    .el(),
                    Button::new(format!("Gpu size ({})", to_byte_unit(total_roots_gpu_size)), {
                        let set_sorting = set_sorting.clone();
                        move |_| set_sorting(Sorting::GpuSize)
                    })
                    .toggled(sorting == Sorting::GpuSize)
                    .style(ButtonStyle::Flat)
                    .el(),
                    Button::new("Name", {
                        let set_sorting = set_sorting.clone();
                        move |_| set_sorting(Sorting::Name)
                    })
                    .toggled(sorting == Sorting::Name)
                    .style(ButtonStyle::Flat)
                    .el(),
                ]),
                FlowRow::el([Text::el("Limit:"), Option::<usize>::editor(limit, set_limit, Default::default())]),
            ])
            .keyboard(),
        );
        FlowColumn::el(children).set(fit_horizontal(), Fit::Parent)
    }
}

#[derive(Debug, Clone)]
struct AssetTimelineRow {
    key: AssetKey,
    value: AssetTimeline,
    timeline: Arc<AssetsTimeline>,
    padding: f32,
    total_gpu_size: Option<usize>,
}
impl ElementComponent for AssetTimelineRow {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { key, value, timeline, padding, total_gpu_size } = *self;
        let (expanded, set_expanded) = hooks.use_state(false);
        let key_text = Text::el(if key.len() > 30 { &key[0..30] } else { &key })
            .set(color(), if value.is_alive { Vec4::ONE } else { vec4(0.5, 0.5, 0.5, 1.) });
        FlowColumn::el([
            Dock(vec![
                Button::new(FlowRow::el([key_text]), move |_| set_expanded(!expanded))
                    .style(ButtonStyle::Flat)
                    .tooltip(format!(
                        "{}{}",
                        value.long_name,
                        if total_gpu_size.is_some() {
                            format!("\nSelf gpu size: {}", value.gpu_size.unwrap_or_default())
                        } else {
                            String::new()
                        }
                    ))
                    .el()
                    .set(width(), 200. - padding)
                    .set(margin(), Borders::left(padding))
                    .set(fit_horizontal(), Fit::None)
                    .set(docking(), Docking::Left),
                if let Some(cpu_size) = value.cpu_size { Text::el(to_byte_unit(cpu_size)) } else { UIBase.el() }
                    .set(width(), 100.)
                    .set(docking(), Docking::Left)
                    .set(margin(), Borders::left(STREET)),
                if let Some(gpu_size) = total_gpu_size.or(value.gpu_size) { Text::el(to_byte_unit(gpu_size)) } else { UIBase.el() }
                    .set(width(), 100.)
                    .set(docking(), Docking::Left)
                    .set(margin(), Borders::left(STREET)),
                AssetLifetimeViz { lifetimes: value.lifetimes }.el(),
            ])
            .el()
            .set(height(), 20.)
            .set(fit_horizontal(), Fit::Parent),
            if expanded {
                let mut stack = value.stack;
                stack.push(key);
                FlowColumn::el(
                    timeline
                        .assets
                        .iter()
                        .filter(|x| x.1.stack == stack)
                        .map(|(key, value)| {
                            AssetTimelineRow {
                                key: key.clone(),
                                value: value.clone(),
                                timeline: timeline.clone(),
                                padding: padding + STREET,
                                total_gpu_size: None,
                            }
                            .el()
                        })
                        .collect_vec(),
                )
            } else {
                Element::new()
            },
        ])
        .set(fit_horizontal(), Fit::Parent)
    }
}

#[derive(Debug, Clone)]
struct AssetLifetimeViz {
    lifetimes: Vec<AssetLifetime>,
}
impl ElementComponent for AssetLifetimeViz {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        let Self { lifetimes } = *self;
        let current_time = chrono::Utc::now();
        let time_scale = 0.001;
        let duration_to_width = |duration: chrono::Duration| duration.num_milliseconds() as f32 * time_scale;
        let time_to_x = |time: chrono::DateTime<chrono::Utc>| duration_to_width(current_time - time);
        UIBase.el().children(
            lifetimes
                .into_iter()
                .filter_map(|lifetime| {
                    if time_to_x(lifetime.end_time()) > 2000. {
                        return None;
                    }
                    let bar_height = 5.;
                    let mut children = vec![];
                    let mut tooltip = vec![];
                    if let Some(aborted_time) = lifetime.aborted {
                        let abort_time = aborted_time - lifetime.start_load;
                        let abort_width = duration_to_width(abort_time);
                        tooltip.push(
                            Text::el(format!("Aborted after: {}", pretty_duration(abort_time.to_std().unwrap())))
                                .set(color(), vec4(1., 0.5, 0.5, 1.)),
                        );
                        if abort_width >= 0.5 {
                            children.push(
                                Rectangle
                                    .el()
                                    .with_background(Color::rgb(1.0, 0.5, 0.5))
                                    .set(width(), abort_width)
                                    .set(height(), bar_height),
                            );
                        }
                    } else {
                        let end_load = if let Some(end_load) = lifetime.end_load { end_load } else { current_time };
                        let load_time = end_load - lifetime.start_load;
                        tooltip.push(
                            Text::el(format!("Load time: {}", pretty_duration(load_time.to_std().unwrap_or_default())))
                                .set(color(), vec4(0., 1., 0., 1.)),
                        );
                        let load_width = duration_to_width(load_time);
                        if load_width >= 0.5 {
                            children.push(
                                Rectangle.el().with_background(Color::rgb(0.0, 1., 0.0)).set(width(), load_width).set(height(), bar_height),
                            );
                        }
                        if let (Some(end_load), Some(k_start), k_end) =
                            (lifetime.end_load, lifetime.keepalive_start, lifetime.keepalive_end)
                        {
                            let k_end = k_end.unwrap_or(current_time);
                            // let keepalive_end = if !lifetime.keepalive {
                            //     end_load
                            // } else if let Some(keepalive_end) = lifetime.keepalive_end {
                            //     keepalive_end
                            // } else {
                            //     current_time
                            // };

                            let keepalive_dur = k_end - k_start;
                            let keepalive_width = duration_to_width(keepalive_dur);

                            tooltip.push(
                                Text::el(format!("Keepalive time: {}", pretty_duration(keepalive_dur.to_std().unwrap())))
                                    .set(color(), vec4(0.5, 0.5, 1., 1.)),
                            );

                            if keepalive_width >= 0.5 {
                                children.push(
                                    Rectangle
                                        .el()
                                        .with_background(Color::rgb(0.5, 0.5, 1.))
                                        .set(width(), keepalive_width)
                                        .set(height(), bar_height),
                                );
                            }

                            let lifetime_end = if let Some(dropped) = lifetime.dropped { dropped } else { current_time };
                            let alive_time = lifetime_end - k_end;
                            let total_alive_time = lifetime_end - end_load;
                            let alive_width = duration_to_width(alive_time);
                            tooltip.push(
                                Text::el(format!("Alive time: {}", pretty_duration(total_alive_time.to_std().unwrap())))
                                    .set(color(), vec4(1., 1., 1., 1.)),
                            );
                            if alive_width >= 0.5 {
                                children.push(
                                    Rectangle
                                        .el()
                                        .with_background(Color::rgb(1., 1., 1.))
                                        .set(width(), alive_width)
                                        .set(height(), bar_height),
                                );
                            }
                        } else if let Some(end_load) = lifetime.end_load {
                            // No keepalive started
                            let lifetime_end = if let Some(dropped) = lifetime.dropped { dropped } else { current_time };
                            let alive_time = lifetime_end - end_load;
                            let total_alive_time = lifetime_end - end_load;
                            let alive_width = duration_to_width(alive_time);
                            tooltip.push(
                                Text::el(format!("Alive time: {}", pretty_duration(total_alive_time.to_std().unwrap())))
                                    .set(color(), vec4(1., 1., 1., 1.)),
                            );
                            if alive_width >= 0.5 {
                                children.push(
                                    Rectangle
                                        .el()
                                        .with_background(Color::rgb(1., 1., 1.))
                                        .set(width(), alive_width)
                                        .set(height(), bar_height),
                                );
                            }
                        }
                    }
                    children.reverse();
                    Some(
                        Tooltip { inner: FlowRow::el(children), tooltip: FlowColumn::el(tooltip) }
                            .el()
                            .set(translation(), vec3(time_to_x(lifetime.end_time()), 0., 0.)),
                    )
                })
                .collect_vec(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct LocalAssetTimelineVisualizer;
impl ElementComponent for LocalAssetTimelineVisualizer {
    fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
        let (timeline, set_timeline) = hooks.use_state(AssetsTimeline::new());
        let assets = world.resource(asset_cache()).clone();
        use_interval(hooks, 1., move || {
            let timeline = assets.timeline.lock().clone();
            set_timeline(timeline);
        });
        AssetTimelineVisualizer { timeline }.el()
    }
}
