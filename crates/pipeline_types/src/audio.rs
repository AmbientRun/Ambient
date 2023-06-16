use serde::{Deserialize, Serialize};

use crate::is_false;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioPipeline {
    /// Whether or not the audio should be converted to Ogg Vorbis.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub convert: bool,
}
