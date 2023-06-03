use paste::paste;

/// A mapping from enum names to Rust types. Instantiate this with a macro that takes `$(($value:ident, $type:ty)),*`.
#[macro_export]
macro_rules! primitive_component_definitions {
    ($macro_to_instantiate:ident) => {
        $macro_to_instantiate!(
            (Empty, ()),
            (Bool, bool),
            (EntityId, EntityId),
            (F32, f32),
            (F64, f64),
            (Mat4, Mat4),
            (I32, i32),
            (Quat, Quat),
            (String, String),
            (U8, u8),
            (U32, u32),
            (U64, u64),
            (Vec2, Vec2),
            (Vec3, Vec3),
            (Vec4, Vec4),
            (Uvec2, UVec2),
            (Uvec3, UVec3),
            (Uvec4, UVec4),
            (Duration, Duration),
            (ProceduralMeshHandle, ProceduralMeshHandle),
            (ProceduralTextureHandle, ProceduralTextureHandle),
            (ProceduralSamplerHandle, ProceduralSamplerHandle),
            (ProceduralMaterialHandle, ProceduralMaterialHandle)
        );
    };
}

#[macro_export]
macro_rules! procedural_storage_handle_definitions {
    ($macro_to_instantiate:ident) => {
        // Handle names must be in snake_case.
        $macro_to_instantiate!(mesh, texture, sampler, material);
    };
}

// The following types are copied from winit, but without everything else winit comes with so that we can use this package in our guest code.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use ulid::Ulid;

/// Describes the appearance of the mouse cursor.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, EnumString, Display, Default, Serialize, Deserialize,
)]
pub enum CursorIcon {
    /// The platform-dependent default cursor.
    #[default]
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    /// Self explanatory.
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done. But in contrast
    /// with "Wait" the user may still interact with the program. Often rendered
    /// as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,

    /// Cursor showing that something cannot be done.
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    /// Indicates something can be grabbed.
    Grab,
    /// Indicates something is grabbed.
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,

