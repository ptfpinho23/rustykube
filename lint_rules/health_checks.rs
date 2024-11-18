use serde_yaml::Value;

use super::LintRule;

pub struct LivenessProbeRule;

impl LintRule for LivenessProbeRule {
    fn check(&self, doc: &Value) -> Option<String> {
        let containers = doc
            .get("spec")?
            .get("template")?
            .get("spec")?
            .get("containers")?
            .as_sequence()?;

        for container in containers {
            if container.get("livenessProbe").is_none() {
                return Some("Container is missing livenessProbe.".to_string());
            }
        }
        None
    }
}

pub struct ReadinessProbeRule;

impl LintRule for ReadinessProbeRule {
    fn check(&self, doc: &Value) -> Option<String> {
        let containers = doc
            .get("spec")?
            .get("template")?
            .get("spec")?
            .get("containers")?
            .as_sequence()?;

        for container in containers {
            if container.get("readinessProbe").is_none() {
                return Some("Container is missing readinessProbe.".to_string())
            }
        }
        return None
    }
}