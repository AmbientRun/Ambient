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
            if my_temp < 32.22 {
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
                .floor() as usize;
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
