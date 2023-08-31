use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use crate::{
    core::messages::WindowFocusChange,
    global::{CursorIcon, Vec2},
    internal::{
        conversion::{FromBindgen, IntoBindgen},
        wit,
    },
    message::Listener,
    prelude::RuntimeMessage,
};

pub use ambient_shared_types::MouseButton;

/// Gets the local player's most recent raw input state.
///
/// To determine if the player just supplied an input, compare it to [get_previous] or use [get_delta].
pub fn get() -> Input {
    wit::client_input::get().from_bindgen()
}

/// Gets the local player's raw input state prior to the most recent update.
pub fn get_previous() -> Input {
    wit::client_input::get_previous().from_bindgen()
}

/// Gets the changes to the local player's raw input state in the last update,
/// as well as the current raw input state.
///
/// This is a wrapper for [get_previous], [get] and [Input::delta].
pub fn get_delta() -> (InputDelta, Input) {
    let (p, c) = (get_previous(), get());
    (c.delta(&p), c)
}

/// Sets the cursor icon.
pub fn set_cursor(icon: CursorIcon) {
    wit::client_input::set_cursor(icon.into_bindgen());
}

/// Sets the cursor's visibility.
pub fn set_cursor_visible(visible: bool) {
    wit::client_input::set_cursor_visible(visible);
}

/// Sets the cursor's lock state. If set, the cursor will not be able to move outside of the window.
///
/// You may want to combine this with [set_cursor_visible] or use [CursorLockGuard] instead.
pub fn set_cursor_lock(locked: bool) {
    wit::client_input::set_cursor_lock(locked);
}

/// Helper utility that will lock and hide the cursor if necessary.
///
/// Will unlock the cursor when dropped.
pub struct CursorLockGuard {
    inner: Arc<Mutex<CursorLockGuardInner>>,
    listener: Listener,
}
impl CursorLockGuard {
    /// Creates a new [CursorLockGuard] with the given lock state.
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(CursorLockGuardInner::new()));
        Self {
            inner: inner.clone(),
            listener: WindowFocusChange::subscribe(move |msg| {
                inner.lock().unwrap().set_locked(msg.focused)
            }),
        }
    }

    /// Locks and hides the cursor if necessary.
    pub fn set_locked(&mut self, locked: bool) {
        self.inner.lock().unwrap().set_locked(locked);
    }

    /// Returns whether or not the cursor is currently locked.
    pub fn is_locked(&self) -> bool {
        self.inner.lock().unwrap().is_locked()
    }

    /// Helper that calls [Self::set_locked] with `true`.
    pub fn lock(&mut self) {
        self.set_locked(true);
    }

    /// Helper that calls [Self::set_locked] with `false`.
    pub fn unlock(&mut self) {
        self.set_locked(false);
    }

    /// Helper that will unlock if `Escape` has been pressed, and lock if anything else is pressed.
    ///
    /// Returns whether or not the cursor is currently locked.
    pub fn auto_unlock_on_escape(&mut self, input: &Input) -> bool {
        if input.keys.contains(&KeyCode::Escape) {
            self.unlock();
        } else if !input.keys.is_empty() || !input.mouse_buttons.is_empty() {
            self.lock();
        }
        self.is_locked()
    }
}
impl Default for CursorLockGuard {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for CursorLockGuard {
    fn drop(&mut self) {
        self.set_locked(false);
        self.listener.stop();
    }
}

struct CursorLockGuardInner {
    locked: bool,
}
impl CursorLockGuardInner {
    fn new() -> Self {
        Self { locked: false }
    }

    /// Locks and hides the cursor if necessary.
    fn set_locked(&mut self, locked: bool) {
        if locked != self.locked {
            set_cursor_lock(locked);
            set_cursor_visible(!locked);
        }
        self.locked = locked;
    }

    /// Returns whether or not the cursor is currently locked.
    fn is_locked(&self) -> bool {
        self.locked
    }
}

#[allow(missing_docs)]
/// The code associated with a key on the keyboard.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl FromBindgen for wit::client_input::VirtualKeyCode {
    type Item = KeyCode;

