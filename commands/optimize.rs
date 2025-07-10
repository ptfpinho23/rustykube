use std::fs;
use std::path::Path;
use std::process;
use crate::utils;

pub fn run_optimize(path: &str, output: Option<&str>, in_place: bool, aggressive: bool, verbose: bool) {
    if verbose {
        println!("🚀 Starting optimization for: {}", path);
        if aggressive {
            println!("⚡ Aggressive optimizations enabled");
        }
    }

    let files = match utils::find_kubernetes_files(path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error finding files: {}", e);
            process::exit(1);
        }
    };

    let mut optimization_results = Vec::new();
    let mut total_optimizations = 0;

    for file_path in files {
        if verbose {
            println!("📄 Optimizing: {}", file_path);
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

        let mut optimized_docs = Vec::new();
        let mut file_optimizations = Vec::new();

        for doc in docs {
            let (optimized_doc, optimizations) = optimize_resource(doc, aggressive);
            optimized_docs.push(optimized_doc);
            file_optimizations.extend(optimizations);
        }

        total_optimizations += file_optimizations.len();

        // Generate optimized YAML content
        let mut optimized_content = String::new();
        for (i, doc) in optimized_docs.iter().enumerate() {
            if i > 0 {
                optimized_content.push_str("---\n");
            }
            optimized_content.push_str(&serde_yaml::to_string(doc).unwrap());
        }

        // Handle output
        let output_path = if in_place {
            file_path.clone()
        } else if let Some(output_dir) = output {
            let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();
            format!("{}/{}", output_dir, file_name)
        } else {
            format!("{}.optimized.yaml", file_path.strip_suffix(".yaml").unwrap_or(&file_path))
        };

        if !file_optimizations.is_empty() {
            if let Err(e) = fs::write(&output_path, &optimized_content) {
                eprintln!("❌ Error writing optimized file {}: {}", output_path, e);
                continue;
            }

            if verbose || !in_place {
                println!("✅ Optimized {} -> {}", file_path, output_path);
                for opt in &file_optimizations {
                    println!("   • {}", opt);
                }
            }
        } else if verbose {
            println!("✅ {} - No optimizations needed", file_path);
        }

        optimization_results.push(OptimizationResult {
            file: file_path,
            output_file: output_path,
            optimizations: file_optimizations,
        });
    }

    // Summary
    println!("\n🎯 Optimization Summary");
    println!("{}", "=".repeat(50));
    println!("  Files processed: {}", optimization_results.len());
    println!("  Total optimizations applied: {}", total_optimizations);
    
    if total_optimizations > 0 {
        println!("  🎉 Successfully optimized {} resources", total_optimizations);
    } else {
        println!("  ✨ All resources are already optimized!");
    }
}

#[derive(Debug)]
struct OptimizationResult {
    file: String,
    output_file: String,
    optimizations: Vec<String>,
}

fn optimize_resource(mut doc: serde_yaml::Value, aggressive: bool) -> (serde_yaml::Value, Vec<String>) {
    let mut optimizations = Vec::new();
    let (kind, _name, _) = utils::get_resource_info(&doc);

    // Apply general optimizations
    if let Some(optimized) = optimize_labels_and_annotations(&mut doc) {
        optimizations.extend(optimized);
    }

    if let Some(optimized) = optimize_resource_requests(&mut doc, aggressive) {
        optimizations.extend(optimized);
    }

    if let Some(optimized) = optimize_container_settings(&mut doc, aggressive) {
        optimizations.extend(optimized);
    }

    // Apply kind-specific optimizations
    match kind.as_str() {
        "Deployment" => {
            if let Some(optimized) = optimize_deployment(&mut doc, aggressive) {
                optimizations.extend(optimized);
            }
        }
        "Service" => {
            if let Some(optimized) = optimize_service(&mut doc) {
                optimizations.extend(optimized);
            }
        }
        "Pod" => {
            if let Some(optimized) = optimize_pod(&mut doc, aggressive) {
                optimizations.extend(optimized);
            }
        }
        _ => {}
    }

    (doc, optimizations)
}

