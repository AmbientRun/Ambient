use ambient_element::{consume_context, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_native_std::Cb;
use ambient_network::{client::ClientState, log_network_result};
use ambient_ui_native::{
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
        let Self {
            selection,
            set_selection: _,
        } = *self;
        let (client_state, _) = consume_context::<ClientState>(hooks).unwrap();
        let (settings, _) = consume_context::<EditorSettings>(hooks).unwrap();

        FlowColumn(vec![
            #[allow(clippy::comparison_chain)]
            if selection.len() == 1 {
                let _state = client_state.game_state.lock();

                EntityEditor { entity_id: selection.entities[0] }.el().with(fit_horizontal(), Fit::Parent)
            } else {
                Text::el(format!("{} entities", selection.len()))
            },
            if !selection.is_empty() && settings.debug_mode {
                Button::new_async(
                    "Toggle collider visualization",
                    closure!(clone selection, clone client_state, || {
                        let client_state = client_state.clone();
                        let selection = selection.iter().collect();
                        async move {
                            log_network_result!(client_state.rpc(rpc_toggle_visualize_colliders, selection).await);
                        }
                    }),
                )
                .el()
            } else {
                Element::new()
            },
        ])
        .el()
        .with(space_between_items(), STREET)
        .with(fit_horizontal(), Fit::None)
        .with(fit_vertical(), Fit::None)
        .with_clickarea()
        .el()
    }
}
