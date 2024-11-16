use serde_yaml::Value;

use super::LintRule;

pub struct MissingLabelsRule;

impl LintRule for MissingLabelsRule {
    fn check(&self, doc: &Value) -> Option<String> {
        if let Some(metadata) = doc.get("metadata") {
            if metadata.get("labels").is_none() {
                return Some("Resource is missing labels.".to_string());
            }
        }
        None
    }
}