fn optimize_labels_and_annotations(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut optimizations = Vec::new();

    if let Some(metadata) = doc.get_mut("metadata") {
        if let Some(metadata_map) = metadata.as_mapping_mut() {
            // Add recommended labels if missing
            let labels = metadata_map
                .entry(serde_yaml::Value::String("labels".to_string()))
                .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

            if let Some(labels_map) = labels.as_mapping_mut() {
                let recommended_labels = vec![
                    ("app.kubernetes.io/name", "application-name"),
                    ("app.kubernetes.io/version", "1.0.0"),
                    ("app.kubernetes.io/component", "component"),
                ];

                for (key, default_value) in recommended_labels {
                    if !labels_map.contains_key(&serde_yaml::Value::String(key.to_string())) {
                        labels_map.insert(
                            serde_yaml::Value::String(key.to_string()),
                            serde_yaml::Value::String(format!("TODO-{}", default_value)),
                        );
                        optimizations.push(format!("Added recommended label: {}", key));
                    }
                }
            }
        }
    }

    if optimizations.is_empty() {
        None
    } else {
        Some(optimizations)
    }
}

fn optimize_resource_requests(doc: &mut serde_yaml::Value, aggressive: bool) -> Option<Vec<String>> {
    let mut optimizations = Vec::new();

    if let Some(containers) = get_containers_mut(doc) {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                let resources = container_map
                    .entry(serde_yaml::Value::String("resources".to_string()))
                    .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

                if let Some(resources_map) = resources.as_mapping_mut() {
                    // Add requests if missing
                    let requests = resources_map
                        .entry(serde_yaml::Value::String("requests".to_string()))
                        .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

                    if let Some(requests_map) = requests.as_mapping_mut() {
                        if !requests_map.contains_key(&serde_yaml::Value::String("cpu".to_string())) {
                            requests_map.insert(
                                serde_yaml::Value::String("cpu".to_string()),
                                serde_yaml::Value::String("100m".to_string()),
                            );
                            optimizations.push("Added CPU request: 100m".to_string());
                        }

                        if !requests_map.contains_key(&serde_yaml::Value::String("memory".to_string())) {
                            requests_map.insert(
                                serde_yaml::Value::String("memory".to_string()),
                                serde_yaml::Value::String("128Mi".to_string()),
                            );
                            optimizations.push("Added memory request: 128Mi".to_string());
                        }
                    }

                    // Add limits if aggressive mode and missing
                    if aggressive {
                        let limits = resources_map
                            .entry(serde_yaml::Value::String("limits".to_string()))
                            .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

                        if let Some(limits_map) = limits.as_mapping_mut() {
                            if !limits_map.contains_key(&serde_yaml::Value::String("cpu".to_string())) {
                                limits_map.insert(
                                    serde_yaml::Value::String("cpu".to_string()),
                                    serde_yaml::Value::String("500m".to_string()),
                                );
                                optimizations.push("Added CPU limit: 500m".to_string());
                            }

                            if !limits_map.contains_key(&serde_yaml::Value::String("memory".to_string())) {
                                limits_map.insert(
                                    serde_yaml::Value::String("memory".to_string()),
                                    serde_yaml::Value::String("512Mi".to_string()),
                                );
                                optimizations.push("Added memory limit: 512Mi".to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if optimizations.is_empty() {
        None
    } else {
        Some(optimizations)
    }
}

fn optimize_container_settings(doc: &mut serde_yaml::Value, aggressive: bool) -> Option<Vec<String>> {
    let mut optimizations = Vec::new();

    if let Some(containers) = get_containers_mut(doc) {
        for container in containers {
            if let Some(container_map) = container.as_mapping_mut() {
                // Add security context
                let security_context = container_map
                    .entry(serde_yaml::Value::String("securityContext".to_string()))
                    .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

                if let Some(security_map) = security_context.as_mapping_mut() {
                    if !security_map.contains_key(&serde_yaml::Value::String("runAsNonRoot".to_string())) {
                        security_map.insert(
                            serde_yaml::Value::String("runAsNonRoot".to_string()),
                            serde_yaml::Value::Bool(true),
                        );
                        optimizations.push("Added runAsNonRoot: true".to_string());
                    }

                    if aggressive && !security_map.contains_key(&serde_yaml::Value::String("readOnlyRootFilesystem".to_string())) {
                        security_map.insert(
                            serde_yaml::Value::String("readOnlyRootFilesystem".to_string()),
                            serde_yaml::Value::Bool(true),
                        );
                        optimizations.push("Added readOnlyRootFilesystem: true".to_string());
                    }
                }

                // Optimize image pull policy
                if let Some(image_pull_policy) = container_map.get("imagePullPolicy") {
                    if image_pull_policy.as_str() == Some("Always") {
                        container_map.insert(
                            serde_yaml::Value::String("imagePullPolicy".to_string()),
                            serde_yaml::Value::String("IfNotPresent".to_string()),
                        );
                        optimizations.push("Changed imagePullPolicy from Always to IfNotPresent".to_string());
                    }
                }
            }
        }
    }

    if optimizations.is_empty() {
        None
    } else {
        Some(optimizations)
    }
}

fn optimize_deployment(doc: &mut serde_yaml::Value, aggressive: bool) -> Option<Vec<String>> {
    let mut optimizations = Vec::new();

    if let Some(spec) = doc.get_mut("spec") {
        if let Some(spec_map) = spec.as_mapping_mut() {
            // Add rolling update strategy if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("strategy".to_string())) {
                let strategy = serde_json::json!({
                    "type": "RollingUpdate",
                    "rollingUpdate": {
                        "maxUnavailable": "25%",
                        "maxSurge": "25%"
                    }
                });
                spec_map.insert(
                    serde_yaml::Value::String("strategy".to_string()),
                    serde_yaml::to_value(strategy).unwrap(),
                );
                optimizations.push("Added RollingUpdate strategy".to_string());
            }

            // Set replicas if missing (aggressive mode)
            if aggressive && !spec_map.contains_key(&serde_yaml::Value::String("replicas".to_string())) {
                spec_map.insert(
                    serde_yaml::Value::String("replicas".to_string()),
                    serde_yaml::Value::Number(3.into()),
                );
                optimizations.push("Set replicas to 3 for high availability".to_string());
            }
        }
    }

    if optimizations.is_empty() {
        None
    } else {
        Some(optimizations)
    }
}

fn optimize_service(doc: &mut serde_yaml::Value) -> Option<Vec<String>> {
    let mut optimizations = Vec::new();

    if let Some(spec) = doc.get_mut("spec") {
        if let Some(spec_map) = spec.as_mapping_mut() {
            // Optimize service type
            if let Some(service_type) = spec_map.get("type") {
                if service_type.as_str() == Some("LoadBalancer") {
                    // Suggest using ClusterIP if no external access needed
                    optimizations.push("Consider using ClusterIP instead of LoadBalancer if external access not required".to_string());
                }
            }

            // Add session affinity for stateful apps
            if !spec_map.contains_key(&serde_yaml::Value::String("sessionAffinity".to_string())) {
                spec_map.insert(
                    serde_yaml::Value::String("sessionAffinity".to_string()),
                    serde_yaml::Value::String("None".to_string()),
                );
                optimizations.push("Added explicit sessionAffinity: None".to_string());
            }
        }
    }

    if optimizations.is_empty() {
        None
    } else {
        Some(optimizations)
    }
}

fn optimize_pod(doc: &mut serde_yaml::Value, aggressive: bool) -> Option<Vec<String>> {
    let mut optimizations = Vec::new();

    if let Some(spec) = doc.get_mut("spec") {
        if let Some(spec_map) = spec.as_mapping_mut() {
            // Add restart policy if missing
            if !spec_map.contains_key(&serde_yaml::Value::String("restartPolicy".to_string())) {
                spec_map.insert(
                    serde_yaml::Value::String("restartPolicy".to_string()),
                    serde_yaml::Value::String("Always".to_string()),
                );
                optimizations.push("Added restartPolicy: Always".to_string());
            }

            // Add DNS policy if aggressive
            if aggressive && !spec_map.contains_key(&serde_yaml::Value::String("dnsPolicy".to_string())) {
                spec_map.insert(
                    serde_yaml::Value::String("dnsPolicy".to_string()),
                    serde_yaml::Value::String("ClusterFirst".to_string()),
                );
                optimizations.push("Added dnsPolicy: ClusterFirst".to_string());
            }
        }
    }

    if optimizations.is_empty() {
        None
    } else {
        Some(optimizations)
    }
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