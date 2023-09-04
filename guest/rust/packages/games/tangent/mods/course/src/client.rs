use ambient_api::prelude::*;
use packages::this::{components::score, messages::CheckpointHit};

#[main]
pub fn main() {
    Hud.el().spawn_interactive();
}

#[element_component]
fn Hud(hooks: &mut Hooks) -> Element {
    let (score, _) = hooks.use_entity_component(player::get_local(), score());
    let (bounce, set_bounce) = hooks.use_state(false);
    let (bounce_amount, set_bounce_amount) = hooks.use_state(0.);

    hooks.use_module_message::<CheckpointHit>(move |_, _, _| {
        set_bounce(true);

        let set_bounce = set_bounce.clone();
        run_async(async move {
            sleep(1.0).await;
            set_bounce(false);
        });
    });

    hooks.use_frame(move |_| {
        if bounce {
            set_bounce_amount((game_time().as_secs_f32() * 5.).sin() * 10.);
        } else {
            set_bounce_amount(0.);
        }
    });

    WindowSized::el([Dock::el([Text::el(format!(
        "Score: {}",
        score.unwrap_or_default()
    ))
    .header_style()
    .with(docking(), Docking::Bottom)
    .with_margin_even(10. + bounce_amount)])])
    .with_padding_even(20.)
}
