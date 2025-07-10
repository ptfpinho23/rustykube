use std::fs;
use std::process;
use crate::utils;
use crate::lint_rules::{LintRule, LivenessProbeRule, MissingLabelsRule, ReadinessProbeRule, ResourceLimitsRule, RunAsNonRootRule, ReadOnlyRootFilesystemRule, LatestImageTagRule};

pub fn run_lint(path: &str, json: bool, strict: bool, rules_filter: Option<&str>, verbose: bool) {
    if verbose {
        println!("🔍 Starting lint analysis for: {}", path);
    }

    let contents = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{}': {}", path, e);
            if strict {
                process::exit(1);
            }
            return;
        }
    };

    let docs = match utils::parse_yaml(&contents) {
        Ok(documents) => documents,
        Err(e) => {
            eprintln!("❌ Error parsing YAML: {}", e);
            if strict {
                process::exit(1);
            }
            return;
        }
    };

    // Initialize all available rules
    let all_rules: Vec<(&str, Box<dyn LintRule>)> = vec![
        ("missing-labels", Box::new(MissingLabelsRule)),
        ("resource-limits", Box::new(ResourceLimitsRule)),
        ("liveness-probe", Box::new(LivenessProbeRule)),
        ("readiness-probe", Box::new(ReadinessProbeRule)),
        ("run-as-non-root", Box::new(RunAsNonRootRule)),
        ("read-only-root-filesystem", Box::new(ReadOnlyRootFilesystemRule)),
        ("latest-image-tag", Box::new(LatestImageTagRule))
    ];

    // Filter rules based on user input
    let active_rules: Vec<&(&str, Box<dyn LintRule>)> = if let Some(filter) = rules_filter {
        let rule_names: Vec<&str> = filter.split(',').map(|s| s.trim()).collect();
        all_rules.iter().filter(|(name, _)| rule_names.contains(name)).collect()
    } else {
        all_rules.iter().collect()
    };

    if verbose {
        println!("📋 Running {} lint rules", active_rules.len());
        for (name, _) in &active_rules {
            println!("  - {}", name);
        }
        println!();
    }

    let mut results = vec![];
    let mut total_issues = 0;

    if !json {
        println!("🔍 Linting Results\n{}", "=".repeat(50));
    }

    for (_i, doc) in docs.iter().enumerate() {
        let resource_kind = doc
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
    
        let resource_name = doc
            .get("metadata")
            .and_then(|metadata| metadata.get("name"))
            .and_then(|name| name.as_str())
            .unwrap_or("Unnamed");
        
        if !json {
            println!("📄 Resource: {} ({})", resource_name, resource_kind);
        }
    
        let mut resource_issues = vec![];

        for (rule_name, rule) in &active_rules {
            if let Some(message) = rule.check(doc) {
                total_issues += 1;
                resource_issues.push(format!("[{}] {}", rule_name, message));
            }
        }

        if !json {
            if resource_issues.is_empty() {
                println!("  ✅ No issues found\n");
            } else {
                for issue in &resource_issues {
                    println!("  ❌ {}", issue);
                }
                println!();
            }
        }

        results.push((format!("{} ({})", resource_name, resource_kind), resource_issues));
    }

    // Output results
    if json {
        let json_output: Vec<_> = results
            .into_iter()
            .map(|(resource, issues)| {
                serde_json::json!({
                    "resource": resource,
                    "issues": issues,
                    "issue_count": issues.len()
                })
            })
            .collect();

        let summary = serde_json::json!({
            "summary": {
                "total_resources": docs.len(),
                "total_issues": total_issues,
                "rules_checked": active_rules.len()
            },
            "results": json_output
        });

        println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    } else {
        // Print summary
        println!("{}", "=".repeat(50));
        println!("📊 Summary");
        println!("  Resources analyzed: {}", docs.len());
        println!("  Total issues found: {}", total_issues);
        
        if total_issues == 0 {
            println!("  🎉 All resources passed linting!");
        } else {
            println!("  ⚠️  {} issue(s) need attention", total_issues);
        }
    }

    // Exit with error code in strict mode if issues found
    if strict && total_issues > 0 {
        if verbose {
            println!("\n🚨 Exiting with error code due to --strict mode");
        }
        process::exit(1);
    }
}
