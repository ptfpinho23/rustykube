use std::fs;
use std::process;
use crate::utils;

pub fn run_fix(path: &str, in_place: bool, dry_run: bool, verbose: bool) {
    if verbose {
        println!("🔧 Starting fix for: {}", path);
        if dry_run {
            println!("🏃 Running in dry-run mode - no changes will be made");
        }
    }

    let files = match utils::find_kubernetes_files(path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error finding files: {}", e);
            process::exit(1);
        }
    };

    let mut total_fixes = 0;
    let mut fix_results = Vec::new();

    for file_path in files {
        if verbose {
            println!("📄 Processing: {}", file_path);
        }

        let contents = match utils::read_file_contents(&file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading {}: {}", file_path, e);
                continue;
            }
        };

        let docs = match utils::parse_yaml(&contents) {
            Ok(documents) => documents,
            Err(e) => {
                eprintln!("❌ Error parsing {}: {}", file_path, e);
                continue;
            }
        };

        let mut fixed_docs = Vec::new();
        let mut file_fixes = Vec::new();

        for doc in docs {
            let (fixed_doc, fixes) = fix_resource(doc);
            fixed_docs.push(fixed_doc);
            file_fixes.extend(fixes);
        }

        total_fixes += file_fixes.len();

        if !file_fixes.is_empty() {
            // Generate fixed YAML content
            let mut fixed_content = String::new();
            for (i, doc) in fixed_docs.iter().enumerate() {
                if i > 0 {
                    fixed_content.push_str("---\n");
                }
                fixed_content.push_str(&serde_yaml::to_string(doc).unwrap());
            }

            let output_path = if in_place {
                file_path.clone()
            } else {
                format!("{}.fixed.yaml", file_path.strip_suffix(".yaml").unwrap_or(&file_path))
            };

            if !dry_run {
                if let Err(e) = fs::write(&output_path, &fixed_content) {
                    eprintln!("❌ Error writing fixed file {}: {}", output_path, e);
                    continue;
                }
            }

            println!("🔧 Fixed {}: {} issue(s)", file_path, file_fixes.len());
            for fix in &file_fixes {
                println!("   ✅ {}", fix);
            }

            if !dry_run && !in_place {
                println!("   📝 Written to: {}", output_path);
            }
        } else if verbose {
            println!("✅ {} - No fixes needed", file_path);
        }

        let is_fixed = !file_fixes.is_empty();
        fix_results.push(FixResult {
            file: file_path,
            fixes: file_fixes,
            fixed: is_fixed,
        });
    }

    // Summary
    println!("\n🔧 Fix Summary");
    println!("{}", "=".repeat(50));
    println!("  Files processed: {}", fix_results.len());
    println!("  Total fixes applied: {}", total_fixes);
    
    if dry_run && total_fixes > 0 {
        println!("  🏃 Dry run completed - {} fixes would be applied", total_fixes);
    } else if total_fixes > 0 {
        println!("  🎉 Successfully fixed {} issues", total_fixes);
    } else {
        println!("  ✨ All files are already correct!");
    }
}

#[derive(Debug)]
struct FixResult {
    file: String,
    fixes: Vec<String>,
    fixed: bool,
}

