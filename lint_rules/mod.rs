pub mod missing_labels;
pub mod resource_limits;

pub use missing_labels::MissingLabelsRule;
pub use resource_limits::ResourceLimitsRule;

pub trait LintRule {
    fn check(&self, doc: &serde_yaml::Value) -> Option<String>;
}
