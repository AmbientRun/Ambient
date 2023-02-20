use std::sync::Arc;

use ambient_core::on_window_event;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_std::Cb;
use closure::closure;
use winit::event::{ElementState, WindowEvent};

use super::{Button, ButtonStyle, FlowColumn, FlowRow, Text, UIExt};
use crate::{
    border_radius,
    layout::{margin, Borders},
    padding, tooltip_background_color, Corners, Dropdown, SMALL_ROUNDING, STREET,
};

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
        let dropdown = Dropdown {
            content: Button::new(
                FlowRow(vec![content, Text::el("\u{f078}").set(margin(), Borders::left(5.))]).el(),
                closure!(clone set_show, |_| set_show(!show)),
            )
            .style(if inline { ButtonStyle::Inline } else { ButtonStyle::Regular })
            .el(),
            dropdown: FlowColumn(
                items
                    .into_iter()
                    .enumerate()
                    .map(move |(i, item)| {
                        Button::new(item, closure!(clone on_select, |_| { on_select.0(i); }))
                            .style(ButtonStyle::Card)
                            .el()
                            .set(margin(), Borders::top(if i != 0 { STREET } else { 0. }))
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
            .set(padding(), Borders::even(STREET))
            .set(border_radius(), Corners::even(SMALL_ROUNDING))
            .with_background(tooltip_background_color()),
            show,
        }
        .el();
        if show {
            dropdown.listener(
                on_window_event(),
                Arc::new(move |_, _, event| {
                    if let WindowEvent::MouseInput { state: ElementState::Released, .. } = event {
                        set_show(false);
                    }
                }),
            )
        } else {
            dropdown
        }
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
