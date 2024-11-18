pub mod missing_labels;
pub mod resource_limits;
pub mod security; 
pub mod health_checks;
pub mod image_tagging;

pub use missing_labels::MissingLabelsRule;
pub use resource_limits::ResourceLimitsRule;
pub use security::{RunAsNonRootRule, ReadOnlyRootFilesystemRule};
pub use health_checks::{LivenessProbeRule, ReadinessProbeRule};
pub use image_tagging::LatestImageTagRule;

pub trait LintRule {
    fn check(&self, doc: &serde_yaml::Value) -> Option<String>;
}
