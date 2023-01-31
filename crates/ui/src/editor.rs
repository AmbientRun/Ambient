use std::{ops::Deref, sync::Arc};

use elements_element::{Element, ElementComponentExt};
use elements_std::{time::Timeout, Cb};
use parking_lot::Mutex;

use crate::{Button, ButtonStyle, DurationEditor, EditableDuration, FlowRow, Text};

#[derive(Clone, Debug)]
pub struct EditorOpts {
    pub enum_can_change_type: bool,
}

impl Default for EditorOpts {
    fn default() -> Self {
        Self { enum_can_change_type: true }
    }
}

pub trait Editor {
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element;
}

impl Editor for EditableDuration {
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        DurationEditor::new(self, on_change.unwrap_or_else(|| Cb(Arc::new(|_| {})))).el()
    }
}

impl<T: Editor + 'static> Editor for Box<T> {
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        T::editor(*self, on_change.map(|cb| Cb(Arc::new(move |new_value| cb.0(Box::new(new_value))) as Arc<dyn Fn(T) + Sync + Send>)), opts)
    }
}

impl<T> Editor for Arc<T>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        T::editor(self.deref().clone(), on_change.map(|f| Cb::new(move |v: T| f(Arc::new(v))) as Cb<dyn Fn(T) + Sync + Send>), opts)
    }
}

impl<T> Editor for Arc<Mutex<T>>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        let v: T = self.lock().clone();
        T::editor(
            v,
            on_change.map(|f| {
                Cb::new(move |v: T| {
                    // Update the shared value
                    *self.lock() = v;
                    // Give the same value which is now internally mutated
                    f(self.clone())
                }) as Cb<dyn Fn(T) + Sync + Send>
            }),
            opts,
        )
    }
}
impl Editor for () {
    fn editor(self, _on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _opts: EditorOpts) -> Element {
        Element::new()
    }
}

impl Editor for Timeout {
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        let on_change = on_change.unwrap_or_else(|| Cb::new(|_| {}));

        DurationEditor::new(
            EditableDuration::new(self.duration(), true, self.duration().as_secs().to_string()),
            Cb::new(move |v| (on_change)(Timeout::new(v.dur()))),
        )
        .el()
    }
}

impl<T: Default + Editor + 'static> Editor for Option<T> {
    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            if let Some(inner_value) = self {
                FlowRow(vec![
                    Button::new("\u{f056}", closure!(clone on_change, |_| on_change.0(None))).style(ButtonStyle::Flat).el(),
                    T::editor(inner_value, Some(Cb(Arc::new(closure!(clone on_change, |value| on_change.0(Some(value)))))), opts),
                ])
                .el()
            } else {
                Button::new("\u{f055}", closure!(clone on_change, |_| on_change.0(Some(T::default())))).style(ButtonStyle::Flat).el()
            }
        } else if let Some(value) = self {
            T::editor(value, None, opts)
        } else {
            Text::el("None")
        }
    }
}
