use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    config::Patchable,
    constants::{DEFAULT_FEAT, DEFAULT_FIX},
    strings,
    utils::capitalize_first_letter,
};

#[derive(Debug, Serialize, serde::Deserialize, Clone)]
pub struct Changelog {
    pub sections: ChangelogSections,
}

impl Default for Changelog {
    fn default() -> Self {
        Changelog {
            sections: ChangelogSections::default(),
        }
    }
}

impl Patchable for Changelog {
    fn patch(&mut self) {
        let feat = DEFAULT_FEAT.to_string();
        let fix = DEFAULT_FIX.to_string();

        if self.sections.added.is_empty() {
            self.sections.added = strings![DEFAULT_FEAT];
        } else if !self.sections.added.contains(&feat) {
            self.sections.added.push(feat);
        }

        if self.sections.fixed.is_empty() {
            self.sections.fixed = strings![DEFAULT_FIX];
        } else if !self.sections.fixed.contains(&fix) {
            self.sections.fixed.push(fix);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangelogSections {
    pub added: Vec<String>,
    pub fixed: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Vec<String>>,
}

impl Default for ChangelogSections {
    fn default() -> Self {
        ChangelogSections {
            added: strings![DEFAULT_FEAT],
            fixed: strings![DEFAULT_FIX],
            extra: BTreeMap::new(),
        }
    }
}

impl ChangelogSections {
    pub fn build_type_to_section_map(self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        let mut seen = HashSet::new();
        macro_rules! insert_mappings {
            ($section_name:literal, $types:expr) => {
                for t in $types {
                    if seen.insert(t) {
                        // only insert if not seen
                        map.insert(t.clone(), $section_name.to_string());
                    }
                }
            };
        }

        insert_mappings!("Added", &self.added);
        // ensure fix is in the fixed section
        insert_mappings!("Fixed", &self.fixed);

        for (section, types) in &self.extra {
            // remove duplicates
            for t in types {
                let key = capitalize_first_letter(t);
                map.insert(key, section.clone());
            }
        }

        map
    }

    pub fn group_commits_by_section(
        &self,
        grouped_commits: &BTreeMap<String, Vec<(String, Option<String>, String)>>,
    ) -> BTreeMap<String, Vec<(Option<String>, String, String)>> {
        let mut section_map: BTreeMap<String, Vec<(Option<String>, String, String)>> =
            BTreeMap::new();
        let type_to_section = self.clone().build_type_to_section_map();

        for (typ, commits) in grouped_commits {
            let section = type_to_section
                .get(typ.as_str())
                .cloned()
                .unwrap_or_else(|| "Uncategorized".to_string());

            for (hash, scope, summary) in commits {
                section_map.entry(section.clone()).or_default().push((
                    scope.clone(),
                    summary.clone(),
                    hash.clone(),
                ));
            }
        }

        section_map
    }
}
