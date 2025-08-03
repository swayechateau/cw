pub const DEFAULT_BRANCH: &str = "main";
pub const DEFAULT_FEAT: &str = "feat";
pub const DEFAULT_FIX: &str = "fix";
pub const DEFAULT_FEAT_MESSAGE: &str = "A new feature";
pub const DEFAULT_FIX_MESSAGE: &str = "A bug fix";
pub const DEFAULT_BREAKING: &[&str] = &["BREAKING CHANGE"];
pub const DEFAULT_SCOPES: &str = "any";
pub const COMMON_SCOPES: &[&str] = &["api", "ui", "db", "core", "cli", "config", "deps"];

pub const COMMIT_FIX_FILE_NAME: &str = ".cw-fix.json";