    fn from_bindgen(self) -> Self::Item {
        match self {
            Self::Key1 => Self::Item::Key1,
            Self::Key2 => Self::Item::Key2,
            Self::Key3 => Self::Item::Key3,
            Self::Key4 => Self::Item::Key4,
            Self::Key5 => Self::Item::Key5,
            Self::Key6 => Self::Item::Key6,
            Self::Key7 => Self::Item::Key7,
            Self::Key8 => Self::Item::Key8,
            Self::Key9 => Self::Item::Key9,
            Self::Key0 => Self::Item::Key0,
            Self::A => Self::Item::A,
            Self::B => Self::Item::B,
            Self::C => Self::Item::C,
            Self::D => Self::Item::D,
            Self::E => Self::Item::E,
            Self::F => Self::Item::F,
            Self::G => Self::Item::G,
            Self::H => Self::Item::H,
            Self::I => Self::Item::I,
            Self::J => Self::Item::J,
            Self::K => Self::Item::K,
            Self::L => Self::Item::L,
            Self::M => Self::Item::M,
            Self::N => Self::Item::N,
            Self::O => Self::Item::O,
            Self::P => Self::Item::P,
            Self::Q => Self::Item::Q,
            Self::R => Self::Item::R,
            Self::S => Self::Item::S,
            Self::T => Self::Item::T,
            Self::U => Self::Item::U,
            Self::V => Self::Item::V,
            Self::W => Self::Item::W,
            Self::X => Self::Item::X,
            Self::Y => Self::Item::Y,
            Self::Z => Self::Item::Z,
            Self::Escape => Self::Item::Escape,
            Self::F1 => Self::Item::F1,
            Self::F2 => Self::Item::F2,
            Self::F3 => Self::Item::F3,
            Self::F4 => Self::Item::F4,
            Self::F5 => Self::Item::F5,
            Self::F6 => Self::Item::F6,
            Self::F7 => Self::Item::F7,
            Self::F8 => Self::Item::F8,
            Self::F9 => Self::Item::F9,
            Self::F10 => Self::Item::F10,
            Self::F11 => Self::Item::F11,
            Self::F12 => Self::Item::F12,
            Self::F13 => Self::Item::F13,
            Self::F14 => Self::Item::F14,
            Self::F15 => Self::Item::F15,
            Self::F16 => Self::Item::F16,
            Self::F17 => Self::Item::F17,
            Self::F18 => Self::Item::F18,
            Self::F19 => Self::Item::F19,
            Self::F20 => Self::Item::F20,
            Self::F21 => Self::Item::F21,
            Self::F22 => Self::Item::F22,
            Self::F23 => Self::Item::F23,
            Self::F24 => Self::Item::F24,
            Self::Snapshot => Self::Item::Snapshot,
            Self::Scroll => Self::Item::Scroll,
            Self::Pause => Self::Item::Pause,
            Self::Insert => Self::Item::Insert,
            Self::Home => Self::Item::Home,
            Self::Delete => Self::Item::Delete,
            Self::End => Self::Item::End,
            Self::PageDown => Self::Item::PageDown,
            Self::PageUp => Self::Item::PageUp,
            Self::Left => Self::Item::Left,
            Self::Up => Self::Item::Up,
            Self::Right => Self::Item::Right,
            Self::Down => Self::Item::Down,
            Self::Back => Self::Item::Back,
            Self::Return => Self::Item::Return,
            Self::Space => Self::Item::Space,
            Self::Compose => Self::Item::Compose,
            Self::Caret => Self::Item::Caret,
            Self::Numlock => Self::Item::Numlock,
            Self::Numpad0 => Self::Item::Numpad0,
            Self::Numpad1 => Self::Item::Numpad1,
            Self::Numpad2 => Self::Item::Numpad2,
            Self::Numpad3 => Self::Item::Numpad3,
            Self::Numpad4 => Self::Item::Numpad4,
            Self::Numpad5 => Self::Item::Numpad5,
            Self::Numpad6 => Self::Item::Numpad6,
            Self::Numpad7 => Self::Item::Numpad7,
            Self::Numpad8 => Self::Item::Numpad8,
            Self::Numpad9 => Self::Item::Numpad9,
            Self::NumpadAdd => Self::Item::NumpadAdd,
            Self::NumpadDivide => Self::Item::NumpadDivide,
            Self::NumpadDecimal => Self::Item::NumpadDecimal,
            Self::NumpadComma => Self::Item::NumpadComma,
            Self::NumpadEnter => Self::Item::NumpadEnter,
            Self::NumpadEquals => Self::Item::NumpadEquals,
            Self::NumpadMultiply => Self::Item::NumpadMultiply,
            Self::NumpadSubtract => Self::Item::NumpadSubtract,
            Self::AbntC1 => Self::Item::AbntC1,
            Self::AbntC2 => Self::Item::AbntC2,
            Self::Apostrophe => Self::Item::Apostrophe,
            Self::Apps => Self::Item::Apps,
            Self::Asterisk => Self::Item::Asterisk,
            Self::At => Self::Item::At,
            Self::Ax => Self::Item::Ax,
            Self::Backslash => Self::Item::Backslash,
            Self::Calculator => Self::Item::Calculator,
            Self::Capital => Self::Item::Capital,
            Self::Colon => Self::Item::Colon,
            Self::Comma => Self::Item::Comma,
            Self::Convert => Self::Item::Convert,
            Self::Equals => Self::Item::Equals,
            Self::Grave => Self::Item::Grave,
            Self::Kana => Self::Item::Kana,
            Self::Kanji => Self::Item::Kanji,
            Self::LAlt => Self::Item::LAlt,
            Self::LBracket => Self::Item::LBracket,
            Self::LControl => Self::Item::LControl,
            Self::LShift => Self::Item::LShift,
            Self::LWin => Self::Item::LWin,
            Self::Mail => Self::Item::Mail,
            Self::MediaSelect => Self::Item::MediaSelect,
            Self::MediaStop => Self::Item::MediaStop,
            Self::Minus => Self::Item::Minus,
            Self::Mute => Self::Item::Mute,
            Self::MyComputer => Self::Item::MyComputer,
            Self::NavigateForward => Self::Item::NavigateForward,
            Self::NavigateBackward => Self::Item::NavigateBackward,
            Self::NextTrack => Self::Item::NextTrack,
            Self::NoConvert => Self::Item::NoConvert,
            Self::Oem102 => Self::Item::OEM102,
            Self::Period => Self::Item::Period,
            Self::PlayPause => Self::Item::PlayPause,
            Self::Plus => Self::Item::Plus,
            Self::Power => Self::Item::Power,
            Self::PrevTrack => Self::Item::PrevTrack,
            Self::RAlt => Self::Item::RAlt,
            Self::RBracket => Self::Item::RBracket,
            Self::RControl => Self::Item::RControl,
            Self::RShift => Self::Item::RShift,
            Self::RWin => Self::Item::RWin,
            Self::Semicolon => Self::Item::Semicolon,
            Self::Slash => Self::Item::Slash,
            Self::Sleep => Self::Item::Sleep,
            Self::Stop => Self::Item::Stop,
            Self::Sysrq => Self::Item::Sysrq,
            Self::Tab => Self::Item::Tab,
            Self::Underline => Self::Item::Underline,
            Self::Unlabeled => Self::Item::Unlabeled,
            Self::VolumeDown => Self::Item::VolumeDown,
            Self::VolumeUp => Self::Item::VolumeUp,
            Self::Wake => Self::Item::Wake,
            Self::WebBack => Self::Item::WebBack,
            Self::WebFavorites => Self::Item::WebFavorites,
            Self::WebForward => Self::Item::WebForward,
            Self::WebHome => Self::Item::WebHome,
            Self::WebRefresh => Self::Item::WebRefresh,
            Self::WebSearch => Self::Item::WebSearch,
            Self::WebStop => Self::Item::WebStop,
            Self::Yen => Self::Item::Yen,
            Self::Copy => Self::Item::Copy,
            Self::Paste => Self::Item::Paste,
            Self::Cut => Self::Item::Cut,
        }
    }
}

