use std::fs;
use crate::utils;
use crate::lint_rules::{LintRule, LivenessProbeRule, MissingLabelsRule, ReadinessProbeRule, ResourceLimitsRule, RunAsNonRootRule, ReadOnlyRootFilesystemRule, LatestImageTagRule};

pub fn run_lint(path: &str, json: bool) {
    let contents = fs::read_to_string(path).expect("Failed to read file");
    let docs = utils::parse_yaml(&contents);

    let rules: Vec<Box<dyn LintRule>> = vec![
        Box::new(MissingLabelsRule),
        Box::new(ResourceLimitsRule),
        Box::new(LivenessProbeRule),
        Box::new(ReadinessProbeRule),
        Box::new(RunAsNonRootRule),
        Box::new(ReadOnlyRootFilesystemRule),
        Box::new(LatestImageTagRule)
    ];

    let mut results = vec![];
    let mut total_issues = 0;

    println!("\n--- Linting Results ---\n");

    for (i, doc) in docs.iter().enumerate() {
        let mut document_issues = vec![];
        println!("📄 Resource {}:", i + 1);

        for rule in &rules {
            if let Some(message) = rule.check(doc) {
                total_issues += 1;
                document_issues.push(message);
            }
        }

        if document_issues.is_empty() {
            println!("  ✅ No issues found.\n");
        } else {
            for issue in &document_issues {
                println!("  ❌ {}", issue);
            }
            println!();
        }

        results.push((format!("Resource {}", i + 1), document_issues));
    }

    // Final Summary
    println!("--- Summary ---");
    if total_issues == 0 {
        println!("🎉 All Resources passed linting with no issues!\n");
    } else {
        println!(
            "⚠️  Linting completed with {} issue(s) across {} resource(s).\n",
            total_issues,
            docs.len()
        );
    }

    if json {
        let json_output: Vec<_> = results
            .into_iter()
            .map(|(doc, issues)| {
                serde_json::json!({
                    "document": doc,
                    "issues": issues,
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
    }
}
