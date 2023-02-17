use std::fmt::Debug;

use closure::closure;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_std::{cb, Cb};

use crate::{space_between_items, Button, FlowColumn, FlowRow, STREET};

#[derive(Clone, Debug)]
pub struct TabBar<T: ToString + PartialEq + Clone + Debug + Sync + Send + 'static> {
    pub tabs: Vec<T>,
    pub value: T,
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}
impl<T: ToString + PartialEq + Clone + Debug + Sync + Send + 'static> ElementComponent for TabBar<T> {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self { tabs, value, on_change } = *self;
        FlowRow(
            tabs.into_iter()
                .map(|tab| {
                    Button::new(tab.to_string(), closure!(clone on_change, clone tab, |_| on_change.0(tab.clone())))
                        .toggled(tab == value)
                        .el()
                })
                .collect(),
        )
        .el()
        .set(space_between_items(), STREET)
    }
}

#[derive(Clone, Debug)]
pub struct Tabs<T: ToString + PartialEq + Default + Clone + Debug + Sync + Send + 'static> {
    pub tabs: Vec<(T, Cb<dyn Fn() -> Element + Sync + Send>)>,
}
impl<T: ToString + PartialEq + Default + Clone + Debug + Sync + Send + 'static> Tabs<T> {
    pub fn new() -> Self {
        Self { tabs: Default::default() }
    }

    pub fn with_tab(mut self, tab: T, callback: impl Fn() -> Element + Sync + Send + 'static) -> Self {
        self.tabs.push((tab, cb(callback)));
        self
    }
}
impl<T: ToString + PartialEq + Default + Clone + Debug + Sync + Send + 'static> ElementComponent for Tabs<T> {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (value, set_value) = hooks.use_state(T::default());
        let selected_tab = self.tabs.iter().find(|it| it.0 == value).map(|it| it.1.clone()).unwrap_or(cb(|| Element::new()));
        let key = value.to_string();

        FlowColumn::el([
            TabBar { tabs: self.tabs.iter().map(|it| it.0.clone()).collect(), value, on_change: cb(move |value| set_value(value)) }.el(),
            selected_tab().key(key),
        ])
        .set(space_between_items(), STREET)
    }
}
