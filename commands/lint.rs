use serde_yaml::Value;
use std::fs;

use crate::utils;
use crate::lint_rules::{LintRule, MissingLabelsRule, ResourceLimitsRule};

pub fn run_lint(path: &str, json: bool) {
    let contents = fs::read_to_string(path).expect("Failed to read file");
    let docs = utils::parse_yaml(&contents);
    

    let rules: Vec<Box<dyn LintRule>> = vec![
        Box::new(MissingLabelsRule),
        Box::new(ResourceLimitsRule),
    ];

    let mut results = vec![];

    for (i, doc) in docs.iter().enumerate() {
        for rule in &rules {
            if let Some(message) = rule.check(doc) {
                results.push(format!("Document {}: {}", i + 1, message));
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        for result in results {
            println!("{}", result);
        }
    }
}
