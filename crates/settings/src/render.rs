use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct RenderSettings {
    #[serde(default)]
    resolution: Resolution,
    #[serde(default)]
    vsync: Vsync,
    #[serde(default)]
    pub render_mode: RenderMode,
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

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderMode {
    #[default]
    MultiIndirect,
    Indirect,
    Direct,
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
