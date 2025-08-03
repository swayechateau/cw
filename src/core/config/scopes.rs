use std::collections::BTreeMap;

use crate::constants::{COMMON_SCOPES, DEFAULT_FEAT, DEFAULT_FIX, DEFAULT_SCOPES};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CommitScopes {
    // optional in config
    pub any: Vec<String>,
    /// optional in config
    pub feat: Vec<String>,
    // optional in config
    pub fix: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Vec<String>>,
}

impl Default for CommitScopes {
    fn default() -> Self {
        CommitScopes {
            any: COMMON_SCOPES.iter().map(|s| s.to_string()).collect(),
            feat: vec![],
            fix: vec![],
            extra: BTreeMap::new(),
        }
    }
}

impl CommitScopes {
    pub fn empty() -> Self {
        CommitScopes {
            any: vec![],
            feat: vec![],
            fix: vec![],
            extra: BTreeMap::new(),
        }
    }

    pub fn all(&self) -> BTreeMap<String, Vec<String>> {
        let mut all = self.extra.clone();
        all.insert(DEFAULT_SCOPES.to_string(), self.any.clone());
        all.insert(DEFAULT_FEAT.to_string(), self.feat.clone());
        all.insert(DEFAULT_FIX.to_string(), self.fix.clone());
        all
    }

    pub fn valid_scopes(&self, commit_type: &str) -> Vec<String> {
        let all = self.all();
        all.get(commit_type)
            .or_else(|| all.get(DEFAULT_SCOPES))
            .cloned()
            .unwrap_or_default()
    }
}
