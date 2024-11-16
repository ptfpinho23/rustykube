use serde_yaml::Value;

use super::LintRule;

pub struct ResourceLimitsRule;

impl LintRule for ResourceLimitsRule {
    fn check(&self, doc: &Value) -> Option<String> {
        let containers = doc
            .get("spec")?
            .get("template")?
            .get("spec")?
            .get("containers")?
            .as_sequence()?;

        for container in containers {
            if container.get("resources").and_then(|r| r.get("limits")).is_none() {
                return Some("Container is missing resource limits.".to_string());
            }
        }
        None
    }
}
