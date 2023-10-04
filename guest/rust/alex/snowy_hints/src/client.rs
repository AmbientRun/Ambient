use ambient_api::{
    core::{
        app::components::window_logical_size, text::components::font_size,
        transform::components::translation,
    },
    prelude::*,
};
use ambient_brand_theme::AmbientInternalStyle;
use packages::temperature::components::temperature;
use packages::this::components::*;

#[main]
pub fn main() {
    BigHintTextUI::el().spawn_interactive();

    entity::add_components(
        packages::this::entity(),
        Entity::new()
            .with(hint_message(), "Cold Chicken".into())
            .with(hint_show_progress(), 0.25)
            .with(hint_speed(), 1.),
    );

    // cold hint.
    ambient_api::core::messages::Frame::subscribe(|_| {
        if let Some(my_temp) = entity::get_component(player::get_local(), temperature()) {
            if my_temp < 32.22 && !entity::has_component(packages::this::entity(), hint_message()) {
                let mut yes_show_hint = false;
                if let Some(last_seen) =
                    entity::get_component(packages::this::entity(), last_seen_cold_hint())
                {
                    if last_seen > game_time().as_secs_f32() * 60. * 5. {
                        // 5 minutes between hints
                        yes_show_hint = true;
                    }
                } else {
                    yes_show_hint = true;
                }

                if yes_show_hint {
                    entity::add_components(
                        packages::this::entity(),
                        Entity::new()
                            .with(hint_message(), "It's too cold to travel alone.".into())
                            .with(last_seen_cold_hint(), game_time().as_secs_f32())
                            .with(hint_show_progress(), 0.)
                            .with(hint_speed(), 0.8),
                    );
                }
            }
        }
    });

    entity::add_component(packages::this::entity(), hint_show_progress(), 0.);

    query(hint_show_progress()).each_frame(|hints| {
        for (hint, progress) in hints {
            let speed = entity::get_component(hint, hint_speed()).unwrap_or(1.);
            let progress2 = progress + delta_time() * 0.15 * speed;
            if progress2 >= 1. {
                entity::remove_component(hint, hint_message());
                entity::remove_component(hint, hint_show_progress());
                entity::remove_component(hint, hint_speed());
            } else {
                entity::set_component(hint, hint_show_progress(), progress2);
            }
        }
    });

    // statue blessings
    query((
        packages::snowy_sceneloader::components::statue_name(),
        translation(),
    ))
    .each_frame(|statues| {
        if !entity::has_component(packages::this::entity(), hint_message()) {
            if let Some(myposition) = entity::get_component(player::get_local(), translation()) {
                for (statue, (statuename, statueposition)) in statues {
                    if myposition.distance(statueposition) < 10. {
                        let last_blessed =
                            entity::get_component(statue, last_blessed_local()).unwrap_or(-100.);
                        let now = game_time().as_secs_f32();
                        if now > last_blessed + 60.0 {
                            // you can only be blessed 1x per minute
                            entity::add_components(
                                packages::this::entity(),
                                Entity::new()
                                    .with(
                                        hint_message(),
                                        format!("Blessed by {statuename}.").replace("-", " "),
                                    )
                                    .with(last_seen_cold_hint(), game_time().as_secs_f32())
                                    .with(hint_show_progress(), 0.)
                                    .with(hint_speed(), 1.2),
                            );
                            entity::add_component(
                                packages::this::entity(),
                                blessings_announced(),
                                0,
                            );

                            if last_blessed < 0. {
                                entity::mutate_component_with_default(
                                    packages::this::entity(),
                                    blessings_to_announce(),
                                    1,
                                    |blessings| *blessings += 1,
                                );
                            }
                        }
                        entity::add_component(statue, last_blessed_local(), now);
                    }
                }
            }
        }
    });

    ambient_api::core::messages::Frame::subscribe(|_| {
        let hints = packages::this::entity();
        let (blessings, blessings2) = (
            entity::get_component(hints, blessings_announced()).unwrap_or(0),
            entity::get_component(hints, blessings_to_announce()).unwrap_or(0),
        );
        if blessings2 != blessings {
            if !entity::has_component(hints, hint_message()) {
                entity::add_component(hints, blessings_announced(), blessings2);
                entity::add_components(
                    packages::this::entity(),
                    Entity::new()
                        .with(
                            hint_message(),
                            match blessings2 {
                                0 => "(Blessings lost.)",
                                1 => "(One blessing.)",
                                2 => "(Two blessings.)",
                                3 => "(All three blessings.)",
                                _ => "(Lost count of blessings...)",
                            }
                            .to_string(),
                        )
                        .with(last_seen_cold_hint(), game_time().as_secs_f32())
                        .with(hint_show_progress(), 0.)
                        .with(hint_speed(), 1.5),
                );
            }
        }
    });

    let find_blessed_statues = query(()).requires(last_blessed_local()).build();

    spawn_query(())
        .requires(packages::snowy_pcs::components::dead_age())
        .bind(move |dead_plrs| {
            for (plr, _) in dead_plrs {
                if plr == player::get_local() {
                    // remove all blessings when local player dies
                    for (blessed_statue, _) in find_blessed_statues.evaluate() {
                        entity::remove_component(blessed_statue, last_blessed_local());
                    }
                    entity::add_component(packages::this::entity(), blessings_to_announce(), 0);
                }
            }
        });
}

// DISPLAYS HINTS
#[element_component]
pub fn BigHintTextUI(hooks: &mut Hooks) -> Element {
    if let Some(hint_show_progress) = ambient_api::element::use_entity_component(
        hooks,
        packages::this::entity(),
        hint_show_progress(),
    ) {
        if hint_show_progress > 0. {
            if let Some(hint_message) = ambient_api::element::use_entity_component(
                hooks,
                packages::this::entity(),
                hint_message(),
            ) {
                let screen_size =
                    entity::get_component(entity::resources(), window_logical_size()).unwrap();
                let fsize = screen_size.y as f32 * 0.08;
                let hint_words: Vec<&str> = hint_message.split(' ').collect();
                let visible_hint_word_count = (hint_words.len() as f32 * {
                    if hint_show_progress < 0.25 {
                        hint_show_progress * 4.
                    } else if hint_show_progress > 0.90 {
                        (1. - hint_show_progress) * 10.
                    } else {
                        1.
                    }
                })
                .round() as usize;
                let visible_hint = hint_words[0..visible_hint_word_count].join(" ");

                // hint_words.
                // for hint_word in hint_words {}

                return FlowColumn::el(visible_hint.split('\n').into_iter().map(
                    |visible_hint_line| {
                        Text::el(visible_hint_line)
                            .font_body_500()
                            .hex_text_color("#000000")
                            .with(font_size(), fsize)
                        // .with(max_width(), screen_size.x as f32)
                    },
                ))
                .with(width(), screen_size.x as f32)
                .with(height(), screen_size.y as f32)
                .with(fit_vertical(), Fit::None)
                .with(fit_horizontal(), Fit::None)
                .with(align_horizontal(), Align::Begin)
                .with(align_vertical(), Align::End)
                // clipping off the edge
                // .with(translation(), vec3(fsize * -0.10, fsize * 0.36, 0.))
                // no clipping
                .with(translation(), vec3(0., fsize * 0.31, 0.))
                .with(space_between_items(), fsize * -0.66);
            }
        }
    }

    return Element::new();
}