impl FromBindgen for wit::client_input::MouseButton {
    type Item = MouseButton;

    fn from_bindgen(self) -> Self::Item {
        match self {
            Self::Left => Self::Item::Left,
            Self::Right => Self::Item::Right,
            Self::Middle => Self::Item::Middle,
            Self::Other(id) => Self::Item::Other(id),
        }
    }
}

/// The state of a player's raw input. Get these with [get] or [get_previous].
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    /// All of the keys being pressed this frame.
    pub keys: HashSet<KeyCode>,
    /// The current position of the mouse.
    pub mouse_position: Vec2,
    /// The movement of the mouse since the last frame. Note that this is not affected by cursor locking, unlike [Self::mouse_position].
    ///
    /// Use this for any kind of movement that should be relative to the mouse's position, such as camera rotation.
    pub mouse_delta: Vec2,
    /// The current scroll position.
    pub mouse_wheel: f32,
    /// All of the mouse buttons being pressed this frame.
    pub mouse_buttons: HashSet<MouseButton>,
}

impl FromBindgen for wit::client_input::Input {
    type Item = Input;
    fn from_bindgen(self) -> Self::Item {
        Self::Item {
            keys: self.keys.into_iter().map(|k| k.from_bindgen()).collect(),
            mouse_position: self.mouse_position.from_bindgen(),
            mouse_delta: self.mouse_delta.from_bindgen(),
            mouse_wheel: self.mouse_wheel,
            mouse_buttons: self
                .mouse_buttons
                .into_iter()
                .map(|b| b.from_bindgen())
                .collect(),
        }
    }
}

