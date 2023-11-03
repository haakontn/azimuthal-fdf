use super::{AFDFSimplified, HeatReleaseRate};
use crate::azimuthal_mode::SystemMode;
use crate::{Quaternion, Settings};
use serde::{Deserialize, Serialize};

/// Conventional Flame Describing Function (FDF).
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct ConventionalFDF {
    #[serde(skip)]
    model: AFDFSimplified,
}

impl ConventionalFDF {
    pub fn new() -> Self {
        let model = AFDFSimplified::new(1.0);

        Self { model }
    }
}

impl Default for ConventionalFDF {
    fn default() -> Self {
        Self::new()
    }
}

impl HeatReleaseRate for ConventionalFDF {
    fn integral(&self, acoustic_mode: &SystemMode, setup: &Settings) -> Quaternion {
        self.model.integral(acoustic_mode, setup)
    }

    fn mode(&self, acoustic_mode: &SystemMode) -> SystemMode {
        self.model.mode(acoustic_mode)
    }
}
