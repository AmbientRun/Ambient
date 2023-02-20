#[allow(clippy::all)]
#[allow(missing_docs)] pub mod host { use super::wit_bindgen_guest_rust;
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct EntityId {
    pub id0: u64,
    pub id1: u64,
  }
  impl core::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("EntityId").field("id0", &self.id0).field("id1", &self.id1).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Vec2 {
    pub x: f32,
    pub y: f32,
  }
  impl core::fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Vec2").field("x", &self.x).field("y", &self.y).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
  }
  impl core::fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Vec3").field("x", &self.x).field("y", &self.y).field("z", &self.z).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
  }
  impl core::fmt::Debug for Vec4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Vec4").field("x", &self.x).field("y", &self.y).field("z", &self.z).field("w", &self.w).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
  }
  impl core::fmt::Debug for Quat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Quat").field("x", &self.x).field("y", &self.y).field("z", &self.z).field("w", &self.w).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
  }
  impl core::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Mat4").field("x", &self.x).field("y", &self.y).field("z", &self.z).field("w", &self.w).finish()}
  }
  #[derive(Clone)]
  pub struct ObjectRefParam<'a,> {
    pub id: &'a  str,
  }
  impl<'a,> core::fmt::Debug for ObjectRefParam<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("ObjectRefParam").field("id", &self.id).finish()}
  }
  #[derive(Clone)]
  pub struct ObjectRefResult {
    pub id: String,
  }
  impl core::fmt::Debug for ObjectRefResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("ObjectRefResult").field("id", &self.id).finish()}
  }
  /// An action in an animation.
  #[derive(Clone)]
  pub struct AnimationAction<'a,> {
    /// The animation clip URL to play.
    pub clip_url: &'a  str,
    /// Whether or not this action should loop
    pub looping: bool,
    /// How strongly this action applies to the final blend [0-1]
    pub weight: f32,
  }
  impl<'a,> core::fmt::Debug for AnimationAction<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("AnimationAction").field("clip-url", &self.clip_url).field("looping", &self.looping).field("weight", &self.weight).finish()}
  }
  /// Controls the animations for an entity.
  #[derive(Clone)]
  pub struct AnimationController<'a,> {
    /// All of the actions that contribute to this animation.
    /// Will be blended together.
    pub actions: &'a [AnimationAction<'a,>],
    /// Whether or not the first action's pose should be used as a base pose.
    pub apply_base_pose: bool,
  }
  impl<'a,> core::fmt::Debug for AnimationController<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("AnimationController").field("actions", &self.actions).field("apply-base-pose", &self.apply_base_pose).finish()}
  }
  #[derive(Clone)]
  pub enum ComponentListTypeParam<'a,>{
    TypeEmpty(&'a [()]),
    TypeBool(&'a [bool]),
    TypeEntityId(&'a [EntityId]),
    TypeF32(&'a [f32]),
    TypeF64(&'a [f64]),
    TypeMat4(&'a [Mat4]),
    TypeI32(&'a [i32]),
    TypeQuat(&'a [Quat]),
    TypeString(&'a [&'a  str]),
    TypeU32(&'a [u32]),
    TypeU64(&'a [u64]),
    TypeVec2(&'a [Vec2]),
    TypeVec3(&'a [Vec3]),
    TypeVec4(&'a [Vec4]),
    TypeObjectRef(&'a [ObjectRefParam<'a,>]),
  }
  impl<'a,> core::fmt::Debug for ComponentListTypeParam<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentListTypeParam::TypeEmpty(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeEmpty").field(e).finish()
        }
        ComponentListTypeParam::TypeBool(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeBool").field(e).finish()
        }
        ComponentListTypeParam::TypeEntityId(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeEntityId").field(e).finish()
        }
        ComponentListTypeParam::TypeF32(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeF32").field(e).finish()
        }
        ComponentListTypeParam::TypeF64(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeF64").field(e).finish()
        }
        ComponentListTypeParam::TypeMat4(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeMat4").field(e).finish()
        }
        ComponentListTypeParam::TypeI32(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeI32").field(e).finish()
        }
        ComponentListTypeParam::TypeQuat(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeQuat").field(e).finish()
        }
        ComponentListTypeParam::TypeString(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeString").field(e).finish()
        }
        ComponentListTypeParam::TypeU32(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeU32").field(e).finish()
        }
        ComponentListTypeParam::TypeU64(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeU64").field(e).finish()
        }
        ComponentListTypeParam::TypeVec2(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeVec2").field(e).finish()
        }
        ComponentListTypeParam::TypeVec3(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeVec3").field(e).finish()
        }
        ComponentListTypeParam::TypeVec4(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeVec4").field(e).finish()
        }
        ComponentListTypeParam::TypeObjectRef(e) => {
          f.debug_tuple("ComponentListTypeParam::TypeObjectRef").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentListTypeResult{
    TypeEmpty(Vec<()>),
    TypeBool(Vec<bool>),
    TypeEntityId(Vec<EntityId>),
    TypeF32(Vec<f32>),
    TypeF64(Vec<f64>),
    TypeMat4(Vec<Mat4>),
    TypeI32(Vec<i32>),
    TypeQuat(Vec<Quat>),
    TypeString(Vec<String>),
    TypeU32(Vec<u32>),
    TypeU64(Vec<u64>),
    TypeVec2(Vec<Vec2>),
    TypeVec3(Vec<Vec3>),
    TypeVec4(Vec<Vec4>),
    TypeObjectRef(Vec<ObjectRefResult>),
  }
  impl core::fmt::Debug for ComponentListTypeResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentListTypeResult::TypeEmpty(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeEmpty").field(e).finish()
        }
        ComponentListTypeResult::TypeBool(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeBool").field(e).finish()
        }
        ComponentListTypeResult::TypeEntityId(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeEntityId").field(e).finish()
        }
        ComponentListTypeResult::TypeF32(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeF32").field(e).finish()
        }
        ComponentListTypeResult::TypeF64(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeF64").field(e).finish()
        }
        ComponentListTypeResult::TypeMat4(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeMat4").field(e).finish()
        }
        ComponentListTypeResult::TypeI32(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeI32").field(e).finish()
        }
        ComponentListTypeResult::TypeQuat(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeQuat").field(e).finish()
        }
        ComponentListTypeResult::TypeString(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeString").field(e).finish()
        }
        ComponentListTypeResult::TypeU32(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeU32").field(e).finish()
        }
        ComponentListTypeResult::TypeU64(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeU64").field(e).finish()
        }
        ComponentListTypeResult::TypeVec2(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeVec2").field(e).finish()
        }
        ComponentListTypeResult::TypeVec3(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeVec3").field(e).finish()
        }
        ComponentListTypeResult::TypeVec4(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeVec4").field(e).finish()
        }
        ComponentListTypeResult::TypeObjectRef(e) => {
          f.debug_tuple("ComponentListTypeResult::TypeObjectRef").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentOptionTypeParam<'a,>{
    TypeEmpty(Option<()>),
    TypeBool(Option<bool>),
    TypeEntityId(Option<EntityId>),
    TypeF32(Option<f32>),
    TypeF64(Option<f64>),
    TypeMat4(Option<Mat4>),
    TypeI32(Option<i32>),
    TypeQuat(Option<Quat>),
    TypeString(Option<&'a  str>),
    TypeU32(Option<u32>),
    TypeU64(Option<u64>),
    TypeVec2(Option<Vec2>),
    TypeVec3(Option<Vec3>),
    TypeVec4(Option<Vec4>),
    TypeObjectRef(Option<ObjectRefParam<'a,>>),
  }
  impl<'a,> core::fmt::Debug for ComponentOptionTypeParam<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentOptionTypeParam::TypeEmpty(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeEmpty").field(e).finish()
        }
        ComponentOptionTypeParam::TypeBool(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeBool").field(e).finish()
        }
        ComponentOptionTypeParam::TypeEntityId(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeEntityId").field(e).finish()
        }
        ComponentOptionTypeParam::TypeF32(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeF32").field(e).finish()
        }
        ComponentOptionTypeParam::TypeF64(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeF64").field(e).finish()
        }
        ComponentOptionTypeParam::TypeMat4(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeMat4").field(e).finish()
        }
        ComponentOptionTypeParam::TypeI32(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeI32").field(e).finish()
        }
        ComponentOptionTypeParam::TypeQuat(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeQuat").field(e).finish()
        }
        ComponentOptionTypeParam::TypeString(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeString").field(e).finish()
        }
        ComponentOptionTypeParam::TypeU32(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeU32").field(e).finish()
        }
        ComponentOptionTypeParam::TypeU64(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeU64").field(e).finish()
        }
        ComponentOptionTypeParam::TypeVec2(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeVec2").field(e).finish()
        }
        ComponentOptionTypeParam::TypeVec3(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeVec3").field(e).finish()
        }
        ComponentOptionTypeParam::TypeVec4(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeVec4").field(e).finish()
        }
        ComponentOptionTypeParam::TypeObjectRef(e) => {
          f.debug_tuple("ComponentOptionTypeParam::TypeObjectRef").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentOptionTypeResult{
    TypeEmpty(Option<()>),
    TypeBool(Option<bool>),
    TypeEntityId(Option<EntityId>),
    TypeF32(Option<f32>),
    TypeF64(Option<f64>),
    TypeMat4(Option<Mat4>),
    TypeI32(Option<i32>),
    TypeQuat(Option<Quat>),
    TypeString(Option<String>),
    TypeU32(Option<u32>),
    TypeU64(Option<u64>),
    TypeVec2(Option<Vec2>),
    TypeVec3(Option<Vec3>),
    TypeVec4(Option<Vec4>),
    TypeObjectRef(Option<ObjectRefResult>),
  }
  impl core::fmt::Debug for ComponentOptionTypeResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentOptionTypeResult::TypeEmpty(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeEmpty").field(e).finish()
        }
        ComponentOptionTypeResult::TypeBool(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeBool").field(e).finish()
        }
        ComponentOptionTypeResult::TypeEntityId(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeEntityId").field(e).finish()
        }
        ComponentOptionTypeResult::TypeF32(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeF32").field(e).finish()
        }
        ComponentOptionTypeResult::TypeF64(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeF64").field(e).finish()
        }
        ComponentOptionTypeResult::TypeMat4(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeMat4").field(e).finish()
        }
        ComponentOptionTypeResult::TypeI32(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeI32").field(e).finish()
        }
        ComponentOptionTypeResult::TypeQuat(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeQuat").field(e).finish()
        }
        ComponentOptionTypeResult::TypeString(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeString").field(e).finish()
        }
        ComponentOptionTypeResult::TypeU32(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeU32").field(e).finish()
        }
        ComponentOptionTypeResult::TypeU64(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeU64").field(e).finish()
        }
        ComponentOptionTypeResult::TypeVec2(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeVec2").field(e).finish()
        }
        ComponentOptionTypeResult::TypeVec3(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeVec3").field(e).finish()
        }
        ComponentOptionTypeResult::TypeVec4(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeVec4").field(e).finish()
        }
        ComponentOptionTypeResult::TypeObjectRef(e) => {
          f.debug_tuple("ComponentOptionTypeResult::TypeObjectRef").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentTypeParam<'a,>{
    TypeEmpty(()),
    TypeBool(bool),
    TypeEntityId(EntityId),
    TypeF32(f32),
    TypeF64(f64),
    TypeMat4(Mat4),
    TypeI32(i32),
    TypeQuat(Quat),
    TypeString(&'a  str),
    TypeU32(u32),
    TypeU64(u64),
    TypeVec2(Vec2),
    TypeVec3(Vec3),
    TypeVec4(Vec4),
    TypeObjectRef(ObjectRefParam<'a,>),
    TypeList(ComponentListTypeParam<'a,>),
    TypeOption(ComponentOptionTypeParam<'a,>),
  }
  impl<'a,> core::fmt::Debug for ComponentTypeParam<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentTypeParam::TypeEmpty(e) => {
          f.debug_tuple("ComponentTypeParam::TypeEmpty").field(e).finish()
        }
        ComponentTypeParam::TypeBool(e) => {
          f.debug_tuple("ComponentTypeParam::TypeBool").field(e).finish()
        }
        ComponentTypeParam::TypeEntityId(e) => {
          f.debug_tuple("ComponentTypeParam::TypeEntityId").field(e).finish()
        }
        ComponentTypeParam::TypeF32(e) => {
          f.debug_tuple("ComponentTypeParam::TypeF32").field(e).finish()
        }
        ComponentTypeParam::TypeF64(e) => {
          f.debug_tuple("ComponentTypeParam::TypeF64").field(e).finish()
        }
        ComponentTypeParam::TypeMat4(e) => {
          f.debug_tuple("ComponentTypeParam::TypeMat4").field(e).finish()
        }
        ComponentTypeParam::TypeI32(e) => {
          f.debug_tuple("ComponentTypeParam::TypeI32").field(e).finish()
        }
        ComponentTypeParam::TypeQuat(e) => {
          f.debug_tuple("ComponentTypeParam::TypeQuat").field(e).finish()
        }
        ComponentTypeParam::TypeString(e) => {
          f.debug_tuple("ComponentTypeParam::TypeString").field(e).finish()
        }
        ComponentTypeParam::TypeU32(e) => {
          f.debug_tuple("ComponentTypeParam::TypeU32").field(e).finish()
        }
        ComponentTypeParam::TypeU64(e) => {
          f.debug_tuple("ComponentTypeParam::TypeU64").field(e).finish()
        }
        ComponentTypeParam::TypeVec2(e) => {
          f.debug_tuple("ComponentTypeParam::TypeVec2").field(e).finish()
        }
        ComponentTypeParam::TypeVec3(e) => {
          f.debug_tuple("ComponentTypeParam::TypeVec3").field(e).finish()
        }
        ComponentTypeParam::TypeVec4(e) => {
          f.debug_tuple("ComponentTypeParam::TypeVec4").field(e).finish()
        }
        ComponentTypeParam::TypeObjectRef(e) => {
          f.debug_tuple("ComponentTypeParam::TypeObjectRef").field(e).finish()
        }
        ComponentTypeParam::TypeList(e) => {
          f.debug_tuple("ComponentTypeParam::TypeList").field(e).finish()
        }
        ComponentTypeParam::TypeOption(e) => {
          f.debug_tuple("ComponentTypeParam::TypeOption").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentTypeResult{
    TypeEmpty(()),
    TypeBool(bool),
    TypeEntityId(EntityId),
    TypeF32(f32),
    TypeF64(f64),
    TypeMat4(Mat4),
    TypeI32(i32),
    TypeQuat(Quat),
    TypeString(String),
    TypeU32(u32),
    TypeU64(u64),
    TypeVec2(Vec2),
    TypeVec3(Vec3),
    TypeVec4(Vec4),
    TypeObjectRef(ObjectRefResult),
    TypeList(ComponentListTypeResult),
    TypeOption(ComponentOptionTypeResult),
  }
  impl core::fmt::Debug for ComponentTypeResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentTypeResult::TypeEmpty(e) => {
          f.debug_tuple("ComponentTypeResult::TypeEmpty").field(e).finish()
        }
        ComponentTypeResult::TypeBool(e) => {
          f.debug_tuple("ComponentTypeResult::TypeBool").field(e).finish()
        }
        ComponentTypeResult::TypeEntityId(e) => {
          f.debug_tuple("ComponentTypeResult::TypeEntityId").field(e).finish()
        }
        ComponentTypeResult::TypeF32(e) => {
          f.debug_tuple("ComponentTypeResult::TypeF32").field(e).finish()
        }
        ComponentTypeResult::TypeF64(e) => {
          f.debug_tuple("ComponentTypeResult::TypeF64").field(e).finish()
        }
        ComponentTypeResult::TypeMat4(e) => {
          f.debug_tuple("ComponentTypeResult::TypeMat4").field(e).finish()
        }
        ComponentTypeResult::TypeI32(e) => {
          f.debug_tuple("ComponentTypeResult::TypeI32").field(e).finish()
        }
        ComponentTypeResult::TypeQuat(e) => {
          f.debug_tuple("ComponentTypeResult::TypeQuat").field(e).finish()
        }
        ComponentTypeResult::TypeString(e) => {
          f.debug_tuple("ComponentTypeResult::TypeString").field(e).finish()
        }
        ComponentTypeResult::TypeU32(e) => {
          f.debug_tuple("ComponentTypeResult::TypeU32").field(e).finish()
        }
        ComponentTypeResult::TypeU64(e) => {
          f.debug_tuple("ComponentTypeResult::TypeU64").field(e).finish()
        }
        ComponentTypeResult::TypeVec2(e) => {
          f.debug_tuple("ComponentTypeResult::TypeVec2").field(e).finish()
        }
        ComponentTypeResult::TypeVec3(e) => {
          f.debug_tuple("ComponentTypeResult::TypeVec3").field(e).finish()
        }
        ComponentTypeResult::TypeVec4(e) => {
          f.debug_tuple("ComponentTypeResult::TypeVec4").field(e).finish()
        }
        ComponentTypeResult::TypeObjectRef(e) => {
          f.debug_tuple("ComponentTypeResult::TypeObjectRef").field(e).finish()
        }
        ComponentTypeResult::TypeList(e) => {
          f.debug_tuple("ComponentTypeResult::TypeList").field(e).finish()
        }
        ComponentTypeResult::TypeOption(e) => {
          f.debug_tuple("ComponentTypeResult::TypeOption").field(e).finish()
        }
      }
    }
  }
  pub type Components<'a,> = &'a [(u32,ComponentTypeParam<'a,>,)];
  #[repr(u8)]
  #[derive(Clone, Copy, PartialEq, Eq)]
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
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    Oem102,
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
  impl core::fmt::Debug for VirtualKeyCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        VirtualKeyCode::Key1 => {
          f.debug_tuple("VirtualKeyCode::Key1").finish()
        }
        VirtualKeyCode::Key2 => {
          f.debug_tuple("VirtualKeyCode::Key2").finish()
        }
        VirtualKeyCode::Key3 => {
          f.debug_tuple("VirtualKeyCode::Key3").finish()
        }
        VirtualKeyCode::Key4 => {
          f.debug_tuple("VirtualKeyCode::Key4").finish()
        }
        VirtualKeyCode::Key5 => {
          f.debug_tuple("VirtualKeyCode::Key5").finish()
        }
        VirtualKeyCode::Key6 => {
          f.debug_tuple("VirtualKeyCode::Key6").finish()
        }
        VirtualKeyCode::Key7 => {
          f.debug_tuple("VirtualKeyCode::Key7").finish()
        }
        VirtualKeyCode::Key8 => {
          f.debug_tuple("VirtualKeyCode::Key8").finish()
        }
        VirtualKeyCode::Key9 => {
          f.debug_tuple("VirtualKeyCode::Key9").finish()
        }
        VirtualKeyCode::Key0 => {
          f.debug_tuple("VirtualKeyCode::Key0").finish()
        }
        VirtualKeyCode::A => {
          f.debug_tuple("VirtualKeyCode::A").finish()
        }
        VirtualKeyCode::B => {
          f.debug_tuple("VirtualKeyCode::B").finish()
        }
        VirtualKeyCode::C => {
          f.debug_tuple("VirtualKeyCode::C").finish()
        }
        VirtualKeyCode::D => {
          f.debug_tuple("VirtualKeyCode::D").finish()
        }
        VirtualKeyCode::E => {
          f.debug_tuple("VirtualKeyCode::E").finish()
        }
        VirtualKeyCode::F => {
          f.debug_tuple("VirtualKeyCode::F").finish()
        }
        VirtualKeyCode::G => {
          f.debug_tuple("VirtualKeyCode::G").finish()
        }
        VirtualKeyCode::H => {
          f.debug_tuple("VirtualKeyCode::H").finish()
        }
        VirtualKeyCode::I => {
          f.debug_tuple("VirtualKeyCode::I").finish()
        }
        VirtualKeyCode::J => {
          f.debug_tuple("VirtualKeyCode::J").finish()
        }
        VirtualKeyCode::K => {
          f.debug_tuple("VirtualKeyCode::K").finish()
        }
        VirtualKeyCode::L => {
          f.debug_tuple("VirtualKeyCode::L").finish()
        }
        VirtualKeyCode::M => {
          f.debug_tuple("VirtualKeyCode::M").finish()
        }
        VirtualKeyCode::N => {
          f.debug_tuple("VirtualKeyCode::N").finish()
        }
        VirtualKeyCode::O => {
          f.debug_tuple("VirtualKeyCode::O").finish()
        }
        VirtualKeyCode::P => {
          f.debug_tuple("VirtualKeyCode::P").finish()
        }
        VirtualKeyCode::Q => {
          f.debug_tuple("VirtualKeyCode::Q").finish()
        }
        VirtualKeyCode::R => {
          f.debug_tuple("VirtualKeyCode::R").finish()
        }
        VirtualKeyCode::S => {
          f.debug_tuple("VirtualKeyCode::S").finish()
        }
        VirtualKeyCode::T => {
          f.debug_tuple("VirtualKeyCode::T").finish()
        }
        VirtualKeyCode::U => {
          f.debug_tuple("VirtualKeyCode::U").finish()
        }
        VirtualKeyCode::V => {
          f.debug_tuple("VirtualKeyCode::V").finish()
        }
        VirtualKeyCode::W => {
          f.debug_tuple("VirtualKeyCode::W").finish()
        }
        VirtualKeyCode::X => {
          f.debug_tuple("VirtualKeyCode::X").finish()
        }
        VirtualKeyCode::Y => {
          f.debug_tuple("VirtualKeyCode::Y").finish()
        }
        VirtualKeyCode::Z => {
          f.debug_tuple("VirtualKeyCode::Z").finish()
        }
        VirtualKeyCode::Escape => {
          f.debug_tuple("VirtualKeyCode::Escape").finish()
        }
        VirtualKeyCode::F1 => {
          f.debug_tuple("VirtualKeyCode::F1").finish()
        }
        VirtualKeyCode::F2 => {
          f.debug_tuple("VirtualKeyCode::F2").finish()
        }
        VirtualKeyCode::F3 => {
          f.debug_tuple("VirtualKeyCode::F3").finish()
        }
        VirtualKeyCode::F4 => {
          f.debug_tuple("VirtualKeyCode::F4").finish()
        }
        VirtualKeyCode::F5 => {
          f.debug_tuple("VirtualKeyCode::F5").finish()
        }
        VirtualKeyCode::F6 => {
          f.debug_tuple("VirtualKeyCode::F6").finish()
        }
        VirtualKeyCode::F7 => {
          f.debug_tuple("VirtualKeyCode::F7").finish()
        }
        VirtualKeyCode::F8 => {
          f.debug_tuple("VirtualKeyCode::F8").finish()
        }
        VirtualKeyCode::F9 => {
          f.debug_tuple("VirtualKeyCode::F9").finish()
        }
        VirtualKeyCode::F10 => {
          f.debug_tuple("VirtualKeyCode::F10").finish()
        }
        VirtualKeyCode::F11 => {
          f.debug_tuple("VirtualKeyCode::F11").finish()
        }
        VirtualKeyCode::F12 => {
          f.debug_tuple("VirtualKeyCode::F12").finish()
        }
        VirtualKeyCode::F13 => {
          f.debug_tuple("VirtualKeyCode::F13").finish()
        }
        VirtualKeyCode::F14 => {
          f.debug_tuple("VirtualKeyCode::F14").finish()
        }
        VirtualKeyCode::F15 => {
          f.debug_tuple("VirtualKeyCode::F15").finish()
        }
        VirtualKeyCode::F16 => {
          f.debug_tuple("VirtualKeyCode::F16").finish()
        }
        VirtualKeyCode::F17 => {
          f.debug_tuple("VirtualKeyCode::F17").finish()
        }
        VirtualKeyCode::F18 => {
          f.debug_tuple("VirtualKeyCode::F18").finish()
        }
        VirtualKeyCode::F19 => {
          f.debug_tuple("VirtualKeyCode::F19").finish()
        }
        VirtualKeyCode::F20 => {
          f.debug_tuple("VirtualKeyCode::F20").finish()
        }
        VirtualKeyCode::F21 => {
          f.debug_tuple("VirtualKeyCode::F21").finish()
        }
        VirtualKeyCode::F22 => {
          f.debug_tuple("VirtualKeyCode::F22").finish()
        }
        VirtualKeyCode::F23 => {
          f.debug_tuple("VirtualKeyCode::F23").finish()
        }
        VirtualKeyCode::F24 => {
          f.debug_tuple("VirtualKeyCode::F24").finish()
        }
        VirtualKeyCode::Snapshot => {
          f.debug_tuple("VirtualKeyCode::Snapshot").finish()
        }
        VirtualKeyCode::Scroll => {
          f.debug_tuple("VirtualKeyCode::Scroll").finish()
        }
        VirtualKeyCode::Pause => {
          f.debug_tuple("VirtualKeyCode::Pause").finish()
        }
        VirtualKeyCode::Insert => {
          f.debug_tuple("VirtualKeyCode::Insert").finish()
        }
        VirtualKeyCode::Home => {
          f.debug_tuple("VirtualKeyCode::Home").finish()
        }
        VirtualKeyCode::Delete => {
          f.debug_tuple("VirtualKeyCode::Delete").finish()
        }
        VirtualKeyCode::End => {
          f.debug_tuple("VirtualKeyCode::End").finish()
        }
        VirtualKeyCode::PageDown => {
          f.debug_tuple("VirtualKeyCode::PageDown").finish()
        }
        VirtualKeyCode::PageUp => {
          f.debug_tuple("VirtualKeyCode::PageUp").finish()
        }
        VirtualKeyCode::Left => {
          f.debug_tuple("VirtualKeyCode::Left").finish()
        }
        VirtualKeyCode::Up => {
          f.debug_tuple("VirtualKeyCode::Up").finish()
        }
        VirtualKeyCode::Right => {
          f.debug_tuple("VirtualKeyCode::Right").finish()
        }
        VirtualKeyCode::Down => {
          f.debug_tuple("VirtualKeyCode::Down").finish()
        }
        VirtualKeyCode::Back => {
          f.debug_tuple("VirtualKeyCode::Back").finish()
        }
        VirtualKeyCode::Return => {
          f.debug_tuple("VirtualKeyCode::Return").finish()
        }
        VirtualKeyCode::Space => {
          f.debug_tuple("VirtualKeyCode::Space").finish()
        }
        VirtualKeyCode::Compose => {
          f.debug_tuple("VirtualKeyCode::Compose").finish()
        }
        VirtualKeyCode::Caret => {
          f.debug_tuple("VirtualKeyCode::Caret").finish()
        }
        VirtualKeyCode::Numlock => {
          f.debug_tuple("VirtualKeyCode::Numlock").finish()
        }
        VirtualKeyCode::Numpad0 => {
          f.debug_tuple("VirtualKeyCode::Numpad0").finish()
        }
        VirtualKeyCode::Numpad1 => {
          f.debug_tuple("VirtualKeyCode::Numpad1").finish()
        }
        VirtualKeyCode::Numpad2 => {
          f.debug_tuple("VirtualKeyCode::Numpad2").finish()
        }
        VirtualKeyCode::Numpad3 => {
          f.debug_tuple("VirtualKeyCode::Numpad3").finish()
        }
        VirtualKeyCode::Numpad4 => {
          f.debug_tuple("VirtualKeyCode::Numpad4").finish()
        }
        VirtualKeyCode::Numpad5 => {
          f.debug_tuple("VirtualKeyCode::Numpad5").finish()
        }
        VirtualKeyCode::Numpad6 => {
          f.debug_tuple("VirtualKeyCode::Numpad6").finish()
        }
        VirtualKeyCode::Numpad7 => {
          f.debug_tuple("VirtualKeyCode::Numpad7").finish()
        }
        VirtualKeyCode::Numpad8 => {
          f.debug_tuple("VirtualKeyCode::Numpad8").finish()
        }
        VirtualKeyCode::Numpad9 => {
          f.debug_tuple("VirtualKeyCode::Numpad9").finish()
        }
        VirtualKeyCode::NumpadAdd => {
          f.debug_tuple("VirtualKeyCode::NumpadAdd").finish()
        }
        VirtualKeyCode::NumpadDivide => {
          f.debug_tuple("VirtualKeyCode::NumpadDivide").finish()
        }
        VirtualKeyCode::NumpadDecimal => {
          f.debug_tuple("VirtualKeyCode::NumpadDecimal").finish()
        }
        VirtualKeyCode::NumpadComma => {
          f.debug_tuple("VirtualKeyCode::NumpadComma").finish()
        }
        VirtualKeyCode::NumpadEnter => {
          f.debug_tuple("VirtualKeyCode::NumpadEnter").finish()
        }
        VirtualKeyCode::NumpadEquals => {
          f.debug_tuple("VirtualKeyCode::NumpadEquals").finish()
        }
        VirtualKeyCode::NumpadMultiply => {
          f.debug_tuple("VirtualKeyCode::NumpadMultiply").finish()
        }
        VirtualKeyCode::NumpadSubtract => {
          f.debug_tuple("VirtualKeyCode::NumpadSubtract").finish()
        }
        VirtualKeyCode::AbntC1 => {
          f.debug_tuple("VirtualKeyCode::AbntC1").finish()
        }
        VirtualKeyCode::AbntC2 => {
          f.debug_tuple("VirtualKeyCode::AbntC2").finish()
        }
        VirtualKeyCode::Apostrophe => {
          f.debug_tuple("VirtualKeyCode::Apostrophe").finish()
        }
        VirtualKeyCode::Apps => {
          f.debug_tuple("VirtualKeyCode::Apps").finish()
        }
        VirtualKeyCode::Asterisk => {
          f.debug_tuple("VirtualKeyCode::Asterisk").finish()
        }
        VirtualKeyCode::At => {
          f.debug_tuple("VirtualKeyCode::At").finish()
        }
        VirtualKeyCode::Ax => {
          f.debug_tuple("VirtualKeyCode::Ax").finish()
        }
        VirtualKeyCode::Backslash => {
          f.debug_tuple("VirtualKeyCode::Backslash").finish()
        }
        VirtualKeyCode::Calculator => {
          f.debug_tuple("VirtualKeyCode::Calculator").finish()
        }
        VirtualKeyCode::Capital => {
          f.debug_tuple("VirtualKeyCode::Capital").finish()
        }
        VirtualKeyCode::Colon => {
          f.debug_tuple("VirtualKeyCode::Colon").finish()
        }
        VirtualKeyCode::Comma => {
          f.debug_tuple("VirtualKeyCode::Comma").finish()
        }
        VirtualKeyCode::Convert => {
          f.debug_tuple("VirtualKeyCode::Convert").finish()
        }
        VirtualKeyCode::Equals => {
          f.debug_tuple("VirtualKeyCode::Equals").finish()
        }
        VirtualKeyCode::Grave => {
          f.debug_tuple("VirtualKeyCode::Grave").finish()
        }
        VirtualKeyCode::Kana => {
          f.debug_tuple("VirtualKeyCode::Kana").finish()
        }
        VirtualKeyCode::Kanji => {
          f.debug_tuple("VirtualKeyCode::Kanji").finish()
        }
        VirtualKeyCode::LAlt => {
          f.debug_tuple("VirtualKeyCode::LAlt").finish()
        }
        VirtualKeyCode::LBracket => {
          f.debug_tuple("VirtualKeyCode::LBracket").finish()
        }
        VirtualKeyCode::LControl => {
          f.debug_tuple("VirtualKeyCode::LControl").finish()
        }
        VirtualKeyCode::LShift => {
          f.debug_tuple("VirtualKeyCode::LShift").finish()
        }
        VirtualKeyCode::LWin => {
          f.debug_tuple("VirtualKeyCode::LWin").finish()
        }
        VirtualKeyCode::Mail => {
          f.debug_tuple("VirtualKeyCode::Mail").finish()
        }
        VirtualKeyCode::MediaSelect => {
          f.debug_tuple("VirtualKeyCode::MediaSelect").finish()
        }
        VirtualKeyCode::MediaStop => {
          f.debug_tuple("VirtualKeyCode::MediaStop").finish()
        }
        VirtualKeyCode::Minus => {
          f.debug_tuple("VirtualKeyCode::Minus").finish()
        }
        VirtualKeyCode::Mute => {
          f.debug_tuple("VirtualKeyCode::Mute").finish()
        }
        VirtualKeyCode::MyComputer => {
          f.debug_tuple("VirtualKeyCode::MyComputer").finish()
        }
        VirtualKeyCode::NavigateForward => {
          f.debug_tuple("VirtualKeyCode::NavigateForward").finish()
        }
        VirtualKeyCode::NavigateBackward => {
          f.debug_tuple("VirtualKeyCode::NavigateBackward").finish()
        }
        VirtualKeyCode::NextTrack => {
          f.debug_tuple("VirtualKeyCode::NextTrack").finish()
        }
        VirtualKeyCode::NoConvert => {
          f.debug_tuple("VirtualKeyCode::NoConvert").finish()
        }
        VirtualKeyCode::Oem102 => {
          f.debug_tuple("VirtualKeyCode::Oem102").finish()
        }
        VirtualKeyCode::Period => {
          f.debug_tuple("VirtualKeyCode::Period").finish()
        }
        VirtualKeyCode::PlayPause => {
          f.debug_tuple("VirtualKeyCode::PlayPause").finish()
        }
        VirtualKeyCode::Plus => {
          f.debug_tuple("VirtualKeyCode::Plus").finish()
        }
        VirtualKeyCode::Power => {
          f.debug_tuple("VirtualKeyCode::Power").finish()
        }
        VirtualKeyCode::PrevTrack => {
          f.debug_tuple("VirtualKeyCode::PrevTrack").finish()
        }
        VirtualKeyCode::RAlt => {
          f.debug_tuple("VirtualKeyCode::RAlt").finish()
        }
        VirtualKeyCode::RBracket => {
          f.debug_tuple("VirtualKeyCode::RBracket").finish()
        }
        VirtualKeyCode::RControl => {
          f.debug_tuple("VirtualKeyCode::RControl").finish()
        }
        VirtualKeyCode::RShift => {
          f.debug_tuple("VirtualKeyCode::RShift").finish()
        }
        VirtualKeyCode::RWin => {
          f.debug_tuple("VirtualKeyCode::RWin").finish()
        }
        VirtualKeyCode::Semicolon => {
          f.debug_tuple("VirtualKeyCode::Semicolon").finish()
        }
        VirtualKeyCode::Slash => {
          f.debug_tuple("VirtualKeyCode::Slash").finish()
        }
        VirtualKeyCode::Sleep => {
          f.debug_tuple("VirtualKeyCode::Sleep").finish()
        }
        VirtualKeyCode::Stop => {
          f.debug_tuple("VirtualKeyCode::Stop").finish()
        }
        VirtualKeyCode::Sysrq => {
          f.debug_tuple("VirtualKeyCode::Sysrq").finish()
        }
        VirtualKeyCode::Tab => {
          f.debug_tuple("VirtualKeyCode::Tab").finish()
        }
        VirtualKeyCode::Underline => {
          f.debug_tuple("VirtualKeyCode::Underline").finish()
        }
        VirtualKeyCode::Unlabeled => {
          f.debug_tuple("VirtualKeyCode::Unlabeled").finish()
        }
        VirtualKeyCode::VolumeDown => {
          f.debug_tuple("VirtualKeyCode::VolumeDown").finish()
        }
        VirtualKeyCode::VolumeUp => {
          f.debug_tuple("VirtualKeyCode::VolumeUp").finish()
        }
        VirtualKeyCode::Wake => {
          f.debug_tuple("VirtualKeyCode::Wake").finish()
        }
        VirtualKeyCode::WebBack => {
          f.debug_tuple("VirtualKeyCode::WebBack").finish()
        }
        VirtualKeyCode::WebFavorites => {
          f.debug_tuple("VirtualKeyCode::WebFavorites").finish()
        }
        VirtualKeyCode::WebForward => {
          f.debug_tuple("VirtualKeyCode::WebForward").finish()
        }
        VirtualKeyCode::WebHome => {
          f.debug_tuple("VirtualKeyCode::WebHome").finish()
        }
        VirtualKeyCode::WebRefresh => {
          f.debug_tuple("VirtualKeyCode::WebRefresh").finish()
        }
        VirtualKeyCode::WebSearch => {
          f.debug_tuple("VirtualKeyCode::WebSearch").finish()
        }
        VirtualKeyCode::WebStop => {
          f.debug_tuple("VirtualKeyCode::WebStop").finish()
        }
        VirtualKeyCode::Yen => {
          f.debug_tuple("VirtualKeyCode::Yen").finish()
        }
        VirtualKeyCode::Copy => {
          f.debug_tuple("VirtualKeyCode::Copy").finish()
        }
        VirtualKeyCode::Paste => {
          f.debug_tuple("VirtualKeyCode::Paste").finish()
        }
        VirtualKeyCode::Cut => {
          f.debug_tuple("VirtualKeyCode::Cut").finish()
        }
      }
    }
  }
  #[derive(Clone, Copy)]
  pub enum MouseButton{
    Left,
    Right,
    Middle,
    Other(u16),
  }
  impl core::fmt::Debug for MouseButton {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        MouseButton::Left => {
          f.debug_tuple("MouseButton::Left").finish()
        }
        MouseButton::Right => {
          f.debug_tuple("MouseButton::Right").finish()
        }
        MouseButton::Middle => {
          f.debug_tuple("MouseButton::Middle").finish()
        }
        MouseButton::Other(e) => {
          f.debug_tuple("MouseButton::Other").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub struct PlayerRawInput {
    pub keys: Vec<VirtualKeyCode>,
    pub mouse_position: Vec2,
    pub mouse_wheel: f32,
    pub mouse_buttons: Vec<MouseButton>,
  }
  impl core::fmt::Debug for PlayerRawInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("PlayerRawInput").field("keys", &self.keys).field("mouse-position", &self.mouse_position).field("mouse-wheel", &self.mouse_wheel).field("mouse-buttons", &self.mouse_buttons).finish()}
  }
  #[derive(Clone)]
  pub struct Query<'a,> {
    pub components: &'a [u32],
    pub include: &'a [u32],
    pub exclude: &'a [u32],
    pub changed: &'a [u32],
  }
  impl<'a,> core::fmt::Debug for Query<'a,> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Query").field("components", &self.components).field("include", &self.include).field("exclude", &self.exclude).field("changed", &self.changed).finish()}
  }
  #[repr(u8)]
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub enum QueryEvent {
    Frame,
    Spawn,
    Despawn,
  }
  impl core::fmt::Debug for QueryEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        QueryEvent::Frame => {
          f.debug_tuple("QueryEvent::Frame").finish()
        }
        QueryEvent::Spawn => {
          f.debug_tuple("QueryEvent::Spawn").finish()
        }
        QueryEvent::Despawn => {
          f.debug_tuple("QueryEvent::Despawn").finish()
        }
      }
    }
  }
  pub fn component_get_index(id: & str,) -> Option<u32>{
    unsafe {
      let vec0 = id;
      let ptr0 = vec0.as_ptr() as i32;
      let len0 = vec0.len() as i32;
      let ptr1 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
      #[link(wasm_import_module = "host")]
      extern "C" {
        #[cfg_attr(target_arch = "wasm32", link_name = "component-get-index: func(id: string) -> option<u32>")]
        #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_component-get-index: func(id: string) -> option<u32>")]
        fn wit_import(_: i32, _: i32, _: i32, );
      }
      wit_import(ptr0, len0, ptr1);
      match i32::from(*((ptr1 + 0) as *const u8)) {
        0 => None,
        1 => Some(*((ptr1 + 4) as *const i32) as u32),
        _ => panic!("invalid enum discriminant"),
      }
    }
  }
  pub fn entity_spawn(data: Components<'_,>,) -> EntityId{
    unsafe {
      let mut cleanup_list = Vec::new();
      let vec47 = data;
      let len47 = vec47.len() as i32;
      let layout47 = core::alloc::Layout::from_size_align_unchecked(vec47.len() * 88, 8);
      let result47 = if layout47.size() != 0
      {
        let ptr = std::alloc::alloc(layout47);
        if ptr.is_null()
        {
          std::alloc::handle_alloc_error(layout47);
        }
        ptr
      }else {
        std::ptr::null_mut()
      };
      for (i, e) in vec47.into_iter().enumerate() {
        let base = result47 as i32 + (i as i32) * 88;
        {
          let (t0_0, t0_1, ) = e;
          *((base + 0) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(t0_0);
          match t0_1 {
            ComponentTypeParam::TypeEmpty(e) => {
              *((base + 8) as *mut u8) = (0i32) as u8;
              let () = e;
              
            },
            ComponentTypeParam::TypeBool(e) => {
              *((base + 8) as *mut u8) = (1i32) as u8;
              *((base + 16) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
              
            },
            ComponentTypeParam::TypeEntityId(e) => {
              *((base + 8) as *mut u8) = (2i32) as u8;
              let EntityId{ id0:id02, id1:id12, } = e;
              *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id02);
              *((base + 24) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id12);
              
            },
            ComponentTypeParam::TypeF32(e) => {
              *((base + 8) as *mut u8) = (3i32) as u8;
              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
              
            },
            ComponentTypeParam::TypeF64(e) => {
              *((base + 8) as *mut u8) = (4i32) as u8;
              *((base + 16) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
              
            },
            ComponentTypeParam::TypeMat4(e) => {
              *((base + 8) as *mut u8) = (5i32) as u8;
              let Mat4{ x:x3, y:y3, z:z3, w:w3, } = e;
              let Vec4{ x:x4, y:y4, z:z4, w:w4, } = x3;
              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x4);
              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y4);
              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z4);
              *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w4);
              let Vec4{ x:x5, y:y5, z:z5, w:w5, } = y3;
              *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x5);
              *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y5);
              *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z5);
              *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w5);
              let Vec4{ x:x6, y:y6, z:z6, w:w6, } = z3;
              *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x6);
              *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y6);
              *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z6);
              *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w6);
              let Vec4{ x:x7, y:y7, z:z7, w:w7, } = w3;
              *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x7);
              *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y7);
              *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z7);
              *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w7);
              
            },
            ComponentTypeParam::TypeI32(e) => {
              *((base + 8) as *mut u8) = (6i32) as u8;
              *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
              
            },
            ComponentTypeParam::TypeQuat(e) => {
              *((base + 8) as *mut u8) = (7i32) as u8;
              let Quat{ x:x8, y:y8, z:z8, w:w8, } = e;
              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x8);
              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y8);
              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z8);
              *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w8);
              
            },
            ComponentTypeParam::TypeString(e) => {
              *((base + 8) as *mut u8) = (8i32) as u8;
              let vec9 = e;
              let ptr9 = vec9.as_ptr() as i32;
              let len9 = vec9.len() as i32;
              *((base + 20) as *mut i32) = len9;
              *((base + 16) as *mut i32) = ptr9;
              
            },
            ComponentTypeParam::TypeU32(e) => {
              *((base + 8) as *mut u8) = (9i32) as u8;
              *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
              
            },
            ComponentTypeParam::TypeU64(e) => {
              *((base + 8) as *mut u8) = (10i32) as u8;
              *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
              
            },
            ComponentTypeParam::TypeVec2(e) => {
              *((base + 8) as *mut u8) = (11i32) as u8;
              let Vec2{ x:x10, y:y10, } = e;
              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x10);
              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y10);
              
            },
            ComponentTypeParam::TypeVec3(e) => {
              *((base + 8) as *mut u8) = (12i32) as u8;
              let Vec3{ x:x11, y:y11, z:z11, } = e;
              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x11);
              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y11);
              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z11);
              
            },
            ComponentTypeParam::TypeVec4(e) => {
              *((base + 8) as *mut u8) = (13i32) as u8;
              let Vec4{ x:x12, y:y12, z:z12, w:w12, } = e;
              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x12);
              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y12);
              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z12);
              *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w12);
              
            },
            ComponentTypeParam::TypeObjectRef(e) => {
              *((base + 8) as *mut u8) = (14i32) as u8;
              let ObjectRefParam{ id:id13, } = e;
              let vec14 = id13;
              let ptr14 = vec14.as_ptr() as i32;
              let len14 = vec14.len() as i32;
              *((base + 20) as *mut i32) = len14;
              *((base + 16) as *mut i32) = ptr14;
              
            },
            ComponentTypeParam::TypeList(e) => {
              *((base + 8) as *mut u8) = (15i32) as u8;
              match e {
                ComponentListTypeParam::TypeEmpty(e) => {
                  *((base + 16) as *mut u8) = (0i32) as u8;
                  let vec15 = e;
                  let ptr15 = vec15.as_ptr() as i32;
                  let len15 = vec15.len() as i32;
                  *((base + 24) as *mut i32) = len15;
                  *((base + 20) as *mut i32) = ptr15;
                  
                },
                ComponentListTypeParam::TypeBool(e) => {
                  *((base + 16) as *mut u8) = (1i32) as u8;
                  let vec16 = e;
                  let len16 = vec16.len() as i32;
                  let layout16 = core::alloc::Layout::from_size_align_unchecked(vec16.len() * 1, 1);
                  let result16 = if layout16.size() != 0
                  {
                    let ptr = std::alloc::alloc(layout16);
                    if ptr.is_null()
                    {
                      std::alloc::handle_alloc_error(layout16);
                    }
                    ptr
                  }else {
                    std::ptr::null_mut()
                  };
                  for (i, e) in vec16.into_iter().enumerate() {
                    let base = result16 as i32 + (i as i32) * 1;
                    {
                      *((base + 0) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                      
                    }}
                    *((base + 24) as *mut i32) = len16;
                    *((base + 20) as *mut i32) = result16 as i32;
                    cleanup_list.extend_from_slice(&[(result16, layout16),]);
                    
                  },
                  ComponentListTypeParam::TypeEntityId(e) => {
                    *((base + 16) as *mut u8) = (2i32) as u8;
                    let vec17 = e;
                    let ptr17 = vec17.as_ptr() as i32;
                    let len17 = vec17.len() as i32;
                    *((base + 24) as *mut i32) = len17;
                    *((base + 20) as *mut i32) = ptr17;
                    
                  },
                  ComponentListTypeParam::TypeF32(e) => {
                    *((base + 16) as *mut u8) = (3i32) as u8;
                    let vec18 = e;
                    let ptr18 = vec18.as_ptr() as i32;
                    let len18 = vec18.len() as i32;
                    *((base + 24) as *mut i32) = len18;
                    *((base + 20) as *mut i32) = ptr18;
                    
                  },
                  ComponentListTypeParam::TypeF64(e) => {
                    *((base + 16) as *mut u8) = (4i32) as u8;
                    let vec19 = e;
                    let ptr19 = vec19.as_ptr() as i32;
                    let len19 = vec19.len() as i32;
                    *((base + 24) as *mut i32) = len19;
                    *((base + 20) as *mut i32) = ptr19;
                    
                  },
                  ComponentListTypeParam::TypeMat4(e) => {
                    *((base + 16) as *mut u8) = (5i32) as u8;
                    let vec20 = e;
                    let ptr20 = vec20.as_ptr() as i32;
                    let len20 = vec20.len() as i32;
                    *((base + 24) as *mut i32) = len20;
                    *((base + 20) as *mut i32) = ptr20;
                    
                  },
                  ComponentListTypeParam::TypeI32(e) => {
                    *((base + 16) as *mut u8) = (6i32) as u8;
                    let vec21 = e;
                    let ptr21 = vec21.as_ptr() as i32;
                    let len21 = vec21.len() as i32;
                    *((base + 24) as *mut i32) = len21;
                    *((base + 20) as *mut i32) = ptr21;
                    
                  },
                  ComponentListTypeParam::TypeQuat(e) => {
                    *((base + 16) as *mut u8) = (7i32) as u8;
                    let vec22 = e;
                    let ptr22 = vec22.as_ptr() as i32;
                    let len22 = vec22.len() as i32;
                    *((base + 24) as *mut i32) = len22;
                    *((base + 20) as *mut i32) = ptr22;
                    
                  },
                  ComponentListTypeParam::TypeString(e) => {
                    *((base + 16) as *mut u8) = (8i32) as u8;
                    let vec24 = e;
                    let len24 = vec24.len() as i32;
                    let layout24 = core::alloc::Layout::from_size_align_unchecked(vec24.len() * 8, 4);
                    let result24 = if layout24.size() != 0
                    {
                      let ptr = std::alloc::alloc(layout24);
                      if ptr.is_null()
                      {
                        std::alloc::handle_alloc_error(layout24);
                      }
                      ptr
                    }else {
                      std::ptr::null_mut()
                    };
                    for (i, e) in vec24.into_iter().enumerate() {
                      let base = result24 as i32 + (i as i32) * 8;
                      {
                        let vec23 = e;
                        let ptr23 = vec23.as_ptr() as i32;
                        let len23 = vec23.len() as i32;
                        *((base + 4) as *mut i32) = len23;
                        *((base + 0) as *mut i32) = ptr23;
                        
                      }}
                      *((base + 24) as *mut i32) = len24;
                      *((base + 20) as *mut i32) = result24 as i32;
                      cleanup_list.extend_from_slice(&[(result24, layout24),]);
                      
                    },
                    ComponentListTypeParam::TypeU32(e) => {
                      *((base + 16) as *mut u8) = (9i32) as u8;
                      let vec25 = e;
                      let ptr25 = vec25.as_ptr() as i32;
                      let len25 = vec25.len() as i32;
                      *((base + 24) as *mut i32) = len25;
                      *((base + 20) as *mut i32) = ptr25;
                      
                    },
                    ComponentListTypeParam::TypeU64(e) => {
                      *((base + 16) as *mut u8) = (10i32) as u8;
                      let vec26 = e;
                      let ptr26 = vec26.as_ptr() as i32;
                      let len26 = vec26.len() as i32;
                      *((base + 24) as *mut i32) = len26;
                      *((base + 20) as *mut i32) = ptr26;
                      
                    },
                    ComponentListTypeParam::TypeVec2(e) => {
                      *((base + 16) as *mut u8) = (11i32) as u8;
                      let vec27 = e;
                      let ptr27 = vec27.as_ptr() as i32;
                      let len27 = vec27.len() as i32;
                      *((base + 24) as *mut i32) = len27;
                      *((base + 20) as *mut i32) = ptr27;
                      
                    },
                    ComponentListTypeParam::TypeVec3(e) => {
                      *((base + 16) as *mut u8) = (12i32) as u8;
                      let vec28 = e;
                      let ptr28 = vec28.as_ptr() as i32;
                      let len28 = vec28.len() as i32;
                      *((base + 24) as *mut i32) = len28;
                      *((base + 20) as *mut i32) = ptr28;
                      
                    },
                    ComponentListTypeParam::TypeVec4(e) => {
                      *((base + 16) as *mut u8) = (13i32) as u8;
                      let vec29 = e;
                      let ptr29 = vec29.as_ptr() as i32;
                      let len29 = vec29.len() as i32;
                      *((base + 24) as *mut i32) = len29;
                      *((base + 20) as *mut i32) = ptr29;
                      
                    },
                    ComponentListTypeParam::TypeObjectRef(e) => {
                      *((base + 16) as *mut u8) = (14i32) as u8;
                      let vec32 = e;
                      let len32 = vec32.len() as i32;
                      let layout32 = core::alloc::Layout::from_size_align_unchecked(vec32.len() * 8, 4);
                      let result32 = if layout32.size() != 0
                      {
                        let ptr = std::alloc::alloc(layout32);
                        if ptr.is_null()
                        {
                          std::alloc::handle_alloc_error(layout32);
                        }
                        ptr
                      }else {
                        std::ptr::null_mut()
                      };
                      for (i, e) in vec32.into_iter().enumerate() {
                        let base = result32 as i32 + (i as i32) * 8;
                        {
                          let ObjectRefParam{ id:id30, } = e;
                          let vec31 = id30;
                          let ptr31 = vec31.as_ptr() as i32;
                          let len31 = vec31.len() as i32;
                          *((base + 4) as *mut i32) = len31;
                          *((base + 0) as *mut i32) = ptr31;
                          
                        }}
                        *((base + 24) as *mut i32) = len32;
                        *((base + 20) as *mut i32) = result32 as i32;
                        cleanup_list.extend_from_slice(&[(result32, layout32),]);
                        
                      },
                    };
                    
                  },
                  ComponentTypeParam::TypeOption(e) => {
                    *((base + 8) as *mut u8) = (16i32) as u8;
                    match e {
                      ComponentOptionTypeParam::TypeEmpty(e) => {
                        *((base + 16) as *mut u8) = (0i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let () = e;
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeBool(e) => {
                        *((base + 16) as *mut u8) = (1i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            *((base + 25) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeEntityId(e) => {
                        *((base + 16) as *mut u8) = (2i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let EntityId{ id0:id034, id1:id134, } = e;
                            *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id034);
                            *((base + 40) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id134);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeF32(e) => {
                        *((base + 16) as *mut u8) = (3i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeF64(e) => {
                        *((base + 16) as *mut u8) = (4i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            *((base + 32) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeMat4(e) => {
                        *((base + 16) as *mut u8) = (5i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let Mat4{ x:x35, y:y35, z:z35, w:w35, } = e;
                            let Vec4{ x:x36, y:y36, z:z36, w:w36, } = x35;
                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x36);
                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y36);
                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z36);
                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w36);
                            let Vec4{ x:x37, y:y37, z:z37, w:w37, } = y35;
                            *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x37);
                            *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y37);
                            *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z37);
                            *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w37);
                            let Vec4{ x:x38, y:y38, z:z38, w:w38, } = z35;
                            *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x38);
                            *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y38);
                            *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z38);
                            *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w38);
                            let Vec4{ x:x39, y:y39, z:z39, w:w39, } = w35;
                            *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x39);
                            *((base + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y39);
                            *((base + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z39);
                            *((base + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w39);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeI32(e) => {
                        *((base + 16) as *mut u8) = (6i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeQuat(e) => {
                        *((base + 16) as *mut u8) = (7i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let Quat{ x:x40, y:y40, z:z40, w:w40, } = e;
                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x40);
                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y40);
                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z40);
                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w40);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeString(e) => {
                        *((base + 16) as *mut u8) = (8i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let vec41 = e;
                            let ptr41 = vec41.as_ptr() as i32;
                            let len41 = vec41.len() as i32;
                            *((base + 32) as *mut i32) = len41;
                            *((base + 28) as *mut i32) = ptr41;
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeU32(e) => {
                        *((base + 16) as *mut u8) = (9i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeU64(e) => {
                        *((base + 16) as *mut u8) = (10i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeVec2(e) => {
                        *((base + 16) as *mut u8) = (11i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let Vec2{ x:x42, y:y42, } = e;
                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x42);
                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y42);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeVec3(e) => {
                        *((base + 16) as *mut u8) = (12i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let Vec3{ x:x43, y:y43, z:z43, } = e;
                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x43);
                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y43);
                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z43);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeVec4(e) => {
                        *((base + 16) as *mut u8) = (13i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let Vec4{ x:x44, y:y44, z:z44, w:w44, } = e;
                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x44);
                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y44);
                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z44);
                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w44);
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                      ComponentOptionTypeParam::TypeObjectRef(e) => {
                        *((base + 16) as *mut u8) = (14i32) as u8;
                        match e {
                          Some(e) => {
                            *((base + 24) as *mut u8) = (1i32) as u8;
                            let ObjectRefParam{ id:id45, } = e;
                            let vec46 = id45;
                            let ptr46 = vec46.as_ptr() as i32;
                            let len46 = vec46.len() as i32;
                            *((base + 32) as *mut i32) = len46;
                            *((base + 28) as *mut i32) = ptr46;
                            
                          },
                          None => {
                            let e = ();
                            {
                              *((base + 24) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            }
                          },
                        };
                      },
                    };
                    
                  },
                };
                
              }}
              let ptr48 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
              #[link(wasm_import_module = "host")]
              extern "C" {
                #[cfg_attr(target_arch = "wasm32", link_name = "entity-spawn: func(data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> record { id0: u64, id1: u64 }")]
                #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-spawn: func(data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> record { id0: u64, id1: u64 }")]
                fn wit_import(_: i32, _: i32, _: i32, );
              }
              wit_import(result47 as i32, len47, ptr48);
              if layout47.size() != 0 {
                std::alloc::dealloc(result47, layout47);
              }
              for (ptr, layout) in cleanup_list {
                
                if layout.size() != 0 {
                  
                  std::alloc::dealloc(ptr, layout);
                  
                }
                
              }
              EntityId{id0:*((ptr48 + 0) as *const i64) as u64, id1:*((ptr48 + 8) as *const i64) as u64, }
            }
          }
          pub fn entity_despawn(entity: EntityId,) -> bool{
            unsafe {
              let EntityId{ id0:id00, id1:id10, } = entity;
              #[link(wasm_import_module = "host")]
              extern "C" {
                #[cfg_attr(target_arch = "wasm32", link_name = "entity-despawn: func(entity: record { id0: u64, id1: u64 }) -> bool")]
                #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-despawn: func(entity: record { id0: u64, id1: u64 }) -> bool")]
                fn wit_import(_: i64, _: i64, ) -> i32;
              }
              let ret = wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10));
              match ret {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
              }
            }
          }
          pub fn entity_set_animation_controller(entity: EntityId,animation_controller: AnimationController<'_,>,) -> (){
            unsafe {
              let EntityId{ id0:id00, id1:id10, } = entity;
              let AnimationController{ actions:actions1, apply_base_pose:apply_base_pose1, } = animation_controller;
              let vec4 = actions1;
              let len4 = vec4.len() as i32;
              let layout4 = core::alloc::Layout::from_size_align_unchecked(vec4.len() * 16, 4);
              let result4 = if layout4.size() != 0
              {
                let ptr = std::alloc::alloc(layout4);
                if ptr.is_null()
                {
                  std::alloc::handle_alloc_error(layout4);
                }
                ptr
              }else {
                std::ptr::null_mut()
              };
              for (i, e) in vec4.into_iter().enumerate() {
                let base = result4 as i32 + (i as i32) * 16;
                {
                  let AnimationAction{ clip_url:clip_url2, looping:looping2, weight:weight2, } = e;
                  let vec3 = clip_url2;
                  let ptr3 = vec3.as_ptr() as i32;
                  let len3 = vec3.len() as i32;
                  *((base + 4) as *mut i32) = len3;
                  *((base + 0) as *mut i32) = ptr3;
                  *((base + 8) as *mut u8) = (match looping2 { true => 1, false => 0 }) as u8;
                  *((base + 12) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(weight2);
                  
                }}
                #[link(wasm_import_module = "host")]
                extern "C" {
                  #[cfg_attr(target_arch = "wasm32", link_name = "entity-set-animation-controller: func(entity: record { id0: u64, id1: u64 }, animation-controller: record { actions: list<record { clip-url: string, looping: bool, weight: float32 }>, apply-base-pose: bool }) -> unit")]
                  #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-set-animation-controller: func(entity: record { id0: u64, id1: u64 }, animation-controller: record { actions: list<record { clip-url: string, looping: bool, weight: float32 }>, apply-base-pose: bool }) -> unit")]
                  fn wit_import(_: i64, _: i64, _: i32, _: i32, _: i32, );
                }
                wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), result4 as i32, len4, match apply_base_pose1 { true => 1, false => 0 });
                if layout4.size() != 0 {
                  std::alloc::dealloc(result4, layout4);
                }
                ()
              }
            }
            pub fn entity_in_area(position: Vec3,radius: f32,) -> Vec<EntityId>{
              unsafe {
                let Vec3{ x:x0, y:y0, z:z0, } = position;
                let ptr1 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "host")]
                extern "C" {
                  #[cfg_attr(target_arch = "wasm32", link_name = "entity-in-area: func(position: record { x: float32, y: float32, z: float32 }, radius: float32) -> list<record { id0: u64, id1: u64 }>")]
                  #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-in-area: func(position: record { x: float32, y: float32, z: float32 }, radius: float32) -> list<record { id0: u64, id1: u64 }>")]
                  fn wit_import(_: f32, _: f32, _: f32, _: f32, _: i32, );
                }
                wit_import(wit_bindgen_guest_rust::rt::as_f32(x0), wit_bindgen_guest_rust::rt::as_f32(y0), wit_bindgen_guest_rust::rt::as_f32(z0), wit_bindgen_guest_rust::rt::as_f32(radius), ptr1);
                let len2 = *((ptr1 + 4) as *const i32) as usize;
                Vec::from_raw_parts(*((ptr1 + 0) as *const i32) as *mut _, len2, len2)
              }
            }
            pub fn entity_get_component(entity: EntityId,index: u32,) -> Option<ComponentTypeResult>{
              unsafe {
                let EntityId{ id0:id00, id1:id10, } = entity;
                let ptr1 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                #[link(wasm_import_module = "host")]
                extern "C" {
                  #[cfg_attr(target_arch = "wasm32", link_name = "entity-get-component: func(entity: record { id0: u64, id1: u64 }, index: u32) -> option<variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>")]
                  #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-get-component: func(entity: record { id0: u64, id1: u64 }, index: u32) -> option<variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>")]
                  fn wit_import(_: i64, _: i64, _: i32, _: i32, );
                }
                wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), wit_bindgen_guest_rust::rt::as_i32(index), ptr1);
                match i32::from(*((ptr1 + 0) as *const u8)) {
                  0 => None,
                  1 => Some(match i32::from(*((ptr1 + 8) as *const u8)) {
                    0 => ComponentTypeResult::TypeEmpty(()),
                    1 => ComponentTypeResult::TypeBool(match i32::from(*((ptr1 + 16) as *const u8)) {
                      0 => false,
                      1 => true,
                      _ => panic!("invalid bool discriminant"),
                    }),
                    2 => ComponentTypeResult::TypeEntityId(EntityId{id0:*((ptr1 + 16) as *const i64) as u64, id1:*((ptr1 + 24) as *const i64) as u64, }),
                    3 => ComponentTypeResult::TypeF32(*((ptr1 + 16) as *const f32)),
                    4 => ComponentTypeResult::TypeF64(*((ptr1 + 16) as *const f64)),
                    5 => ComponentTypeResult::TypeMat4(Mat4{x:Vec4{x:*((ptr1 + 16) as *const f32), y:*((ptr1 + 20) as *const f32), z:*((ptr1 + 24) as *const f32), w:*((ptr1 + 28) as *const f32), }, y:Vec4{x:*((ptr1 + 32) as *const f32), y:*((ptr1 + 36) as *const f32), z:*((ptr1 + 40) as *const f32), w:*((ptr1 + 44) as *const f32), }, z:Vec4{x:*((ptr1 + 48) as *const f32), y:*((ptr1 + 52) as *const f32), z:*((ptr1 + 56) as *const f32), w:*((ptr1 + 60) as *const f32), }, w:Vec4{x:*((ptr1 + 64) as *const f32), y:*((ptr1 + 68) as *const f32), z:*((ptr1 + 72) as *const f32), w:*((ptr1 + 76) as *const f32), }, }),
                    6 => ComponentTypeResult::TypeI32(*((ptr1 + 16) as *const i32)),
                    7 => ComponentTypeResult::TypeQuat(Quat{x:*((ptr1 + 16) as *const f32), y:*((ptr1 + 20) as *const f32), z:*((ptr1 + 24) as *const f32), w:*((ptr1 + 28) as *const f32), }),
                    8 => ComponentTypeResult::TypeString({
                      let len2 = *((ptr1 + 20) as *const i32) as usize;
                      
                      String::from_utf8(Vec::from_raw_parts(*((ptr1 + 16) as *const i32) as *mut _, len2, len2)).unwrap()
                    }),
                    9 => ComponentTypeResult::TypeU32(*((ptr1 + 16) as *const i32) as u32),
                    10 => ComponentTypeResult::TypeU64(*((ptr1 + 16) as *const i64) as u64),
                    11 => ComponentTypeResult::TypeVec2(Vec2{x:*((ptr1 + 16) as *const f32), y:*((ptr1 + 20) as *const f32), }),
                    12 => ComponentTypeResult::TypeVec3(Vec3{x:*((ptr1 + 16) as *const f32), y:*((ptr1 + 20) as *const f32), z:*((ptr1 + 24) as *const f32), }),
                    13 => ComponentTypeResult::TypeVec4(Vec4{x:*((ptr1 + 16) as *const f32), y:*((ptr1 + 20) as *const f32), z:*((ptr1 + 24) as *const f32), w:*((ptr1 + 28) as *const f32), }),
                    14 => ComponentTypeResult::TypeObjectRef({
                      let len3 = *((ptr1 + 20) as *const i32) as usize;
                      
                      ObjectRefResult{id:String::from_utf8(Vec::from_raw_parts(*((ptr1 + 16) as *const i32) as *mut _, len3, len3)).unwrap(), }
                    }),
                    15 => ComponentTypeResult::TypeList(match i32::from(*((ptr1 + 16) as *const u8)) {
                      0 => ComponentListTypeResult::TypeEmpty({
                        let len4 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len4, len4)
                      }),
                      1 => ComponentListTypeResult::TypeBool({
                        let base5 = *((ptr1 + 20) as *const i32);
                        let len5 = *((ptr1 + 24) as *const i32);
                        let mut result5 = Vec::with_capacity(len5 as usize);
                        for i in 0..len5 {
                          let base = base5 + i *1;
                          result5.push(match i32::from(*((base + 0) as *const u8)) {
                            0 => false,
                            1 => true,
                            _ => panic!("invalid bool discriminant"),
                          });
                        }
                        if len5 != 0 {
                          std::alloc::dealloc(base5 as *mut _, std::alloc::Layout::from_size_align_unchecked((len5 as usize) * 1, 1));
                        }
                        
                        result5
                      }),
                      2 => ComponentListTypeResult::TypeEntityId({
                        let len6 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len6, len6)
                      }),
                      3 => ComponentListTypeResult::TypeF32({
                        let len7 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len7, len7)
                      }),
                      4 => ComponentListTypeResult::TypeF64({
                        let len8 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len8, len8)
                      }),
                      5 => ComponentListTypeResult::TypeMat4({
                        let len9 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len9, len9)
                      }),
                      6 => ComponentListTypeResult::TypeI32({
                        let len10 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len10, len10)
                      }),
                      7 => ComponentListTypeResult::TypeQuat({
                        let len11 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len11, len11)
                      }),
                      8 => ComponentListTypeResult::TypeString({
                        let base13 = *((ptr1 + 20) as *const i32);
                        let len13 = *((ptr1 + 24) as *const i32);
                        let mut result13 = Vec::with_capacity(len13 as usize);
                        for i in 0..len13 {
                          let base = base13 + i *8;
                          result13.push({
                            let len12 = *((base + 4) as *const i32) as usize;
                            
                            String::from_utf8(Vec::from_raw_parts(*((base + 0) as *const i32) as *mut _, len12, len12)).unwrap()
                          });
                        }
                        if len13 != 0 {
                          std::alloc::dealloc(base13 as *mut _, std::alloc::Layout::from_size_align_unchecked((len13 as usize) * 8, 4));
                        }
                        
                        result13
                      }),
                      9 => ComponentListTypeResult::TypeU32({
                        let len14 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len14, len14)
                      }),
                      10 => ComponentListTypeResult::TypeU64({
                        let len15 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len15, len15)
                      }),
                      11 => ComponentListTypeResult::TypeVec2({
                        let len16 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len16, len16)
                      }),
                      12 => ComponentListTypeResult::TypeVec3({
                        let len17 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len17, len17)
                      }),
                      13 => ComponentListTypeResult::TypeVec4({
                        let len18 = *((ptr1 + 24) as *const i32) as usize;
                        
                        Vec::from_raw_parts(*((ptr1 + 20) as *const i32) as *mut _, len18, len18)
                      }),
                      14 => ComponentListTypeResult::TypeObjectRef({
                        let base20 = *((ptr1 + 20) as *const i32);
                        let len20 = *((ptr1 + 24) as *const i32);
                        let mut result20 = Vec::with_capacity(len20 as usize);
                        for i in 0..len20 {
                          let base = base20 + i *8;
                          result20.push({
                            let len19 = *((base + 4) as *const i32) as usize;
                            
                            ObjectRefResult{id:String::from_utf8(Vec::from_raw_parts(*((base + 0) as *const i32) as *mut _, len19, len19)).unwrap(), }
                          });
                        }
                        if len20 != 0 {
                          std::alloc::dealloc(base20 as *mut _, std::alloc::Layout::from_size_align_unchecked((len20 as usize) * 8, 4));
                        }
                        
                        result20
                      }),
                      _ => panic!("invalid enum discriminant"),
                    }),
                    16 => ComponentTypeResult::TypeOption(match i32::from(*((ptr1 + 16) as *const u8)) {
                      0 => ComponentOptionTypeResult::TypeEmpty(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(()),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      1 => ComponentOptionTypeResult::TypeBool(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(match i32::from(*((ptr1 + 25) as *const u8)) {
                          0 => false,
                          1 => true,
                          _ => panic!("invalid bool discriminant"),
                        }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      2 => ComponentOptionTypeResult::TypeEntityId(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(EntityId{id0:*((ptr1 + 32) as *const i64) as u64, id1:*((ptr1 + 40) as *const i64) as u64, }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      3 => ComponentOptionTypeResult::TypeF32(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(*((ptr1 + 28) as *const f32)),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      4 => ComponentOptionTypeResult::TypeF64(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(*((ptr1 + 32) as *const f64)),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      5 => ComponentOptionTypeResult::TypeMat4(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(Mat4{x:Vec4{x:*((ptr1 + 28) as *const f32), y:*((ptr1 + 32) as *const f32), z:*((ptr1 + 36) as *const f32), w:*((ptr1 + 40) as *const f32), }, y:Vec4{x:*((ptr1 + 44) as *const f32), y:*((ptr1 + 48) as *const f32), z:*((ptr1 + 52) as *const f32), w:*((ptr1 + 56) as *const f32), }, z:Vec4{x:*((ptr1 + 60) as *const f32), y:*((ptr1 + 64) as *const f32), z:*((ptr1 + 68) as *const f32), w:*((ptr1 + 72) as *const f32), }, w:Vec4{x:*((ptr1 + 76) as *const f32), y:*((ptr1 + 80) as *const f32), z:*((ptr1 + 84) as *const f32), w:*((ptr1 + 88) as *const f32), }, }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      6 => ComponentOptionTypeResult::TypeI32(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(*((ptr1 + 28) as *const i32)),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      7 => ComponentOptionTypeResult::TypeQuat(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(Quat{x:*((ptr1 + 28) as *const f32), y:*((ptr1 + 32) as *const f32), z:*((ptr1 + 36) as *const f32), w:*((ptr1 + 40) as *const f32), }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      8 => ComponentOptionTypeResult::TypeString(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some({
                          let len21 = *((ptr1 + 32) as *const i32) as usize;
                          
                          String::from_utf8(Vec::from_raw_parts(*((ptr1 + 28) as *const i32) as *mut _, len21, len21)).unwrap()
                        }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      9 => ComponentOptionTypeResult::TypeU32(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(*((ptr1 + 28) as *const i32) as u32),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      10 => ComponentOptionTypeResult::TypeU64(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(*((ptr1 + 32) as *const i64) as u64),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      11 => ComponentOptionTypeResult::TypeVec2(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(Vec2{x:*((ptr1 + 28) as *const f32), y:*((ptr1 + 32) as *const f32), }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      12 => ComponentOptionTypeResult::TypeVec3(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(Vec3{x:*((ptr1 + 28) as *const f32), y:*((ptr1 + 32) as *const f32), z:*((ptr1 + 36) as *const f32), }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      13 => ComponentOptionTypeResult::TypeVec4(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some(Vec4{x:*((ptr1 + 28) as *const f32), y:*((ptr1 + 32) as *const f32), z:*((ptr1 + 36) as *const f32), w:*((ptr1 + 40) as *const f32), }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      14 => ComponentOptionTypeResult::TypeObjectRef(match i32::from(*((ptr1 + 24) as *const u8)) {
                        0 => None,
                        1 => Some({
                          let len22 = *((ptr1 + 32) as *const i32) as usize;
                          
                          ObjectRefResult{id:String::from_utf8(Vec::from_raw_parts(*((ptr1 + 28) as *const i32) as *mut _, len22, len22)).unwrap(), }
                        }),
                        _ => panic!("invalid enum discriminant"),
                      }),
                      _ => panic!("invalid enum discriminant"),
                    }),
                    _ => panic!("invalid enum discriminant"),
                  }),
                  _ => panic!("invalid enum discriminant"),
                }
              }
            }
            pub fn entity_add_component(entity: EntityId,index: u32,value: ComponentTypeParam<'_,>,) -> (){
              unsafe {
                let mut cleanup_list = Vec::new();
                let ptr0 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                let EntityId{ id0:id01, id1:id11, } = entity;
                *((ptr0 + 0) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id01);
                *((ptr0 + 8) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id11);
                *((ptr0 + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(index);
                match value {
                  ComponentTypeParam::TypeEmpty(e) => {
                    *((ptr0 + 24) as *mut u8) = (0i32) as u8;
                    let () = e;
                    
                  },
                  ComponentTypeParam::TypeBool(e) => {
                    *((ptr0 + 24) as *mut u8) = (1i32) as u8;
                    *((ptr0 + 32) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                    
                  },
                  ComponentTypeParam::TypeEntityId(e) => {
                    *((ptr0 + 24) as *mut u8) = (2i32) as u8;
                    let EntityId{ id0:id03, id1:id13, } = e;
                    *((ptr0 + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id03);
                    *((ptr0 + 40) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id13);
                    
                  },
                  ComponentTypeParam::TypeF32(e) => {
                    *((ptr0 + 24) as *mut u8) = (3i32) as u8;
                    *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                    
                  },
                  ComponentTypeParam::TypeF64(e) => {
                    *((ptr0 + 24) as *mut u8) = (4i32) as u8;
                    *((ptr0 + 32) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                    
                  },
                  ComponentTypeParam::TypeMat4(e) => {
                    *((ptr0 + 24) as *mut u8) = (5i32) as u8;
                    let Mat4{ x:x4, y:y4, z:z4, w:w4, } = e;
                    let Vec4{ x:x5, y:y5, z:z5, w:w5, } = x4;
                    *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x5);
                    *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y5);
                    *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z5);
                    *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w5);
                    let Vec4{ x:x6, y:y6, z:z6, w:w6, } = y4;
                    *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x6);
                    *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y6);
                    *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z6);
                    *((ptr0 + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w6);
                    let Vec4{ x:x7, y:y7, z:z7, w:w7, } = z4;
                    *((ptr0 + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x7);
                    *((ptr0 + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y7);
                    *((ptr0 + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z7);
                    *((ptr0 + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w7);
                    let Vec4{ x:x8, y:y8, z:z8, w:w8, } = w4;
                    *((ptr0 + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x8);
                    *((ptr0 + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y8);
                    *((ptr0 + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z8);
                    *((ptr0 + 92) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w8);
                    
                  },
                  ComponentTypeParam::TypeI32(e) => {
                    *((ptr0 + 24) as *mut u8) = (6i32) as u8;
                    *((ptr0 + 32) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                    
                  },
                  ComponentTypeParam::TypeQuat(e) => {
                    *((ptr0 + 24) as *mut u8) = (7i32) as u8;
                    let Quat{ x:x9, y:y9, z:z9, w:w9, } = e;
                    *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x9);
                    *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y9);
                    *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z9);
                    *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w9);
                    
                  },
                  ComponentTypeParam::TypeString(e) => {
                    *((ptr0 + 24) as *mut u8) = (8i32) as u8;
                    let vec10 = e;
                    let ptr10 = vec10.as_ptr() as i32;
                    let len10 = vec10.len() as i32;
                    *((ptr0 + 36) as *mut i32) = len10;
                    *((ptr0 + 32) as *mut i32) = ptr10;
                    
                  },
                  ComponentTypeParam::TypeU32(e) => {
                    *((ptr0 + 24) as *mut u8) = (9i32) as u8;
                    *((ptr0 + 32) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                    
                  },
                  ComponentTypeParam::TypeU64(e) => {
                    *((ptr0 + 24) as *mut u8) = (10i32) as u8;
                    *((ptr0 + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                    
                  },
                  ComponentTypeParam::TypeVec2(e) => {
                    *((ptr0 + 24) as *mut u8) = (11i32) as u8;
                    let Vec2{ x:x11, y:y11, } = e;
                    *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x11);
                    *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y11);
                    
                  },
                  ComponentTypeParam::TypeVec3(e) => {
                    *((ptr0 + 24) as *mut u8) = (12i32) as u8;
                    let Vec3{ x:x12, y:y12, z:z12, } = e;
                    *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x12);
                    *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y12);
                    *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z12);
                    
                  },
                  ComponentTypeParam::TypeVec4(e) => {
                    *((ptr0 + 24) as *mut u8) = (13i32) as u8;
                    let Vec4{ x:x13, y:y13, z:z13, w:w13, } = e;
                    *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x13);
                    *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y13);
                    *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z13);
                    *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w13);
                    
                  },
                  ComponentTypeParam::TypeObjectRef(e) => {
                    *((ptr0 + 24) as *mut u8) = (14i32) as u8;
                    let ObjectRefParam{ id:id14, } = e;
                    let vec15 = id14;
                    let ptr15 = vec15.as_ptr() as i32;
                    let len15 = vec15.len() as i32;
                    *((ptr0 + 36) as *mut i32) = len15;
                    *((ptr0 + 32) as *mut i32) = ptr15;
                    
                  },
                  ComponentTypeParam::TypeList(e) => {
                    *((ptr0 + 24) as *mut u8) = (15i32) as u8;
                    match e {
                      ComponentListTypeParam::TypeEmpty(e) => {
                        *((ptr0 + 32) as *mut u8) = (0i32) as u8;
                        let vec16 = e;
                        let ptr16 = vec16.as_ptr() as i32;
                        let len16 = vec16.len() as i32;
                        *((ptr0 + 40) as *mut i32) = len16;
                        *((ptr0 + 36) as *mut i32) = ptr16;
                        
                      },
                      ComponentListTypeParam::TypeBool(e) => {
                        *((ptr0 + 32) as *mut u8) = (1i32) as u8;
                        let vec17 = e;
                        let len17 = vec17.len() as i32;
                        let layout17 = core::alloc::Layout::from_size_align_unchecked(vec17.len() * 1, 1);
                        let result17 = if layout17.size() != 0
                        {
                          let ptr = std::alloc::alloc(layout17);
                          if ptr.is_null()
                          {
                            std::alloc::handle_alloc_error(layout17);
                          }
                          ptr
                        }else {
                          std::ptr::null_mut()
                        };
                        for (i, e) in vec17.into_iter().enumerate() {
                          let base = result17 as i32 + (i as i32) * 1;
                          {
                            *((base + 0) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                            
                          }}
                          *((ptr0 + 40) as *mut i32) = len17;
                          *((ptr0 + 36) as *mut i32) = result17 as i32;
                          cleanup_list.extend_from_slice(&[(result17, layout17),]);
                          
                        },
                        ComponentListTypeParam::TypeEntityId(e) => {
                          *((ptr0 + 32) as *mut u8) = (2i32) as u8;
                          let vec18 = e;
                          let ptr18 = vec18.as_ptr() as i32;
                          let len18 = vec18.len() as i32;
                          *((ptr0 + 40) as *mut i32) = len18;
                          *((ptr0 + 36) as *mut i32) = ptr18;
                          
                        },
                        ComponentListTypeParam::TypeF32(e) => {
                          *((ptr0 + 32) as *mut u8) = (3i32) as u8;
                          let vec19 = e;
                          let ptr19 = vec19.as_ptr() as i32;
                          let len19 = vec19.len() as i32;
                          *((ptr0 + 40) as *mut i32) = len19;
                          *((ptr0 + 36) as *mut i32) = ptr19;
                          
                        },
                        ComponentListTypeParam::TypeF64(e) => {
                          *((ptr0 + 32) as *mut u8) = (4i32) as u8;
                          let vec20 = e;
                          let ptr20 = vec20.as_ptr() as i32;
                          let len20 = vec20.len() as i32;
                          *((ptr0 + 40) as *mut i32) = len20;
                          *((ptr0 + 36) as *mut i32) = ptr20;
                          
                        },
                        ComponentListTypeParam::TypeMat4(e) => {
                          *((ptr0 + 32) as *mut u8) = (5i32) as u8;
                          let vec21 = e;
                          let ptr21 = vec21.as_ptr() as i32;
                          let len21 = vec21.len() as i32;
                          *((ptr0 + 40) as *mut i32) = len21;
                          *((ptr0 + 36) as *mut i32) = ptr21;
                          
                        },
                        ComponentListTypeParam::TypeI32(e) => {
                          *((ptr0 + 32) as *mut u8) = (6i32) as u8;
                          let vec22 = e;
                          let ptr22 = vec22.as_ptr() as i32;
                          let len22 = vec22.len() as i32;
                          *((ptr0 + 40) as *mut i32) = len22;
                          *((ptr0 + 36) as *mut i32) = ptr22;
                          
                        },
                        ComponentListTypeParam::TypeQuat(e) => {
                          *((ptr0 + 32) as *mut u8) = (7i32) as u8;
                          let vec23 = e;
                          let ptr23 = vec23.as_ptr() as i32;
                          let len23 = vec23.len() as i32;
                          *((ptr0 + 40) as *mut i32) = len23;
                          *((ptr0 + 36) as *mut i32) = ptr23;
                          
                        },
                        ComponentListTypeParam::TypeString(e) => {
                          *((ptr0 + 32) as *mut u8) = (8i32) as u8;
                          let vec25 = e;
                          let len25 = vec25.len() as i32;
                          let layout25 = core::alloc::Layout::from_size_align_unchecked(vec25.len() * 8, 4);
                          let result25 = if layout25.size() != 0
                          {
                            let ptr = std::alloc::alloc(layout25);
                            if ptr.is_null()
                            {
                              std::alloc::handle_alloc_error(layout25);
                            }
                            ptr
                          }else {
                            std::ptr::null_mut()
                          };
                          for (i, e) in vec25.into_iter().enumerate() {
                            let base = result25 as i32 + (i as i32) * 8;
                            {
                              let vec24 = e;
                              let ptr24 = vec24.as_ptr() as i32;
                              let len24 = vec24.len() as i32;
                              *((base + 4) as *mut i32) = len24;
                              *((base + 0) as *mut i32) = ptr24;
                              
                            }}
                            *((ptr0 + 40) as *mut i32) = len25;
                            *((ptr0 + 36) as *mut i32) = result25 as i32;
                            cleanup_list.extend_from_slice(&[(result25, layout25),]);
                            
                          },
                          ComponentListTypeParam::TypeU32(e) => {
                            *((ptr0 + 32) as *mut u8) = (9i32) as u8;
                            let vec26 = e;
                            let ptr26 = vec26.as_ptr() as i32;
                            let len26 = vec26.len() as i32;
                            *((ptr0 + 40) as *mut i32) = len26;
                            *((ptr0 + 36) as *mut i32) = ptr26;
                            
                          },
                          ComponentListTypeParam::TypeU64(e) => {
                            *((ptr0 + 32) as *mut u8) = (10i32) as u8;
                            let vec27 = e;
                            let ptr27 = vec27.as_ptr() as i32;
                            let len27 = vec27.len() as i32;
                            *((ptr0 + 40) as *mut i32) = len27;
                            *((ptr0 + 36) as *mut i32) = ptr27;
                            
                          },
                          ComponentListTypeParam::TypeVec2(e) => {
                            *((ptr0 + 32) as *mut u8) = (11i32) as u8;
                            let vec28 = e;
                            let ptr28 = vec28.as_ptr() as i32;
                            let len28 = vec28.len() as i32;
                            *((ptr0 + 40) as *mut i32) = len28;
                            *((ptr0 + 36) as *mut i32) = ptr28;
                            
                          },
                          ComponentListTypeParam::TypeVec3(e) => {
                            *((ptr0 + 32) as *mut u8) = (12i32) as u8;
                            let vec29 = e;
                            let ptr29 = vec29.as_ptr() as i32;
                            let len29 = vec29.len() as i32;
                            *((ptr0 + 40) as *mut i32) = len29;
                            *((ptr0 + 36) as *mut i32) = ptr29;
                            
                          },
                          ComponentListTypeParam::TypeVec4(e) => {
                            *((ptr0 + 32) as *mut u8) = (13i32) as u8;
                            let vec30 = e;
                            let ptr30 = vec30.as_ptr() as i32;
                            let len30 = vec30.len() as i32;
                            *((ptr0 + 40) as *mut i32) = len30;
                            *((ptr0 + 36) as *mut i32) = ptr30;
                            
                          },
                          ComponentListTypeParam::TypeObjectRef(e) => {
                            *((ptr0 + 32) as *mut u8) = (14i32) as u8;
                            let vec33 = e;
                            let len33 = vec33.len() as i32;
                            let layout33 = core::alloc::Layout::from_size_align_unchecked(vec33.len() * 8, 4);
                            let result33 = if layout33.size() != 0
                            {
                              let ptr = std::alloc::alloc(layout33);
                              if ptr.is_null()
                              {
                                std::alloc::handle_alloc_error(layout33);
                              }
                              ptr
                            }else {
                              std::ptr::null_mut()
                            };
                            for (i, e) in vec33.into_iter().enumerate() {
                              let base = result33 as i32 + (i as i32) * 8;
                              {
                                let ObjectRefParam{ id:id31, } = e;
                                let vec32 = id31;
                                let ptr32 = vec32.as_ptr() as i32;
                                let len32 = vec32.len() as i32;
                                *((base + 4) as *mut i32) = len32;
                                *((base + 0) as *mut i32) = ptr32;
                                
                              }}
                              *((ptr0 + 40) as *mut i32) = len33;
                              *((ptr0 + 36) as *mut i32) = result33 as i32;
                              cleanup_list.extend_from_slice(&[(result33, layout33),]);
                              
                            },
                          };
                          
                        },
                        ComponentTypeParam::TypeOption(e) => {
                          *((ptr0 + 24) as *mut u8) = (16i32) as u8;
                          match e {
                            ComponentOptionTypeParam::TypeEmpty(e) => {
                              *((ptr0 + 32) as *mut u8) = (0i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let () = e;
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeBool(e) => {
                              *((ptr0 + 32) as *mut u8) = (1i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 41) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeEntityId(e) => {
                              *((ptr0 + 32) as *mut u8) = (2i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let EntityId{ id0:id035, id1:id135, } = e;
                                  *((ptr0 + 48) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id035);
                                  *((ptr0 + 56) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id135);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeF32(e) => {
                              *((ptr0 + 32) as *mut u8) = (3i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeF64(e) => {
                              *((ptr0 + 32) as *mut u8) = (4i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 48) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeMat4(e) => {
                              *((ptr0 + 32) as *mut u8) = (5i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let Mat4{ x:x36, y:y36, z:z36, w:w36, } = e;
                                  let Vec4{ x:x37, y:y37, z:z37, w:w37, } = x36;
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x37);
                                  *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y37);
                                  *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z37);
                                  *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w37);
                                  let Vec4{ x:x38, y:y38, z:z38, w:w38, } = y36;
                                  *((ptr0 + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x38);
                                  *((ptr0 + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y38);
                                  *((ptr0 + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z38);
                                  *((ptr0 + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w38);
                                  let Vec4{ x:x39, y:y39, z:z39, w:w39, } = z36;
                                  *((ptr0 + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x39);
                                  *((ptr0 + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y39);
                                  *((ptr0 + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z39);
                                  *((ptr0 + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w39);
                                  let Vec4{ x:x40, y:y40, z:z40, w:w40, } = w36;
                                  *((ptr0 + 92) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x40);
                                  *((ptr0 + 96) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y40);
                                  *((ptr0 + 100) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z40);
                                  *((ptr0 + 104) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w40);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeI32(e) => {
                              *((ptr0 + 32) as *mut u8) = (6i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 44) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeQuat(e) => {
                              *((ptr0 + 32) as *mut u8) = (7i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let Quat{ x:x41, y:y41, z:z41, w:w41, } = e;
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x41);
                                  *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y41);
                                  *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z41);
                                  *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w41);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeString(e) => {
                              *((ptr0 + 32) as *mut u8) = (8i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let vec42 = e;
                                  let ptr42 = vec42.as_ptr() as i32;
                                  let len42 = vec42.len() as i32;
                                  *((ptr0 + 48) as *mut i32) = len42;
                                  *((ptr0 + 44) as *mut i32) = ptr42;
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeU32(e) => {
                              *((ptr0 + 32) as *mut u8) = (9i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 44) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeU64(e) => {
                              *((ptr0 + 32) as *mut u8) = (10i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 48) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeVec2(e) => {
                              *((ptr0 + 32) as *mut u8) = (11i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let Vec2{ x:x43, y:y43, } = e;
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x43);
                                  *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y43);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeVec3(e) => {
                              *((ptr0 + 32) as *mut u8) = (12i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let Vec3{ x:x44, y:y44, z:z44, } = e;
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x44);
                                  *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y44);
                                  *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z44);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeVec4(e) => {
                              *((ptr0 + 32) as *mut u8) = (13i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let Vec4{ x:x45, y:y45, z:z45, w:w45, } = e;
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x45);
                                  *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y45);
                                  *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z45);
                                  *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w45);
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                            ComponentOptionTypeParam::TypeObjectRef(e) => {
                              *((ptr0 + 32) as *mut u8) = (14i32) as u8;
                              match e {
                                Some(e) => {
                                  *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                  let ObjectRefParam{ id:id46, } = e;
                                  let vec47 = id46;
                                  let ptr47 = vec47.as_ptr() as i32;
                                  let len47 = vec47.len() as i32;
                                  *((ptr0 + 48) as *mut i32) = len47;
                                  *((ptr0 + 44) as *mut i32) = ptr47;
                                  
                                },
                                None => {
                                  let e = ();
                                  {
                                    *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                    let () = e;
                                    
                                  }
                                },
                              };
                            },
                          };
                          
                        },
                      };
                      #[link(wasm_import_module = "host")]
                      extern "C" {
                        #[cfg_attr(target_arch = "wasm32", link_name = "entity-add-component: func(entity: record { id0: u64, id1: u64 }, index: u32, value: variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }) -> unit")]
                        #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-add-component: func(entity: record { id0: u64, id1: u64 }, index: u32, value: variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }) -> unit")]
                        fn wit_import(_: i32, );
                      }
                      wit_import(ptr0);
                      for (ptr, layout) in cleanup_list {
                        
                        if layout.size() != 0 {
                          
                          std::alloc::dealloc(ptr, layout);
                          
                        }
                        
                      }
                      ()
                    }
                  }
                  pub fn entity_add_components(entity: EntityId,data: Components<'_,>,) -> (){
                    unsafe {
                      let mut cleanup_list = Vec::new();
                      let EntityId{ id0:id00, id1:id10, } = entity;
                      let vec48 = data;
                      let len48 = vec48.len() as i32;
                      let layout48 = core::alloc::Layout::from_size_align_unchecked(vec48.len() * 88, 8);
                      let result48 = if layout48.size() != 0
                      {
                        let ptr = std::alloc::alloc(layout48);
                        if ptr.is_null()
                        {
                          std::alloc::handle_alloc_error(layout48);
                        }
                        ptr
                      }else {
                        std::ptr::null_mut()
                      };
                      for (i, e) in vec48.into_iter().enumerate() {
                        let base = result48 as i32 + (i as i32) * 88;
                        {
                          let (t1_0, t1_1, ) = e;
                          *((base + 0) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(t1_0);
                          match t1_1 {
                            ComponentTypeParam::TypeEmpty(e) => {
                              *((base + 8) as *mut u8) = (0i32) as u8;
                              let () = e;
                              
                            },
                            ComponentTypeParam::TypeBool(e) => {
                              *((base + 8) as *mut u8) = (1i32) as u8;
                              *((base + 16) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                              
                            },
                            ComponentTypeParam::TypeEntityId(e) => {
                              *((base + 8) as *mut u8) = (2i32) as u8;
                              let EntityId{ id0:id03, id1:id13, } = e;
                              *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id03);
                              *((base + 24) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id13);
                              
                            },
                            ComponentTypeParam::TypeF32(e) => {
                              *((base + 8) as *mut u8) = (3i32) as u8;
                              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                              
                            },
                            ComponentTypeParam::TypeF64(e) => {
                              *((base + 8) as *mut u8) = (4i32) as u8;
                              *((base + 16) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                              
                            },
                            ComponentTypeParam::TypeMat4(e) => {
                              *((base + 8) as *mut u8) = (5i32) as u8;
                              let Mat4{ x:x4, y:y4, z:z4, w:w4, } = e;
                              let Vec4{ x:x5, y:y5, z:z5, w:w5, } = x4;
                              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x5);
                              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y5);
                              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z5);
                              *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w5);
                              let Vec4{ x:x6, y:y6, z:z6, w:w6, } = y4;
                              *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x6);
                              *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y6);
                              *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z6);
                              *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w6);
                              let Vec4{ x:x7, y:y7, z:z7, w:w7, } = z4;
                              *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x7);
                              *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y7);
                              *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z7);
                              *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w7);
                              let Vec4{ x:x8, y:y8, z:z8, w:w8, } = w4;
                              *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x8);
                              *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y8);
                              *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z8);
                              *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w8);
                              
                            },
                            ComponentTypeParam::TypeI32(e) => {
                              *((base + 8) as *mut u8) = (6i32) as u8;
                              *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                              
                            },
                            ComponentTypeParam::TypeQuat(e) => {
                              *((base + 8) as *mut u8) = (7i32) as u8;
                              let Quat{ x:x9, y:y9, z:z9, w:w9, } = e;
                              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x9);
                              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y9);
                              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z9);
                              *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w9);
                              
                            },
                            ComponentTypeParam::TypeString(e) => {
                              *((base + 8) as *mut u8) = (8i32) as u8;
                              let vec10 = e;
                              let ptr10 = vec10.as_ptr() as i32;
                              let len10 = vec10.len() as i32;
                              *((base + 20) as *mut i32) = len10;
                              *((base + 16) as *mut i32) = ptr10;
                              
                            },
                            ComponentTypeParam::TypeU32(e) => {
                              *((base + 8) as *mut u8) = (9i32) as u8;
                              *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                              
                            },
                            ComponentTypeParam::TypeU64(e) => {
                              *((base + 8) as *mut u8) = (10i32) as u8;
                              *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                              
                            },
                            ComponentTypeParam::TypeVec2(e) => {
                              *((base + 8) as *mut u8) = (11i32) as u8;
                              let Vec2{ x:x11, y:y11, } = e;
                              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x11);
                              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y11);
                              
                            },
                            ComponentTypeParam::TypeVec3(e) => {
                              *((base + 8) as *mut u8) = (12i32) as u8;
                              let Vec3{ x:x12, y:y12, z:z12, } = e;
                              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x12);
                              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y12);
                              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z12);
                              
                            },
                            ComponentTypeParam::TypeVec4(e) => {
                              *((base + 8) as *mut u8) = (13i32) as u8;
                              let Vec4{ x:x13, y:y13, z:z13, w:w13, } = e;
                              *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x13);
                              *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y13);
                              *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z13);
                              *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w13);
                              
                            },
                            ComponentTypeParam::TypeObjectRef(e) => {
                              *((base + 8) as *mut u8) = (14i32) as u8;
                              let ObjectRefParam{ id:id14, } = e;
                              let vec15 = id14;
                              let ptr15 = vec15.as_ptr() as i32;
                              let len15 = vec15.len() as i32;
                              *((base + 20) as *mut i32) = len15;
                              *((base + 16) as *mut i32) = ptr15;
                              
                            },
                            ComponentTypeParam::TypeList(e) => {
                              *((base + 8) as *mut u8) = (15i32) as u8;
                              match e {
                                ComponentListTypeParam::TypeEmpty(e) => {
                                  *((base + 16) as *mut u8) = (0i32) as u8;
                                  let vec16 = e;
                                  let ptr16 = vec16.as_ptr() as i32;
                                  let len16 = vec16.len() as i32;
                                  *((base + 24) as *mut i32) = len16;
                                  *((base + 20) as *mut i32) = ptr16;
                                  
                                },
                                ComponentListTypeParam::TypeBool(e) => {
                                  *((base + 16) as *mut u8) = (1i32) as u8;
                                  let vec17 = e;
                                  let len17 = vec17.len() as i32;
                                  let layout17 = core::alloc::Layout::from_size_align_unchecked(vec17.len() * 1, 1);
                                  let result17 = if layout17.size() != 0
                                  {
                                    let ptr = std::alloc::alloc(layout17);
                                    if ptr.is_null()
                                    {
                                      std::alloc::handle_alloc_error(layout17);
                                    }
                                    ptr
                                  }else {
                                    std::ptr::null_mut()
                                  };
                                  for (i, e) in vec17.into_iter().enumerate() {
                                    let base = result17 as i32 + (i as i32) * 1;
                                    {
                                      *((base + 0) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                      
                                    }}
                                    *((base + 24) as *mut i32) = len17;
                                    *((base + 20) as *mut i32) = result17 as i32;
                                    cleanup_list.extend_from_slice(&[(result17, layout17),]);
                                    
                                  },
                                  ComponentListTypeParam::TypeEntityId(e) => {
                                    *((base + 16) as *mut u8) = (2i32) as u8;
                                    let vec18 = e;
                                    let ptr18 = vec18.as_ptr() as i32;
                                    let len18 = vec18.len() as i32;
                                    *((base + 24) as *mut i32) = len18;
                                    *((base + 20) as *mut i32) = ptr18;
                                    
                                  },
                                  ComponentListTypeParam::TypeF32(e) => {
                                    *((base + 16) as *mut u8) = (3i32) as u8;
                                    let vec19 = e;
                                    let ptr19 = vec19.as_ptr() as i32;
                                    let len19 = vec19.len() as i32;
                                    *((base + 24) as *mut i32) = len19;
                                    *((base + 20) as *mut i32) = ptr19;
                                    
                                  },
                                  ComponentListTypeParam::TypeF64(e) => {
                                    *((base + 16) as *mut u8) = (4i32) as u8;
                                    let vec20 = e;
                                    let ptr20 = vec20.as_ptr() as i32;
                                    let len20 = vec20.len() as i32;
                                    *((base + 24) as *mut i32) = len20;
                                    *((base + 20) as *mut i32) = ptr20;
                                    
                                  },
                                  ComponentListTypeParam::TypeMat4(e) => {
                                    *((base + 16) as *mut u8) = (5i32) as u8;
                                    let vec21 = e;
                                    let ptr21 = vec21.as_ptr() as i32;
                                    let len21 = vec21.len() as i32;
                                    *((base + 24) as *mut i32) = len21;
                                    *((base + 20) as *mut i32) = ptr21;
                                    
                                  },
                                  ComponentListTypeParam::TypeI32(e) => {
                                    *((base + 16) as *mut u8) = (6i32) as u8;
                                    let vec22 = e;
                                    let ptr22 = vec22.as_ptr() as i32;
                                    let len22 = vec22.len() as i32;
                                    *((base + 24) as *mut i32) = len22;
                                    *((base + 20) as *mut i32) = ptr22;
                                    
                                  },
                                  ComponentListTypeParam::TypeQuat(e) => {
                                    *((base + 16) as *mut u8) = (7i32) as u8;
                                    let vec23 = e;
                                    let ptr23 = vec23.as_ptr() as i32;
                                    let len23 = vec23.len() as i32;
                                    *((base + 24) as *mut i32) = len23;
                                    *((base + 20) as *mut i32) = ptr23;
                                    
                                  },
                                  ComponentListTypeParam::TypeString(e) => {
                                    *((base + 16) as *mut u8) = (8i32) as u8;
                                    let vec25 = e;
                                    let len25 = vec25.len() as i32;
                                    let layout25 = core::alloc::Layout::from_size_align_unchecked(vec25.len() * 8, 4);
                                    let result25 = if layout25.size() != 0
                                    {
                                      let ptr = std::alloc::alloc(layout25);
                                      if ptr.is_null()
                                      {
                                        std::alloc::handle_alloc_error(layout25);
                                      }
                                      ptr
                                    }else {
                                      std::ptr::null_mut()
                                    };
                                    for (i, e) in vec25.into_iter().enumerate() {
                                      let base = result25 as i32 + (i as i32) * 8;
                                      {
                                        let vec24 = e;
                                        let ptr24 = vec24.as_ptr() as i32;
                                        let len24 = vec24.len() as i32;
                                        *((base + 4) as *mut i32) = len24;
                                        *((base + 0) as *mut i32) = ptr24;
                                        
                                      }}
                                      *((base + 24) as *mut i32) = len25;
                                      *((base + 20) as *mut i32) = result25 as i32;
                                      cleanup_list.extend_from_slice(&[(result25, layout25),]);
                                      
                                    },
                                    ComponentListTypeParam::TypeU32(e) => {
                                      *((base + 16) as *mut u8) = (9i32) as u8;
                                      let vec26 = e;
                                      let ptr26 = vec26.as_ptr() as i32;
                                      let len26 = vec26.len() as i32;
                                      *((base + 24) as *mut i32) = len26;
                                      *((base + 20) as *mut i32) = ptr26;
                                      
                                    },
                                    ComponentListTypeParam::TypeU64(e) => {
                                      *((base + 16) as *mut u8) = (10i32) as u8;
                                      let vec27 = e;
                                      let ptr27 = vec27.as_ptr() as i32;
                                      let len27 = vec27.len() as i32;
                                      *((base + 24) as *mut i32) = len27;
                                      *((base + 20) as *mut i32) = ptr27;
                                      
                                    },
                                    ComponentListTypeParam::TypeVec2(e) => {
                                      *((base + 16) as *mut u8) = (11i32) as u8;
                                      let vec28 = e;
                                      let ptr28 = vec28.as_ptr() as i32;
                                      let len28 = vec28.len() as i32;
                                      *((base + 24) as *mut i32) = len28;
                                      *((base + 20) as *mut i32) = ptr28;
                                      
                                    },
                                    ComponentListTypeParam::TypeVec3(e) => {
                                      *((base + 16) as *mut u8) = (12i32) as u8;
                                      let vec29 = e;
                                      let ptr29 = vec29.as_ptr() as i32;
                                      let len29 = vec29.len() as i32;
                                      *((base + 24) as *mut i32) = len29;
                                      *((base + 20) as *mut i32) = ptr29;
                                      
                                    },
                                    ComponentListTypeParam::TypeVec4(e) => {
                                      *((base + 16) as *mut u8) = (13i32) as u8;
                                      let vec30 = e;
                                      let ptr30 = vec30.as_ptr() as i32;
                                      let len30 = vec30.len() as i32;
                                      *((base + 24) as *mut i32) = len30;
                                      *((base + 20) as *mut i32) = ptr30;
                                      
                                    },
                                    ComponentListTypeParam::TypeObjectRef(e) => {
                                      *((base + 16) as *mut u8) = (14i32) as u8;
                                      let vec33 = e;
                                      let len33 = vec33.len() as i32;
                                      let layout33 = core::alloc::Layout::from_size_align_unchecked(vec33.len() * 8, 4);
                                      let result33 = if layout33.size() != 0
                                      {
                                        let ptr = std::alloc::alloc(layout33);
                                        if ptr.is_null()
                                        {
                                          std::alloc::handle_alloc_error(layout33);
                                        }
                                        ptr
                                      }else {
                                        std::ptr::null_mut()
                                      };
                                      for (i, e) in vec33.into_iter().enumerate() {
                                        let base = result33 as i32 + (i as i32) * 8;
                                        {
                                          let ObjectRefParam{ id:id31, } = e;
                                          let vec32 = id31;
                                          let ptr32 = vec32.as_ptr() as i32;
                                          let len32 = vec32.len() as i32;
                                          *((base + 4) as *mut i32) = len32;
                                          *((base + 0) as *mut i32) = ptr32;
                                          
                                        }}
                                        *((base + 24) as *mut i32) = len33;
                                        *((base + 20) as *mut i32) = result33 as i32;
                                        cleanup_list.extend_from_slice(&[(result33, layout33),]);
                                        
                                      },
                                    };
                                    
                                  },
                                  ComponentTypeParam::TypeOption(e) => {
                                    *((base + 8) as *mut u8) = (16i32) as u8;
                                    match e {
                                      ComponentOptionTypeParam::TypeEmpty(e) => {
                                        *((base + 16) as *mut u8) = (0i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let () = e;
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeBool(e) => {
                                        *((base + 16) as *mut u8) = (1i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            *((base + 25) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeEntityId(e) => {
                                        *((base + 16) as *mut u8) = (2i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let EntityId{ id0:id035, id1:id135, } = e;
                                            *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id035);
                                            *((base + 40) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id135);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeF32(e) => {
                                        *((base + 16) as *mut u8) = (3i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeF64(e) => {
                                        *((base + 16) as *mut u8) = (4i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            *((base + 32) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeMat4(e) => {
                                        *((base + 16) as *mut u8) = (5i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let Mat4{ x:x36, y:y36, z:z36, w:w36, } = e;
                                            let Vec4{ x:x37, y:y37, z:z37, w:w37, } = x36;
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x37);
                                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y37);
                                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z37);
                                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w37);
                                            let Vec4{ x:x38, y:y38, z:z38, w:w38, } = y36;
                                            *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x38);
                                            *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y38);
                                            *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z38);
                                            *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w38);
                                            let Vec4{ x:x39, y:y39, z:z39, w:w39, } = z36;
                                            *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x39);
                                            *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y39);
                                            *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z39);
                                            *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w39);
                                            let Vec4{ x:x40, y:y40, z:z40, w:w40, } = w36;
                                            *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x40);
                                            *((base + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y40);
                                            *((base + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z40);
                                            *((base + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w40);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeI32(e) => {
                                        *((base + 16) as *mut u8) = (6i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeQuat(e) => {
                                        *((base + 16) as *mut u8) = (7i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let Quat{ x:x41, y:y41, z:z41, w:w41, } = e;
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x41);
                                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y41);
                                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z41);
                                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w41);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeString(e) => {
                                        *((base + 16) as *mut u8) = (8i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let vec42 = e;
                                            let ptr42 = vec42.as_ptr() as i32;
                                            let len42 = vec42.len() as i32;
                                            *((base + 32) as *mut i32) = len42;
                                            *((base + 28) as *mut i32) = ptr42;
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeU32(e) => {
                                        *((base + 16) as *mut u8) = (9i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeU64(e) => {
                                        *((base + 16) as *mut u8) = (10i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeVec2(e) => {
                                        *((base + 16) as *mut u8) = (11i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let Vec2{ x:x43, y:y43, } = e;
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x43);
                                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y43);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeVec3(e) => {
                                        *((base + 16) as *mut u8) = (12i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let Vec3{ x:x44, y:y44, z:z44, } = e;
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x44);
                                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y44);
                                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z44);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeVec4(e) => {
                                        *((base + 16) as *mut u8) = (13i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let Vec4{ x:x45, y:y45, z:z45, w:w45, } = e;
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x45);
                                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y45);
                                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z45);
                                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w45);
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                      ComponentOptionTypeParam::TypeObjectRef(e) => {
                                        *((base + 16) as *mut u8) = (14i32) as u8;
                                        match e {
                                          Some(e) => {
                                            *((base + 24) as *mut u8) = (1i32) as u8;
                                            let ObjectRefParam{ id:id46, } = e;
                                            let vec47 = id46;
                                            let ptr47 = vec47.as_ptr() as i32;
                                            let len47 = vec47.len() as i32;
                                            *((base + 32) as *mut i32) = len47;
                                            *((base + 28) as *mut i32) = ptr47;
                                            
                                          },
                                          None => {
                                            let e = ();
                                            {
                                              *((base + 24) as *mut u8) = (0i32) as u8;
                                              let () = e;
                                              
                                            }
                                          },
                                        };
                                      },
                                    };
                                    
                                  },
                                };
                                
                              }}
                              #[link(wasm_import_module = "host")]
                              extern "C" {
                                #[cfg_attr(target_arch = "wasm32", link_name = "entity-add-components: func(entity: record { id0: u64, id1: u64 }, data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit")]
                                #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-add-components: func(entity: record { id0: u64, id1: u64 }, data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit")]
                                fn wit_import(_: i64, _: i64, _: i32, _: i32, );
                              }
                              wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), result48 as i32, len48);
                              if layout48.size() != 0 {
                                std::alloc::dealloc(result48, layout48);
                              }
                              for (ptr, layout) in cleanup_list {
                                
                                if layout.size() != 0 {
                                  
                                  std::alloc::dealloc(ptr, layout);
                                  
                                }
                                
                              }
                              ()
                            }
                          }
                          pub fn entity_set_component(entity: EntityId,index: u32,value: ComponentTypeParam<'_,>,) -> (){
                            unsafe {
                              let mut cleanup_list = Vec::new();
                              let ptr0 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                              let EntityId{ id0:id01, id1:id11, } = entity;
                              *((ptr0 + 0) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id01);
                              *((ptr0 + 8) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id11);
                              *((ptr0 + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(index);
                              match value {
                                ComponentTypeParam::TypeEmpty(e) => {
                                  *((ptr0 + 24) as *mut u8) = (0i32) as u8;
                                  let () = e;
                                  
                                },
                                ComponentTypeParam::TypeBool(e) => {
                                  *((ptr0 + 24) as *mut u8) = (1i32) as u8;
                                  *((ptr0 + 32) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                  
                                },
                                ComponentTypeParam::TypeEntityId(e) => {
                                  *((ptr0 + 24) as *mut u8) = (2i32) as u8;
                                  let EntityId{ id0:id03, id1:id13, } = e;
                                  *((ptr0 + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id03);
                                  *((ptr0 + 40) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id13);
                                  
                                },
                                ComponentTypeParam::TypeF32(e) => {
                                  *((ptr0 + 24) as *mut u8) = (3i32) as u8;
                                  *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                  
                                },
                                ComponentTypeParam::TypeF64(e) => {
                                  *((ptr0 + 24) as *mut u8) = (4i32) as u8;
                                  *((ptr0 + 32) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                  
                                },
                                ComponentTypeParam::TypeMat4(e) => {
                                  *((ptr0 + 24) as *mut u8) = (5i32) as u8;
                                  let Mat4{ x:x4, y:y4, z:z4, w:w4, } = e;
                                  let Vec4{ x:x5, y:y5, z:z5, w:w5, } = x4;
                                  *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x5);
                                  *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y5);
                                  *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z5);
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w5);
                                  let Vec4{ x:x6, y:y6, z:z6, w:w6, } = y4;
                                  *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x6);
                                  *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y6);
                                  *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z6);
                                  *((ptr0 + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w6);
                                  let Vec4{ x:x7, y:y7, z:z7, w:w7, } = z4;
                                  *((ptr0 + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x7);
                                  *((ptr0 + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y7);
                                  *((ptr0 + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z7);
                                  *((ptr0 + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w7);
                                  let Vec4{ x:x8, y:y8, z:z8, w:w8, } = w4;
                                  *((ptr0 + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x8);
                                  *((ptr0 + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y8);
                                  *((ptr0 + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z8);
                                  *((ptr0 + 92) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w8);
                                  
                                },
                                ComponentTypeParam::TypeI32(e) => {
                                  *((ptr0 + 24) as *mut u8) = (6i32) as u8;
                                  *((ptr0 + 32) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                  
                                },
                                ComponentTypeParam::TypeQuat(e) => {
                                  *((ptr0 + 24) as *mut u8) = (7i32) as u8;
                                  let Quat{ x:x9, y:y9, z:z9, w:w9, } = e;
                                  *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x9);
                                  *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y9);
                                  *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z9);
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w9);
                                  
                                },
                                ComponentTypeParam::TypeString(e) => {
                                  *((ptr0 + 24) as *mut u8) = (8i32) as u8;
                                  let vec10 = e;
                                  let ptr10 = vec10.as_ptr() as i32;
                                  let len10 = vec10.len() as i32;
                                  *((ptr0 + 36) as *mut i32) = len10;
                                  *((ptr0 + 32) as *mut i32) = ptr10;
                                  
                                },
                                ComponentTypeParam::TypeU32(e) => {
                                  *((ptr0 + 24) as *mut u8) = (9i32) as u8;
                                  *((ptr0 + 32) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                  
                                },
                                ComponentTypeParam::TypeU64(e) => {
                                  *((ptr0 + 24) as *mut u8) = (10i32) as u8;
                                  *((ptr0 + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                  
                                },
                                ComponentTypeParam::TypeVec2(e) => {
                                  *((ptr0 + 24) as *mut u8) = (11i32) as u8;
                                  let Vec2{ x:x11, y:y11, } = e;
                                  *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x11);
                                  *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y11);
                                  
                                },
                                ComponentTypeParam::TypeVec3(e) => {
                                  *((ptr0 + 24) as *mut u8) = (12i32) as u8;
                                  let Vec3{ x:x12, y:y12, z:z12, } = e;
                                  *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x12);
                                  *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y12);
                                  *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z12);
                                  
                                },
                                ComponentTypeParam::TypeVec4(e) => {
                                  *((ptr0 + 24) as *mut u8) = (13i32) as u8;
                                  let Vec4{ x:x13, y:y13, z:z13, w:w13, } = e;
                                  *((ptr0 + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x13);
                                  *((ptr0 + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y13);
                                  *((ptr0 + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z13);
                                  *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w13);
                                  
                                },
                                ComponentTypeParam::TypeObjectRef(e) => {
                                  *((ptr0 + 24) as *mut u8) = (14i32) as u8;
                                  let ObjectRefParam{ id:id14, } = e;
                                  let vec15 = id14;
                                  let ptr15 = vec15.as_ptr() as i32;
                                  let len15 = vec15.len() as i32;
                                  *((ptr0 + 36) as *mut i32) = len15;
                                  *((ptr0 + 32) as *mut i32) = ptr15;
                                  
                                },
                                ComponentTypeParam::TypeList(e) => {
                                  *((ptr0 + 24) as *mut u8) = (15i32) as u8;
                                  match e {
                                    ComponentListTypeParam::TypeEmpty(e) => {
                                      *((ptr0 + 32) as *mut u8) = (0i32) as u8;
                                      let vec16 = e;
                                      let ptr16 = vec16.as_ptr() as i32;
                                      let len16 = vec16.len() as i32;
                                      *((ptr0 + 40) as *mut i32) = len16;
                                      *((ptr0 + 36) as *mut i32) = ptr16;
                                      
                                    },
                                    ComponentListTypeParam::TypeBool(e) => {
                                      *((ptr0 + 32) as *mut u8) = (1i32) as u8;
                                      let vec17 = e;
                                      let len17 = vec17.len() as i32;
                                      let layout17 = core::alloc::Layout::from_size_align_unchecked(vec17.len() * 1, 1);
                                      let result17 = if layout17.size() != 0
                                      {
                                        let ptr = std::alloc::alloc(layout17);
                                        if ptr.is_null()
                                        {
                                          std::alloc::handle_alloc_error(layout17);
                                        }
                                        ptr
                                      }else {
                                        std::ptr::null_mut()
                                      };
                                      for (i, e) in vec17.into_iter().enumerate() {
                                        let base = result17 as i32 + (i as i32) * 1;
                                        {
                                          *((base + 0) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                          
                                        }}
                                        *((ptr0 + 40) as *mut i32) = len17;
                                        *((ptr0 + 36) as *mut i32) = result17 as i32;
                                        cleanup_list.extend_from_slice(&[(result17, layout17),]);
                                        
                                      },
                                      ComponentListTypeParam::TypeEntityId(e) => {
                                        *((ptr0 + 32) as *mut u8) = (2i32) as u8;
                                        let vec18 = e;
                                        let ptr18 = vec18.as_ptr() as i32;
                                        let len18 = vec18.len() as i32;
                                        *((ptr0 + 40) as *mut i32) = len18;
                                        *((ptr0 + 36) as *mut i32) = ptr18;
                                        
                                      },
                                      ComponentListTypeParam::TypeF32(e) => {
                                        *((ptr0 + 32) as *mut u8) = (3i32) as u8;
                                        let vec19 = e;
                                        let ptr19 = vec19.as_ptr() as i32;
                                        let len19 = vec19.len() as i32;
                                        *((ptr0 + 40) as *mut i32) = len19;
                                        *((ptr0 + 36) as *mut i32) = ptr19;
                                        
                                      },
                                      ComponentListTypeParam::TypeF64(e) => {
                                        *((ptr0 + 32) as *mut u8) = (4i32) as u8;
                                        let vec20 = e;
                                        let ptr20 = vec20.as_ptr() as i32;
                                        let len20 = vec20.len() as i32;
                                        *((ptr0 + 40) as *mut i32) = len20;
                                        *((ptr0 + 36) as *mut i32) = ptr20;
                                        
                                      },
                                      ComponentListTypeParam::TypeMat4(e) => {
                                        *((ptr0 + 32) as *mut u8) = (5i32) as u8;
                                        let vec21 = e;
                                        let ptr21 = vec21.as_ptr() as i32;
                                        let len21 = vec21.len() as i32;
                                        *((ptr0 + 40) as *mut i32) = len21;
                                        *((ptr0 + 36) as *mut i32) = ptr21;
                                        
                                      },
                                      ComponentListTypeParam::TypeI32(e) => {
                                        *((ptr0 + 32) as *mut u8) = (6i32) as u8;
                                        let vec22 = e;
                                        let ptr22 = vec22.as_ptr() as i32;
                                        let len22 = vec22.len() as i32;
                                        *((ptr0 + 40) as *mut i32) = len22;
                                        *((ptr0 + 36) as *mut i32) = ptr22;
                                        
                                      },
                                      ComponentListTypeParam::TypeQuat(e) => {
                                        *((ptr0 + 32) as *mut u8) = (7i32) as u8;
                                        let vec23 = e;
                                        let ptr23 = vec23.as_ptr() as i32;
                                        let len23 = vec23.len() as i32;
                                        *((ptr0 + 40) as *mut i32) = len23;
                                        *((ptr0 + 36) as *mut i32) = ptr23;
                                        
                                      },
                                      ComponentListTypeParam::TypeString(e) => {
                                        *((ptr0 + 32) as *mut u8) = (8i32) as u8;
                                        let vec25 = e;
                                        let len25 = vec25.len() as i32;
                                        let layout25 = core::alloc::Layout::from_size_align_unchecked(vec25.len() * 8, 4);
                                        let result25 = if layout25.size() != 0
                                        {
                                          let ptr = std::alloc::alloc(layout25);
                                          if ptr.is_null()
                                          {
                                            std::alloc::handle_alloc_error(layout25);
                                          }
                                          ptr
                                        }else {
                                          std::ptr::null_mut()
                                        };
                                        for (i, e) in vec25.into_iter().enumerate() {
                                          let base = result25 as i32 + (i as i32) * 8;
                                          {
                                            let vec24 = e;
                                            let ptr24 = vec24.as_ptr() as i32;
                                            let len24 = vec24.len() as i32;
                                            *((base + 4) as *mut i32) = len24;
                                            *((base + 0) as *mut i32) = ptr24;
                                            
                                          }}
                                          *((ptr0 + 40) as *mut i32) = len25;
                                          *((ptr0 + 36) as *mut i32) = result25 as i32;
                                          cleanup_list.extend_from_slice(&[(result25, layout25),]);
                                          
                                        },
                                        ComponentListTypeParam::TypeU32(e) => {
                                          *((ptr0 + 32) as *mut u8) = (9i32) as u8;
                                          let vec26 = e;
                                          let ptr26 = vec26.as_ptr() as i32;
                                          let len26 = vec26.len() as i32;
                                          *((ptr0 + 40) as *mut i32) = len26;
                                          *((ptr0 + 36) as *mut i32) = ptr26;
                                          
                                        },
                                        ComponentListTypeParam::TypeU64(e) => {
                                          *((ptr0 + 32) as *mut u8) = (10i32) as u8;
                                          let vec27 = e;
                                          let ptr27 = vec27.as_ptr() as i32;
                                          let len27 = vec27.len() as i32;
                                          *((ptr0 + 40) as *mut i32) = len27;
                                          *((ptr0 + 36) as *mut i32) = ptr27;
                                          
                                        },
                                        ComponentListTypeParam::TypeVec2(e) => {
                                          *((ptr0 + 32) as *mut u8) = (11i32) as u8;
                                          let vec28 = e;
                                          let ptr28 = vec28.as_ptr() as i32;
                                          let len28 = vec28.len() as i32;
                                          *((ptr0 + 40) as *mut i32) = len28;
                                          *((ptr0 + 36) as *mut i32) = ptr28;
                                          
                                        },
                                        ComponentListTypeParam::TypeVec3(e) => {
                                          *((ptr0 + 32) as *mut u8) = (12i32) as u8;
                                          let vec29 = e;
                                          let ptr29 = vec29.as_ptr() as i32;
                                          let len29 = vec29.len() as i32;
                                          *((ptr0 + 40) as *mut i32) = len29;
                                          *((ptr0 + 36) as *mut i32) = ptr29;
                                          
                                        },
                                        ComponentListTypeParam::TypeVec4(e) => {
                                          *((ptr0 + 32) as *mut u8) = (13i32) as u8;
                                          let vec30 = e;
                                          let ptr30 = vec30.as_ptr() as i32;
                                          let len30 = vec30.len() as i32;
                                          *((ptr0 + 40) as *mut i32) = len30;
                                          *((ptr0 + 36) as *mut i32) = ptr30;
                                          
                                        },
                                        ComponentListTypeParam::TypeObjectRef(e) => {
                                          *((ptr0 + 32) as *mut u8) = (14i32) as u8;
                                          let vec33 = e;
                                          let len33 = vec33.len() as i32;
                                          let layout33 = core::alloc::Layout::from_size_align_unchecked(vec33.len() * 8, 4);
                                          let result33 = if layout33.size() != 0
                                          {
                                            let ptr = std::alloc::alloc(layout33);
                                            if ptr.is_null()
                                            {
                                              std::alloc::handle_alloc_error(layout33);
                                            }
                                            ptr
                                          }else {
                                            std::ptr::null_mut()
                                          };
                                          for (i, e) in vec33.into_iter().enumerate() {
                                            let base = result33 as i32 + (i as i32) * 8;
                                            {
                                              let ObjectRefParam{ id:id31, } = e;
                                              let vec32 = id31;
                                              let ptr32 = vec32.as_ptr() as i32;
                                              let len32 = vec32.len() as i32;
                                              *((base + 4) as *mut i32) = len32;
                                              *((base + 0) as *mut i32) = ptr32;
                                              
                                            }}
                                            *((ptr0 + 40) as *mut i32) = len33;
                                            *((ptr0 + 36) as *mut i32) = result33 as i32;
                                            cleanup_list.extend_from_slice(&[(result33, layout33),]);
                                            
                                          },
                                        };
                                        
                                      },
                                      ComponentTypeParam::TypeOption(e) => {
                                        *((ptr0 + 24) as *mut u8) = (16i32) as u8;
                                        match e {
                                          ComponentOptionTypeParam::TypeEmpty(e) => {
                                            *((ptr0 + 32) as *mut u8) = (0i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let () = e;
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeBool(e) => {
                                            *((ptr0 + 32) as *mut u8) = (1i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                *((ptr0 + 41) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeEntityId(e) => {
                                            *((ptr0 + 32) as *mut u8) = (2i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let EntityId{ id0:id035, id1:id135, } = e;
                                                *((ptr0 + 48) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id035);
                                                *((ptr0 + 56) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id135);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeF32(e) => {
                                            *((ptr0 + 32) as *mut u8) = (3i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeF64(e) => {
                                            *((ptr0 + 32) as *mut u8) = (4i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                *((ptr0 + 48) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeMat4(e) => {
                                            *((ptr0 + 32) as *mut u8) = (5i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let Mat4{ x:x36, y:y36, z:z36, w:w36, } = e;
                                                let Vec4{ x:x37, y:y37, z:z37, w:w37, } = x36;
                                                *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x37);
                                                *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y37);
                                                *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z37);
                                                *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w37);
                                                let Vec4{ x:x38, y:y38, z:z38, w:w38, } = y36;
                                                *((ptr0 + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x38);
                                                *((ptr0 + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y38);
                                                *((ptr0 + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z38);
                                                *((ptr0 + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w38);
                                                let Vec4{ x:x39, y:y39, z:z39, w:w39, } = z36;
                                                *((ptr0 + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x39);
                                                *((ptr0 + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y39);
                                                *((ptr0 + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z39);
                                                *((ptr0 + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w39);
                                                let Vec4{ x:x40, y:y40, z:z40, w:w40, } = w36;
                                                *((ptr0 + 92) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x40);
                                                *((ptr0 + 96) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y40);
                                                *((ptr0 + 100) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z40);
                                                *((ptr0 + 104) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w40);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeI32(e) => {
                                            *((ptr0 + 32) as *mut u8) = (6i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                *((ptr0 + 44) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeQuat(e) => {
                                            *((ptr0 + 32) as *mut u8) = (7i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let Quat{ x:x41, y:y41, z:z41, w:w41, } = e;
                                                *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x41);
                                                *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y41);
                                                *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z41);
                                                *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w41);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeString(e) => {
                                            *((ptr0 + 32) as *mut u8) = (8i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let vec42 = e;
                                                let ptr42 = vec42.as_ptr() as i32;
                                                let len42 = vec42.len() as i32;
                                                *((ptr0 + 48) as *mut i32) = len42;
                                                *((ptr0 + 44) as *mut i32) = ptr42;
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeU32(e) => {
                                            *((ptr0 + 32) as *mut u8) = (9i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                *((ptr0 + 44) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeU64(e) => {
                                            *((ptr0 + 32) as *mut u8) = (10i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                *((ptr0 + 48) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeVec2(e) => {
                                            *((ptr0 + 32) as *mut u8) = (11i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let Vec2{ x:x43, y:y43, } = e;
                                                *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x43);
                                                *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y43);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeVec3(e) => {
                                            *((ptr0 + 32) as *mut u8) = (12i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let Vec3{ x:x44, y:y44, z:z44, } = e;
                                                *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x44);
                                                *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y44);
                                                *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z44);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeVec4(e) => {
                                            *((ptr0 + 32) as *mut u8) = (13i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let Vec4{ x:x45, y:y45, z:z45, w:w45, } = e;
                                                *((ptr0 + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x45);
                                                *((ptr0 + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y45);
                                                *((ptr0 + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z45);
                                                *((ptr0 + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w45);
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                          ComponentOptionTypeParam::TypeObjectRef(e) => {
                                            *((ptr0 + 32) as *mut u8) = (14i32) as u8;
                                            match e {
                                              Some(e) => {
                                                *((ptr0 + 40) as *mut u8) = (1i32) as u8;
                                                let ObjectRefParam{ id:id46, } = e;
                                                let vec47 = id46;
                                                let ptr47 = vec47.as_ptr() as i32;
                                                let len47 = vec47.len() as i32;
                                                *((ptr0 + 48) as *mut i32) = len47;
                                                *((ptr0 + 44) as *mut i32) = ptr47;
                                                
                                              },
                                              None => {
                                                let e = ();
                                                {
                                                  *((ptr0 + 40) as *mut u8) = (0i32) as u8;
                                                  let () = e;
                                                  
                                                }
                                              },
                                            };
                                          },
                                        };
                                        
                                      },
                                    };
                                    #[link(wasm_import_module = "host")]
                                    extern "C" {
                                      #[cfg_attr(target_arch = "wasm32", link_name = "entity-set-component: func(entity: record { id0: u64, id1: u64 }, index: u32, value: variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }) -> unit")]
                                      #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-set-component: func(entity: record { id0: u64, id1: u64 }, index: u32, value: variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }) -> unit")]
                                      fn wit_import(_: i32, );
                                    }
                                    wit_import(ptr0);
                                    for (ptr, layout) in cleanup_list {
                                      
                                      if layout.size() != 0 {
                                        
                                        std::alloc::dealloc(ptr, layout);
                                        
                                      }
                                      
                                    }
                                    ()
                                  }
                                }
                                pub fn entity_set_components(entity: EntityId,data: Components<'_,>,) -> (){
                                  unsafe {
                                    let mut cleanup_list = Vec::new();
                                    let EntityId{ id0:id00, id1:id10, } = entity;
                                    let vec48 = data;
                                    let len48 = vec48.len() as i32;
                                    let layout48 = core::alloc::Layout::from_size_align_unchecked(vec48.len() * 88, 8);
                                    let result48 = if layout48.size() != 0
                                    {
                                      let ptr = std::alloc::alloc(layout48);
                                      if ptr.is_null()
                                      {
                                        std::alloc::handle_alloc_error(layout48);
                                      }
                                      ptr
                                    }else {
                                      std::ptr::null_mut()
                                    };
                                    for (i, e) in vec48.into_iter().enumerate() {
                                      let base = result48 as i32 + (i as i32) * 88;
                                      {
                                        let (t1_0, t1_1, ) = e;
                                        *((base + 0) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(t1_0);
                                        match t1_1 {
                                          ComponentTypeParam::TypeEmpty(e) => {
                                            *((base + 8) as *mut u8) = (0i32) as u8;
                                            let () = e;
                                            
                                          },
                                          ComponentTypeParam::TypeBool(e) => {
                                            *((base + 8) as *mut u8) = (1i32) as u8;
                                            *((base + 16) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                            
                                          },
                                          ComponentTypeParam::TypeEntityId(e) => {
                                            *((base + 8) as *mut u8) = (2i32) as u8;
                                            let EntityId{ id0:id03, id1:id13, } = e;
                                            *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id03);
                                            *((base + 24) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id13);
                                            
                                          },
                                          ComponentTypeParam::TypeF32(e) => {
                                            *((base + 8) as *mut u8) = (3i32) as u8;
                                            *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                            
                                          },
                                          ComponentTypeParam::TypeF64(e) => {
                                            *((base + 8) as *mut u8) = (4i32) as u8;
                                            *((base + 16) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                            
                                          },
                                          ComponentTypeParam::TypeMat4(e) => {
                                            *((base + 8) as *mut u8) = (5i32) as u8;
                                            let Mat4{ x:x4, y:y4, z:z4, w:w4, } = e;
                                            let Vec4{ x:x5, y:y5, z:z5, w:w5, } = x4;
                                            *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x5);
                                            *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y5);
                                            *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z5);
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w5);
                                            let Vec4{ x:x6, y:y6, z:z6, w:w6, } = y4;
                                            *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x6);
                                            *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y6);
                                            *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z6);
                                            *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w6);
                                            let Vec4{ x:x7, y:y7, z:z7, w:w7, } = z4;
                                            *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x7);
                                            *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y7);
                                            *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z7);
                                            *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w7);
                                            let Vec4{ x:x8, y:y8, z:z8, w:w8, } = w4;
                                            *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x8);
                                            *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y8);
                                            *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z8);
                                            *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w8);
                                            
                                          },
                                          ComponentTypeParam::TypeI32(e) => {
                                            *((base + 8) as *mut u8) = (6i32) as u8;
                                            *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                            
                                          },
                                          ComponentTypeParam::TypeQuat(e) => {
                                            *((base + 8) as *mut u8) = (7i32) as u8;
                                            let Quat{ x:x9, y:y9, z:z9, w:w9, } = e;
                                            *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x9);
                                            *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y9);
                                            *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z9);
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w9);
                                            
                                          },
                                          ComponentTypeParam::TypeString(e) => {
                                            *((base + 8) as *mut u8) = (8i32) as u8;
                                            let vec10 = e;
                                            let ptr10 = vec10.as_ptr() as i32;
                                            let len10 = vec10.len() as i32;
                                            *((base + 20) as *mut i32) = len10;
                                            *((base + 16) as *mut i32) = ptr10;
                                            
                                          },
                                          ComponentTypeParam::TypeU32(e) => {
                                            *((base + 8) as *mut u8) = (9i32) as u8;
                                            *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                            
                                          },
                                          ComponentTypeParam::TypeU64(e) => {
                                            *((base + 8) as *mut u8) = (10i32) as u8;
                                            *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                            
                                          },
                                          ComponentTypeParam::TypeVec2(e) => {
                                            *((base + 8) as *mut u8) = (11i32) as u8;
                                            let Vec2{ x:x11, y:y11, } = e;
                                            *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x11);
                                            *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y11);
                                            
                                          },
                                          ComponentTypeParam::TypeVec3(e) => {
                                            *((base + 8) as *mut u8) = (12i32) as u8;
                                            let Vec3{ x:x12, y:y12, z:z12, } = e;
                                            *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x12);
                                            *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y12);
                                            *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z12);
                                            
                                          },
                                          ComponentTypeParam::TypeVec4(e) => {
                                            *((base + 8) as *mut u8) = (13i32) as u8;
                                            let Vec4{ x:x13, y:y13, z:z13, w:w13, } = e;
                                            *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x13);
                                            *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y13);
                                            *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z13);
                                            *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w13);
                                            
                                          },
                                          ComponentTypeParam::TypeObjectRef(e) => {
                                            *((base + 8) as *mut u8) = (14i32) as u8;
                                            let ObjectRefParam{ id:id14, } = e;
                                            let vec15 = id14;
                                            let ptr15 = vec15.as_ptr() as i32;
                                            let len15 = vec15.len() as i32;
                                            *((base + 20) as *mut i32) = len15;
                                            *((base + 16) as *mut i32) = ptr15;
                                            
                                          },
                                          ComponentTypeParam::TypeList(e) => {
                                            *((base + 8) as *mut u8) = (15i32) as u8;
                                            match e {
                                              ComponentListTypeParam::TypeEmpty(e) => {
                                                *((base + 16) as *mut u8) = (0i32) as u8;
                                                let vec16 = e;
                                                let ptr16 = vec16.as_ptr() as i32;
                                                let len16 = vec16.len() as i32;
                                                *((base + 24) as *mut i32) = len16;
                                                *((base + 20) as *mut i32) = ptr16;
                                                
                                              },
                                              ComponentListTypeParam::TypeBool(e) => {
                                                *((base + 16) as *mut u8) = (1i32) as u8;
                                                let vec17 = e;
                                                let len17 = vec17.len() as i32;
                                                let layout17 = core::alloc::Layout::from_size_align_unchecked(vec17.len() * 1, 1);
                                                let result17 = if layout17.size() != 0
                                                {
                                                  let ptr = std::alloc::alloc(layout17);
                                                  if ptr.is_null()
                                                  {
                                                    std::alloc::handle_alloc_error(layout17);
                                                  }
                                                  ptr
                                                }else {
                                                  std::ptr::null_mut()
                                                };
                                                for (i, e) in vec17.into_iter().enumerate() {
                                                  let base = result17 as i32 + (i as i32) * 1;
                                                  {
                                                    *((base + 0) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                                    
                                                  }}
                                                  *((base + 24) as *mut i32) = len17;
                                                  *((base + 20) as *mut i32) = result17 as i32;
                                                  cleanup_list.extend_from_slice(&[(result17, layout17),]);
                                                  
                                                },
                                                ComponentListTypeParam::TypeEntityId(e) => {
                                                  *((base + 16) as *mut u8) = (2i32) as u8;
                                                  let vec18 = e;
                                                  let ptr18 = vec18.as_ptr() as i32;
                                                  let len18 = vec18.len() as i32;
                                                  *((base + 24) as *mut i32) = len18;
                                                  *((base + 20) as *mut i32) = ptr18;
                                                  
                                                },
                                                ComponentListTypeParam::TypeF32(e) => {
                                                  *((base + 16) as *mut u8) = (3i32) as u8;
                                                  let vec19 = e;
                                                  let ptr19 = vec19.as_ptr() as i32;
                                                  let len19 = vec19.len() as i32;
                                                  *((base + 24) as *mut i32) = len19;
                                                  *((base + 20) as *mut i32) = ptr19;
                                                  
                                                },
                                                ComponentListTypeParam::TypeF64(e) => {
                                                  *((base + 16) as *mut u8) = (4i32) as u8;
                                                  let vec20 = e;
                                                  let ptr20 = vec20.as_ptr() as i32;
                                                  let len20 = vec20.len() as i32;
                                                  *((base + 24) as *mut i32) = len20;
                                                  *((base + 20) as *mut i32) = ptr20;
                                                  
                                                },
                                                ComponentListTypeParam::TypeMat4(e) => {
                                                  *((base + 16) as *mut u8) = (5i32) as u8;
                                                  let vec21 = e;
                                                  let ptr21 = vec21.as_ptr() as i32;
                                                  let len21 = vec21.len() as i32;
                                                  *((base + 24) as *mut i32) = len21;
                                                  *((base + 20) as *mut i32) = ptr21;
                                                  
                                                },
                                                ComponentListTypeParam::TypeI32(e) => {
                                                  *((base + 16) as *mut u8) = (6i32) as u8;
                                                  let vec22 = e;
                                                  let ptr22 = vec22.as_ptr() as i32;
                                                  let len22 = vec22.len() as i32;
                                                  *((base + 24) as *mut i32) = len22;
                                                  *((base + 20) as *mut i32) = ptr22;
                                                  
                                                },
                                                ComponentListTypeParam::TypeQuat(e) => {
                                                  *((base + 16) as *mut u8) = (7i32) as u8;
                                                  let vec23 = e;
                                                  let ptr23 = vec23.as_ptr() as i32;
                                                  let len23 = vec23.len() as i32;
                                                  *((base + 24) as *mut i32) = len23;
                                                  *((base + 20) as *mut i32) = ptr23;
                                                  
                                                },
                                                ComponentListTypeParam::TypeString(e) => {
                                                  *((base + 16) as *mut u8) = (8i32) as u8;
                                                  let vec25 = e;
                                                  let len25 = vec25.len() as i32;
                                                  let layout25 = core::alloc::Layout::from_size_align_unchecked(vec25.len() * 8, 4);
                                                  let result25 = if layout25.size() != 0
                                                  {
                                                    let ptr = std::alloc::alloc(layout25);
                                                    if ptr.is_null()
                                                    {
                                                      std::alloc::handle_alloc_error(layout25);
                                                    }
                                                    ptr
                                                  }else {
                                                    std::ptr::null_mut()
                                                  };
                                                  for (i, e) in vec25.into_iter().enumerate() {
                                                    let base = result25 as i32 + (i as i32) * 8;
                                                    {
                                                      let vec24 = e;
                                                      let ptr24 = vec24.as_ptr() as i32;
                                                      let len24 = vec24.len() as i32;
                                                      *((base + 4) as *mut i32) = len24;
                                                      *((base + 0) as *mut i32) = ptr24;
                                                      
                                                    }}
                                                    *((base + 24) as *mut i32) = len25;
                                                    *((base + 20) as *mut i32) = result25 as i32;
                                                    cleanup_list.extend_from_slice(&[(result25, layout25),]);
                                                    
                                                  },
                                                  ComponentListTypeParam::TypeU32(e) => {
                                                    *((base + 16) as *mut u8) = (9i32) as u8;
                                                    let vec26 = e;
                                                    let ptr26 = vec26.as_ptr() as i32;
                                                    let len26 = vec26.len() as i32;
                                                    *((base + 24) as *mut i32) = len26;
                                                    *((base + 20) as *mut i32) = ptr26;
                                                    
                                                  },
                                                  ComponentListTypeParam::TypeU64(e) => {
                                                    *((base + 16) as *mut u8) = (10i32) as u8;
                                                    let vec27 = e;
                                                    let ptr27 = vec27.as_ptr() as i32;
                                                    let len27 = vec27.len() as i32;
                                                    *((base + 24) as *mut i32) = len27;
                                                    *((base + 20) as *mut i32) = ptr27;
                                                    
                                                  },
                                                  ComponentListTypeParam::TypeVec2(e) => {
                                                    *((base + 16) as *mut u8) = (11i32) as u8;
                                                    let vec28 = e;
                                                    let ptr28 = vec28.as_ptr() as i32;
                                                    let len28 = vec28.len() as i32;
                                                    *((base + 24) as *mut i32) = len28;
                                                    *((base + 20) as *mut i32) = ptr28;
                                                    
                                                  },
                                                  ComponentListTypeParam::TypeVec3(e) => {
                                                    *((base + 16) as *mut u8) = (12i32) as u8;
                                                    let vec29 = e;
                                                    let ptr29 = vec29.as_ptr() as i32;
                                                    let len29 = vec29.len() as i32;
                                                    *((base + 24) as *mut i32) = len29;
                                                    *((base + 20) as *mut i32) = ptr29;
                                                    
                                                  },
                                                  ComponentListTypeParam::TypeVec4(e) => {
                                                    *((base + 16) as *mut u8) = (13i32) as u8;
                                                    let vec30 = e;
                                                    let ptr30 = vec30.as_ptr() as i32;
                                                    let len30 = vec30.len() as i32;
                                                    *((base + 24) as *mut i32) = len30;
                                                    *((base + 20) as *mut i32) = ptr30;
                                                    
                                                  },
                                                  ComponentListTypeParam::TypeObjectRef(e) => {
                                                    *((base + 16) as *mut u8) = (14i32) as u8;
                                                    let vec33 = e;
                                                    let len33 = vec33.len() as i32;
                                                    let layout33 = core::alloc::Layout::from_size_align_unchecked(vec33.len() * 8, 4);
                                                    let result33 = if layout33.size() != 0
                                                    {
                                                      let ptr = std::alloc::alloc(layout33);
                                                      if ptr.is_null()
                                                      {
                                                        std::alloc::handle_alloc_error(layout33);
                                                      }
                                                      ptr
                                                    }else {
                                                      std::ptr::null_mut()
                                                    };
                                                    for (i, e) in vec33.into_iter().enumerate() {
                                                      let base = result33 as i32 + (i as i32) * 8;
                                                      {
                                                        let ObjectRefParam{ id:id31, } = e;
                                                        let vec32 = id31;
                                                        let ptr32 = vec32.as_ptr() as i32;
                                                        let len32 = vec32.len() as i32;
                                                        *((base + 4) as *mut i32) = len32;
                                                        *((base + 0) as *mut i32) = ptr32;
                                                        
                                                      }}
                                                      *((base + 24) as *mut i32) = len33;
                                                      *((base + 20) as *mut i32) = result33 as i32;
                                                      cleanup_list.extend_from_slice(&[(result33, layout33),]);
                                                      
                                                    },
                                                  };
                                                  
                                                },
                                                ComponentTypeParam::TypeOption(e) => {
                                                  *((base + 8) as *mut u8) = (16i32) as u8;
                                                  match e {
                                                    ComponentOptionTypeParam::TypeEmpty(e) => {
                                                      *((base + 16) as *mut u8) = (0i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let () = e;
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeBool(e) => {
                                                      *((base + 16) as *mut u8) = (1i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          *((base + 25) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeEntityId(e) => {
                                                      *((base + 16) as *mut u8) = (2i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let EntityId{ id0:id035, id1:id135, } = e;
                                                          *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id035);
                                                          *((base + 40) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id135);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeF32(e) => {
                                                      *((base + 16) as *mut u8) = (3i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeF64(e) => {
                                                      *((base + 16) as *mut u8) = (4i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          *((base + 32) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeMat4(e) => {
                                                      *((base + 16) as *mut u8) = (5i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let Mat4{ x:x36, y:y36, z:z36, w:w36, } = e;
                                                          let Vec4{ x:x37, y:y37, z:z37, w:w37, } = x36;
                                                          *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x37);
                                                          *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y37);
                                                          *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z37);
                                                          *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w37);
                                                          let Vec4{ x:x38, y:y38, z:z38, w:w38, } = y36;
                                                          *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x38);
                                                          *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y38);
                                                          *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z38);
                                                          *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w38);
                                                          let Vec4{ x:x39, y:y39, z:z39, w:w39, } = z36;
                                                          *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x39);
                                                          *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y39);
                                                          *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z39);
                                                          *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w39);
                                                          let Vec4{ x:x40, y:y40, z:z40, w:w40, } = w36;
                                                          *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x40);
                                                          *((base + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y40);
                                                          *((base + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z40);
                                                          *((base + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w40);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeI32(e) => {
                                                      *((base + 16) as *mut u8) = (6i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeQuat(e) => {
                                                      *((base + 16) as *mut u8) = (7i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let Quat{ x:x41, y:y41, z:z41, w:w41, } = e;
                                                          *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x41);
                                                          *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y41);
                                                          *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z41);
                                                          *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w41);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeString(e) => {
                                                      *((base + 16) as *mut u8) = (8i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let vec42 = e;
                                                          let ptr42 = vec42.as_ptr() as i32;
                                                          let len42 = vec42.len() as i32;
                                                          *((base + 32) as *mut i32) = len42;
                                                          *((base + 28) as *mut i32) = ptr42;
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeU32(e) => {
                                                      *((base + 16) as *mut u8) = (9i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeU64(e) => {
                                                      *((base + 16) as *mut u8) = (10i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeVec2(e) => {
                                                      *((base + 16) as *mut u8) = (11i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let Vec2{ x:x43, y:y43, } = e;
                                                          *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x43);
                                                          *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y43);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeVec3(e) => {
                                                      *((base + 16) as *mut u8) = (12i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let Vec3{ x:x44, y:y44, z:z44, } = e;
                                                          *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x44);
                                                          *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y44);
                                                          *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z44);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeVec4(e) => {
                                                      *((base + 16) as *mut u8) = (13i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let Vec4{ x:x45, y:y45, z:z45, w:w45, } = e;
                                                          *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x45);
                                                          *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y45);
                                                          *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z45);
                                                          *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w45);
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                    ComponentOptionTypeParam::TypeObjectRef(e) => {
                                                      *((base + 16) as *mut u8) = (14i32) as u8;
                                                      match e {
                                                        Some(e) => {
                                                          *((base + 24) as *mut u8) = (1i32) as u8;
                                                          let ObjectRefParam{ id:id46, } = e;
                                                          let vec47 = id46;
                                                          let ptr47 = vec47.as_ptr() as i32;
                                                          let len47 = vec47.len() as i32;
                                                          *((base + 32) as *mut i32) = len47;
                                                          *((base + 28) as *mut i32) = ptr47;
                                                          
                                                        },
                                                        None => {
                                                          let e = ();
                                                          {
                                                            *((base + 24) as *mut u8) = (0i32) as u8;
                                                            let () = e;
                                                            
                                                          }
                                                        },
                                                      };
                                                    },
                                                  };
                                                  
                                                },
                                              };
                                              
                                            }}
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-set-components: func(entity: record { id0: u64, id1: u64 }, data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-set-components: func(entity: record { id0: u64, id1: u64 }, data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit")]
                                              fn wit_import(_: i64, _: i64, _: i32, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), result48 as i32, len48);
                                            if layout48.size() != 0 {
                                              std::alloc::dealloc(result48, layout48);
                                            }
                                            for (ptr, layout) in cleanup_list {
                                              
                                              if layout.size() != 0 {
                                                
                                                std::alloc::dealloc(ptr, layout);
                                                
                                              }
                                              
                                            }
                                            ()
                                          }
                                        }
                                        pub fn entity_has_component(entity: EntityId,index: u32,) -> bool{
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-has-component: func(entity: record { id0: u64, id1: u64 }, index: u32) -> bool")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-has-component: func(entity: record { id0: u64, id1: u64 }, index: u32) -> bool")]
                                              fn wit_import(_: i64, _: i64, _: i32, ) -> i32;
                                            }
                                            let ret = wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), wit_bindgen_guest_rust::rt::as_i32(index));
                                            match ret {
                                              0 => false,
                                              1 => true,
                                              _ => panic!("invalid bool discriminant"),
                                            }
                                          }
                                        }
                                        pub fn entity_has_components(entity: EntityId,indices: &[u32],) -> bool{
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            let vec1 = indices;
                                            let ptr1 = vec1.as_ptr() as i32;
                                            let len1 = vec1.len() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-has-components: func(entity: record { id0: u64, id1: u64 }, indices: list<u32>) -> bool")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-has-components: func(entity: record { id0: u64, id1: u64 }, indices: list<u32>) -> bool")]
                                              fn wit_import(_: i64, _: i64, _: i32, _: i32, ) -> i32;
                                            }
                                            let ret = wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), ptr1, len1);
                                            match ret {
                                              0 => false,
                                              1 => true,
                                              _ => panic!("invalid bool discriminant"),
                                            }
                                          }
                                        }
                                        pub fn entity_remove_component(entity: EntityId,index: u32,) -> (){
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-remove-component: func(entity: record { id0: u64, id1: u64 }, index: u32) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-remove-component: func(entity: record { id0: u64, id1: u64 }, index: u32) -> unit")]
                                              fn wit_import(_: i64, _: i64, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), wit_bindgen_guest_rust::rt::as_i32(index));
                                            ()
                                          }
                                        }
                                        pub fn entity_remove_components(entity: EntityId,indices: &[u32],) -> (){
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            let vec1 = indices;
                                            let ptr1 = vec1.as_ptr() as i32;
                                            let len1 = vec1.len() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-remove-components: func(entity: record { id0: u64, id1: u64 }, indices: list<u32>) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-remove-components: func(entity: record { id0: u64, id1: u64 }, indices: list<u32>) -> unit")]
                                              fn wit_import(_: i64, _: i64, _: i32, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), ptr1, len1);
                                            ()
                                          }
                                        }
                                        pub fn entity_exists(entity: EntityId,) -> bool{
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-exists: func(entity: record { id0: u64, id1: u64 }) -> bool")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-exists: func(entity: record { id0: u64, id1: u64 }) -> bool")]
                                              fn wit_import(_: i64, _: i64, ) -> i32;
                                            }
                                            let ret = wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10));
                                            match ret {
                                              0 => false,
                                              1 => true,
                                              _ => panic!("invalid bool discriminant"),
                                            }
                                          }
                                        }
                                        pub fn entity_query(index: u32,) -> Vec<EntityId>{
                                          unsafe {
                                            let ptr0 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-query: func(index: u32) -> list<record { id0: u64, id1: u64 }>")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-query: func(index: u32) -> list<record { id0: u64, id1: u64 }>")]
                                              fn wit_import(_: i32, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i32(index), ptr0);
                                            let len1 = *((ptr0 + 4) as *const i32) as usize;
                                            Vec::from_raw_parts(*((ptr0 + 0) as *const i32) as *mut _, len1, len1)
                                          }
                                        }
                                        pub fn entity_resources() -> EntityId{
                                          unsafe {
                                            let ptr0 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-resources: func() -> record { id0: u64, id1: u64 }")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-resources: func() -> record { id0: u64, id1: u64 }")]
                                              fn wit_import(_: i32, );
                                            }
                                            wit_import(ptr0);
                                            EntityId{id0:*((ptr0 + 0) as *const i64) as u64, id1:*((ptr0 + 8) as *const i64) as u64, }
                                          }
                                        }
                                        pub fn entity_query2(q: Query<'_,>,t: QueryEvent,) -> u64{
                                          unsafe {
                                            let Query{ components:components0, include:include0, exclude:exclude0, changed:changed0, } = q;
                                            let vec1 = components0;
                                            let ptr1 = vec1.as_ptr() as i32;
                                            let len1 = vec1.len() as i32;
                                            let vec2 = include0;
                                            let ptr2 = vec2.as_ptr() as i32;
                                            let len2 = vec2.len() as i32;
                                            let vec3 = exclude0;
                                            let ptr3 = vec3.as_ptr() as i32;
                                            let len3 = vec3.len() as i32;
                                            let vec4 = changed0;
                                            let ptr4 = vec4.as_ptr() as i32;
                                            let len4 = vec4.len() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "entity-query2: func(q: record { components: list<u32>, include: list<u32>, exclude: list<u32>, changed: list<u32> }, t: enum { frame, spawn, despawn }) -> u64")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_entity-query2: func(q: record { components: list<u32>, include: list<u32>, exclude: list<u32>, changed: list<u32> }, t: enum { frame, spawn, despawn }) -> u64")]
                                              fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32, ) -> i64;
                                            }
                                            let ret = wit_import(ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4, match t {
                                              QueryEvent::Frame => 0,
                                              QueryEvent::Spawn => 1,
                                              QueryEvent::Despawn => 2,
                                            });
                                            ret as u64
                                          }
                                        }
                                        pub fn query_eval(q: u64,) -> Vec<(EntityId,Vec<ComponentTypeResult>,)>{
                                          unsafe {
                                            let ptr0 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "query-eval: func(q: u64) -> list<tuple<record { id0: u64, id1: u64 }, list<variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>>")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_query-eval: func(q: u64) -> list<tuple<record { id0: u64, id1: u64 }, list<variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>>")]
                                              fn wit_import(_: i64, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(q), ptr0);
                                            let base23 = *((ptr0 + 0) as *const i32);
                                            let len23 = *((ptr0 + 4) as *const i32);
                                            let mut result23 = Vec::with_capacity(len23 as usize);
                                            for i in 0..len23 {
                                              let base = base23 + i *24;
                                              result23.push({
                                                let base22 = *((base + 16) as *const i32);
                                                let len22 = *((base + 20) as *const i32);
                                                let mut result22 = Vec::with_capacity(len22 as usize);
                                                for i in 0..len22 {
                                                  let base = base22 + i *80;
                                                  result22.push(match i32::from(*((base + 0) as *const u8)) {
                                                    0 => ComponentTypeResult::TypeEmpty(()),
                                                    1 => ComponentTypeResult::TypeBool(match i32::from(*((base + 8) as *const u8)) {
                                                      0 => false,
                                                      1 => true,
                                                      _ => panic!("invalid bool discriminant"),
                                                    }),
                                                    2 => ComponentTypeResult::TypeEntityId(EntityId{id0:*((base + 8) as *const i64) as u64, id1:*((base + 16) as *const i64) as u64, }),
                                                    3 => ComponentTypeResult::TypeF32(*((base + 8) as *const f32)),
                                                    4 => ComponentTypeResult::TypeF64(*((base + 8) as *const f64)),
                                                    5 => ComponentTypeResult::TypeMat4(Mat4{x:Vec4{x:*((base + 8) as *const f32), y:*((base + 12) as *const f32), z:*((base + 16) as *const f32), w:*((base + 20) as *const f32), }, y:Vec4{x:*((base + 24) as *const f32), y:*((base + 28) as *const f32), z:*((base + 32) as *const f32), w:*((base + 36) as *const f32), }, z:Vec4{x:*((base + 40) as *const f32), y:*((base + 44) as *const f32), z:*((base + 48) as *const f32), w:*((base + 52) as *const f32), }, w:Vec4{x:*((base + 56) as *const f32), y:*((base + 60) as *const f32), z:*((base + 64) as *const f32), w:*((base + 68) as *const f32), }, }),
                                                    6 => ComponentTypeResult::TypeI32(*((base + 8) as *const i32)),
                                                    7 => ComponentTypeResult::TypeQuat(Quat{x:*((base + 8) as *const f32), y:*((base + 12) as *const f32), z:*((base + 16) as *const f32), w:*((base + 20) as *const f32), }),
                                                    8 => ComponentTypeResult::TypeString({
                                                      let len1 = *((base + 12) as *const i32) as usize;
                                                      
                                                      String::from_utf8(Vec::from_raw_parts(*((base + 8) as *const i32) as *mut _, len1, len1)).unwrap()
                                                    }),
                                                    9 => ComponentTypeResult::TypeU32(*((base + 8) as *const i32) as u32),
                                                    10 => ComponentTypeResult::TypeU64(*((base + 8) as *const i64) as u64),
                                                    11 => ComponentTypeResult::TypeVec2(Vec2{x:*((base + 8) as *const f32), y:*((base + 12) as *const f32), }),
                                                    12 => ComponentTypeResult::TypeVec3(Vec3{x:*((base + 8) as *const f32), y:*((base + 12) as *const f32), z:*((base + 16) as *const f32), }),
                                                    13 => ComponentTypeResult::TypeVec4(Vec4{x:*((base + 8) as *const f32), y:*((base + 12) as *const f32), z:*((base + 16) as *const f32), w:*((base + 20) as *const f32), }),
                                                    14 => ComponentTypeResult::TypeObjectRef({
                                                      let len2 = *((base + 12) as *const i32) as usize;
                                                      
                                                      ObjectRefResult{id:String::from_utf8(Vec::from_raw_parts(*((base + 8) as *const i32) as *mut _, len2, len2)).unwrap(), }
                                                    }),
                                                    15 => ComponentTypeResult::TypeList(match i32::from(*((base + 8) as *const u8)) {
                                                      0 => ComponentListTypeResult::TypeEmpty({
                                                        let len3 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len3, len3)
                                                      }),
                                                      1 => ComponentListTypeResult::TypeBool({
                                                        let base4 = *((base + 12) as *const i32);
                                                        let len4 = *((base + 16) as *const i32);
                                                        let mut result4 = Vec::with_capacity(len4 as usize);
                                                        for i in 0..len4 {
                                                          let base = base4 + i *1;
                                                          result4.push(match i32::from(*((base + 0) as *const u8)) {
                                                            0 => false,
                                                            1 => true,
                                                            _ => panic!("invalid bool discriminant"),
                                                          });
                                                        }
                                                        if len4 != 0 {
                                                          std::alloc::dealloc(base4 as *mut _, std::alloc::Layout::from_size_align_unchecked((len4 as usize) * 1, 1));
                                                        }
                                                        
                                                        result4
                                                      }),
                                                      2 => ComponentListTypeResult::TypeEntityId({
                                                        let len5 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len5, len5)
                                                      }),
                                                      3 => ComponentListTypeResult::TypeF32({
                                                        let len6 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len6, len6)
                                                      }),
                                                      4 => ComponentListTypeResult::TypeF64({
                                                        let len7 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len7, len7)
                                                      }),
                                                      5 => ComponentListTypeResult::TypeMat4({
                                                        let len8 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len8, len8)
                                                      }),
                                                      6 => ComponentListTypeResult::TypeI32({
                                                        let len9 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len9, len9)
                                                      }),
                                                      7 => ComponentListTypeResult::TypeQuat({
                                                        let len10 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len10, len10)
                                                      }),
                                                      8 => ComponentListTypeResult::TypeString({
                                                        let base12 = *((base + 12) as *const i32);
                                                        let len12 = *((base + 16) as *const i32);
                                                        let mut result12 = Vec::with_capacity(len12 as usize);
                                                        for i in 0..len12 {
                                                          let base = base12 + i *8;
                                                          result12.push({
                                                            let len11 = *((base + 4) as *const i32) as usize;
                                                            
                                                            String::from_utf8(Vec::from_raw_parts(*((base + 0) as *const i32) as *mut _, len11, len11)).unwrap()
                                                          });
                                                        }
                                                        if len12 != 0 {
                                                          std::alloc::dealloc(base12 as *mut _, std::alloc::Layout::from_size_align_unchecked((len12 as usize) * 8, 4));
                                                        }
                                                        
                                                        result12
                                                      }),
                                                      9 => ComponentListTypeResult::TypeU32({
                                                        let len13 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len13, len13)
                                                      }),
                                                      10 => ComponentListTypeResult::TypeU64({
                                                        let len14 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len14, len14)
                                                      }),
                                                      11 => ComponentListTypeResult::TypeVec2({
                                                        let len15 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len15, len15)
                                                      }),
                                                      12 => ComponentListTypeResult::TypeVec3({
                                                        let len16 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len16, len16)
                                                      }),
                                                      13 => ComponentListTypeResult::TypeVec4({
                                                        let len17 = *((base + 16) as *const i32) as usize;
                                                        
                                                        Vec::from_raw_parts(*((base + 12) as *const i32) as *mut _, len17, len17)
                                                      }),
                                                      14 => ComponentListTypeResult::TypeObjectRef({
                                                        let base19 = *((base + 12) as *const i32);
                                                        let len19 = *((base + 16) as *const i32);
                                                        let mut result19 = Vec::with_capacity(len19 as usize);
                                                        for i in 0..len19 {
                                                          let base = base19 + i *8;
                                                          result19.push({
                                                            let len18 = *((base + 4) as *const i32) as usize;
                                                            
                                                            ObjectRefResult{id:String::from_utf8(Vec::from_raw_parts(*((base + 0) as *const i32) as *mut _, len18, len18)).unwrap(), }
                                                          });
                                                        }
                                                        if len19 != 0 {
                                                          std::alloc::dealloc(base19 as *mut _, std::alloc::Layout::from_size_align_unchecked((len19 as usize) * 8, 4));
                                                        }
                                                        
                                                        result19
                                                      }),
                                                      _ => panic!("invalid enum discriminant"),
                                                    }),
                                                    16 => ComponentTypeResult::TypeOption(match i32::from(*((base + 8) as *const u8)) {
                                                      0 => ComponentOptionTypeResult::TypeEmpty(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(()),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      1 => ComponentOptionTypeResult::TypeBool(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(match i32::from(*((base + 17) as *const u8)) {
                                                          0 => false,
                                                          1 => true,
                                                          _ => panic!("invalid bool discriminant"),
                                                        }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      2 => ComponentOptionTypeResult::TypeEntityId(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(EntityId{id0:*((base + 24) as *const i64) as u64, id1:*((base + 32) as *const i64) as u64, }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      3 => ComponentOptionTypeResult::TypeF32(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(*((base + 20) as *const f32)),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      4 => ComponentOptionTypeResult::TypeF64(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(*((base + 24) as *const f64)),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      5 => ComponentOptionTypeResult::TypeMat4(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(Mat4{x:Vec4{x:*((base + 20) as *const f32), y:*((base + 24) as *const f32), z:*((base + 28) as *const f32), w:*((base + 32) as *const f32), }, y:Vec4{x:*((base + 36) as *const f32), y:*((base + 40) as *const f32), z:*((base + 44) as *const f32), w:*((base + 48) as *const f32), }, z:Vec4{x:*((base + 52) as *const f32), y:*((base + 56) as *const f32), z:*((base + 60) as *const f32), w:*((base + 64) as *const f32), }, w:Vec4{x:*((base + 68) as *const f32), y:*((base + 72) as *const f32), z:*((base + 76) as *const f32), w:*((base + 80) as *const f32), }, }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      6 => ComponentOptionTypeResult::TypeI32(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(*((base + 20) as *const i32)),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      7 => ComponentOptionTypeResult::TypeQuat(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(Quat{x:*((base + 20) as *const f32), y:*((base + 24) as *const f32), z:*((base + 28) as *const f32), w:*((base + 32) as *const f32), }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      8 => ComponentOptionTypeResult::TypeString(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some({
                                                          let len20 = *((base + 24) as *const i32) as usize;
                                                          
                                                          String::from_utf8(Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len20, len20)).unwrap()
                                                        }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      9 => ComponentOptionTypeResult::TypeU32(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(*((base + 20) as *const i32) as u32),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      10 => ComponentOptionTypeResult::TypeU64(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(*((base + 24) as *const i64) as u64),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      11 => ComponentOptionTypeResult::TypeVec2(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(Vec2{x:*((base + 20) as *const f32), y:*((base + 24) as *const f32), }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      12 => ComponentOptionTypeResult::TypeVec3(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(Vec3{x:*((base + 20) as *const f32), y:*((base + 24) as *const f32), z:*((base + 28) as *const f32), }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      13 => ComponentOptionTypeResult::TypeVec4(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some(Vec4{x:*((base + 20) as *const f32), y:*((base + 24) as *const f32), z:*((base + 28) as *const f32), w:*((base + 32) as *const f32), }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      14 => ComponentOptionTypeResult::TypeObjectRef(match i32::from(*((base + 16) as *const u8)) {
                                                        0 => None,
                                                        1 => Some({
                                                          let len21 = *((base + 24) as *const i32) as usize;
                                                          
                                                          ObjectRefResult{id:String::from_utf8(Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len21, len21)).unwrap(), }
                                                        }),
                                                        _ => panic!("invalid enum discriminant"),
                                                      }),
                                                      _ => panic!("invalid enum discriminant"),
                                                    }),
                                                    _ => panic!("invalid enum discriminant"),
                                                  });
                                                }
                                                if len22 != 0 {
                                                  std::alloc::dealloc(base22 as *mut _, std::alloc::Layout::from_size_align_unchecked((len22 as usize) * 80, 8));
                                                }
                                                
                                                (EntityId{id0:*((base + 0) as *const i64) as u64, id1:*((base + 8) as *const i64) as u64, }, result22)
                                              });
                                            }
                                            if len23 != 0 {
                                              std::alloc::dealloc(base23 as *mut _, std::alloc::Layout::from_size_align_unchecked((len23 as usize) * 24, 8));
                                            }
                                            result23
                                          }
                                        }
                                        pub fn player_get_raw_input(player: EntityId,) -> Option<PlayerRawInput>{
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = player;
                                            let ptr1 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "player-get-raw-input: func(player: record { id0: u64, id1: u64 }) -> option<record { keys: list<enum { key1, key2, key3, key4, key5, key6, key7, key8, key9, key0, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, escape, f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16, f17, f18, f19, f20, f21, f22, f23, f24, snapshot, scroll, pause, insert, home, delete, end, page-down, page-up, left, up, right, down, back, return, space, compose, caret, numlock, numpad0, numpad1, numpad2, numpad3, numpad4, numpad5, numpad6, numpad7, numpad8, numpad9, numpad-add, numpad-divide, numpad-decimal, numpad-comma, numpad-enter, numpad-equals, numpad-multiply, numpad-subtract, abnt-c1, abnt-c2, apostrophe, apps, asterisk, at, ax, backslash, calculator, capital, colon, comma, convert, equals, grave, kana, kanji, l-alt, l-bracket, l-control, l-shift, l-win, mail, media-select, media-stop, minus, mute, my-computer, navigate-forward, navigate-backward, next-track, no-convert, oem102, period, play-pause, plus, power, prev-track, r-alt, r-bracket, r-control, r-shift, r-win, semicolon, slash, sleep, stop, sysrq, tab, underline, unlabeled, volume-down, volume-up, wake, web-back, web-favorites, web-forward, web-home, web-refresh, web-search, web-stop, yen, copy, paste, cut }>, mouse-position: record { x: float32, y: float32 }, mouse-wheel: float32, mouse-buttons: list<variant { left(unit), right(unit), middle(unit), other(u16) }> }>")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_player-get-raw-input: func(player: record { id0: u64, id1: u64 }) -> option<record { keys: list<enum { key1, key2, key3, key4, key5, key6, key7, key8, key9, key0, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, escape, f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16, f17, f18, f19, f20, f21, f22, f23, f24, snapshot, scroll, pause, insert, home, delete, end, page-down, page-up, left, up, right, down, back, return, space, compose, caret, numlock, numpad0, numpad1, numpad2, numpad3, numpad4, numpad5, numpad6, numpad7, numpad8, numpad9, numpad-add, numpad-divide, numpad-decimal, numpad-comma, numpad-enter, numpad-equals, numpad-multiply, numpad-subtract, abnt-c1, abnt-c2, apostrophe, apps, asterisk, at, ax, backslash, calculator, capital, colon, comma, convert, equals, grave, kana, kanji, l-alt, l-bracket, l-control, l-shift, l-win, mail, media-select, media-stop, minus, mute, my-computer, navigate-forward, navigate-backward, next-track, no-convert, oem102, period, play-pause, plus, power, prev-track, r-alt, r-bracket, r-control, r-shift, r-win, semicolon, slash, sleep, stop, sysrq, tab, underline, unlabeled, volume-down, volume-up, wake, web-back, web-favorites, web-forward, web-home, web-refresh, web-search, web-stop, yen, copy, paste, cut }>, mouse-position: record { x: float32, y: float32 }, mouse-wheel: float32, mouse-buttons: list<variant { left(unit), right(unit), middle(unit), other(u16) }> }>")]
                                              fn wit_import(_: i64, _: i64, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), ptr1);
                                            match i32::from(*((ptr1 + 0) as *const u8)) {
                                              0 => None,
                                              1 => Some({
                                                let base2 = *((ptr1 + 4) as *const i32);
                                                let len2 = *((ptr1 + 8) as *const i32);
                                                let mut result2 = Vec::with_capacity(len2 as usize);
                                                for i in 0..len2 {
                                                  let base = base2 + i *1;
                                                  result2.push(match i32::from(*((base + 0) as *const u8)) {
                                                    0 => VirtualKeyCode::Key1,
                                                    1 => VirtualKeyCode::Key2,
                                                    2 => VirtualKeyCode::Key3,
                                                    3 => VirtualKeyCode::Key4,
                                                    4 => VirtualKeyCode::Key5,
                                                    5 => VirtualKeyCode::Key6,
                                                    6 => VirtualKeyCode::Key7,
                                                    7 => VirtualKeyCode::Key8,
                                                    8 => VirtualKeyCode::Key9,
                                                    9 => VirtualKeyCode::Key0,
                                                    10 => VirtualKeyCode::A,
                                                    11 => VirtualKeyCode::B,
                                                    12 => VirtualKeyCode::C,
                                                    13 => VirtualKeyCode::D,
                                                    14 => VirtualKeyCode::E,
                                                    15 => VirtualKeyCode::F,
                                                    16 => VirtualKeyCode::G,
                                                    17 => VirtualKeyCode::H,
                                                    18 => VirtualKeyCode::I,
                                                    19 => VirtualKeyCode::J,
                                                    20 => VirtualKeyCode::K,
                                                    21 => VirtualKeyCode::L,
                                                    22 => VirtualKeyCode::M,
                                                    23 => VirtualKeyCode::N,
                                                    24 => VirtualKeyCode::O,
                                                    25 => VirtualKeyCode::P,
                                                    26 => VirtualKeyCode::Q,
                                                    27 => VirtualKeyCode::R,
                                                    28 => VirtualKeyCode::S,
                                                    29 => VirtualKeyCode::T,
                                                    30 => VirtualKeyCode::U,
                                                    31 => VirtualKeyCode::V,
                                                    32 => VirtualKeyCode::W,
                                                    33 => VirtualKeyCode::X,
                                                    34 => VirtualKeyCode::Y,
                                                    35 => VirtualKeyCode::Z,
                                                    36 => VirtualKeyCode::Escape,
                                                    37 => VirtualKeyCode::F1,
                                                    38 => VirtualKeyCode::F2,
                                                    39 => VirtualKeyCode::F3,
                                                    40 => VirtualKeyCode::F4,
                                                    41 => VirtualKeyCode::F5,
                                                    42 => VirtualKeyCode::F6,
                                                    43 => VirtualKeyCode::F7,
                                                    44 => VirtualKeyCode::F8,
                                                    45 => VirtualKeyCode::F9,
                                                    46 => VirtualKeyCode::F10,
                                                    47 => VirtualKeyCode::F11,
                                                    48 => VirtualKeyCode::F12,
                                                    49 => VirtualKeyCode::F13,
                                                    50 => VirtualKeyCode::F14,
                                                    51 => VirtualKeyCode::F15,
                                                    52 => VirtualKeyCode::F16,
                                                    53 => VirtualKeyCode::F17,
                                                    54 => VirtualKeyCode::F18,
                                                    55 => VirtualKeyCode::F19,
                                                    56 => VirtualKeyCode::F20,
                                                    57 => VirtualKeyCode::F21,
                                                    58 => VirtualKeyCode::F22,
                                                    59 => VirtualKeyCode::F23,
                                                    60 => VirtualKeyCode::F24,
                                                    61 => VirtualKeyCode::Snapshot,
                                                    62 => VirtualKeyCode::Scroll,
                                                    63 => VirtualKeyCode::Pause,
                                                    64 => VirtualKeyCode::Insert,
                                                    65 => VirtualKeyCode::Home,
                                                    66 => VirtualKeyCode::Delete,
                                                    67 => VirtualKeyCode::End,
                                                    68 => VirtualKeyCode::PageDown,
                                                    69 => VirtualKeyCode::PageUp,
                                                    70 => VirtualKeyCode::Left,
                                                    71 => VirtualKeyCode::Up,
                                                    72 => VirtualKeyCode::Right,
                                                    73 => VirtualKeyCode::Down,
                                                    74 => VirtualKeyCode::Back,
                                                    75 => VirtualKeyCode::Return,
                                                    76 => VirtualKeyCode::Space,
                                                    77 => VirtualKeyCode::Compose,
                                                    78 => VirtualKeyCode::Caret,
                                                    79 => VirtualKeyCode::Numlock,
                                                    80 => VirtualKeyCode::Numpad0,
                                                    81 => VirtualKeyCode::Numpad1,
                                                    82 => VirtualKeyCode::Numpad2,
                                                    83 => VirtualKeyCode::Numpad3,
                                                    84 => VirtualKeyCode::Numpad4,
                                                    85 => VirtualKeyCode::Numpad5,
                                                    86 => VirtualKeyCode::Numpad6,
                                                    87 => VirtualKeyCode::Numpad7,
                                                    88 => VirtualKeyCode::Numpad8,
                                                    89 => VirtualKeyCode::Numpad9,
                                                    90 => VirtualKeyCode::NumpadAdd,
                                                    91 => VirtualKeyCode::NumpadDivide,
                                                    92 => VirtualKeyCode::NumpadDecimal,
                                                    93 => VirtualKeyCode::NumpadComma,
                                                    94 => VirtualKeyCode::NumpadEnter,
                                                    95 => VirtualKeyCode::NumpadEquals,
                                                    96 => VirtualKeyCode::NumpadMultiply,
                                                    97 => VirtualKeyCode::NumpadSubtract,
                                                    98 => VirtualKeyCode::AbntC1,
                                                    99 => VirtualKeyCode::AbntC2,
                                                    100 => VirtualKeyCode::Apostrophe,
                                                    101 => VirtualKeyCode::Apps,
                                                    102 => VirtualKeyCode::Asterisk,
                                                    103 => VirtualKeyCode::At,
                                                    104 => VirtualKeyCode::Ax,
                                                    105 => VirtualKeyCode::Backslash,
                                                    106 => VirtualKeyCode::Calculator,
                                                    107 => VirtualKeyCode::Capital,
                                                    108 => VirtualKeyCode::Colon,
                                                    109 => VirtualKeyCode::Comma,
                                                    110 => VirtualKeyCode::Convert,
                                                    111 => VirtualKeyCode::Equals,
                                                    112 => VirtualKeyCode::Grave,
                                                    113 => VirtualKeyCode::Kana,
                                                    114 => VirtualKeyCode::Kanji,
                                                    115 => VirtualKeyCode::LAlt,
                                                    116 => VirtualKeyCode::LBracket,
                                                    117 => VirtualKeyCode::LControl,
                                                    118 => VirtualKeyCode::LShift,
                                                    119 => VirtualKeyCode::LWin,
                                                    120 => VirtualKeyCode::Mail,
                                                    121 => VirtualKeyCode::MediaSelect,
                                                    122 => VirtualKeyCode::MediaStop,
                                                    123 => VirtualKeyCode::Minus,
                                                    124 => VirtualKeyCode::Mute,
                                                    125 => VirtualKeyCode::MyComputer,
                                                    126 => VirtualKeyCode::NavigateForward,
                                                    127 => VirtualKeyCode::NavigateBackward,
                                                    128 => VirtualKeyCode::NextTrack,
                                                    129 => VirtualKeyCode::NoConvert,
                                                    130 => VirtualKeyCode::Oem102,
                                                    131 => VirtualKeyCode::Period,
                                                    132 => VirtualKeyCode::PlayPause,
                                                    133 => VirtualKeyCode::Plus,
                                                    134 => VirtualKeyCode::Power,
                                                    135 => VirtualKeyCode::PrevTrack,
                                                    136 => VirtualKeyCode::RAlt,
                                                    137 => VirtualKeyCode::RBracket,
                                                    138 => VirtualKeyCode::RControl,
                                                    139 => VirtualKeyCode::RShift,
                                                    140 => VirtualKeyCode::RWin,
                                                    141 => VirtualKeyCode::Semicolon,
                                                    142 => VirtualKeyCode::Slash,
                                                    143 => VirtualKeyCode::Sleep,
                                                    144 => VirtualKeyCode::Stop,
                                                    145 => VirtualKeyCode::Sysrq,
                                                    146 => VirtualKeyCode::Tab,
                                                    147 => VirtualKeyCode::Underline,
                                                    148 => VirtualKeyCode::Unlabeled,
                                                    149 => VirtualKeyCode::VolumeDown,
                                                    150 => VirtualKeyCode::VolumeUp,
                                                    151 => VirtualKeyCode::Wake,
                                                    152 => VirtualKeyCode::WebBack,
                                                    153 => VirtualKeyCode::WebFavorites,
                                                    154 => VirtualKeyCode::WebForward,
                                                    155 => VirtualKeyCode::WebHome,
                                                    156 => VirtualKeyCode::WebRefresh,
                                                    157 => VirtualKeyCode::WebSearch,
                                                    158 => VirtualKeyCode::WebStop,
                                                    159 => VirtualKeyCode::Yen,
                                                    160 => VirtualKeyCode::Copy,
                                                    161 => VirtualKeyCode::Paste,
                                                    162 => VirtualKeyCode::Cut,
                                                    _ => panic!("invalid enum discriminant"),
                                                  });
                                                }
                                                if len2 != 0 {
                                                  std::alloc::dealloc(base2 as *mut _, std::alloc::Layout::from_size_align_unchecked((len2 as usize) * 1, 1));
                                                }
                                                let base3 = *((ptr1 + 24) as *const i32);
                                                let len3 = *((ptr1 + 28) as *const i32);
                                                let mut result3 = Vec::with_capacity(len3 as usize);
                                                for i in 0..len3 {
                                                  let base = base3 + i *4;
                                                  result3.push(match i32::from(*((base + 0) as *const u8)) {
                                                    0 => MouseButton::Left,
                                                    1 => MouseButton::Right,
                                                    2 => MouseButton::Middle,
                                                    3 => MouseButton::Other(i32::from(*((base + 2) as *const u16)) as u16),
                                                    _ => panic!("invalid enum discriminant"),
                                                  });
                                                }
                                                if len3 != 0 {
                                                  std::alloc::dealloc(base3 as *mut _, std::alloc::Layout::from_size_align_unchecked((len3 as usize) * 4, 2));
                                                }
                                                
                                                PlayerRawInput{keys:result2, mouse_position:Vec2{x:*((ptr1 + 12) as *const f32), y:*((ptr1 + 16) as *const f32), }, mouse_wheel:*((ptr1 + 20) as *const f32), mouse_buttons:result3, }
                                              }),
                                              _ => panic!("invalid enum discriminant"),
                                            }
                                          }
                                        }
                                        pub fn player_get_prev_raw_input(player: EntityId,) -> Option<PlayerRawInput>{
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = player;
                                            let ptr1 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "player-get-prev-raw-input: func(player: record { id0: u64, id1: u64 }) -> option<record { keys: list<enum { key1, key2, key3, key4, key5, key6, key7, key8, key9, key0, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, escape, f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16, f17, f18, f19, f20, f21, f22, f23, f24, snapshot, scroll, pause, insert, home, delete, end, page-down, page-up, left, up, right, down, back, return, space, compose, caret, numlock, numpad0, numpad1, numpad2, numpad3, numpad4, numpad5, numpad6, numpad7, numpad8, numpad9, numpad-add, numpad-divide, numpad-decimal, numpad-comma, numpad-enter, numpad-equals, numpad-multiply, numpad-subtract, abnt-c1, abnt-c2, apostrophe, apps, asterisk, at, ax, backslash, calculator, capital, colon, comma, convert, equals, grave, kana, kanji, l-alt, l-bracket, l-control, l-shift, l-win, mail, media-select, media-stop, minus, mute, my-computer, navigate-forward, navigate-backward, next-track, no-convert, oem102, period, play-pause, plus, power, prev-track, r-alt, r-bracket, r-control, r-shift, r-win, semicolon, slash, sleep, stop, sysrq, tab, underline, unlabeled, volume-down, volume-up, wake, web-back, web-favorites, web-forward, web-home, web-refresh, web-search, web-stop, yen, copy, paste, cut }>, mouse-position: record { x: float32, y: float32 }, mouse-wheel: float32, mouse-buttons: list<variant { left(unit), right(unit), middle(unit), other(u16) }> }>")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_player-get-prev-raw-input: func(player: record { id0: u64, id1: u64 }) -> option<record { keys: list<enum { key1, key2, key3, key4, key5, key6, key7, key8, key9, key0, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z, escape, f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16, f17, f18, f19, f20, f21, f22, f23, f24, snapshot, scroll, pause, insert, home, delete, end, page-down, page-up, left, up, right, down, back, return, space, compose, caret, numlock, numpad0, numpad1, numpad2, numpad3, numpad4, numpad5, numpad6, numpad7, numpad8, numpad9, numpad-add, numpad-divide, numpad-decimal, numpad-comma, numpad-enter, numpad-equals, numpad-multiply, numpad-subtract, abnt-c1, abnt-c2, apostrophe, apps, asterisk, at, ax, backslash, calculator, capital, colon, comma, convert, equals, grave, kana, kanji, l-alt, l-bracket, l-control, l-shift, l-win, mail, media-select, media-stop, minus, mute, my-computer, navigate-forward, navigate-backward, next-track, no-convert, oem102, period, play-pause, plus, power, prev-track, r-alt, r-bracket, r-control, r-shift, r-win, semicolon, slash, sleep, stop, sysrq, tab, underline, unlabeled, volume-down, volume-up, wake, web-back, web-favorites, web-forward, web-home, web-refresh, web-search, web-stop, yen, copy, paste, cut }>, mouse-position: record { x: float32, y: float32 }, mouse-wheel: float32, mouse-buttons: list<variant { left(unit), right(unit), middle(unit), other(u16) }> }>")]
                                              fn wit_import(_: i64, _: i64, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), ptr1);
                                            match i32::from(*((ptr1 + 0) as *const u8)) {
                                              0 => None,
                                              1 => Some({
                                                let base2 = *((ptr1 + 4) as *const i32);
                                                let len2 = *((ptr1 + 8) as *const i32);
                                                let mut result2 = Vec::with_capacity(len2 as usize);
                                                for i in 0..len2 {
                                                  let base = base2 + i *1;
                                                  result2.push(match i32::from(*((base + 0) as *const u8)) {
                                                    0 => VirtualKeyCode::Key1,
                                                    1 => VirtualKeyCode::Key2,
                                                    2 => VirtualKeyCode::Key3,
                                                    3 => VirtualKeyCode::Key4,
                                                    4 => VirtualKeyCode::Key5,
                                                    5 => VirtualKeyCode::Key6,
                                                    6 => VirtualKeyCode::Key7,
                                                    7 => VirtualKeyCode::Key8,
                                                    8 => VirtualKeyCode::Key9,
                                                    9 => VirtualKeyCode::Key0,
                                                    10 => VirtualKeyCode::A,
                                                    11 => VirtualKeyCode::B,
                                                    12 => VirtualKeyCode::C,
                                                    13 => VirtualKeyCode::D,
                                                    14 => VirtualKeyCode::E,
                                                    15 => VirtualKeyCode::F,
                                                    16 => VirtualKeyCode::G,
                                                    17 => VirtualKeyCode::H,
                                                    18 => VirtualKeyCode::I,
                                                    19 => VirtualKeyCode::J,
                                                    20 => VirtualKeyCode::K,
                                                    21 => VirtualKeyCode::L,
                                                    22 => VirtualKeyCode::M,
                                                    23 => VirtualKeyCode::N,
                                                    24 => VirtualKeyCode::O,
                                                    25 => VirtualKeyCode::P,
                                                    26 => VirtualKeyCode::Q,
                                                    27 => VirtualKeyCode::R,
                                                    28 => VirtualKeyCode::S,
                                                    29 => VirtualKeyCode::T,
                                                    30 => VirtualKeyCode::U,
                                                    31 => VirtualKeyCode::V,
                                                    32 => VirtualKeyCode::W,
                                                    33 => VirtualKeyCode::X,
                                                    34 => VirtualKeyCode::Y,
                                                    35 => VirtualKeyCode::Z,
                                                    36 => VirtualKeyCode::Escape,
                                                    37 => VirtualKeyCode::F1,
                                                    38 => VirtualKeyCode::F2,
                                                    39 => VirtualKeyCode::F3,
                                                    40 => VirtualKeyCode::F4,
                                                    41 => VirtualKeyCode::F5,
                                                    42 => VirtualKeyCode::F6,
                                                    43 => VirtualKeyCode::F7,
                                                    44 => VirtualKeyCode::F8,
                                                    45 => VirtualKeyCode::F9,
                                                    46 => VirtualKeyCode::F10,
                                                    47 => VirtualKeyCode::F11,
                                                    48 => VirtualKeyCode::F12,
                                                    49 => VirtualKeyCode::F13,
                                                    50 => VirtualKeyCode::F14,
                                                    51 => VirtualKeyCode::F15,
                                                    52 => VirtualKeyCode::F16,
                                                    53 => VirtualKeyCode::F17,
                                                    54 => VirtualKeyCode::F18,
                                                    55 => VirtualKeyCode::F19,
                                                    56 => VirtualKeyCode::F20,
                                                    57 => VirtualKeyCode::F21,
                                                    58 => VirtualKeyCode::F22,
                                                    59 => VirtualKeyCode::F23,
                                                    60 => VirtualKeyCode::F24,
                                                    61 => VirtualKeyCode::Snapshot,
                                                    62 => VirtualKeyCode::Scroll,
                                                    63 => VirtualKeyCode::Pause,
                                                    64 => VirtualKeyCode::Insert,
                                                    65 => VirtualKeyCode::Home,
                                                    66 => VirtualKeyCode::Delete,
                                                    67 => VirtualKeyCode::End,
                                                    68 => VirtualKeyCode::PageDown,
                                                    69 => VirtualKeyCode::PageUp,
                                                    70 => VirtualKeyCode::Left,
                                                    71 => VirtualKeyCode::Up,
                                                    72 => VirtualKeyCode::Right,
                                                    73 => VirtualKeyCode::Down,
                                                    74 => VirtualKeyCode::Back,
                                                    75 => VirtualKeyCode::Return,
                                                    76 => VirtualKeyCode::Space,
                                                    77 => VirtualKeyCode::Compose,
                                                    78 => VirtualKeyCode::Caret,
                                                    79 => VirtualKeyCode::Numlock,
                                                    80 => VirtualKeyCode::Numpad0,
                                                    81 => VirtualKeyCode::Numpad1,
                                                    82 => VirtualKeyCode::Numpad2,
                                                    83 => VirtualKeyCode::Numpad3,
                                                    84 => VirtualKeyCode::Numpad4,
                                                    85 => VirtualKeyCode::Numpad5,
                                                    86 => VirtualKeyCode::Numpad6,
                                                    87 => VirtualKeyCode::Numpad7,
                                                    88 => VirtualKeyCode::Numpad8,
                                                    89 => VirtualKeyCode::Numpad9,
                                                    90 => VirtualKeyCode::NumpadAdd,
                                                    91 => VirtualKeyCode::NumpadDivide,
                                                    92 => VirtualKeyCode::NumpadDecimal,
                                                    93 => VirtualKeyCode::NumpadComma,
                                                    94 => VirtualKeyCode::NumpadEnter,
                                                    95 => VirtualKeyCode::NumpadEquals,
                                                    96 => VirtualKeyCode::NumpadMultiply,
                                                    97 => VirtualKeyCode::NumpadSubtract,
                                                    98 => VirtualKeyCode::AbntC1,
                                                    99 => VirtualKeyCode::AbntC2,
                                                    100 => VirtualKeyCode::Apostrophe,
                                                    101 => VirtualKeyCode::Apps,
                                                    102 => VirtualKeyCode::Asterisk,
                                                    103 => VirtualKeyCode::At,
                                                    104 => VirtualKeyCode::Ax,
                                                    105 => VirtualKeyCode::Backslash,
                                                    106 => VirtualKeyCode::Calculator,
                                                    107 => VirtualKeyCode::Capital,
                                                    108 => VirtualKeyCode::Colon,
                                                    109 => VirtualKeyCode::Comma,
                                                    110 => VirtualKeyCode::Convert,
                                                    111 => VirtualKeyCode::Equals,
                                                    112 => VirtualKeyCode::Grave,
                                                    113 => VirtualKeyCode::Kana,
                                                    114 => VirtualKeyCode::Kanji,
                                                    115 => VirtualKeyCode::LAlt,
                                                    116 => VirtualKeyCode::LBracket,
                                                    117 => VirtualKeyCode::LControl,
                                                    118 => VirtualKeyCode::LShift,
                                                    119 => VirtualKeyCode::LWin,
                                                    120 => VirtualKeyCode::Mail,
                                                    121 => VirtualKeyCode::MediaSelect,
                                                    122 => VirtualKeyCode::MediaStop,
                                                    123 => VirtualKeyCode::Minus,
                                                    124 => VirtualKeyCode::Mute,
                                                    125 => VirtualKeyCode::MyComputer,
                                                    126 => VirtualKeyCode::NavigateForward,
                                                    127 => VirtualKeyCode::NavigateBackward,
                                                    128 => VirtualKeyCode::NextTrack,
                                                    129 => VirtualKeyCode::NoConvert,
                                                    130 => VirtualKeyCode::Oem102,
                                                    131 => VirtualKeyCode::Period,
                                                    132 => VirtualKeyCode::PlayPause,
                                                    133 => VirtualKeyCode::Plus,
                                                    134 => VirtualKeyCode::Power,
                                                    135 => VirtualKeyCode::PrevTrack,
                                                    136 => VirtualKeyCode::RAlt,
                                                    137 => VirtualKeyCode::RBracket,
                                                    138 => VirtualKeyCode::RControl,
                                                    139 => VirtualKeyCode::RShift,
                                                    140 => VirtualKeyCode::RWin,
                                                    141 => VirtualKeyCode::Semicolon,
                                                    142 => VirtualKeyCode::Slash,
                                                    143 => VirtualKeyCode::Sleep,
                                                    144 => VirtualKeyCode::Stop,
                                                    145 => VirtualKeyCode::Sysrq,
                                                    146 => VirtualKeyCode::Tab,
                                                    147 => VirtualKeyCode::Underline,
                                                    148 => VirtualKeyCode::Unlabeled,
                                                    149 => VirtualKeyCode::VolumeDown,
                                                    150 => VirtualKeyCode::VolumeUp,
                                                    151 => VirtualKeyCode::Wake,
                                                    152 => VirtualKeyCode::WebBack,
                                                    153 => VirtualKeyCode::WebFavorites,
                                                    154 => VirtualKeyCode::WebForward,
                                                    155 => VirtualKeyCode::WebHome,
                                                    156 => VirtualKeyCode::WebRefresh,
                                                    157 => VirtualKeyCode::WebSearch,
                                                    158 => VirtualKeyCode::WebStop,
                                                    159 => VirtualKeyCode::Yen,
                                                    160 => VirtualKeyCode::Copy,
                                                    161 => VirtualKeyCode::Paste,
                                                    162 => VirtualKeyCode::Cut,
                                                    _ => panic!("invalid enum discriminant"),
                                                  });
                                                }
                                                if len2 != 0 {
                                                  std::alloc::dealloc(base2 as *mut _, std::alloc::Layout::from_size_align_unchecked((len2 as usize) * 1, 1));
                                                }
                                                let base3 = *((ptr1 + 24) as *const i32);
                                                let len3 = *((ptr1 + 28) as *const i32);
                                                let mut result3 = Vec::with_capacity(len3 as usize);
                                                for i in 0..len3 {
                                                  let base = base3 + i *4;
                                                  result3.push(match i32::from(*((base + 0) as *const u8)) {
                                                    0 => MouseButton::Left,
                                                    1 => MouseButton::Right,
                                                    2 => MouseButton::Middle,
                                                    3 => MouseButton::Other(i32::from(*((base + 2) as *const u16)) as u16),
                                                    _ => panic!("invalid enum discriminant"),
                                                  });
                                                }
                                                if len3 != 0 {
                                                  std::alloc::dealloc(base3 as *mut _, std::alloc::Layout::from_size_align_unchecked((len3 as usize) * 4, 2));
                                                }
                                                
                                                PlayerRawInput{keys:result2, mouse_position:Vec2{x:*((ptr1 + 12) as *const f32), y:*((ptr1 + 16) as *const f32), }, mouse_wheel:*((ptr1 + 20) as *const f32), mouse_buttons:result3, }
                                              }),
                                              _ => panic!("invalid enum discriminant"),
                                            }
                                          }
                                        }
                                        pub fn physics_apply_force(entities: &[EntityId],force: Vec3,) -> (){
                                          unsafe {
                                            let vec0 = entities;
                                            let ptr0 = vec0.as_ptr() as i32;
                                            let len0 = vec0.len() as i32;
                                            let Vec3{ x:x1, y:y1, z:z1, } = force;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-apply-force: func(entities: list<record { id0: u64, id1: u64 }>, force: record { x: float32, y: float32, z: float32 }) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-apply-force: func(entities: list<record { id0: u64, id1: u64 }>, force: record { x: float32, y: float32, z: float32 }) -> unit")]
                                              fn wit_import(_: i32, _: i32, _: f32, _: f32, _: f32, );
                                            }
                                            wit_import(ptr0, len0, wit_bindgen_guest_rust::rt::as_f32(x1), wit_bindgen_guest_rust::rt::as_f32(y1), wit_bindgen_guest_rust::rt::as_f32(z1));
                                            ()
                                          }
                                        }
                                        pub fn physics_explode_bomb(position: Vec3,force: f32,radius: f32,falloff_radius: Option<f32>,) -> (){
                                          unsafe {
                                            let Vec3{ x:x0, y:y0, z:z0, } = position;
                                            let (result1_0,result1_1,) = match falloff_radius {
                                              Some(e) => (1i32, wit_bindgen_guest_rust::rt::as_f32(e)),
                                              None => {
                                                let e = ();
                                                {
                                                  let () = e;
                                                  
                                                  (0i32, 0.0f32)
                                                }
                                              },
                                            };#[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-explode-bomb: func(position: record { x: float32, y: float32, z: float32 }, force: float32, radius: float32, falloff-radius: option<float32>) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-explode-bomb: func(position: record { x: float32, y: float32, z: float32 }, force: float32, radius: float32, falloff-radius: option<float32>) -> unit")]
                                              fn wit_import(_: f32, _: f32, _: f32, _: f32, _: f32, _: i32, _: f32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_f32(x0), wit_bindgen_guest_rust::rt::as_f32(y0), wit_bindgen_guest_rust::rt::as_f32(z0), wit_bindgen_guest_rust::rt::as_f32(force), wit_bindgen_guest_rust::rt::as_f32(radius), result1_0, result1_1);
                                            ()
                                          }
                                        }
                                        pub fn physics_set_gravity(gravity: Vec3,) -> (){
                                          unsafe {
                                            let Vec3{ x:x0, y:y0, z:z0, } = gravity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-set-gravity: func(gravity: record { x: float32, y: float32, z: float32 }) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-set-gravity: func(gravity: record { x: float32, y: float32, z: float32 }) -> unit")]
                                              fn wit_import(_: f32, _: f32, _: f32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_f32(x0), wit_bindgen_guest_rust::rt::as_f32(y0), wit_bindgen_guest_rust::rt::as_f32(z0));
                                            ()
                                          }
                                        }
                                        pub fn physics_unfreeze(entity: EntityId,) -> (){
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-unfreeze: func(entity: record { id0: u64, id1: u64 }) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-unfreeze: func(entity: record { id0: u64, id1: u64 }) -> unit")]
                                              fn wit_import(_: i64, _: i64, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10));
                                            ()
                                          }
                                        }
                                        pub fn physics_freeze(entity: EntityId,) -> (){
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-freeze: func(entity: record { id0: u64, id1: u64 }) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-freeze: func(entity: record { id0: u64, id1: u64 }) -> unit")]
                                              fn wit_import(_: i64, _: i64, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10));
                                            ()
                                          }
                                        }
                                        pub fn physics_start_motor(entity: EntityId,velocity: f32,) -> (){
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-start-motor: func(entity: record { id0: u64, id1: u64 }, velocity: float32) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-start-motor: func(entity: record { id0: u64, id1: u64 }, velocity: float32) -> unit")]
                                              fn wit_import(_: i64, _: i64, _: f32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10), wit_bindgen_guest_rust::rt::as_f32(velocity));
                                            ()
                                          }
                                        }
                                        pub fn physics_stop_motor(entity: EntityId,) -> (){
                                          unsafe {
                                            let EntityId{ id0:id00, id1:id10, } = entity;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-stop-motor: func(entity: record { id0: u64, id1: u64 }) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-stop-motor: func(entity: record { id0: u64, id1: u64 }) -> unit")]
                                              fn wit_import(_: i64, _: i64, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_i64(id00), wit_bindgen_guest_rust::rt::as_i64(id10));
                                            ()
                                          }
                                        }
                                        pub fn physics_raycast_first(origin: Vec3,direction: Vec3,) -> Option<(EntityId,f32,)>{
                                          unsafe {
                                            let Vec3{ x:x0, y:y0, z:z0, } = origin;
                                            let Vec3{ x:x1, y:y1, z:z1, } = direction;
                                            let ptr2 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-raycast-first: func(origin: record { x: float32, y: float32, z: float32 }, direction: record { x: float32, y: float32, z: float32 }) -> option<tuple<record { id0: u64, id1: u64 }, float32>>")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-raycast-first: func(origin: record { x: float32, y: float32, z: float32 }, direction: record { x: float32, y: float32, z: float32 }) -> option<tuple<record { id0: u64, id1: u64 }, float32>>")]
                                              fn wit_import(_: f32, _: f32, _: f32, _: f32, _: f32, _: f32, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_f32(x0), wit_bindgen_guest_rust::rt::as_f32(y0), wit_bindgen_guest_rust::rt::as_f32(z0), wit_bindgen_guest_rust::rt::as_f32(x1), wit_bindgen_guest_rust::rt::as_f32(y1), wit_bindgen_guest_rust::rt::as_f32(z1), ptr2);
                                            match i32::from(*((ptr2 + 0) as *const u8)) {
                                              0 => None,
                                              1 => Some((EntityId{id0:*((ptr2 + 8) as *const i64) as u64, id1:*((ptr2 + 16) as *const i64) as u64, }, *((ptr2 + 24) as *const f32))),
                                              _ => panic!("invalid enum discriminant"),
                                            }
                                          }
                                        }
                                        pub fn physics_raycast(origin: Vec3,direction: Vec3,) -> Vec<(EntityId,f32,)>{
                                          unsafe {
                                            let Vec3{ x:x0, y:y0, z:z0, } = origin;
                                            let Vec3{ x:x1, y:y1, z:z1, } = direction;
                                            let ptr2 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "physics-raycast: func(origin: record { x: float32, y: float32, z: float32 }, direction: record { x: float32, y: float32, z: float32 }) -> list<tuple<record { id0: u64, id1: u64 }, float32>>")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_physics-raycast: func(origin: record { x: float32, y: float32, z: float32 }, direction: record { x: float32, y: float32, z: float32 }) -> list<tuple<record { id0: u64, id1: u64 }, float32>>")]
                                              fn wit_import(_: f32, _: f32, _: f32, _: f32, _: f32, _: f32, _: i32, );
                                            }
                                            wit_import(wit_bindgen_guest_rust::rt::as_f32(x0), wit_bindgen_guest_rust::rt::as_f32(y0), wit_bindgen_guest_rust::rt::as_f32(z0), wit_bindgen_guest_rust::rt::as_f32(x1), wit_bindgen_guest_rust::rt::as_f32(y1), wit_bindgen_guest_rust::rt::as_f32(z1), ptr2);
                                            let len3 = *((ptr2 + 4) as *const i32) as usize;
                                            Vec::from_raw_parts(*((ptr2 + 0) as *const i32) as *mut _, len3, len3)
                                          }
                                        }
                                        pub fn event_subscribe(name: & str,) -> (){
                                          unsafe {
                                            let vec0 = name;
                                            let ptr0 = vec0.as_ptr() as i32;
                                            let len0 = vec0.len() as i32;
                                            #[link(wasm_import_module = "host")]
                                            extern "C" {
                                              #[cfg_attr(target_arch = "wasm32", link_name = "event-subscribe: func(name: string) -> unit")]
                                              #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_event-subscribe: func(name: string) -> unit")]
                                              fn wit_import(_: i32, _: i32, );
                                            }
                                            wit_import(ptr0, len0);
                                            ()
                                          }
                                        }
                                        pub fn event_send(name: & str,data: Components<'_,>,) -> (){
                                          unsafe {
                                            let mut cleanup_list = Vec::new();
                                            let vec0 = name;
                                            let ptr0 = vec0.as_ptr() as i32;
                                            let len0 = vec0.len() as i32;
                                            let vec48 = data;
                                            let len48 = vec48.len() as i32;
                                            let layout48 = core::alloc::Layout::from_size_align_unchecked(vec48.len() * 88, 8);
                                            let result48 = if layout48.size() != 0
                                            {
                                              let ptr = std::alloc::alloc(layout48);
                                              if ptr.is_null()
                                              {
                                                std::alloc::handle_alloc_error(layout48);
                                              }
                                              ptr
                                            }else {
                                              std::ptr::null_mut()
                                            };
                                            for (i, e) in vec48.into_iter().enumerate() {
                                              let base = result48 as i32 + (i as i32) * 88;
                                              {
                                                let (t1_0, t1_1, ) = e;
                                                *((base + 0) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(t1_0);
                                                match t1_1 {
                                                  ComponentTypeParam::TypeEmpty(e) => {
                                                    *((base + 8) as *mut u8) = (0i32) as u8;
                                                    let () = e;
                                                    
                                                  },
                                                  ComponentTypeParam::TypeBool(e) => {
                                                    *((base + 8) as *mut u8) = (1i32) as u8;
                                                    *((base + 16) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                                    
                                                  },
                                                  ComponentTypeParam::TypeEntityId(e) => {
                                                    *((base + 8) as *mut u8) = (2i32) as u8;
                                                    let EntityId{ id0:id03, id1:id13, } = e;
                                                    *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id03);
                                                    *((base + 24) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id13);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeF32(e) => {
                                                    *((base + 8) as *mut u8) = (3i32) as u8;
                                                    *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeF64(e) => {
                                                    *((base + 8) as *mut u8) = (4i32) as u8;
                                                    *((base + 16) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeMat4(e) => {
                                                    *((base + 8) as *mut u8) = (5i32) as u8;
                                                    let Mat4{ x:x4, y:y4, z:z4, w:w4, } = e;
                                                    let Vec4{ x:x5, y:y5, z:z5, w:w5, } = x4;
                                                    *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x5);
                                                    *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y5);
                                                    *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z5);
                                                    *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w5);
                                                    let Vec4{ x:x6, y:y6, z:z6, w:w6, } = y4;
                                                    *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x6);
                                                    *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y6);
                                                    *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z6);
                                                    *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w6);
                                                    let Vec4{ x:x7, y:y7, z:z7, w:w7, } = z4;
                                                    *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x7);
                                                    *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y7);
                                                    *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z7);
                                                    *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w7);
                                                    let Vec4{ x:x8, y:y8, z:z8, w:w8, } = w4;
                                                    *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x8);
                                                    *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y8);
                                                    *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z8);
                                                    *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w8);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeI32(e) => {
                                                    *((base + 8) as *mut u8) = (6i32) as u8;
                                                    *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeQuat(e) => {
                                                    *((base + 8) as *mut u8) = (7i32) as u8;
                                                    let Quat{ x:x9, y:y9, z:z9, w:w9, } = e;
                                                    *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x9);
                                                    *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y9);
                                                    *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z9);
                                                    *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w9);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeString(e) => {
                                                    *((base + 8) as *mut u8) = (8i32) as u8;
                                                    let vec10 = e;
                                                    let ptr10 = vec10.as_ptr() as i32;
                                                    let len10 = vec10.len() as i32;
                                                    *((base + 20) as *mut i32) = len10;
                                                    *((base + 16) as *mut i32) = ptr10;
                                                    
                                                  },
                                                  ComponentTypeParam::TypeU32(e) => {
                                                    *((base + 8) as *mut u8) = (9i32) as u8;
                                                    *((base + 16) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeU64(e) => {
                                                    *((base + 8) as *mut u8) = (10i32) as u8;
                                                    *((base + 16) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeVec2(e) => {
                                                    *((base + 8) as *mut u8) = (11i32) as u8;
                                                    let Vec2{ x:x11, y:y11, } = e;
                                                    *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x11);
                                                    *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y11);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeVec3(e) => {
                                                    *((base + 8) as *mut u8) = (12i32) as u8;
                                                    let Vec3{ x:x12, y:y12, z:z12, } = e;
                                                    *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x12);
                                                    *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y12);
                                                    *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z12);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeVec4(e) => {
                                                    *((base + 8) as *mut u8) = (13i32) as u8;
                                                    let Vec4{ x:x13, y:y13, z:z13, w:w13, } = e;
                                                    *((base + 16) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x13);
                                                    *((base + 20) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y13);
                                                    *((base + 24) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z13);
                                                    *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w13);
                                                    
                                                  },
                                                  ComponentTypeParam::TypeObjectRef(e) => {
                                                    *((base + 8) as *mut u8) = (14i32) as u8;
                                                    let ObjectRefParam{ id:id14, } = e;
                                                    let vec15 = id14;
                                                    let ptr15 = vec15.as_ptr() as i32;
                                                    let len15 = vec15.len() as i32;
                                                    *((base + 20) as *mut i32) = len15;
                                                    *((base + 16) as *mut i32) = ptr15;
                                                    
                                                  },
                                                  ComponentTypeParam::TypeList(e) => {
                                                    *((base + 8) as *mut u8) = (15i32) as u8;
                                                    match e {
                                                      ComponentListTypeParam::TypeEmpty(e) => {
                                                        *((base + 16) as *mut u8) = (0i32) as u8;
                                                        let vec16 = e;
                                                        let ptr16 = vec16.as_ptr() as i32;
                                                        let len16 = vec16.len() as i32;
                                                        *((base + 24) as *mut i32) = len16;
                                                        *((base + 20) as *mut i32) = ptr16;
                                                        
                                                      },
                                                      ComponentListTypeParam::TypeBool(e) => {
                                                        *((base + 16) as *mut u8) = (1i32) as u8;
                                                        let vec17 = e;
                                                        let len17 = vec17.len() as i32;
                                                        let layout17 = core::alloc::Layout::from_size_align_unchecked(vec17.len() * 1, 1);
                                                        let result17 = if layout17.size() != 0
                                                        {
                                                          let ptr = std::alloc::alloc(layout17);
                                                          if ptr.is_null()
                                                          {
                                                            std::alloc::handle_alloc_error(layout17);
                                                          }
                                                          ptr
                                                        }else {
                                                          std::ptr::null_mut()
                                                        };
                                                        for (i, e) in vec17.into_iter().enumerate() {
                                                          let base = result17 as i32 + (i as i32) * 1;
                                                          {
                                                            *((base + 0) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                                            
                                                          }}
                                                          *((base + 24) as *mut i32) = len17;
                                                          *((base + 20) as *mut i32) = result17 as i32;
                                                          cleanup_list.extend_from_slice(&[(result17, layout17),]);
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeEntityId(e) => {
                                                          *((base + 16) as *mut u8) = (2i32) as u8;
                                                          let vec18 = e;
                                                          let ptr18 = vec18.as_ptr() as i32;
                                                          let len18 = vec18.len() as i32;
                                                          *((base + 24) as *mut i32) = len18;
                                                          *((base + 20) as *mut i32) = ptr18;
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeF32(e) => {
                                                          *((base + 16) as *mut u8) = (3i32) as u8;
                                                          let vec19 = e;
                                                          let ptr19 = vec19.as_ptr() as i32;
                                                          let len19 = vec19.len() as i32;
                                                          *((base + 24) as *mut i32) = len19;
                                                          *((base + 20) as *mut i32) = ptr19;
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeF64(e) => {
                                                          *((base + 16) as *mut u8) = (4i32) as u8;
                                                          let vec20 = e;
                                                          let ptr20 = vec20.as_ptr() as i32;
                                                          let len20 = vec20.len() as i32;
                                                          *((base + 24) as *mut i32) = len20;
                                                          *((base + 20) as *mut i32) = ptr20;
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeMat4(e) => {
                                                          *((base + 16) as *mut u8) = (5i32) as u8;
                                                          let vec21 = e;
                                                          let ptr21 = vec21.as_ptr() as i32;
                                                          let len21 = vec21.len() as i32;
                                                          *((base + 24) as *mut i32) = len21;
                                                          *((base + 20) as *mut i32) = ptr21;
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeI32(e) => {
                                                          *((base + 16) as *mut u8) = (6i32) as u8;
                                                          let vec22 = e;
                                                          let ptr22 = vec22.as_ptr() as i32;
                                                          let len22 = vec22.len() as i32;
                                                          *((base + 24) as *mut i32) = len22;
                                                          *((base + 20) as *mut i32) = ptr22;
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeQuat(e) => {
                                                          *((base + 16) as *mut u8) = (7i32) as u8;
                                                          let vec23 = e;
                                                          let ptr23 = vec23.as_ptr() as i32;
                                                          let len23 = vec23.len() as i32;
                                                          *((base + 24) as *mut i32) = len23;
                                                          *((base + 20) as *mut i32) = ptr23;
                                                          
                                                        },
                                                        ComponentListTypeParam::TypeString(e) => {
                                                          *((base + 16) as *mut u8) = (8i32) as u8;
                                                          let vec25 = e;
                                                          let len25 = vec25.len() as i32;
                                                          let layout25 = core::alloc::Layout::from_size_align_unchecked(vec25.len() * 8, 4);
                                                          let result25 = if layout25.size() != 0
                                                          {
                                                            let ptr = std::alloc::alloc(layout25);
                                                            if ptr.is_null()
                                                            {
                                                              std::alloc::handle_alloc_error(layout25);
                                                            }
                                                            ptr
                                                          }else {
                                                            std::ptr::null_mut()
                                                          };
                                                          for (i, e) in vec25.into_iter().enumerate() {
                                                            let base = result25 as i32 + (i as i32) * 8;
                                                            {
                                                              let vec24 = e;
                                                              let ptr24 = vec24.as_ptr() as i32;
                                                              let len24 = vec24.len() as i32;
                                                              *((base + 4) as *mut i32) = len24;
                                                              *((base + 0) as *mut i32) = ptr24;
                                                              
                                                            }}
                                                            *((base + 24) as *mut i32) = len25;
                                                            *((base + 20) as *mut i32) = result25 as i32;
                                                            cleanup_list.extend_from_slice(&[(result25, layout25),]);
                                                            
                                                          },
                                                          ComponentListTypeParam::TypeU32(e) => {
                                                            *((base + 16) as *mut u8) = (9i32) as u8;
                                                            let vec26 = e;
                                                            let ptr26 = vec26.as_ptr() as i32;
                                                            let len26 = vec26.len() as i32;
                                                            *((base + 24) as *mut i32) = len26;
                                                            *((base + 20) as *mut i32) = ptr26;
                                                            
                                                          },
                                                          ComponentListTypeParam::TypeU64(e) => {
                                                            *((base + 16) as *mut u8) = (10i32) as u8;
                                                            let vec27 = e;
                                                            let ptr27 = vec27.as_ptr() as i32;
                                                            let len27 = vec27.len() as i32;
                                                            *((base + 24) as *mut i32) = len27;
                                                            *((base + 20) as *mut i32) = ptr27;
                                                            
                                                          },
                                                          ComponentListTypeParam::TypeVec2(e) => {
                                                            *((base + 16) as *mut u8) = (11i32) as u8;
                                                            let vec28 = e;
                                                            let ptr28 = vec28.as_ptr() as i32;
                                                            let len28 = vec28.len() as i32;
                                                            *((base + 24) as *mut i32) = len28;
                                                            *((base + 20) as *mut i32) = ptr28;
                                                            
                                                          },
                                                          ComponentListTypeParam::TypeVec3(e) => {
                                                            *((base + 16) as *mut u8) = (12i32) as u8;
                                                            let vec29 = e;
                                                            let ptr29 = vec29.as_ptr() as i32;
                                                            let len29 = vec29.len() as i32;
                                                            *((base + 24) as *mut i32) = len29;
                                                            *((base + 20) as *mut i32) = ptr29;
                                                            
                                                          },
                                                          ComponentListTypeParam::TypeVec4(e) => {
                                                            *((base + 16) as *mut u8) = (13i32) as u8;
                                                            let vec30 = e;
                                                            let ptr30 = vec30.as_ptr() as i32;
                                                            let len30 = vec30.len() as i32;
                                                            *((base + 24) as *mut i32) = len30;
                                                            *((base + 20) as *mut i32) = ptr30;
                                                            
                                                          },
                                                          ComponentListTypeParam::TypeObjectRef(e) => {
                                                            *((base + 16) as *mut u8) = (14i32) as u8;
                                                            let vec33 = e;
                                                            let len33 = vec33.len() as i32;
                                                            let layout33 = core::alloc::Layout::from_size_align_unchecked(vec33.len() * 8, 4);
                                                            let result33 = if layout33.size() != 0
                                                            {
                                                              let ptr = std::alloc::alloc(layout33);
                                                              if ptr.is_null()
                                                              {
                                                                std::alloc::handle_alloc_error(layout33);
                                                              }
                                                              ptr
                                                            }else {
                                                              std::ptr::null_mut()
                                                            };
                                                            for (i, e) in vec33.into_iter().enumerate() {
                                                              let base = result33 as i32 + (i as i32) * 8;
                                                              {
                                                                let ObjectRefParam{ id:id31, } = e;
                                                                let vec32 = id31;
                                                                let ptr32 = vec32.as_ptr() as i32;
                                                                let len32 = vec32.len() as i32;
                                                                *((base + 4) as *mut i32) = len32;
                                                                *((base + 0) as *mut i32) = ptr32;
                                                                
                                                              }}
                                                              *((base + 24) as *mut i32) = len33;
                                                              *((base + 20) as *mut i32) = result33 as i32;
                                                              cleanup_list.extend_from_slice(&[(result33, layout33),]);
                                                              
                                                            },
                                                          };
                                                          
                                                        },
                                                        ComponentTypeParam::TypeOption(e) => {
                                                          *((base + 8) as *mut u8) = (16i32) as u8;
                                                          match e {
                                                            ComponentOptionTypeParam::TypeEmpty(e) => {
                                                              *((base + 16) as *mut u8) = (0i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let () = e;
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeBool(e) => {
                                                              *((base + 16) as *mut u8) = (1i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  *((base + 25) as *mut u8) = (match e { true => 1, false => 0 }) as u8;
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeEntityId(e) => {
                                                              *((base + 16) as *mut u8) = (2i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let EntityId{ id0:id035, id1:id135, } = e;
                                                                  *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id035);
                                                                  *((base + 40) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(id135);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeF32(e) => {
                                                              *((base + 16) as *mut u8) = (3i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(e);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeF64(e) => {
                                                              *((base + 16) as *mut u8) = (4i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  *((base + 32) as *mut f64) = wit_bindgen_guest_rust::rt::as_f64(e);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeMat4(e) => {
                                                              *((base + 16) as *mut u8) = (5i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let Mat4{ x:x36, y:y36, z:z36, w:w36, } = e;
                                                                  let Vec4{ x:x37, y:y37, z:z37, w:w37, } = x36;
                                                                  *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x37);
                                                                  *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y37);
                                                                  *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z37);
                                                                  *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w37);
                                                                  let Vec4{ x:x38, y:y38, z:z38, w:w38, } = y36;
                                                                  *((base + 44) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x38);
                                                                  *((base + 48) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y38);
                                                                  *((base + 52) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z38);
                                                                  *((base + 56) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w38);
                                                                  let Vec4{ x:x39, y:y39, z:z39, w:w39, } = z36;
                                                                  *((base + 60) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x39);
                                                                  *((base + 64) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y39);
                                                                  *((base + 68) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z39);
                                                                  *((base + 72) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w39);
                                                                  let Vec4{ x:x40, y:y40, z:z40, w:w40, } = w36;
                                                                  *((base + 76) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x40);
                                                                  *((base + 80) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y40);
                                                                  *((base + 84) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z40);
                                                                  *((base + 88) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w40);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeI32(e) => {
                                                              *((base + 16) as *mut u8) = (6i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeQuat(e) => {
                                                              *((base + 16) as *mut u8) = (7i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let Quat{ x:x41, y:y41, z:z41, w:w41, } = e;
                                                                  *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x41);
                                                                  *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y41);
                                                                  *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z41);
                                                                  *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w41);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeString(e) => {
                                                              *((base + 16) as *mut u8) = (8i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let vec42 = e;
                                                                  let ptr42 = vec42.as_ptr() as i32;
                                                                  let len42 = vec42.len() as i32;
                                                                  *((base + 32) as *mut i32) = len42;
                                                                  *((base + 28) as *mut i32) = ptr42;
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeU32(e) => {
                                                              *((base + 16) as *mut u8) = (9i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  *((base + 28) as *mut i32) = wit_bindgen_guest_rust::rt::as_i32(e);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeU64(e) => {
                                                              *((base + 16) as *mut u8) = (10i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  *((base + 32) as *mut i64) = wit_bindgen_guest_rust::rt::as_i64(e);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeVec2(e) => {
                                                              *((base + 16) as *mut u8) = (11i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let Vec2{ x:x43, y:y43, } = e;
                                                                  *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x43);
                                                                  *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y43);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeVec3(e) => {
                                                              *((base + 16) as *mut u8) = (12i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let Vec3{ x:x44, y:y44, z:z44, } = e;
                                                                  *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x44);
                                                                  *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y44);
                                                                  *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z44);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeVec4(e) => {
                                                              *((base + 16) as *mut u8) = (13i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let Vec4{ x:x45, y:y45, z:z45, w:w45, } = e;
                                                                  *((base + 28) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(x45);
                                                                  *((base + 32) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(y45);
                                                                  *((base + 36) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(z45);
                                                                  *((base + 40) as *mut f32) = wit_bindgen_guest_rust::rt::as_f32(w45);
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                            ComponentOptionTypeParam::TypeObjectRef(e) => {
                                                              *((base + 16) as *mut u8) = (14i32) as u8;
                                                              match e {
                                                                Some(e) => {
                                                                  *((base + 24) as *mut u8) = (1i32) as u8;
                                                                  let ObjectRefParam{ id:id46, } = e;
                                                                  let vec47 = id46;
                                                                  let ptr47 = vec47.as_ptr() as i32;
                                                                  let len47 = vec47.len() as i32;
                                                                  *((base + 32) as *mut i32) = len47;
                                                                  *((base + 28) as *mut i32) = ptr47;
                                                                  
                                                                },
                                                                None => {
                                                                  let e = ();
                                                                  {
                                                                    *((base + 24) as *mut u8) = (0i32) as u8;
                                                                    let () = e;
                                                                    
                                                                  }
                                                                },
                                                              };
                                                            },
                                                          };
                                                          
                                                        },
                                                      };
                                                      
                                                    }}
                                                    #[link(wasm_import_module = "host")]
                                                    extern "C" {
                                                      #[cfg_attr(target_arch = "wasm32", link_name = "event-send: func(name: string, data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit")]
                                                      #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_event-send: func(name: string, data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit")]
                                                      fn wit_import(_: i32, _: i32, _: i32, _: i32, );
                                                    }
                                                    wit_import(ptr0, len0, result48 as i32, len48);
                                                    if layout48.size() != 0 {
                                                      std::alloc::dealloc(result48, layout48);
                                                    }
                                                    for (ptr, layout) in cleanup_list {
                                                      
                                                      if layout.size() != 0 {
                                                        
                                                        std::alloc::dealloc(ptr, layout);
                                                        
                                                      }
                                                      
                                                    }
                                                    ()
                                                  }
                                                }
                                                pub fn asset_url(path: & str,) -> Option<String>{
                                                  unsafe {
                                                    let vec0 = path;
                                                    let ptr0 = vec0.as_ptr() as i32;
                                                    let len0 = vec0.len() as i32;
                                                    let ptr1 = __HOST_RET_AREA.0.as_mut_ptr() as i32;
                                                    #[link(wasm_import_module = "host")]
                                                    extern "C" {
                                                      #[cfg_attr(target_arch = "wasm32", link_name = "asset-url: func(path: string) -> option<string>")]
                                                      #[cfg_attr(not(target_arch = "wasm32"), link_name = "host_asset-url: func(path: string) -> option<string>")]
                                                      fn wit_import(_: i32, _: i32, _: i32, );
                                                    }
                                                    wit_import(ptr0, len0, ptr1);
                                                    match i32::from(*((ptr1 + 0) as *const u8)) {
                                                      0 => None,
                                                      1 => Some({
                                                        let len2 = *((ptr1 + 8) as *const i32) as usize;
                                                        
                                                        String::from_utf8(Vec::from_raw_parts(*((ptr1 + 4) as *const i32) as *mut _, len2, len2)).unwrap()
                                                      }),
                                                      _ => panic!("invalid enum discriminant"),
                                                    }
                                                  }
                                                }
                                                
                                                #[repr(align(8))]
                                                struct __HostRetArea([u8; 104]);
                                                static mut __HOST_RET_AREA: __HostRetArea = __HostRetArea([0; 104]);
                                              }
                                              #[allow(clippy::all)]
#[allow(missing_docs)] pub mod guest {
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct RunContext {
    pub time: f32,
  }
  impl core::fmt::Debug for RunContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("RunContext").field("time", &self.time).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct EntityId {
    pub id0: u64,
    pub id1: u64,
  }
  impl core::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("EntityId").field("id0", &self.id0).field("id1", &self.id1).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Vec2 {
    pub x: f32,
    pub y: f32,
  }
  impl core::fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Vec2").field("x", &self.x).field("y", &self.y).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
  }
  impl core::fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Vec3").field("x", &self.x).field("y", &self.y).field("z", &self.z).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
  }
  impl core::fmt::Debug for Vec4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Vec4").field("x", &self.x).field("y", &self.y).field("z", &self.z).field("w", &self.w).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
  }
  impl core::fmt::Debug for Quat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Quat").field("x", &self.x).field("y", &self.y).field("z", &self.z).field("w", &self.w).finish()}
  }
  #[repr(C)]
  #[derive(Copy, Clone)]
  pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
  }
  impl core::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("Mat4").field("x", &self.x).field("y", &self.y).field("z", &self.z).field("w", &self.w).finish()}
  }
  #[derive(Clone)]
  pub struct ObjectRef {
    pub id: String,
  }
  impl core::fmt::Debug for ObjectRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      f.debug_struct("ObjectRef").field("id", &self.id).finish()}
  }
  #[derive(Clone)]
  pub enum ComponentListType{
    TypeEmpty(Vec<()>),
    TypeBool(Vec<bool>),
    TypeEntityId(Vec<EntityId>),
    TypeF32(Vec<f32>),
    TypeF64(Vec<f64>),
    TypeMat4(Vec<Mat4>),
    TypeI32(Vec<i32>),
    TypeQuat(Vec<Quat>),
    TypeString(Vec<String>),
    TypeU32(Vec<u32>),
    TypeU64(Vec<u64>),
    TypeVec2(Vec<Vec2>),
    TypeVec3(Vec<Vec3>),
    TypeVec4(Vec<Vec4>),
    TypeObjectRef(Vec<ObjectRef>),
  }
  impl core::fmt::Debug for ComponentListType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentListType::TypeEmpty(e) => {
          f.debug_tuple("ComponentListType::TypeEmpty").field(e).finish()
        }
        ComponentListType::TypeBool(e) => {
          f.debug_tuple("ComponentListType::TypeBool").field(e).finish()
        }
        ComponentListType::TypeEntityId(e) => {
          f.debug_tuple("ComponentListType::TypeEntityId").field(e).finish()
        }
        ComponentListType::TypeF32(e) => {
          f.debug_tuple("ComponentListType::TypeF32").field(e).finish()
        }
        ComponentListType::TypeF64(e) => {
          f.debug_tuple("ComponentListType::TypeF64").field(e).finish()
        }
        ComponentListType::TypeMat4(e) => {
          f.debug_tuple("ComponentListType::TypeMat4").field(e).finish()
        }
        ComponentListType::TypeI32(e) => {
          f.debug_tuple("ComponentListType::TypeI32").field(e).finish()
        }
        ComponentListType::TypeQuat(e) => {
          f.debug_tuple("ComponentListType::TypeQuat").field(e).finish()
        }
        ComponentListType::TypeString(e) => {
          f.debug_tuple("ComponentListType::TypeString").field(e).finish()
        }
        ComponentListType::TypeU32(e) => {
          f.debug_tuple("ComponentListType::TypeU32").field(e).finish()
        }
        ComponentListType::TypeU64(e) => {
          f.debug_tuple("ComponentListType::TypeU64").field(e).finish()
        }
        ComponentListType::TypeVec2(e) => {
          f.debug_tuple("ComponentListType::TypeVec2").field(e).finish()
        }
        ComponentListType::TypeVec3(e) => {
          f.debug_tuple("ComponentListType::TypeVec3").field(e).finish()
        }
        ComponentListType::TypeVec4(e) => {
          f.debug_tuple("ComponentListType::TypeVec4").field(e).finish()
        }
        ComponentListType::TypeObjectRef(e) => {
          f.debug_tuple("ComponentListType::TypeObjectRef").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentOptionType{
    TypeEmpty(Option<()>),
    TypeBool(Option<bool>),
    TypeEntityId(Option<EntityId>),
    TypeF32(Option<f32>),
    TypeF64(Option<f64>),
    TypeMat4(Option<Mat4>),
    TypeI32(Option<i32>),
    TypeQuat(Option<Quat>),
    TypeString(Option<String>),
    TypeU32(Option<u32>),
    TypeU64(Option<u64>),
    TypeVec2(Option<Vec2>),
    TypeVec3(Option<Vec3>),
    TypeVec4(Option<Vec4>),
    TypeObjectRef(Option<ObjectRef>),
  }
  impl core::fmt::Debug for ComponentOptionType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentOptionType::TypeEmpty(e) => {
          f.debug_tuple("ComponentOptionType::TypeEmpty").field(e).finish()
        }
        ComponentOptionType::TypeBool(e) => {
          f.debug_tuple("ComponentOptionType::TypeBool").field(e).finish()
        }
        ComponentOptionType::TypeEntityId(e) => {
          f.debug_tuple("ComponentOptionType::TypeEntityId").field(e).finish()
        }
        ComponentOptionType::TypeF32(e) => {
          f.debug_tuple("ComponentOptionType::TypeF32").field(e).finish()
        }
        ComponentOptionType::TypeF64(e) => {
          f.debug_tuple("ComponentOptionType::TypeF64").field(e).finish()
        }
        ComponentOptionType::TypeMat4(e) => {
          f.debug_tuple("ComponentOptionType::TypeMat4").field(e).finish()
        }
        ComponentOptionType::TypeI32(e) => {
          f.debug_tuple("ComponentOptionType::TypeI32").field(e).finish()
        }
        ComponentOptionType::TypeQuat(e) => {
          f.debug_tuple("ComponentOptionType::TypeQuat").field(e).finish()
        }
        ComponentOptionType::TypeString(e) => {
          f.debug_tuple("ComponentOptionType::TypeString").field(e).finish()
        }
        ComponentOptionType::TypeU32(e) => {
          f.debug_tuple("ComponentOptionType::TypeU32").field(e).finish()
        }
        ComponentOptionType::TypeU64(e) => {
          f.debug_tuple("ComponentOptionType::TypeU64").field(e).finish()
        }
        ComponentOptionType::TypeVec2(e) => {
          f.debug_tuple("ComponentOptionType::TypeVec2").field(e).finish()
        }
        ComponentOptionType::TypeVec3(e) => {
          f.debug_tuple("ComponentOptionType::TypeVec3").field(e).finish()
        }
        ComponentOptionType::TypeVec4(e) => {
          f.debug_tuple("ComponentOptionType::TypeVec4").field(e).finish()
        }
        ComponentOptionType::TypeObjectRef(e) => {
          f.debug_tuple("ComponentOptionType::TypeObjectRef").field(e).finish()
        }
      }
    }
  }
  #[derive(Clone)]
  pub enum ComponentType{
    TypeEmpty(()),
    TypeBool(bool),
    TypeEntityId(EntityId),
    TypeF32(f32),
    TypeF64(f64),
    TypeMat4(Mat4),
    TypeI32(i32),
    TypeQuat(Quat),
    TypeString(String),
    TypeU32(u32),
    TypeU64(u64),
    TypeVec2(Vec2),
    TypeVec3(Vec3),
    TypeVec4(Vec4),
    TypeObjectRef(ObjectRef),
    TypeList(ComponentListType),
    TypeOption(ComponentOptionType),
  }
  impl core::fmt::Debug for ComponentType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      match self {
        ComponentType::TypeEmpty(e) => {
          f.debug_tuple("ComponentType::TypeEmpty").field(e).finish()
        }
        ComponentType::TypeBool(e) => {
          f.debug_tuple("ComponentType::TypeBool").field(e).finish()
        }
        ComponentType::TypeEntityId(e) => {
          f.debug_tuple("ComponentType::TypeEntityId").field(e).finish()
        }
        ComponentType::TypeF32(e) => {
          f.debug_tuple("ComponentType::TypeF32").field(e).finish()
        }
        ComponentType::TypeF64(e) => {
          f.debug_tuple("ComponentType::TypeF64").field(e).finish()
        }
        ComponentType::TypeMat4(e) => {
          f.debug_tuple("ComponentType::TypeMat4").field(e).finish()
        }
        ComponentType::TypeI32(e) => {
          f.debug_tuple("ComponentType::TypeI32").field(e).finish()
        }
        ComponentType::TypeQuat(e) => {
          f.debug_tuple("ComponentType::TypeQuat").field(e).finish()
        }
        ComponentType::TypeString(e) => {
          f.debug_tuple("ComponentType::TypeString").field(e).finish()
        }
        ComponentType::TypeU32(e) => {
          f.debug_tuple("ComponentType::TypeU32").field(e).finish()
        }
        ComponentType::TypeU64(e) => {
          f.debug_tuple("ComponentType::TypeU64").field(e).finish()
        }
        ComponentType::TypeVec2(e) => {
          f.debug_tuple("ComponentType::TypeVec2").field(e).finish()
        }
        ComponentType::TypeVec3(e) => {
          f.debug_tuple("ComponentType::TypeVec3").field(e).finish()
        }
        ComponentType::TypeVec4(e) => {
          f.debug_tuple("ComponentType::TypeVec4").field(e).finish()
        }
        ComponentType::TypeObjectRef(e) => {
          f.debug_tuple("ComponentType::TypeObjectRef").field(e).finish()
        }
        ComponentType::TypeList(e) => {
          f.debug_tuple("ComponentType::TypeList").field(e).finish()
        }
        ComponentType::TypeOption(e) => {
          f.debug_tuple("ComponentType::TypeOption").field(e).finish()
        }
      }
    }
  }
  #[export_name = "init: func() -> unit"]
  unsafe extern "C" fn __wit_bindgen_guest_init(){
    let result = <super::Guest as Guest>::init();
    let () = result;
  }
  #[export_name = "exec: func(ctx: record { time: float32 }, event-name: string, event-data: list<tuple<u32, variant { type-empty(tuple<>), type-bool(bool), type-entity-id(record { id0: u64, id1: u64 }), type-f32(float32), type-f64(float64), type-mat4(record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }), type-i32(s32), type-quat(record { x: float32, y: float32, z: float32, w: float32 }), type-string(string), type-u32(u32), type-u64(u64), type-vec2(record { x: float32, y: float32 }), type-vec3(record { x: float32, y: float32, z: float32 }), type-vec4(record { x: float32, y: float32, z: float32, w: float32 }), type-object-ref(record { id: string }), type-list(variant { type-empty(list<tuple<>>), type-bool(list<bool>), type-entity-id(list<record { id0: u64, id1: u64 }>), type-f32(list<float32>), type-f64(list<float64>), type-mat4(list<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(list<s32>), type-quat(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(list<string>), type-u32(list<u32>), type-u64(list<u64>), type-vec2(list<record { x: float32, y: float32 }>), type-vec3(list<record { x: float32, y: float32, z: float32 }>), type-vec4(list<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(list<record { id: string }>) }), type-option(variant { type-empty(option<tuple<>>), type-bool(option<bool>), type-entity-id(option<record { id0: u64, id1: u64 }>), type-f32(option<float32>), type-f64(option<float64>), type-mat4(option<record { x: record { x: float32, y: float32, z: float32, w: float32 }, y: record { x: float32, y: float32, z: float32, w: float32 }, z: record { x: float32, y: float32, z: float32, w: float32 }, w: record { x: float32, y: float32, z: float32, w: float32 } }>), type-i32(option<s32>), type-quat(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-string(option<string>), type-u32(option<u32>), type-u64(option<u64>), type-vec2(option<record { x: float32, y: float32 }>), type-vec3(option<record { x: float32, y: float32, z: float32 }>), type-vec4(option<record { x: float32, y: float32, z: float32, w: float32 }>), type-object-ref(option<record { id: string }>) }) }>>) -> unit"]
  unsafe extern "C" fn __wit_bindgen_guest_exec(arg0: f32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, ){
    let len0 = arg2 as usize;
    let base22 = arg3;
    let len22 = arg4;
    let mut result22 = Vec::with_capacity(len22 as usize);
    for i in 0..len22 {
      let base = base22 + i *88;
      result22.push((*((base + 0) as *const i32) as u32, match i32::from(*((base + 8) as *const u8)) {
        0 => ComponentType::TypeEmpty(()),
        1 => ComponentType::TypeBool(match i32::from(*((base + 16) as *const u8)) {
          0 => false,
          1 => true,
          _ => panic!("invalid bool discriminant"),
        }),
        2 => ComponentType::TypeEntityId(EntityId{id0:*((base + 16) as *const i64) as u64, id1:*((base + 24) as *const i64) as u64, }),
        3 => ComponentType::TypeF32(*((base + 16) as *const f32)),
        4 => ComponentType::TypeF64(*((base + 16) as *const f64)),
        5 => ComponentType::TypeMat4(Mat4{x:Vec4{x:*((base + 16) as *const f32), y:*((base + 20) as *const f32), z:*((base + 24) as *const f32), w:*((base + 28) as *const f32), }, y:Vec4{x:*((base + 32) as *const f32), y:*((base + 36) as *const f32), z:*((base + 40) as *const f32), w:*((base + 44) as *const f32), }, z:Vec4{x:*((base + 48) as *const f32), y:*((base + 52) as *const f32), z:*((base + 56) as *const f32), w:*((base + 60) as *const f32), }, w:Vec4{x:*((base + 64) as *const f32), y:*((base + 68) as *const f32), z:*((base + 72) as *const f32), w:*((base + 76) as *const f32), }, }),
        6 => ComponentType::TypeI32(*((base + 16) as *const i32)),
        7 => ComponentType::TypeQuat(Quat{x:*((base + 16) as *const f32), y:*((base + 20) as *const f32), z:*((base + 24) as *const f32), w:*((base + 28) as *const f32), }),
        8 => ComponentType::TypeString({
          let len1 = *((base + 20) as *const i32) as usize;
          
          String::from_utf8(Vec::from_raw_parts(*((base + 16) as *const i32) as *mut _, len1, len1)).unwrap()
        }),
        9 => ComponentType::TypeU32(*((base + 16) as *const i32) as u32),
        10 => ComponentType::TypeU64(*((base + 16) as *const i64) as u64),
        11 => ComponentType::TypeVec2(Vec2{x:*((base + 16) as *const f32), y:*((base + 20) as *const f32), }),
        12 => ComponentType::TypeVec3(Vec3{x:*((base + 16) as *const f32), y:*((base + 20) as *const f32), z:*((base + 24) as *const f32), }),
        13 => ComponentType::TypeVec4(Vec4{x:*((base + 16) as *const f32), y:*((base + 20) as *const f32), z:*((base + 24) as *const f32), w:*((base + 28) as *const f32), }),
        14 => ComponentType::TypeObjectRef({
          let len2 = *((base + 20) as *const i32) as usize;
          
          ObjectRef{id:String::from_utf8(Vec::from_raw_parts(*((base + 16) as *const i32) as *mut _, len2, len2)).unwrap(), }
        }),
        15 => ComponentType::TypeList(match i32::from(*((base + 16) as *const u8)) {
          0 => ComponentListType::TypeEmpty({
            let len3 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len3, len3)
          }),
          1 => ComponentListType::TypeBool({
            let base4 = *((base + 20) as *const i32);
            let len4 = *((base + 24) as *const i32);
            let mut result4 = Vec::with_capacity(len4 as usize);
            for i in 0..len4 {
              let base = base4 + i *1;
              result4.push(match i32::from(*((base + 0) as *const u8)) {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
              });
            }
            if len4 != 0 {
              std::alloc::dealloc(base4 as *mut _, std::alloc::Layout::from_size_align_unchecked((len4 as usize) * 1, 1));
            }
            
            result4
          }),
          2 => ComponentListType::TypeEntityId({
            let len5 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len5, len5)
          }),
          3 => ComponentListType::TypeF32({
            let len6 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len6, len6)
          }),
          4 => ComponentListType::TypeF64({
            let len7 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len7, len7)
          }),
          5 => ComponentListType::TypeMat4({
            let len8 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len8, len8)
          }),
          6 => ComponentListType::TypeI32({
            let len9 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len9, len9)
          }),
          7 => ComponentListType::TypeQuat({
            let len10 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len10, len10)
          }),
          8 => ComponentListType::TypeString({
            let base12 = *((base + 20) as *const i32);
            let len12 = *((base + 24) as *const i32);
            let mut result12 = Vec::with_capacity(len12 as usize);
            for i in 0..len12 {
              let base = base12 + i *8;
              result12.push({
                let len11 = *((base + 4) as *const i32) as usize;
                
                String::from_utf8(Vec::from_raw_parts(*((base + 0) as *const i32) as *mut _, len11, len11)).unwrap()
              });
            }
            if len12 != 0 {
              std::alloc::dealloc(base12 as *mut _, std::alloc::Layout::from_size_align_unchecked((len12 as usize) * 8, 4));
            }
            
            result12
          }),
          9 => ComponentListType::TypeU32({
            let len13 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len13, len13)
          }),
          10 => ComponentListType::TypeU64({
            let len14 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len14, len14)
          }),
          11 => ComponentListType::TypeVec2({
            let len15 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len15, len15)
          }),
          12 => ComponentListType::TypeVec3({
            let len16 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len16, len16)
          }),
          13 => ComponentListType::TypeVec4({
            let len17 = *((base + 24) as *const i32) as usize;
            
            Vec::from_raw_parts(*((base + 20) as *const i32) as *mut _, len17, len17)
          }),
          14 => ComponentListType::TypeObjectRef({
            let base19 = *((base + 20) as *const i32);
            let len19 = *((base + 24) as *const i32);
            let mut result19 = Vec::with_capacity(len19 as usize);
            for i in 0..len19 {
              let base = base19 + i *8;
              result19.push({
                let len18 = *((base + 4) as *const i32) as usize;
                
                ObjectRef{id:String::from_utf8(Vec::from_raw_parts(*((base + 0) as *const i32) as *mut _, len18, len18)).unwrap(), }
              });
            }
            if len19 != 0 {
              std::alloc::dealloc(base19 as *mut _, std::alloc::Layout::from_size_align_unchecked((len19 as usize) * 8, 4));
            }
            
            result19
          }),
          _ => panic!("invalid enum discriminant"),
        }),
        16 => ComponentType::TypeOption(match i32::from(*((base + 16) as *const u8)) {
          0 => ComponentOptionType::TypeEmpty(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(()),
            _ => panic!("invalid enum discriminant"),
          }),
          1 => ComponentOptionType::TypeBool(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(match i32::from(*((base + 25) as *const u8)) {
              0 => false,
              1 => true,
              _ => panic!("invalid bool discriminant"),
            }),
            _ => panic!("invalid enum discriminant"),
          }),
          2 => ComponentOptionType::TypeEntityId(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(EntityId{id0:*((base + 32) as *const i64) as u64, id1:*((base + 40) as *const i64) as u64, }),
            _ => panic!("invalid enum discriminant"),
          }),
          3 => ComponentOptionType::TypeF32(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(*((base + 28) as *const f32)),
            _ => panic!("invalid enum discriminant"),
          }),
          4 => ComponentOptionType::TypeF64(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(*((base + 32) as *const f64)),
            _ => panic!("invalid enum discriminant"),
          }),
          5 => ComponentOptionType::TypeMat4(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(Mat4{x:Vec4{x:*((base + 28) as *const f32), y:*((base + 32) as *const f32), z:*((base + 36) as *const f32), w:*((base + 40) as *const f32), }, y:Vec4{x:*((base + 44) as *const f32), y:*((base + 48) as *const f32), z:*((base + 52) as *const f32), w:*((base + 56) as *const f32), }, z:Vec4{x:*((base + 60) as *const f32), y:*((base + 64) as *const f32), z:*((base + 68) as *const f32), w:*((base + 72) as *const f32), }, w:Vec4{x:*((base + 76) as *const f32), y:*((base + 80) as *const f32), z:*((base + 84) as *const f32), w:*((base + 88) as *const f32), }, }),
            _ => panic!("invalid enum discriminant"),
          }),
          6 => ComponentOptionType::TypeI32(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(*((base + 28) as *const i32)),
            _ => panic!("invalid enum discriminant"),
          }),
          7 => ComponentOptionType::TypeQuat(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(Quat{x:*((base + 28) as *const f32), y:*((base + 32) as *const f32), z:*((base + 36) as *const f32), w:*((base + 40) as *const f32), }),
            _ => panic!("invalid enum discriminant"),
          }),
          8 => ComponentOptionType::TypeString(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some({
              let len20 = *((base + 32) as *const i32) as usize;
              
              String::from_utf8(Vec::from_raw_parts(*((base + 28) as *const i32) as *mut _, len20, len20)).unwrap()
            }),
            _ => panic!("invalid enum discriminant"),
          }),
          9 => ComponentOptionType::TypeU32(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(*((base + 28) as *const i32) as u32),
            _ => panic!("invalid enum discriminant"),
          }),
          10 => ComponentOptionType::TypeU64(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(*((base + 32) as *const i64) as u64),
            _ => panic!("invalid enum discriminant"),
          }),
          11 => ComponentOptionType::TypeVec2(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(Vec2{x:*((base + 28) as *const f32), y:*((base + 32) as *const f32), }),
            _ => panic!("invalid enum discriminant"),
          }),
          12 => ComponentOptionType::TypeVec3(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(Vec3{x:*((base + 28) as *const f32), y:*((base + 32) as *const f32), z:*((base + 36) as *const f32), }),
            _ => panic!("invalid enum discriminant"),
          }),
          13 => ComponentOptionType::TypeVec4(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some(Vec4{x:*((base + 28) as *const f32), y:*((base + 32) as *const f32), z:*((base + 36) as *const f32), w:*((base + 40) as *const f32), }),
            _ => panic!("invalid enum discriminant"),
          }),
          14 => ComponentOptionType::TypeObjectRef(match i32::from(*((base + 24) as *const u8)) {
            0 => None,
            1 => Some({
              let len21 = *((base + 32) as *const i32) as usize;
              
              ObjectRef{id:String::from_utf8(Vec::from_raw_parts(*((base + 28) as *const i32) as *mut _, len21, len21)).unwrap(), }
            }),
            _ => panic!("invalid enum discriminant"),
          }),
          _ => panic!("invalid enum discriminant"),
        }),
        _ => panic!("invalid enum discriminant"),
      }));
    }
    if len22 != 0 {
      std::alloc::dealloc(base22 as *mut _, std::alloc::Layout::from_size_align_unchecked((len22 as usize) * 88, 8));
    }
    let result = <super::Guest as Guest>::exec(RunContext{time:arg0, }, String::from_utf8(Vec::from_raw_parts(arg1 as *mut _, len0, len0)).unwrap(), result22);
    let () = result;
  }
  
  #[repr(align(8))]
  struct __GuestRetArea([u8; 104]);
  static mut __GUEST_RET_AREA: __GuestRetArea = __GuestRetArea([0; 104]);
  pub trait Guest {
    fn init() -> ();
    fn exec(ctx: RunContext,event_name: String,event_data: Vec<(u32,ComponentType,)>,) -> ();
  }
}
#[allow(missing_docs)] pub const INTERFACE_VERSION: u32 = 10;