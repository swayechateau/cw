use serde::Serialize;

use crate::{
    config::Patchable,
    constants::{DEFAULT_FEAT, DEFAULT_FIX},
    strings,
};

#[derive(Debug, Serialize, serde::Deserialize, Clone)]
pub struct CWVersioning {
    pub minor: Vec<String>,
    pub patch: Vec<String>,
}

impl Default for CWVersioning {
    fn default() -> Self {
        CWVersioning {
            minor: strings![DEFAULT_FEAT],
            patch: strings![DEFAULT_FIX],
        }
    }
}

impl Patchable for CWVersioning {
    fn patch(&mut self) {
        let feat = DEFAULT_FEAT.to_string();
        let fix = DEFAULT_FIX.to_string();

        if self.minor.is_empty() {
            self.minor = strings![DEFAULT_FEAT];
        } else if !self.minor.contains(&feat) {
            self.minor.push(feat);
        }

        if self.patch.is_empty() {
            self.patch = strings![DEFAULT_FIX];
        } else if !self.patch.contains(&fix) {
            self.patch.push(fix);
        }
    }
}