/// The changes between the player's input state this update ([get]) and their input state
/// last update ([get_previous]). Get this with [get_delta] or [Input::delta].
#[derive(Clone, Debug, PartialEq)]
pub struct InputDelta {
    /// All of the keys that were pressed this frame, but not last frame.
    pub keys: HashSet<KeyCode>,
    /// All of the keys that were released this frame.
    pub keys_released: HashSet<KeyCode>,
    /// The change between last frame's mouse position and this frame.
    ///
    /// Note that this is equal to [Input::mouse_delta], not the delta of [Input::mouse_position].
    pub mouse_position: Vec2,
    /// The amount the mouse wheel has scrolled since the last frame.
    pub mouse_wheel: f32,
    /// All of the mouse buttons that were pressed this frame, but not last frame.
    pub mouse_buttons: HashSet<MouseButton>,
    /// All of the mouse buttons that were released this frame.
    pub mouse_buttons_released: HashSet<MouseButton>,
}

impl Input {
    /// Returns whether or not each input has changed from `previous` to this [Input].
    pub fn delta(&self, previous: &Input) -> InputDelta {
        let (p, c) = (previous, self);

        InputDelta {
            keys: &c.keys - &p.keys,
            keys_released: &p.keys - &c.keys,
            mouse_position: c.mouse_delta,
            mouse_wheel: c.mouse_wheel - p.mouse_wheel,
            mouse_buttons: &c.mouse_buttons - &p.mouse_buttons,
            mouse_buttons_released: &p.mouse_buttons - &c.mouse_buttons,
        }
    }
}

impl IntoBindgen for CursorIcon {
    type Item = wit::client_input::CursorIcon;
    fn into_bindgen(self) -> Self::Item {
        type Ci = CursorIcon;
        type Wci = wit::client_input::CursorIcon;

        match self {
            Ci::Default => Wci::DefaultIcon,
            Ci::Crosshair => Wci::Crosshair,
            Ci::Hand => Wci::Hand,
            Ci::Arrow => Wci::Arrow,
            Ci::Move => Wci::Move,
            Ci::Text => Wci::Text,
            Ci::Wait => Wci::Wait,
            Ci::Help => Wci::Help,
            Ci::Progress => Wci::Progress,

            Ci::NotAllowed => Wci::NotAllowed,
            Ci::ContextMenu => Wci::ContextMenu,
            Ci::Cell => Wci::Cell,
            Ci::VerticalText => Wci::VerticalText,
            Ci::Alias => Wci::Alias,
            Ci::Copy => Wci::Copy,
            Ci::NoDrop => Wci::NoDrop,
            Ci::Grab => Wci::Grab,
            Ci::Grabbing => Wci::Grabbing,
            Ci::AllScroll => Wci::AllScroll,
            Ci::ZoomIn => Wci::ZoomIn,
            Ci::ZoomOut => Wci::ZoomOut,

            Ci::EResize => Wci::EResize,
            Ci::NResize => Wci::NResize,
            Ci::NeResize => Wci::NeResize,
            Ci::NwResize => Wci::NwResize,
            Ci::SResize => Wci::SResize,
            Ci::SeResize => Wci::SeResize,
            Ci::SwResize => Wci::SwResize,
            Ci::WResize => Wci::WResize,
            Ci::EwResize => Wci::EwResize,
            Ci::NsResize => Wci::NsResize,
            Ci::NeswResize => Wci::NeswResize,
            Ci::NwseResize => Wci::NwseResize,
            Ci::ColResize => Wci::ColResize,
            Ci::RowResize => Wci::RowResize,
        }
    }
}
