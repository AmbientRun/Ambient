#[cfg(feature = "native")]
use ambient_sys::time::SystemTime;
use std::time::Duration;
#[cfg(feature = "guest")]
use std::time::SystemTime;

use ambient_cb::{cb, Cb};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{layout::space_between_items, rendering::color};
use ambient_time::parse_duration;
use glam::vec4;

use crate::{layout::FlowRow, text::Text};

use super::{ChangeCb, Editor, EditorOpts, TextEditor};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
/// A duration that can be edited.
pub struct EditableDuration {
    dur: Duration,
    valid: bool,
    input: String,
}

impl EditableDuration {
    /// Create a new [EditableDuration].
    pub fn new(dur: Duration, valid: bool, input: String) -> Self {
        Self { dur, valid, input }
    }

    /// Get the duration.
    pub fn dur(&self) -> Duration {
        self.dur
    }
}

impl From<Duration> for EditableDuration {
    fn from(v: Duration) -> Self {
        Self { dur: v, input: format!("{}s", v.as_secs()), valid: true }
    }
}

impl From<EditableDuration> for Duration {
    fn from(v: EditableDuration) -> Self {
        v.dur
    }
}

impl From<&EditableDuration> for Duration {
    fn from(v: &EditableDuration) -> Self {
        v.dur
    }
}

impl From<String> for EditableDuration {
    fn from(s: String) -> Self {
        let dur = parse_duration(&s);
        let valid = dur.is_ok();
        Self { dur: dur.unwrap_or_default(), valid, input: s }
    }
}

#[derive(Debug, Clone)]
/// An editor for [EditableDuration].
pub struct DurationEditor {
    /// The value to edit.
    pub value: EditableDuration,
    /// Callback for when the value changes.
    pub on_change: Cb<dyn Fn(EditableDuration) + Sync + Send>,
}

impl DurationEditor {
    /// Create a new [DurationEditor].
    pub fn new(value: EditableDuration, on_change: Cb<dyn Fn(EditableDuration) + Sync + Send>) -> Self {
        Self { value, on_change }
    }
}

impl ElementComponent for DurationEditor {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self { value: EditableDuration { input, dur, valid }, on_change } = *self;
        let input = TextEditor::new(input, cb(move |upd: String| on_change(EditableDuration::from(upd)))).el();
        let value = Text::el(format!("{dur:#?}"));

        if valid {
            FlowRow(vec![input, value]).el().with(space_between_items(), 10.0)
        } else {
            FlowRow(vec![input, Text::el("invalid duration").with(color(), vec4(1.0, 0.0, 0.0, 1.0))])
                .el()
                .with(space_between_items(), 10.0)
        }
    }
}

impl Editor for EditableDuration {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        DurationEditor::new(self, on_change).el()
    }
}

#[derive(Debug, Clone)]
/// An editor for [SystemTime].
///
/// At present, this does not support editing and is purely display-only.
pub struct SystemTimeEditor {
    /// The value to edit.
    pub value: SystemTime,
    /// Callback for when the value changes.
    pub on_change: Option<Cb<dyn Fn(SystemTime) + Sync + Send>>,
}
impl ElementComponent for SystemTimeEditor {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Text::el(format!("{:?}", self.value))
    }
}
impl Editor for SystemTime {
    fn editor(self, _: ChangeCb<Self>, _: EditorOpts) -> Element {
        SystemTimeEditor { value: self, on_change: None }.el()
    }
}
