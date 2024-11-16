use serde_yaml::Value;

use super::LintRule;

pub struct RunAsNonRootRule;

impl LintRule for RunAsNonRootRule {
    fn check(&self, doc: &Value) -> Option<String> {
        let containers = doc
            .get("spec")?
            .get("template")?
            .get("spec")?
            .get("containers")?
            .as_sequence()?;

        for container in containers {
            if let Some(security_context) = container.get("securityContext") {
                if security_context.get("runAsNonRoot").is_none() {
                    return Some("Container does not have runAsNonRoot set.".to_string());
                }
            }
        }
        None
    }
}

pub struct ReadOnlyRootFilesystemRule;

impl LintRule for ReadOnlyRootFilesystemRule {
    fn check(&self, doc: &Value) -> Option<String> {
        let containers = doc
            .get("spec")?
            .get("template")?
            .get("spec")?
            .get("containers")?
            .as_sequence()?;

        for container in containers {
            if let Some(security_context) = container.get("securityContext") {
                if security_context.get("readOnlyRootFilesystem").is_none() {
                    return Some("Container does not have readOnlyRootFilesystem set.".to_string());
                }
            }
        }
        None
    }
}
