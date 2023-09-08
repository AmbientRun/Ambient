//! Defines elements that can be used to select an item from a list.
use ambient_cb::Cb;
use ambient_element::{
    to_owned, use_runtime_message, use_state, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::{
    layout::components::margin, messages, rect::components::border_radius,
};
use glam::{vec4, Vec4};

use crate::{
    button::{Button, ButtonStyle},
    default_theme::{tooltip_background_color, SMALL_ROUNDING, STREET},
    dropdown::Dropdown,
    layout::{FlowColumn, FlowRow},
    text::Text,
    UIExt,
};

#[derive(Debug, Clone)]
/// A dropdown select element. Presents a button next to `content` that, when clicked, shows a dropdown with the items in `items`.
pub struct DropdownSelect {
    /// The content (always shown)
    pub content: Element,
    /// The callback to call when an item is selected. Called with the index of the item.
    pub on_select: Cb<dyn Fn(usize) + Sync + Send>,
    /// The items to select from.
    pub items: Vec<Element>,
    /// Whether or not the button used for the dropdown should be inline or not.
    pub inline: bool,
}
impl ElementComponent for DropdownSelect {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            content,
            on_select,
            items,
            inline,
        } = *self;
        let (show, set_show) = use_state(hooks, false);
        use_runtime_message::<messages::WindowMouseInput>(hooks, {
            to_owned![set_show];
            move |_world, event| {
                if show && !event.pressed {
                    set_show(false);
                }
            }
        });
        Dropdown {
            content: Button::new(
                FlowRow(vec![
                    content,
                    Text::el("\u{f078}").with(margin(), vec4(0., 0., 0., 5.)),
                ])
                .el(),
                {
                    to_owned![set_show];
                    move |_| set_show(!show)
                },
            )
            .style(if inline {
                ButtonStyle::Inline
            } else {
                ButtonStyle::Regular
            })
            .el(),
            dropdown: FlowColumn(
                items
                    .into_iter()
                    .enumerate()
                    .map(move |(i, item)| {
                        Button::new(item, {
                            to_owned![on_select];
                            move |_| {
                                on_select.0(i);
                            }
                        })
                        .style(ButtonStyle::Card)
                        .el()
                        .with(margin(), vec4(if i != 0 { STREET } else { 0. }, 0., 0., 0.))
                    })
                    .collect(), //     vec![Bookcase(
                                //     items
                                //         .into_iter()
                                //         .enumerate()
                                //         .map(move |(i, item)| BookFile {
                                //             container: Button::new(item, closure!(clone on_select, |_, _, _| { on_select.0(i); }))
                                //             .style(ButtonStyle::Card)
                                //                 .el()
                                //                 .set(margin(), Borders::even(5.)),
                                //             book: item,
                                //         })
                                //         .collect(),
                                // )
                                // .el()
                                // .set(orientation(), Orientation::Vertical)]
            )
            .el()
            .with_padding_even(STREET)
            .with(border_radius(), Vec4::ONE * SMALL_ROUNDING)
            .with_background(tooltip_background_color().into()),
            show,
        }
        .el()
    }
}

#[derive(Debug, Clone)]
/// A [DropdownSelect] that shows the current item for you automatically.
pub struct ListSelect {
    /// The index of the currently selected item.
    pub value: usize,
    /// The callback to call when an item is selected. Called with the index of the item.
    pub on_change: Cb<dyn Fn(usize) + Sync + Send>,
    /// The items to select from.
    pub items: Vec<Element>,
    /// Whether or not the button used for the dropdown should be inline or not.
    pub inline: bool,
}
impl ElementComponent for ListSelect {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self {
            value,
            on_change,
            items,
            inline,
        } = *self;
        DropdownSelect {
            content: FlowRow(vec![if let Some(item) = items.get(value) {
                item.clone()
            } else {
                Text::el("-")
            }])
            .el(),
            on_select: on_change,
            items,
            inline,
        }
        .el()
    }
}
