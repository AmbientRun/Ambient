use ambient_api::prelude::*;

#[main]
pub fn main() {
    App.el().spawn_interactive();
}

#[element_component]
pub fn App(hooks: &mut Hooks) -> Element {
    // let (players, set_players) = hooks.use_state(vec![]);

    // change_query((anim_character(), health(), components::death_count())).track_change((health(), components::death_count())).bind(move |players| {
    //     let mut v = vec![];
    //     // this id is actually the model id, thus model health :P
    //     for (id, (_, health, death_count)) in players {
    //         println!("changes detected in players! {:?} {:?}", health, death_count);
    //         v.push((id, health, death_count));
    //     }
    //     set_players(v);
    // });

    let size_info = hooks.use_query(window_logical_size());
    for (resource_id, xy) in &size_info {
        println!("window size change: {:?} {:?}", resource_id, xy);
    }
    let center_x = size_info[0].1.x as f32 / 2.;
    let center_y = size_info[0].1.y as f32 / 2.;

    // let zombie_health_list = hooks.use_query(crate::components::zombie_health());
    // println!("zombie health list: {:?}", zombie_health_list);
    // for (zombie_id, health) in &zombie_health_list {
    //     println!("zombie health: {:?} {:?}", zombie_id, health);
    // }

    Group::el([
        // FocusRoot::el([FlowColumn::el(
        //     zombie_health_list.iter().map(|(id, health)| {
        //             FlowRow::el([
        //                 Text::el(format!("Player {}", id)),
        //                 Text::el(format!("Health: {}", health)),
        //             ])
        //     }).collect::<Vec<_>>()
        // )])
        // .with_background(vec4(0., 0., 0., 0.9))
        // .with_padding_even(10.),
        Line.el()
            .with(line_from(), vec3(center_x - 10., center_y, 0.))
            .with(line_to(), vec3(center_x + 10., center_y, 0.))
            .with(line_width(), 2.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
        Line.el()
            .with(line_from(), vec3(center_x, center_y - 10., 0.))
            .with(line_to(), vec3(center_x, center_y + 10., 0.))
            .with(line_width(), 2.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
    ])
}
