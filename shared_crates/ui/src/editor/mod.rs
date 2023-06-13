//! Provides an [Editor] trait for values that can be edited in the UI, and implementations for common types.

mod collections;
mod primitives;
mod screens;
mod slider;
mod text_editor;
mod time;
use std::{ops::Deref, sync::Arc};

use ambient_cb::{cb, Cb};
use ambient_element::{to_owned, Element, ElementComponentExt};
pub use collections::*;
use parking_lot::Mutex;
pub use primitives::*;
pub use screens::*;
pub use slider::*;
pub use text_editor::*;
pub use time::*;

use crate::{
    button::{Button, ButtonStyle},
    layout::FlowRow,
    text::Text,
};

#[derive(Clone, Debug)]
/// Options for the [Editor] type.
pub struct EditorOpts {
    /// If true, enums can be changed to a different variant.
    pub enum_can_change_type: bool,
}

impl Default for EditorOpts {
    fn default() -> Self {
        Self {
            enum_can_change_type: true,
        }
    }
}

/// Callback for when an editor changes its value.
pub type ChangeCb<T> = Cb<dyn Fn(T) + Sync + Send>;

/// A trait for types that can be edited in the UI.
///
/// Implementing this trait allows you to make a UI editor for any value of your type.
pub trait Editor {
    /// Create an editor [Element] for this value.
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element;
    /// Creates an editor [Element] for this value, or a view [Element] if `on_change` is `None`.
    fn edit_or_view(self, on_change: Option<ChangeCb<Self>>, opts: EditorOpts) -> Element
    where
        Self: Sized,
    {
        if let Some(on_change) = on_change {
            self.editor(on_change, opts)
        } else {
            self.view(opts)
        }
    }
    /// Viewer for this value.
    ///
    /// By default, this will call [Editor::editor] with a no-op callback.
    fn view(self, opts: EditorOpts) -> Element
    where
        Self: Sized,
    {
        self.editor(cb(|_| {}), opts)
    }
}

impl<T: Editor + 'static> Editor for Box<T> {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        T::editor(
            *self,
            cb(move |new_value| (on_change)(Box::new(new_value))),
            opts,
        )
    }
}

impl<T> Editor for Arc<T>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        T::editor(
            self.deref().clone(),
            cb(move |v: T| on_change(Arc::new(v))) as Cb<dyn Fn(T) + Sync + Send>,
            opts,
        )
    }
}

impl<T> Editor for Arc<Mutex<T>>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        let v: T = self.lock().clone();
        T::editor(
            v,
            cb(move |v: T| {
                // Update the shared value
                *self.lock() = v;
                // Give the same value which is now internally mutated
                on_change(self.clone())
            }),
            opts,
        )
    }
}

impl Editor for () {
    fn editor(self, _on_change: ChangeCb<Self>, _opts: EditorOpts) -> Element {
        Element::new()
    }
}

impl<T: Default + Editor + 'static> Editor for Option<T> {
    fn editor(self, on_change: ChangeCb<Self>, opts: EditorOpts) -> Element {
        if let Some(inner_value) = self {
            FlowRow(vec![
                Button::new("\u{f056}", {
                    to_owned![on_change];
                    move |_| on_change.0(None)
                })
                .style(ButtonStyle::Flat)
                .el(),
                T::editor(
                    inner_value,
                    cb({
                        to_owned![on_change];
                        move |value| on_change.0(Some(value))
                    }),
                    opts,
                ),
            ])
            .el()
        } else {
            Button::new("\u{f055}", {
                to_owned![on_change];
                move |_| on_change.0(Some(T::default()))
            })
            .style(ButtonStyle::Flat)
            .el()
        }
    }

    fn view(self, opts: EditorOpts) -> Element
    where
        Self: Sized,
    {
        if let Some(value) = self {
            T::view(value, opts)
        } else {
            Text::el("None")
        }
    }
}
