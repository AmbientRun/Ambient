use ambient_sys::time::SystemTime;

use ambient_animation as ea;
use ambient_ecs::EntityId;
use ambient_std::asset_url::TypedAssetUrl;
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use wit_bindgen_host_wasmtime_rust::{Endian, Le};

use super::interface::host;

/// Converts from a Rust representation to a wit-bindgen representation.
pub trait IntoBindgen {
    type Item;
    fn into_bindgen(self) -> Self::Item;
}

/// Converts from a wit-bindgen representation to a Rust representation.
#[allow(clippy::wrong_self_convention)]
pub trait FromBindgen {
    type Item;
    fn from_bindgen(self) -> Self::Item;
}

impl IntoBindgen for EntityId {
    type Item = host::EntityId;
    fn into_bindgen(self) -> Self::Item {
        let (id0, id1) = self.to_u64s();

        host::EntityId { id0, id1 }
    }
}
impl FromBindgen for host::EntityId {
    type Item = EntityId;
    fn from_bindgen(self) -> Self::Item {
        EntityId::from_u64s(self.id0, self.id1)
    }
}

impl IntoBindgen for Vec2 {
    type Item = host::Vec2;
    fn into_bindgen(self) -> Self::Item {
        host::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for host::Vec2 {
    type Item = Vec2;
    fn from_bindgen(self) -> Self::Item {
        Vec2::new(self.x, self.y)
    }
}

impl IntoBindgen for Vec3 {
    type Item = host::Vec3;
    fn into_bindgen(self) -> Self::Item {
        host::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for host::Vec3 {
    type Item = Vec3;
    fn from_bindgen(self) -> Self::Item {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for Vec4 {
    type Item = host::Vec4;
    fn into_bindgen(self) -> Self::Item {
        host::Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for host::Vec4 {
    type Item = Vec4;
    fn from_bindgen(self) -> Self::Item {
        Vec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for UVec2 {
    type Item = host::Uvec2;
    fn into_bindgen(self) -> Self::Item {
        host::Uvec2 {
            x: self.x,
            y: self.y,
        }
    }
}
impl FromBindgen for host::Uvec2 {
    type Item = UVec2;
    fn from_bindgen(self) -> Self::Item {
        UVec2::new(self.x, self.y)
    }
}

impl IntoBindgen for UVec3 {
    type Item = host::Uvec3;
    fn into_bindgen(self) -> Self::Item {
        host::Uvec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl FromBindgen for host::Uvec3 {
    type Item = UVec3;
    fn from_bindgen(self) -> Self::Item {
        UVec3::new(self.x, self.y, self.z)
    }
}

impl IntoBindgen for UVec4 {
    type Item = host::Uvec4;
    fn into_bindgen(self) -> Self::Item {
        host::Uvec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for host::Uvec4 {
    type Item = UVec4;
    fn from_bindgen(self) -> Self::Item {
        UVec4::new(self.x, self.y, self.z, self.w)
    }
}

impl IntoBindgen for Quat {
    type Item = host::Quat;
    fn into_bindgen(self) -> Self::Item {
        host::Quat {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl FromBindgen for host::Quat {
    type Item = Quat;
    fn from_bindgen(self) -> Self::Item {
        Quat::from_array([self.x, self.y, self.z, self.w])
    }
}

impl IntoBindgen for Mat4 {
    type Item = host::Mat4;
    fn into_bindgen(self) -> Self::Item {
        host::Mat4 {
            x: self.x_axis.into_bindgen(),
            y: self.y_axis.into_bindgen(),
            z: self.z_axis.into_bindgen(),
            w: self.w_axis.into_bindgen(),
        }
    }
}
impl FromBindgen for host::Mat4 {
    type Item = Mat4;
    fn from_bindgen(self) -> Self::Item {
        Mat4::from_cols(
            self.x.from_bindgen(),
            self.y.from_bindgen(),
            self.z.from_bindgen(),
            self.w.from_bindgen(),
        )
    }
}

macro_rules! bindgen_passthrough {
    ($type:ty) => {
        impl IntoBindgen for $type {
            type Item = Self;
            fn into_bindgen(self) -> Self::Item {
                self
            }
        }
        impl FromBindgen for $type {
            type Item = Self;
            fn from_bindgen(self) -> Self::Item {
                self
            }
        }
    };
}

bindgen_passthrough!(());
bindgen_passthrough!(bool);
bindgen_passthrough!(f32);
bindgen_passthrough!(f64);
bindgen_passthrough!(i32);
bindgen_passthrough!(String);
bindgen_passthrough!(u32);
bindgen_passthrough!(u64);

impl<'a> FromBindgen for &'a str {
    type Item = String;
    fn from_bindgen(self) -> Self::Item {
        self.to_owned()
    }
}

impl<T> IntoBindgen for Option<T>
where
    T: IntoBindgen,
{
    type Item = Option<T::Item>;
    fn into_bindgen(self) -> Self::Item {
        self.map(|i| i.into_bindgen())
    }
}
impl<T> FromBindgen for Option<T>
where
    T: FromBindgen,
{
    type Item = Option<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.map(|i| i.from_bindgen())
    }
}

impl<T> IntoBindgen for Vec<T>
where
    T: IntoBindgen,
{
    type Item = Vec<T::Item>;
    fn into_bindgen(self) -> Self::Item {
        self.into_iter().map(|i| i.into_bindgen()).collect()
    }
}
impl<T> FromBindgen for Vec<T>
where
    T: FromBindgen,
{
    type Item = Vec<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.into_iter().map(|i| i.from_bindgen()).collect()
    }
}
impl<T> FromBindgen for &[T]
where
    T: FromBindgen + Clone,
{
    type Item = Vec<T::Item>;
    fn from_bindgen(self) -> Self::Item {
        self.iter().map(|i| i.clone().from_bindgen()).collect()
    }
}

impl<T> FromBindgen for Le<T>
where
    T: FromBindgen + Endian,
{
    type Item = T::Item;

    fn from_bindgen(self) -> Self::Item {
        self.get().from_bindgen()
    }
}

impl FromBindgen for host::AnimationAction<'_> {
    type Item = ea::AnimationAction;
    fn from_bindgen(self) -> Self::Item {
        ea::AnimationAction {
            clip: ea::AnimationClipRef::FromModelAsset(
                TypedAssetUrl::parse(self.clip_url).unwrap(),
            ),
            time: ea::AnimationActionTime::Offset {
                start_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
                speed: 1.0,
            },
            looping: self.looping,
            weight: self.weight,
        }
    }
}

impl FromBindgen for host::AnimationController<'_> {
    type Item = ea::AnimationController;
    fn from_bindgen(self) -> Self::Item {
        ea::AnimationController {
            actions: self.actions.into_iter().map(|s| s.from_bindgen()).collect(),
            apply_base_pose: self.apply_base_pose,
        }
    }
}

impl IntoBindgen for ambient_input::PlayerRawInput {
    type Item = host::PlayerRawInput;

    fn into_bindgen(self) -> Self::Item {
        Self::Item {
            keys: self.keys.into_iter().map(|k| k.into_bindgen()).collect(),
            mouse_position: self.mouse_position.into_bindgen(),
            cursor_position: self.cursor_position.into_bindgen(),
            mouse_wheel: self.mouse_wheel,
            mouse_buttons: self
                .mouse_buttons
                .into_iter()
                .map(|b| b.into_bindgen())
                .collect(),
        }
    }
}

impl IntoBindgen for ambient_input::VirtualKeyCode {
    type Item = host::VirtualKeyCode;

    fn into_bindgen(self) -> Self::Item {
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
            Self::OEM102 => Self::Item::Oem102,
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

impl IntoBindgen for ambient_input::MouseButton {
    type Item = host::MouseButton;

    fn into_bindgen(self) -> Self::Item {
        match self {
            Self::Left => Self::Item::Left,
            Self::Right => Self::Item::Right,
            Self::Middle => Self::Item::Middle,
            Self::Other(id) => Self::Item::Other(id),
        }
    }
}
