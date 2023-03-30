use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};

use crate::{
    button::{Button, ButtonStyle},
    default_theme::{tooltip_background_color, SMALL_ROUNDING, STREET},
    dropdown::Dropdown,
    layout::{FlowColumn, FlowRow},
    text::Text,
    UIExt,
};
use ambient_cb::Cb;
use ambient_guest_bridge::components::{
    input::event_mouse_input,
    layout::{margin_left, margin_top},
    rect::border_radius,
};
use ambient_shared_types::events::WINDOW_MOUSE_INPUT;
use glam::Vec4;

#[derive(Debug, Clone)]
pub struct DropdownSelect {
    pub content: Element,
    pub on_select: Cb<dyn Fn(usize) + Sync + Send>,
    pub items: Vec<Element>,
    pub inline: bool,
}
impl ElementComponent for DropdownSelect {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { content, on_select, items, inline } = *self;
        let (show, set_show) = hooks.use_state(false);
        hooks.use_event(WINDOW_MOUSE_INPUT, {
            let set_show = set_show.clone();
            move |_world, event| {
                if let Some(pressed) = event.get(event_mouse_input()) {
                    if show && !pressed {
                        set_show(false);
                    }
                }
            }
        });
        Dropdown {
            content: Button::new(FlowRow(vec![content, Text::el("\u{f078}").with(margin_left(), 5.)]).el(), {
                let set_show = set_show.clone();
                move |_| set_show(!show)
            })
            .style(if inline { ButtonStyle::Inline } else { ButtonStyle::Regular })
            .el(),
            dropdown: FlowColumn(
                items
                    .into_iter()
                    .enumerate()
                    .map(move |(i, item)| {
                        Button::new(item, {
                            let on_select = on_select.clone();
                            move |_| {
                                on_select.0(i);
                            }
                        })
                        .style(ButtonStyle::Card)
                        .el()
                        .with(margin_top(), if i != 0 { STREET } else { 0. })
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
pub struct ListSelect {
    pub value: usize,
    pub on_change: Cb<dyn Fn(usize) + Sync + Send>,
    pub items: Vec<Element>,
    pub inline: bool,
}
impl ElementComponent for ListSelect {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self { value, on_change, items, inline } = *self;
        DropdownSelect {
            content: FlowRow(vec![if let Some(item) = items.get(value) { item.clone() } else { Text::el("-") }]).el(),
            on_select: on_change,
            items,
            inline,
        }
        .el()
    }
}
