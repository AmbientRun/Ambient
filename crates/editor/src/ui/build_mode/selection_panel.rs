use kiwi_ecs::World;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_network::{client::GameClient, log_network_result};
use kiwi_std::Cb;
use kiwi_ui::{
    layout::{fit_horizontal, fit_vertical, space_between_items, Fit},
    Button, FlowColumn, Text, UIExt, STREET,
};

use super::super::entity_editor::EntityEditor;
use crate::{rpc::rpc_toggle_visualize_colliders, ui::EditorSettings, Selection};

#[derive(Debug, Clone)]
pub struct SelectionPanel {
    pub selection: Selection,
    pub set_selection: Cb<dyn Fn(Selection) + Sync + Send>,
}

impl ElementComponent for SelectionPanel {
    #[profiling::function]
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { selection, set_selection: _ } = *self;
        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
        let (settings, _) = hooks.consume_context::<EditorSettings>().unwrap();

        FlowColumn(vec![
            #[allow(clippy::comparison_chain)]
            if selection.len() == 1 {
                let _state = game_client.game_state.lock();

                EntityEditor { entity_id: selection.entities[0] }.el().set(fit_horizontal(), Fit::Parent)
            } else {
                Text::el(format!("{} entities", selection.len()))
            },
            if !selection.is_empty() && settings.debug_mode {
                Button::new_async(
                    "Toggle collider visualization",
                    closure!(clone selection, clone game_client, || {
                        let game_client = game_client.clone();
                        let selection = selection.iter().collect();
                        async move {
                            log_network_result!(game_client.rpc(rpc_toggle_visualize_colliders, selection).await);
                        }
                    }),
                )
                .el()
            } else {
                Element::new()
            },
        ])
        .el()
        .set(space_between_items(), STREET)
        .set(fit_horizontal(), Fit::None)
        .set(fit_vertical(), Fit::None)
        .with_clickarea()
    }
}
