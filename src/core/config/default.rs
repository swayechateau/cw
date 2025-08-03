use serde::{Deserialize, Serialize};

use crate::constants::DEFAULT_BRANCH;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CWDefault {
    /// Optional default target branch (e.g., "main", "develop")
    /// Defaults to "main" if not set.
    #[serde(default)]
    pub branch: Option<String>,
}

impl Default for CWDefault {
    fn default() -> Self {
        CWDefault {
            branch: Some(DEFAULT_BRANCH.to_string()),
        }
    }
}