fn fix_resource(mut doc: serde_yaml::Value) -> (serde_yaml::Value, Vec<String>) {
    let mut fixes = Vec::new();
    let (kind, _name, _) = utils::get_resource_info(&doc);

    // Fix common issues
    if let Some(applied_fixes) = fix_missing_labels(&mut doc) {
        fixes.extend(applied_fixes);
    }

    if let Some(applied_fixes) = fix_resource_limits(&mut doc) {
        fixes.extend(applied_fixes);
    }

    if let Some(applied_fixes) = fix_health_probes(&mut doc) {
        fixes.extend(applied_fixes);
    }

    if let Some(applied_fixes) = fix_security_context(&mut doc) {
        fixes.extend(applied_fixes);
    }

    if let Some(applied_fixes) = fix_image_tags(&mut doc) {
        fixes.extend(applied_fixes);
    }

    // Apply kind-specific fixes
    match kind.as_str() {
        "Deployment" => {
            if let Some(applied_fixes) = fix_deployment_specific(&mut doc) {
                fixes.extend(applied_fixes);
            }
        }
        "Service" => {
            if let Some(applied_fixes) = fix_service_specific(&mut doc) {
                fixes.extend(applied_fixes);
            }
        }
        "Pod" => {
            if let Some(applied_fixes) = fix_pod_specific(&mut doc) {
                fixes.extend(applied_fixes);
            }
        }
        _ => {}
    }

    (doc, fixes)
}

