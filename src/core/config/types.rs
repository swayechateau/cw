use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    config::Patchable,
    constants::{DEFAULT_FEAT, DEFAULT_FEAT_MESSAGE, DEFAULT_FIX, DEFAULT_FIX_MESSAGE},
    utils::is_blank,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitTypes {
    pub feat: String,
    pub fix: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, String>,
}

impl Default for CommitTypes {
    fn default() -> Self {
        CommitTypes {
            feat: DEFAULT_FEAT_MESSAGE.to_string(),
            fix: DEFAULT_FIX_MESSAGE.to_string(),
            extra: BTreeMap::new(),
        }
    }
}

impl Patchable for CommitTypes {
    fn patch(&mut self) {
        if is_blank(&self.feat) {
            self.feat = DEFAULT_FEAT_MESSAGE.to_string();
        }
        if is_blank(&self.fix) {
            self.fix = DEFAULT_FIX_MESSAGE.to_string();
        }
    }
}

impl CommitTypes {
    pub fn all(&self) -> std::collections::BTreeMap<String, String> {
        let mut all = self.extra.clone();
        all.insert(DEFAULT_FEAT.to_string(), self.feat.clone());
        all.insert(DEFAULT_FIX.to_string(), self.fix.clone());
        all
    }

    pub fn allowed(&self) -> Vec<String> {
        self.all().keys().cloned().collect()
    }
}
