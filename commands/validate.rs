use std::process;
use crate::utils;

pub fn run_validate(path: &str, api_version: &str, json: bool, verbose: bool) {
    if verbose {
        println!("🔍 Starting validation for: {} (API version: {})", path, api_version);
    }

    let files = match utils::find_kubernetes_files(path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error finding files: {}", e);
            process::exit(1);
        }
    };

    let mut total_files = 0;
    let mut valid_files = 0;
    let mut validation_results = Vec::new();
    let mut all_errors = Vec::new();

    for file_path in files {
        total_files += 1;
        
        if verbose {
            println!("📄 Validating: {}", file_path);
        }

        let contents = match utils::read_file_contents(&file_path) {
            Ok(content) => content,
            Err(e) => {
                let error_msg = format!("Failed to read file: {}", e);
                all_errors.push(error_msg.clone());
                validation_results.push(ValidationResult {
                    file: file_path,
                    valid: false,
                    errors: vec![error_msg],
                    resources_count: 0,
                });
                continue;
            }
        };

        let docs = match utils::parse_yaml(&contents) {
            Ok(documents) => documents,
            Err(e) => {
                let error_msg = format!("YAML parsing error: {}", e);
                all_errors.push(error_msg.clone());
                validation_results.push(ValidationResult {
                    file: file_path,
                    valid: false,
                    errors: vec![error_msg],
                    resources_count: 0,
                });
                continue;
            }
        };

        let mut file_errors = Vec::new();
        let mut _valid_resources = 0;

        for (_idx, doc) in docs.iter().enumerate() {
            let (kind, name, namespace) = utils::get_resource_info(doc);
            
            // Basic structure validation
            if let Err(validation_errors) = validate_kubernetes_resource(doc, api_version) {
                for error in validation_errors {
                    let error_msg = format!("Resource {}/{} ({}): {}", namespace, name, kind, error);
                    file_errors.push(error_msg.clone());
                    all_errors.push(error_msg);
                }
            } else {
                _valid_resources += 1;
            }
        }

        let is_valid = file_errors.is_empty();
        if is_valid {
            valid_files += 1;
        }

        validation_results.push(ValidationResult {
            file: file_path,
            valid: is_valid,
            errors: file_errors,
            resources_count: docs.len(),
        });
    }

    // Output results
    if json {
        let json_output = serde_json::json!({
            "summary": {
                "total_files": total_files,
                "valid_files": valid_files,
                "invalid_files": total_files - valid_files,
                "total_errors": all_errors.len(),
                "api_version": api_version
            },
            "results": validation_results
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
    } else {
        println!("\n✅ Validation Results");
        println!("{}", "=".repeat(50));
        
        for result in &validation_results {
            if result.valid {
                println!("✅ {} ({} resources)", result.file, result.resources_count);
            } else {
                println!("❌ {} ({} resources)", result.file, result.resources_count);
                for error in &result.errors {
                    println!("   • {}", error);
                }
            }
        }
        
        println!("\n{}", "=".repeat(50));
        println!("📊 Summary");
        println!("  Files validated: {}", total_files);
        println!("  Valid files: {}", valid_files);
        println!("  Invalid files: {}", total_files - valid_files);
        println!("  Total errors: {}", all_errors.len());
        
        if valid_files == total_files {
            println!("  🎉 All files are valid!");
        }
    }

    // Exit with error if any validation failed
    if valid_files < total_files {
        process::exit(1);
    }
}

#[derive(serde::Serialize)]
struct ValidationResult {
    file: String,
    valid: bool,
    errors: Vec<String>,
    resources_count: usize,
}

fn validate_kubernetes_resource(doc: &serde_yaml::Value, _api_version: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Check for required fields
    if doc.get("apiVersion").is_none() {
        errors.push("Missing required field 'apiVersion'".to_string());
    }

    if doc.get("kind").is_none() {
        errors.push("Missing required field 'kind'".to_string());
    }

    let metadata = doc.get("metadata");
    if metadata.is_none() {
        errors.push("Missing required field 'metadata'".to_string());
    } else {
        let metadata = metadata.unwrap();
        if metadata.get("name").is_none() {
            errors.push("Missing required field 'metadata.name'".to_string());
        }
    }

    // Validate specific resource types
    if let Some(kind) = doc.get("kind").and_then(|k| k.as_str()) {
        match kind {
            "Pod" => validate_pod(doc, &mut errors),
            "Deployment" => validate_deployment(doc, &mut errors),
            "Service" => validate_service(doc, &mut errors),
            "ConfigMap" => validate_configmap(doc, &mut errors),
            "Secret" => validate_secret(doc, &mut errors),
            _ => {} // Unknown kind, skip specific validation
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_pod(doc: &serde_yaml::Value, errors: &mut Vec<String>) {
    if let Some(spec) = doc.get("spec") {
        if spec.get("containers").is_none() {
            errors.push("Pod spec missing 'containers' field".to_string());
        }
    } else {
        errors.push("Pod missing 'spec' field".to_string());
    }
}

fn validate_deployment(doc: &serde_yaml::Value, errors: &mut Vec<String>) {
    if let Some(spec) = doc.get("spec") {
        if spec.get("selector").is_none() {
            errors.push("Deployment spec missing 'selector' field".to_string());
        }
        if spec.get("template").is_none() {
            errors.push("Deployment spec missing 'template' field".to_string());
        }
    } else {
        errors.push("Deployment missing 'spec' field".to_string());
    }
}

fn validate_service(doc: &serde_yaml::Value, errors: &mut Vec<String>) {
    if let Some(spec) = doc.get("spec") {
        if spec.get("ports").is_none() {
            errors.push("Service spec missing 'ports' field".to_string());
        }
    } else {
        errors.push("Service missing 'spec' field".to_string());
    }
}

fn validate_configmap(_doc: &serde_yaml::Value, _errors: &mut Vec<String>) {
    // ConfigMaps are generally valid if they have the basic structure
    // Additional validation could be added here for data format
}

fn validate_secret(_doc: &serde_yaml::Value, _errors: &mut Vec<String>) {
    // Secrets are generally valid if they have the basic structure
    // Additional validation could be added here for data format and type
}