fn fix_missing_labels(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(metadata) = doc.get_mut("metadata") {
        if let Some(metadata_map) = metadata.as_mapping_mut() {
            // Extract the name first to avoid borrowing conflicts
            let resource_name = metadata_map.get("name")
                .and_then(|name_value| name_value.as_str())
                .map(|s| s.to_string());

            // Ensure labels exist
            let labels = metadata_map
                .entry(serde_yaml::Value::String("labels".to_string()))
                .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

            if let Some(labels_map) = labels.as_mapping_mut() {
                // Add basic app label if missing
                if !labels_map.contains_key(&serde_yaml::Value::String("app".to_string())) {
                    if let Some(name) = resource_name {
                        labels_map.insert(
                            serde_yaml::Value::String("app".to_string()),
                            serde_yaml::Value::String(name),
                        );
                        fixes.push("Added 'app' label based on resource name".to_string());
                    }
                }
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_resource_limits(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(containers) = get_containers_mut(doc) {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                let container_name = container_map
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Add resources if completely missing
                if !container_map.contains_key(&serde_yaml::Value::String("resources".to_string())) {
                    let resources = serde_json::json!({
                        "requests": {
                            "cpu": "100m",
                            "memory": "128Mi"
                        },
                        "limits": {
                            "cpu": "500m",
                            "memory": "512Mi"
                        }
                    });
                    container_map.insert(
                        serde_yaml::Value::String("resources".to_string()),
                        serde_yaml::to_value(resources).unwrap(),
                    );
                    fixes.push(format!("Added resource requests and limits for container '{}'", container_name));
                }
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_health_probes(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(containers) = get_containers_mut(doc) {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                let container_name = container_map
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Add readiness probe if missing
                if !container_map.contains_key(&serde_yaml::Value::String("readinessProbe".to_string())) {
                    let readiness_probe = serde_json::json!({
                        "httpGet": {
                            "path": "/health",
                            "port": 8080
                        },
                        "initialDelaySeconds": 10,
                        "periodSeconds": 10
                    });
                    container_map.insert(
                        serde_yaml::Value::String("readinessProbe".to_string()),
                        serde_yaml::to_value(readiness_probe).unwrap(),
                    );
                    fixes.push(format!("Added readiness probe for container '{}'", container_name));
                }

                // Add liveness probe if missing
                if !container_map.contains_key(&serde_yaml::Value::String("livenessProbe".to_string())) {
                    let liveness_probe = serde_json::json!({
                        "httpGet": {
                            "path": "/health",
                            "port": 8080
                        },
                        "initialDelaySeconds": 30,
                        "periodSeconds": 30
                    });
                    container_map.insert(
                        serde_yaml::Value::String("livenessProbe".to_string()),
                        serde_yaml::to_value(liveness_probe).unwrap(),
                    );
                    fixes.push(format!("Added liveness probe for container '{}'", container_name));
                }
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_security_context(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(containers) = get_containers_mut(doc) {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                let container_name = container_map
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Add security context if missing
                if !container_map.contains_key(&serde_yaml::Value::String("securityContext".to_string())) {
                    let security_context = serde_json::json!({
                        "runAsNonRoot": true,
                        "runAsUser": 1000,
                        "allowPrivilegeEscalation": false,
                        "capabilities": {
                            "drop": ["ALL"]
                        }
                    });
                    container_map.insert(
                        serde_yaml::Value::String("securityContext".to_string()),
                        serde_yaml::to_value(security_context).unwrap(),
                    );
                    fixes.push(format!("Added security context for container '{}'", container_name));
                }
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_image_tags(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(containers) = get_containers_mut(doc) {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                let image_info = container_map.get("image")
                    .and_then(|i| i.as_str())
                    .map(|s| s.to_string());
                
                if let Some(image_str) = image_info {
                    if image_str.ends_with(":latest") || !image_str.contains(':') {
                        let fixed_image = if image_str.ends_with(":latest") {
                            image_str.replace(":latest", ":1.0.0")
                        } else {
                            format!("{}:1.0.0", image_str)
                        };
                        
                        container_map.insert(
                            serde_yaml::Value::String("image".to_string()),
                            serde_yaml::Value::String(fixed_image.clone())
                        );
                        fixes.push(format!("Changed image from '{}' to '{}'", image_str, fixed_image));
                    }
                }
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_deployment_specific(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(spec) = doc.get_mut("spec") {
        if let Some(spec_map) = spec.as_mapping_mut() {
            // Add selector if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("selector".to_string())) {
                let selector = serde_json::json!({
                    "matchLabels": {
                        "app": "app-name"
                    }
                });
                spec_map.insert(
                    serde_yaml::Value::String("selector".to_string()),
                    serde_yaml::to_value(selector).unwrap(),
                );
                fixes.push("Added selector for Deployment".to_string());
            }

            // Add replicas if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("replicas".to_string())) {
                spec_map.insert(
                    serde_yaml::Value::String("replicas".to_string()),
                    serde_yaml::Value::Number(2.into()),
                );
                fixes.push("Set replicas to 2".to_string());
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_service_specific(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(spec) = doc.get_mut("spec") {
        if let Some(spec_map) = spec.as_mapping_mut() {
            // Add ports if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("ports".to_string())) {
                let ports = serde_json::json!([{
                    "port": 80,
                    "targetPort": 8080,
                    "protocol": "TCP"
                }]);
                spec_map.insert(
                    serde_yaml::Value::String("ports".to_string()),
                    serde_yaml::to_value(ports).unwrap(),
                );
                fixes.push("Added default port configuration".to_string());
            }

            // Add selector if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("selector".to_string())) {
                let selector = serde_json::json!({
                    "app": "app-name"
                });
                spec_map.insert(
                    serde_yaml::Value::String("selector".to_string()),
                    serde_yaml::to_value(selector).unwrap(),
                );
                fixes.push("Added selector for Service".to_string());
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn fix_pod_specific(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut fixes = Vec::new();

    if let Some(spec) = doc.get_mut("spec") {
        if let Some(spec_map) = spec.as_mapping_mut() {
            // Add restart policy if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("restartPolicy".to_string())) {
                spec_map.insert(
                    serde_yaml::Value::String("restartPolicy".to_string()),
                    serde_yaml::Value::String("Always".to_string()),
                );
                fixes.push("Added restartPolicy: Always".to_string());
            }
        }
    }

    if fixes.is_empty() { None } else { Some(fixes) }
}

fn get_containers_mut(doc: &mut serde_yaml::Value) -> Option<&mut Vec<serde_yaml::Value>> {
    doc.get_mut("spec").and_then(|spec| {
        if spec.get("template").is_some() {
            spec.get_mut("template")
                .and_then(|template| template.get_mut("spec"))
                .and_then(|template_spec| template_spec.get_mut("containers"))
                .and_then(|containers| containers.as_sequence_mut())
        } else {
            spec.get_mut("containers")
                .and_then(|containers| containers.as_sequence_mut())
        }
    })
}