    /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
    /// is used when the movement starts from the south-east corner of the box.
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

#[cfg(feature = "native")]
impl From<CursorIcon> for winit::window::CursorIcon {
    fn from(value: CursorIcon) -> Self {
        match value {
            CursorIcon::Default => winit::window::CursorIcon::Default,
            CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
            CursorIcon::Hand => winit::window::CursorIcon::Hand,
            CursorIcon::Arrow => winit::window::CursorIcon::Arrow,
            CursorIcon::Move => winit::window::CursorIcon::Move,
            CursorIcon::Text => winit::window::CursorIcon::Text,
            CursorIcon::Wait => winit::window::CursorIcon::Wait,
            CursorIcon::Help => winit::window::CursorIcon::Help,
            CursorIcon::Progress => winit::window::CursorIcon::Progress,
            CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
            CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
            CursorIcon::Cell => winit::window::CursorIcon::Cell,
            CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
            CursorIcon::Alias => winit::window::CursorIcon::Alias,
            CursorIcon::Copy => winit::window::CursorIcon::Copy,
            CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
            CursorIcon::Grab => winit::window::CursorIcon::Grab,
            CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
            CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
            CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
            CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
            CursorIcon::EResize => winit::window::CursorIcon::EResize,
            CursorIcon::NResize => winit::window::CursorIcon::NResize,
            CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
            CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
            CursorIcon::SResize => winit::window::CursorIcon::SResize,
            CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
            CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
            CursorIcon::WResize => winit::window::CursorIcon::WResize,
            CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
            CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
            CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
            CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
            CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
            CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
        }
    }
}

#[cfg(feature = "native")]
impl From<winit::window::CursorIcon> for CursorIcon {
    fn from(value: winit::window::CursorIcon) -> Self {
        match value {
            winit::window::CursorIcon::Default => CursorIcon::Default,
            winit::window::CursorIcon::Crosshair => CursorIcon::Crosshair,
            winit::window::CursorIcon::Hand => CursorIcon::Hand,
            winit::window::CursorIcon::Arrow => CursorIcon::Arrow,
            winit::window::CursorIcon::Move => CursorIcon::Move,
            winit::window::CursorIcon::Text => CursorIcon::Text,
            winit::window::CursorIcon::Wait => CursorIcon::Wait,
            winit::window::CursorIcon::Help => CursorIcon::Help,
            winit::window::CursorIcon::Progress => CursorIcon::Progress,
            winit::window::CursorIcon::NotAllowed => CursorIcon::NotAllowed,
            winit::window::CursorIcon::ContextMenu => CursorIcon::ContextMenu,
            winit::window::CursorIcon::Cell => CursorIcon::Cell,
            winit::window::CursorIcon::VerticalText => CursorIcon::VerticalText,
            winit::window::CursorIcon::Alias => CursorIcon::Alias,
            winit::window::CursorIcon::Copy => CursorIcon::Copy,
            winit::window::CursorIcon::NoDrop => CursorIcon::NoDrop,
            winit::window::CursorIcon::Grab => CursorIcon::Grab,
            winit::window::CursorIcon::Grabbing => CursorIcon::Grabbing,
            winit::window::CursorIcon::AllScroll => CursorIcon::AllScroll,
            winit::window::CursorIcon::ZoomIn => CursorIcon::ZoomIn,
            winit::window::CursorIcon::ZoomOut => CursorIcon::ZoomOut,
            winit::window::CursorIcon::EResize => CursorIcon::EResize,
            winit::window::CursorIcon::NResize => CursorIcon::NResize,
            winit::window::CursorIcon::NeResize => CursorIcon::NeResize,
            winit::window::CursorIcon::NwResize => CursorIcon::NwResize,
            winit::window::CursorIcon::SResize => CursorIcon::SResize,
            winit::window::CursorIcon::SeResize => CursorIcon::SeResize,
            winit::window::CursorIcon::SwResize => CursorIcon::SwResize,
            winit::window::CursorIcon::WResize => CursorIcon::WResize,
            winit::window::CursorIcon::EwResize => CursorIcon::EwResize,
            winit::window::CursorIcon::NsResize => CursorIcon::NsResize,
            winit::window::CursorIcon::NeswResize => CursorIcon::NeswResize,
            winit::window::CursorIcon::NwseResize => CursorIcon::NwseResize,
            winit::window::CursorIcon::ColResize => CursorIcon::ColResize,
            winit::window::CursorIcon::RowResize => CursorIcon::RowResize,
        }
    }
}

/// Symbolic name for a keyboard key.
#[derive(
    Debug,
    Hash,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    EnumString,
    Display,
    Serialize,
    Deserialize,
)]
#[repr(u32)]
pub enum VirtualKeyCode {
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

#[cfg(feature = "native")]
impl From<VirtualKeyCode> for winit::event::VirtualKeyCode {
    fn from(value: VirtualKeyCode) -> Self {
        match value {
            VirtualKeyCode::Key1 => winit::event::VirtualKeyCode::Key1,
            VirtualKeyCode::Key2 => winit::event::VirtualKeyCode::Key2,
            VirtualKeyCode::Key3 => winit::event::VirtualKeyCode::Key3,
            VirtualKeyCode::Key4 => winit::event::VirtualKeyCode::Key4,
            VirtualKeyCode::Key5 => winit::event::VirtualKeyCode::Key5,
            VirtualKeyCode::Key6 => winit::event::VirtualKeyCode::Key6,
            VirtualKeyCode::Key7 => winit::event::VirtualKeyCode::Key7,
            VirtualKeyCode::Key8 => winit::event::VirtualKeyCode::Key8,
            VirtualKeyCode::Key9 => winit::event::VirtualKeyCode::Key9,
            VirtualKeyCode::Key0 => winit::event::VirtualKeyCode::Key0,
            VirtualKeyCode::A => winit::event::VirtualKeyCode::A,
            VirtualKeyCode::B => winit::event::VirtualKeyCode::B,
            VirtualKeyCode::C => winit::event::VirtualKeyCode::C,
            VirtualKeyCode::D => winit::event::VirtualKeyCode::D,
            VirtualKeyCode::E => winit::event::VirtualKeyCode::E,
            VirtualKeyCode::F => winit::event::VirtualKeyCode::F,
            VirtualKeyCode::G => winit::event::VirtualKeyCode::G,
            VirtualKeyCode::H => winit::event::VirtualKeyCode::H,
            VirtualKeyCode::I => winit::event::VirtualKeyCode::I,
            VirtualKeyCode::J => winit::event::VirtualKeyCode::J,
            VirtualKeyCode::K => winit::event::VirtualKeyCode::K,
            VirtualKeyCode::L => winit::event::VirtualKeyCode::L,
            VirtualKeyCode::M => winit::event::VirtualKeyCode::M,
            VirtualKeyCode::N => winit::event::VirtualKeyCode::N,
            VirtualKeyCode::O => winit::event::VirtualKeyCode::O,
            VirtualKeyCode::P => winit::event::VirtualKeyCode::P,
            VirtualKeyCode::Q => winit::event::VirtualKeyCode::Q,
            VirtualKeyCode::R => winit::event::VirtualKeyCode::R,
            VirtualKeyCode::S => winit::event::VirtualKeyCode::S,
            VirtualKeyCode::T => winit::event::VirtualKeyCode::T,
            VirtualKeyCode::U => winit::event::VirtualKeyCode::U,
            VirtualKeyCode::V => winit::event::VirtualKeyCode::V,
            VirtualKeyCode::W => winit::event::VirtualKeyCode::W,
            VirtualKeyCode::X => winit::event::VirtualKeyCode::X,
            VirtualKeyCode::Y => winit::event::VirtualKeyCode::Y,
            VirtualKeyCode::Z => winit::event::VirtualKeyCode::Z,
            VirtualKeyCode::Escape => winit::event::VirtualKeyCode::Escape,
            VirtualKeyCode::F1 => winit::event::VirtualKeyCode::F1,
            VirtualKeyCode::F2 => winit::event::VirtualKeyCode::F2,
            VirtualKeyCode::F3 => winit::event::VirtualKeyCode::F3,
            VirtualKeyCode::F4 => winit::event::VirtualKeyCode::F4,
            VirtualKeyCode::F5 => winit::event::VirtualKeyCode::F5,
            VirtualKeyCode::F6 => winit::event::VirtualKeyCode::F6,
            VirtualKeyCode::F7 => winit::event::VirtualKeyCode::F7,
            VirtualKeyCode::F8 => winit::event::VirtualKeyCode::F8,
            VirtualKeyCode::F9 => winit::event::VirtualKeyCode::F9,
            VirtualKeyCode::F10 => winit::event::VirtualKeyCode::F10,
            VirtualKeyCode::F11 => winit::event::VirtualKeyCode::F11,
            VirtualKeyCode::F12 => winit::event::VirtualKeyCode::F12,
            VirtualKeyCode::F13 => winit::event::VirtualKeyCode::F13,
            VirtualKeyCode::F14 => winit::event::VirtualKeyCode::F14,
            VirtualKeyCode::F15 => winit::event::VirtualKeyCode::F15,
            VirtualKeyCode::F16 => winit::event::VirtualKeyCode::F16,
            VirtualKeyCode::F17 => winit::event::VirtualKeyCode::F17,
            VirtualKeyCode::F18 => winit::event::VirtualKeyCode::F18,
            VirtualKeyCode::F19 => winit::event::VirtualKeyCode::F19,
            VirtualKeyCode::F20 => winit::event::VirtualKeyCode::F20,
            VirtualKeyCode::F21 => winit::event::VirtualKeyCode::F21,
            VirtualKeyCode::F22 => winit::event::VirtualKeyCode::F22,
            VirtualKeyCode::F23 => winit::event::VirtualKeyCode::F23,
            VirtualKeyCode::F24 => winit::event::VirtualKeyCode::F24,
            VirtualKeyCode::Snapshot => winit::event::VirtualKeyCode::Snapshot,
            VirtualKeyCode::Scroll => winit::event::VirtualKeyCode::Scroll,
            VirtualKeyCode::Pause => winit::event::VirtualKeyCode::Pause,
            VirtualKeyCode::Insert => winit::event::VirtualKeyCode::Insert,
            VirtualKeyCode::Home => winit::event::VirtualKeyCode::Home,
            VirtualKeyCode::Delete => winit::event::VirtualKeyCode::Delete,
            VirtualKeyCode::End => winit::event::VirtualKeyCode::End,
            VirtualKeyCode::PageDown => winit::event::VirtualKeyCode::PageDown,
            VirtualKeyCode::PageUp => winit::event::VirtualKeyCode::PageUp,
            VirtualKeyCode::Left => winit::event::VirtualKeyCode::Left,
            VirtualKeyCode::Up => winit::event::VirtualKeyCode::Up,
            VirtualKeyCode::Right => winit::event::VirtualKeyCode::Right,
            VirtualKeyCode::Down => winit::event::VirtualKeyCode::Down,
            VirtualKeyCode::Back => winit::event::VirtualKeyCode::Back,
            VirtualKeyCode::Return => winit::event::VirtualKeyCode::Return,
            VirtualKeyCode::Space => winit::event::VirtualKeyCode::Space,
            VirtualKeyCode::Compose => winit::event::VirtualKeyCode::Compose,
            VirtualKeyCode::Caret => winit::event::VirtualKeyCode::Caret,
            VirtualKeyCode::Numlock => winit::event::VirtualKeyCode::Numlock,
            VirtualKeyCode::Numpad0 => winit::event::VirtualKeyCode::Numpad0,
            VirtualKeyCode::Numpad1 => winit::event::VirtualKeyCode::Numpad1,
            VirtualKeyCode::Numpad2 => winit::event::VirtualKeyCode::Numpad2,
            VirtualKeyCode::Numpad3 => winit::event::VirtualKeyCode::Numpad3,
            VirtualKeyCode::Numpad4 => winit::event::VirtualKeyCode::Numpad4,
            VirtualKeyCode::Numpad5 => winit::event::VirtualKeyCode::Numpad5,
            VirtualKeyCode::Numpad6 => winit::event::VirtualKeyCode::Numpad6,
            VirtualKeyCode::Numpad7 => winit::event::VirtualKeyCode::Numpad7,
            VirtualKeyCode::Numpad8 => winit::event::VirtualKeyCode::Numpad8,
            VirtualKeyCode::Numpad9 => winit::event::VirtualKeyCode::Numpad9,
            VirtualKeyCode::NumpadAdd => winit::event::VirtualKeyCode::NumpadAdd,
            VirtualKeyCode::NumpadDivide => winit::event::VirtualKeyCode::NumpadDivide,
            VirtualKeyCode::NumpadDecimal => winit::event::VirtualKeyCode::NumpadDecimal,
            VirtualKeyCode::NumpadComma => winit::event::VirtualKeyCode::NumpadComma,
            VirtualKeyCode::NumpadEnter => winit::event::VirtualKeyCode::NumpadEnter,
            VirtualKeyCode::NumpadEquals => winit::event::VirtualKeyCode::NumpadEquals,
            VirtualKeyCode::NumpadMultiply => winit::event::VirtualKeyCode::NumpadMultiply,
            VirtualKeyCode::NumpadSubtract => winit::event::VirtualKeyCode::NumpadSubtract,
            VirtualKeyCode::AbntC1 => winit::event::VirtualKeyCode::AbntC1,
            VirtualKeyCode::AbntC2 => winit::event::VirtualKeyCode::AbntC2,
            VirtualKeyCode::Apostrophe => winit::event::VirtualKeyCode::Apostrophe,
            VirtualKeyCode::Apps => winit::event::VirtualKeyCode::Apps,
            VirtualKeyCode::Asterisk => winit::event::VirtualKeyCode::Asterisk,
            VirtualKeyCode::At => winit::event::VirtualKeyCode::At,
            VirtualKeyCode::Ax => winit::event::VirtualKeyCode::Ax,
            VirtualKeyCode::Backslash => winit::event::VirtualKeyCode::Backslash,
            VirtualKeyCode::Calculator => winit::event::VirtualKeyCode::Calculator,
            VirtualKeyCode::Capital => winit::event::VirtualKeyCode::Capital,
            VirtualKeyCode::Colon => winit::event::VirtualKeyCode::Colon,
            VirtualKeyCode::Comma => winit::event::VirtualKeyCode::Comma,
            VirtualKeyCode::Convert => winit::event::VirtualKeyCode::Convert,
            VirtualKeyCode::Equals => winit::event::VirtualKeyCode::Equals,
            VirtualKeyCode::Grave => winit::event::VirtualKeyCode::Grave,
            VirtualKeyCode::Kana => winit::event::VirtualKeyCode::Kana,
            VirtualKeyCode::Kanji => winit::event::VirtualKeyCode::Kanji,
            VirtualKeyCode::LAlt => winit::event::VirtualKeyCode::LAlt,
            VirtualKeyCode::LBracket => winit::event::VirtualKeyCode::LBracket,
            VirtualKeyCode::LControl => winit::event::VirtualKeyCode::LControl,
            VirtualKeyCode::LShift => winit::event::VirtualKeyCode::LShift,
            VirtualKeyCode::LWin => winit::event::VirtualKeyCode::LWin,
            VirtualKeyCode::Mail => winit::event::VirtualKeyCode::Mail,
            VirtualKeyCode::MediaSelect => winit::event::VirtualKeyCode::MediaSelect,
            VirtualKeyCode::MediaStop => winit::event::VirtualKeyCode::MediaStop,
            VirtualKeyCode::Minus => winit::event::VirtualKeyCode::Minus,
            VirtualKeyCode::Mute => winit::event::VirtualKeyCode::Mute,
            VirtualKeyCode::MyComputer => winit::event::VirtualKeyCode::MyComputer,
            VirtualKeyCode::NavigateForward => winit::event::VirtualKeyCode::NavigateForward,
            VirtualKeyCode::NavigateBackward => winit::event::VirtualKeyCode::NavigateBackward,
            VirtualKeyCode::NextTrack => winit::event::VirtualKeyCode::NextTrack,
            VirtualKeyCode::NoConvert => winit::event::VirtualKeyCode::NoConvert,
            VirtualKeyCode::OEM102 => winit::event::VirtualKeyCode::OEM102,
            VirtualKeyCode::Period => winit::event::VirtualKeyCode::Period,
            VirtualKeyCode::PlayPause => winit::event::VirtualKeyCode::PlayPause,
            VirtualKeyCode::Plus => winit::event::VirtualKeyCode::Plus,
            VirtualKeyCode::Power => winit::event::VirtualKeyCode::Power,
            VirtualKeyCode::PrevTrack => winit::event::VirtualKeyCode::PrevTrack,
            VirtualKeyCode::RAlt => winit::event::VirtualKeyCode::RAlt,
            VirtualKeyCode::RBracket => winit::event::VirtualKeyCode::RBracket,
            VirtualKeyCode::RControl => winit::event::VirtualKeyCode::RControl,
            VirtualKeyCode::RShift => winit::event::VirtualKeyCode::RShift,
            VirtualKeyCode::RWin => winit::event::VirtualKeyCode::RWin,
            VirtualKeyCode::Semicolon => winit::event::VirtualKeyCode::Semicolon,
            VirtualKeyCode::Slash => winit::event::VirtualKeyCode::Slash,
            VirtualKeyCode::Sleep => winit::event::VirtualKeyCode::Sleep,
            VirtualKeyCode::Stop => winit::event::VirtualKeyCode::Stop,
            VirtualKeyCode::Sysrq => winit::event::VirtualKeyCode::Sysrq,
            VirtualKeyCode::Tab => winit::event::VirtualKeyCode::Tab,
            VirtualKeyCode::Underline => winit::event::VirtualKeyCode::Underline,
            VirtualKeyCode::Unlabeled => winit::event::VirtualKeyCode::Unlabeled,
            VirtualKeyCode::VolumeDown => winit::event::VirtualKeyCode::VolumeDown,
            VirtualKeyCode::VolumeUp => winit::event::VirtualKeyCode::VolumeUp,
            VirtualKeyCode::Wake => winit::event::VirtualKeyCode::Wake,
            VirtualKeyCode::WebBack => winit::event::VirtualKeyCode::WebBack,
            VirtualKeyCode::WebFavorites => winit::event::VirtualKeyCode::WebFavorites,
            VirtualKeyCode::WebForward => winit::event::VirtualKeyCode::WebForward,
            VirtualKeyCode::WebHome => winit::event::VirtualKeyCode::WebHome,
            VirtualKeyCode::WebRefresh => winit::event::VirtualKeyCode::WebRefresh,
            VirtualKeyCode::WebSearch => winit::event::VirtualKeyCode::WebSearch,
            VirtualKeyCode::WebStop => winit::event::VirtualKeyCode::WebStop,
            VirtualKeyCode::Yen => winit::event::VirtualKeyCode::Yen,
            VirtualKeyCode::Copy => winit::event::VirtualKeyCode::Copy,
            VirtualKeyCode::Paste => winit::event::VirtualKeyCode::Paste,
            VirtualKeyCode::Cut => winit::event::VirtualKeyCode::Cut,
        }
    }
}

#[cfg(feature = "native")]
impl From<winit::event::VirtualKeyCode> for VirtualKeyCode {
    fn from(value: winit::event::VirtualKeyCode) -> Self {
        match value {
            winit::event::VirtualKeyCode::Key1 => VirtualKeyCode::Key1,
            winit::event::VirtualKeyCode::Key2 => VirtualKeyCode::Key2,
            winit::event::VirtualKeyCode::Key3 => VirtualKeyCode::Key3,
            winit::event::VirtualKeyCode::Key4 => VirtualKeyCode::Key4,
            winit::event::VirtualKeyCode::Key5 => VirtualKeyCode::Key5,
            winit::event::VirtualKeyCode::Key6 => VirtualKeyCode::Key6,
            winit::event::VirtualKeyCode::Key7 => VirtualKeyCode::Key7,
            winit::event::VirtualKeyCode::Key8 => VirtualKeyCode::Key8,
            winit::event::VirtualKeyCode::Key9 => VirtualKeyCode::Key9,
            winit::event::VirtualKeyCode::Key0 => VirtualKeyCode::Key0,
            winit::event::VirtualKeyCode::A => VirtualKeyCode::A,
            winit::event::VirtualKeyCode::B => VirtualKeyCode::B,
            winit::event::VirtualKeyCode::C => VirtualKeyCode::C,
            winit::event::VirtualKeyCode::D => VirtualKeyCode::D,
            winit::event::VirtualKeyCode::E => VirtualKeyCode::E,
            winit::event::VirtualKeyCode::F => VirtualKeyCode::F,
            winit::event::VirtualKeyCode::G => VirtualKeyCode::G,
            winit::event::VirtualKeyCode::H => VirtualKeyCode::H,
            winit::event::VirtualKeyCode::I => VirtualKeyCode::I,
            winit::event::VirtualKeyCode::J => VirtualKeyCode::J,
            winit::event::VirtualKeyCode::K => VirtualKeyCode::K,
            winit::event::VirtualKeyCode::L => VirtualKeyCode::L,
            winit::event::VirtualKeyCode::M => VirtualKeyCode::M,
            winit::event::VirtualKeyCode::N => VirtualKeyCode::N,
            winit::event::VirtualKeyCode::O => VirtualKeyCode::O,
            winit::event::VirtualKeyCode::P => VirtualKeyCode::P,
            winit::event::VirtualKeyCode::Q => VirtualKeyCode::Q,
            winit::event::VirtualKeyCode::R => VirtualKeyCode::R,
            winit::event::VirtualKeyCode::S => VirtualKeyCode::S,
            winit::event::VirtualKeyCode::T => VirtualKeyCode::T,
            winit::event::VirtualKeyCode::U => VirtualKeyCode::U,
            winit::event::VirtualKeyCode::V => VirtualKeyCode::V,
            winit::event::VirtualKeyCode::W => VirtualKeyCode::W,
            winit::event::VirtualKeyCode::X => VirtualKeyCode::X,
            winit::event::VirtualKeyCode::Y => VirtualKeyCode::Y,
            winit::event::VirtualKeyCode::Z => VirtualKeyCode::Z,
            winit::event::VirtualKeyCode::Escape => VirtualKeyCode::Escape,
            winit::event::VirtualKeyCode::F1 => VirtualKeyCode::F1,
            winit::event::VirtualKeyCode::F2 => VirtualKeyCode::F2,
            winit::event::VirtualKeyCode::F3 => VirtualKeyCode::F3,
            winit::event::VirtualKeyCode::F4 => VirtualKeyCode::F4,
            winit::event::VirtualKeyCode::F5 => VirtualKeyCode::F5,
            winit::event::VirtualKeyCode::F6 => VirtualKeyCode::F6,
            winit::event::VirtualKeyCode::F7 => VirtualKeyCode::F7,
            winit::event::VirtualKeyCode::F8 => VirtualKeyCode::F8,
            winit::event::VirtualKeyCode::F9 => VirtualKeyCode::F9,
            winit::event::VirtualKeyCode::F10 => VirtualKeyCode::F10,
            winit::event::VirtualKeyCode::F11 => VirtualKeyCode::F11,
            winit::event::VirtualKeyCode::F12 => VirtualKeyCode::F12,
            winit::event::VirtualKeyCode::F13 => VirtualKeyCode::F13,
            winit::event::VirtualKeyCode::F14 => VirtualKeyCode::F14,
            winit::event::VirtualKeyCode::F15 => VirtualKeyCode::F15,
            winit::event::VirtualKeyCode::F16 => VirtualKeyCode::F16,
            winit::event::VirtualKeyCode::F17 => VirtualKeyCode::F17,
            winit::event::VirtualKeyCode::F18 => VirtualKeyCode::F18,
            winit::event::VirtualKeyCode::F19 => VirtualKeyCode::F19,
            winit::event::VirtualKeyCode::F20 => VirtualKeyCode::F20,
            winit::event::VirtualKeyCode::F21 => VirtualKeyCode::F21,
            winit::event::VirtualKeyCode::F22 => VirtualKeyCode::F22,
            winit::event::VirtualKeyCode::F23 => VirtualKeyCode::F23,
            winit::event::VirtualKeyCode::F24 => VirtualKeyCode::F24,
            winit::event::VirtualKeyCode::Snapshot => VirtualKeyCode::Snapshot,
            winit::event::VirtualKeyCode::Scroll => VirtualKeyCode::Scroll,
            winit::event::VirtualKeyCode::Pause => VirtualKeyCode::Pause,
            winit::event::VirtualKeyCode::Insert => VirtualKeyCode::Insert,
            winit::event::VirtualKeyCode::Home => VirtualKeyCode::Home,
            winit::event::VirtualKeyCode::Delete => VirtualKeyCode::Delete,
            winit::event::VirtualKeyCode::End => VirtualKeyCode::End,
            winit::event::VirtualKeyCode::PageDown => VirtualKeyCode::PageDown,
            winit::event::VirtualKeyCode::PageUp => VirtualKeyCode::PageUp,
            winit::event::VirtualKeyCode::Left => VirtualKeyCode::Left,
            winit::event::VirtualKeyCode::Up => VirtualKeyCode::Up,
            winit::event::VirtualKeyCode::Right => VirtualKeyCode::Right,
            winit::event::VirtualKeyCode::Down => VirtualKeyCode::Down,
            winit::event::VirtualKeyCode::Back => VirtualKeyCode::Back,
            winit::event::VirtualKeyCode::Return => VirtualKeyCode::Return,
            winit::event::VirtualKeyCode::Space => VirtualKeyCode::Space,
            winit::event::VirtualKeyCode::Compose => VirtualKeyCode::Compose,
            winit::event::VirtualKeyCode::Caret => VirtualKeyCode::Caret,
            winit::event::VirtualKeyCode::Numlock => VirtualKeyCode::Numlock,
            winit::event::VirtualKeyCode::Numpad0 => VirtualKeyCode::Numpad0,
            winit::event::VirtualKeyCode::Numpad1 => VirtualKeyCode::Numpad1,
            winit::event::VirtualKeyCode::Numpad2 => VirtualKeyCode::Numpad2,
            winit::event::VirtualKeyCode::Numpad3 => VirtualKeyCode::Numpad3,
            winit::event::VirtualKeyCode::Numpad4 => VirtualKeyCode::Numpad4,
            winit::event::VirtualKeyCode::Numpad5 => VirtualKeyCode::Numpad5,
            winit::event::VirtualKeyCode::Numpad6 => VirtualKeyCode::Numpad6,
            winit::event::VirtualKeyCode::Numpad7 => VirtualKeyCode::Numpad7,
            winit::event::VirtualKeyCode::Numpad8 => VirtualKeyCode::Numpad8,
            winit::event::VirtualKeyCode::Numpad9 => VirtualKeyCode::Numpad9,
            winit::event::VirtualKeyCode::NumpadAdd => VirtualKeyCode::NumpadAdd,
            winit::event::VirtualKeyCode::NumpadDivide => VirtualKeyCode::NumpadDivide,
            winit::event::VirtualKeyCode::NumpadDecimal => VirtualKeyCode::NumpadDecimal,
            winit::event::VirtualKeyCode::NumpadComma => VirtualKeyCode::NumpadComma,
            winit::event::VirtualKeyCode::NumpadEnter => VirtualKeyCode::NumpadEnter,
            winit::event::VirtualKeyCode::NumpadEquals => VirtualKeyCode::NumpadEquals,
            winit::event::VirtualKeyCode::NumpadMultiply => VirtualKeyCode::NumpadMultiply,
            winit::event::VirtualKeyCode::NumpadSubtract => VirtualKeyCode::NumpadSubtract,
            winit::event::VirtualKeyCode::AbntC1 => VirtualKeyCode::AbntC1,
            winit::event::VirtualKeyCode::AbntC2 => VirtualKeyCode::AbntC2,
            winit::event::VirtualKeyCode::Apostrophe => VirtualKeyCode::Apostrophe,
            winit::event::VirtualKeyCode::Apps => VirtualKeyCode::Apps,
            winit::event::VirtualKeyCode::Asterisk => VirtualKeyCode::Asterisk,
            winit::event::VirtualKeyCode::At => VirtualKeyCode::At,
            winit::event::VirtualKeyCode::Ax => VirtualKeyCode::Ax,
            winit::event::VirtualKeyCode::Backslash => VirtualKeyCode::Backslash,
            winit::event::VirtualKeyCode::Calculator => VirtualKeyCode::Calculator,
            winit::event::VirtualKeyCode::Capital => VirtualKeyCode::Capital,
            winit::event::VirtualKeyCode::Colon => VirtualKeyCode::Colon,
            winit::event::VirtualKeyCode::Comma => VirtualKeyCode::Comma,
            winit::event::VirtualKeyCode::Convert => VirtualKeyCode::Convert,
            winit::event::VirtualKeyCode::Equals => VirtualKeyCode::Equals,
            winit::event::VirtualKeyCode::Grave => VirtualKeyCode::Grave,
            winit::event::VirtualKeyCode::Kana => VirtualKeyCode::Kana,
            winit::event::VirtualKeyCode::Kanji => VirtualKeyCode::Kanji,
            winit::event::VirtualKeyCode::LAlt => VirtualKeyCode::LAlt,
            winit::event::VirtualKeyCode::LBracket => VirtualKeyCode::LBracket,
            winit::event::VirtualKeyCode::LControl => VirtualKeyCode::LControl,
            winit::event::VirtualKeyCode::LShift => VirtualKeyCode::LShift,
            winit::event::VirtualKeyCode::LWin => VirtualKeyCode::LWin,
            winit::event::VirtualKeyCode::Mail => VirtualKeyCode::Mail,
            winit::event::VirtualKeyCode::MediaSelect => VirtualKeyCode::MediaSelect,
            winit::event::VirtualKeyCode::MediaStop => VirtualKeyCode::MediaStop,
            winit::event::VirtualKeyCode::Minus => VirtualKeyCode::Minus,
            winit::event::VirtualKeyCode::Mute => VirtualKeyCode::Mute,
            winit::event::VirtualKeyCode::MyComputer => VirtualKeyCode::MyComputer,
            winit::event::VirtualKeyCode::NavigateForward => VirtualKeyCode::NavigateForward,
            winit::event::VirtualKeyCode::NavigateBackward => VirtualKeyCode::NavigateBackward,
            winit::event::VirtualKeyCode::NextTrack => VirtualKeyCode::NextTrack,
            winit::event::VirtualKeyCode::NoConvert => VirtualKeyCode::NoConvert,
            winit::event::VirtualKeyCode::OEM102 => VirtualKeyCode::OEM102,
            winit::event::VirtualKeyCode::Period => VirtualKeyCode::Period,
            winit::event::VirtualKeyCode::PlayPause => VirtualKeyCode::PlayPause,
            winit::event::VirtualKeyCode::Plus => VirtualKeyCode::Plus,
            winit::event::VirtualKeyCode::Power => VirtualKeyCode::Power,
            winit::event::VirtualKeyCode::PrevTrack => VirtualKeyCode::PrevTrack,
            winit::event::VirtualKeyCode::RAlt => VirtualKeyCode::RAlt,
            winit::event::VirtualKeyCode::RBracket => VirtualKeyCode::RBracket,
            winit::event::VirtualKeyCode::RControl => VirtualKeyCode::RControl,
            winit::event::VirtualKeyCode::RShift => VirtualKeyCode::RShift,
            winit::event::VirtualKeyCode::RWin => VirtualKeyCode::RWin,
            winit::event::VirtualKeyCode::Semicolon => VirtualKeyCode::Semicolon,
            winit::event::VirtualKeyCode::Slash => VirtualKeyCode::Slash,
            winit::event::VirtualKeyCode::Sleep => VirtualKeyCode::Sleep,
            winit::event::VirtualKeyCode::Stop => VirtualKeyCode::Stop,
            winit::event::VirtualKeyCode::Sysrq => VirtualKeyCode::Sysrq,
            winit::event::VirtualKeyCode::Tab => VirtualKeyCode::Tab,
            winit::event::VirtualKeyCode::Underline => VirtualKeyCode::Underline,
            winit::event::VirtualKeyCode::Unlabeled => VirtualKeyCode::Unlabeled,
            winit::event::VirtualKeyCode::VolumeDown => VirtualKeyCode::VolumeDown,
            winit::event::VirtualKeyCode::VolumeUp => VirtualKeyCode::VolumeUp,
            winit::event::VirtualKeyCode::Wake => VirtualKeyCode::Wake,
            winit::event::VirtualKeyCode::WebBack => VirtualKeyCode::WebBack,
            winit::event::VirtualKeyCode::WebFavorites => VirtualKeyCode::WebFavorites,
            winit::event::VirtualKeyCode::WebForward => VirtualKeyCode::WebForward,
            winit::event::VirtualKeyCode::WebHome => VirtualKeyCode::WebHome,
            winit::event::VirtualKeyCode::WebRefresh => VirtualKeyCode::WebRefresh,
            winit::event::VirtualKeyCode::WebSearch => VirtualKeyCode::WebSearch,
            winit::event::VirtualKeyCode::WebStop => VirtualKeyCode::WebStop,
            winit::event::VirtualKeyCode::Yen => VirtualKeyCode::Yen,
            winit::event::VirtualKeyCode::Copy => VirtualKeyCode::Copy,
            winit::event::VirtualKeyCode::Paste => VirtualKeyCode::Paste,
            winit::event::VirtualKeyCode::Cut => VirtualKeyCode::Cut,
        }
    }
}

impl ModifiersState {
    /// Returns `true` if the shift key is pressed.
    pub fn shift(&self) -> bool {
        self.intersects(Self::SHIFT)
    }
    /// Returns `true` if the control key is pressed.
    pub fn ctrl(&self) -> bool {
        self.intersects(Self::CTRL)
    }
    /// Returns `true` if the alt key is pressed.
    pub fn alt(&self) -> bool {
        self.intersects(Self::ALT)
    }
    /// Returns `true` if the logo key is pressed.
    pub fn logo(&self) -> bool {
        self.intersects(Self::LOGO)
    }
}

bitflags! {
    /// Represents the current state of the keyboard modifiers
    ///
    /// Each flag represents a modifier and is set if this modifier is active.
    #[derive(Default)]
    pub struct ModifiersState: u32 {
        // left and right modifiers are currently commented out, but we should be able to support
        // them in a future release
        /// The "shift" key.
        const SHIFT = 0b100;
        // const LSHIFT = 0b010;
        // const RSHIFT = 0b001;
        /// The "control" key.
        const CTRL = 0b100 << 3;
        // const LCTRL = 0b010 << 3;
        // const RCTRL = 0b001 << 3;
        /// The "alt" key.
        const ALT = 0b100 << 6;
        // const LALT = 0b010 << 6;
        // const RALT = 0b001 << 6;
        /// This is the "windows" key on PC and "command" key on Mac.
        const LOGO = 0b100 << 9;
        // const LLOGO = 0b010 << 9;
        // const RLOGO = 0b001 << 9;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}
impl From<u32> for MouseButton {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Middle,
            x => Self::Other(x as _),
        }
    }
}
impl From<MouseButton> for u32 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => x as _,
        }
    }
}
#[cfg(feature = "native")]
impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Other(x) => Self::Other(x),
        }
    }
}
#[cfg(feature = "native")]
impl From<MouseButton> for winit::event::MouseButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
            MouseButton::Other(x) => Self::Other(x),
        }
    }
}

macro_rules! make_procedural_storage_handles {
    ($($name:ident),*) => { paste!{$(
        #[derive(
            Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize,
        )]
        pub struct [<Procedural $name:camel Handle>](Ulid);

        impl Default for [<Procedural $name:camel Handle>] {
            fn default() -> Self {
                Self(Ulid::nil())
            }
        }

        impl std::fmt::Display for [<Procedural $name:camel Handle>] {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!(stringify!([<Procedural $name:camel Handle>]), "({})"), self.0)
            }
        }

        impl From<Ulid> for [<Procedural $name:camel Handle>] {
            fn from(ulid: Ulid) -> Self {
                Self(ulid)
            }
        }

        impl From<[<Procedural $name:camel Handle>]> for Ulid {
            fn from(handle: [<Procedural $name:camel Handle>]) -> Self {
                handle.0
            }
        }
    )*}};
}

procedural_storage_handle_definitions!(make_procedural_storage_handles);
