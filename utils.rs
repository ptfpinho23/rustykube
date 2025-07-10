use serde_yaml::{Deserializer, Value};
use serde::de::Deserialize;
use std::fs;
use std::path::Path;

pub fn parse_yaml(contents: &str) -> Result<Vec<Value>, String> {
    let mut documents = Vec::new();
    
    for doc in Deserializer::from_str(contents) {
        match Value::deserialize(doc) {
            Ok(value) => documents.push(value),
            Err(e) => return Err(format!("Failed to deserialize YAML document: {}", e)),
        }
    }
    
    if documents.is_empty() {
        return Err("No valid YAML documents found".to_string());
    }
    
    Ok(documents)
}

pub fn read_file_contents(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

pub fn is_kubernetes_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        ext == "yaml" || ext == "yml"
    } else {
        false
    }
}

pub fn find_kubernetes_files(dir_path: &str) -> Result<Vec<String>, String> {
    let path = Path::new(dir_path);
    
    if !path.exists() {
        return Err(format!("Path '{}' does not exist", dir_path));
    }
    
    if path.is_file() {
        if is_kubernetes_file(path) {
            return Ok(vec![dir_path.to_string()]);
        } else {
            return Err(format!("File '{}' is not a YAML file", dir_path));
        }
    }
    
    let mut yaml_files = Vec::new();
    
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir_path, e))?;
    
    for entry in entries {
        let entry = entry
            .map_err(|e| format!("Failed to read directory entry: {}", e))?;
        
        let file_path = entry.path();
        
        if file_path.is_file() && is_kubernetes_file(&file_path) {
            if let Some(path_str) = file_path.to_str() {
                yaml_files.push(path_str.to_string());
            }
        }
    }
    
    if yaml_files.is_empty() {
        return Err(format!("No YAML files found in directory '{}'", dir_path));
    }
    
    yaml_files.sort();
    Ok(yaml_files)
}

pub fn get_resource_info(doc: &Value) -> (String, String, String) {
    let kind = doc
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let name = doc
        .get("metadata")
        .and_then(|metadata| metadata.get("name"))
        .and_then(|name| name.as_str())
        .unwrap_or("Unnamed")
        .to_string();
    
    let namespace = doc
        .get("metadata")
        .and_then(|metadata| metadata.get("namespace"))
        .and_then(|ns| ns.as_str())
        .unwrap_or("default")
        .to_string();
    
    (kind, name, namespace)
}
