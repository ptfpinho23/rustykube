use serde_yaml::Value;
use super::LintRule;

pub struct LatestImageTagRule;

impl LintRule for LatestImageTagRule {
    fn check(&self, doc: &serde_yaml::Value) -> Option<String> {
        let containers = doc.get("spec")?
        .get("template")?.get("spec")?
        .get("containers")?
        .as_sequence()?;

        for container in containers {
            if let Some(image) = container.get("image").and_then(Value::as_str) {
                if image.ends_with(":latest") {
                    return Some("Container uses a 'latest' image tag. Which should be avoided. ".to_string());
                }
        }
    }
    return None
}
}