use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct RenderSettings {
    #[serde(default)]
    pub(crate) resolution: Resolution,
    #[serde(default)]
    pub(crate) vsync: Vsync,
    #[serde(default)]
    /// If `None` fall back to the platform default
    pub render_mode: Option<RenderMode>,
    #[serde(default)]
    pub software_culling: bool,
}

impl RenderSettings {
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution.0
    }

    pub fn vsync(&self) -> bool {
        self.vsync.0
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderMode {
    MultiIndirect,
    Indirect,
    Direct,
}

impl RenderMode {
    pub const fn instrinsic_render_mode() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "windows", target_os = "linux"))] {
                Self::MultiIndirect
            } else if #[cfg(target_os = "macos")] {
                Self::Indirect
            } else if #[cfg(target_os = "unknown")] {
                // Chrome uses DirectX12 which does not correctly implement `INDIRECT_FIRST_INSTANCE` which causes the wrong instance index to be passed to indirect draws.
                // This in turn causes a dispatch of X vertices to unconditionally use instance/entity 0, and therefore the wrong mesh for the dispatch count.
                // Not good
               Self::Direct
            } else {
                Self::Direct
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Resolution(pub (u32, u32));
impl Default for Resolution {
    fn default() -> Self {
        Self((800, 600))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vsync(pub bool);
impl Default for Vsync {
    fn default() -> Self {
        Self(true)
    }
}
