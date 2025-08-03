use std::collections::HashSet;

use crate::{config::Patchable, constants::DEFAULT_BREAKING};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Breaking {
    pub keywords: Vec<String>,
}

impl Default for Breaking {
    fn default() -> Self {
        Breaking {
            keywords: DEFAULT_BREAKING.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Patchable for Breaking {
    fn patch(&mut self) {
        let existing: HashSet<_> = self.keywords.iter().cloned().collect();
        for &kw in DEFAULT_BREAKING {
            if !existing.contains(kw) {
                self.keywords.push(kw.to_string());
            }
        }
    }
}
