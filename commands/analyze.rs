use std::collections::HashMap;
use std::process;
use crate::utils;

pub fn run_analyze(path: &str, detailed: bool, json: bool, verbose: bool) {
    if verbose {
        println!("📊 Starting analysis for: {}", path);
    }

    let files = match utils::find_kubernetes_files(path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error finding files: {}", e);
            process::exit(1);
        }
    };

    let mut analysis_results = Vec::new();
    let mut overall_stats = OverallStats::new();

    for file_path in files {
        if verbose {
            println!("📄 Analyzing: {}", file_path);
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

        let mut file_analysis = FileAnalysis {
            file: file_path.clone(),
            resources: Vec::new(),
            summary: FileSummary::new(),
            recommendations: Vec::new(),
        };

        for doc in docs {
            let resource_analysis = analyze_resource(&doc, detailed);
            overall_stats.update_from_resource(&resource_analysis);
            file_analysis.summary.update_from_resource(&resource_analysis);
            file_analysis.resources.push(resource_analysis);
        }

        // Generate file-level recommendations
        file_analysis.recommendations = generate_file_recommendations(&file_analysis, detailed);
        analysis_results.push(file_analysis);
    }

    // Output results
    if json {
        let json_output = serde_json::json!({
            "overall_stats": overall_stats,
            "files": analysis_results,
            "recommendations": generate_overall_recommendations(&overall_stats, detailed)
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
    } else {
        print_analysis_report(&analysis_results, &overall_stats, detailed);
    }
}

#[derive(Debug, serde::Serialize)]
struct FileAnalysis {
    file: String,
    resources: Vec<ResourceAnalysis>,
    summary: FileSummary,
    recommendations: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ResourceAnalysis {
    kind: String,
    name: String,
    namespace: String,
    complexity_score: u32,
    security_score: u32,
    performance_score: u32,
    reliability_score: u32,
    issues: Vec<Issue>,
    insights: Vec<String>,
    resource_usage: ResourceUsage,
}

#[derive(Debug, serde::Serialize)]
struct Issue {
    severity: String,
    category: String,
    message: String,
    recommendation: String,
}

#[derive(Debug, serde::Serialize)]
struct ResourceUsage {
    cpu_requests: Option<String>,
    memory_requests: Option<String>,
    cpu_limits: Option<String>,
    memory_limits: Option<String>,
    has_probes: bool,
    has_security_context: bool,
}

#[derive(Debug, serde::Serialize)]
struct FileSummary {
    total_resources: usize,
    resource_types: HashMap<String, usize>,
    avg_complexity_score: f32,
    avg_security_score: f32,
    avg_performance_score: f32,
    avg_reliability_score: f32,
    total_issues: usize,
    high_severity_issues: usize,
}

#[derive(Debug, serde::Serialize)]
struct OverallStats {
    total_files: usize,
    total_resources: usize,
    resource_types: HashMap<String, usize>,
    avg_complexity_score: f32,
    avg_security_score: f32,
    avg_performance_score: f32,
    avg_reliability_score: f32,
    total_issues: usize,
    high_severity_issues: usize,
    namespaces: HashMap<String, usize>,
}

impl FileSummary {
    fn new() -> Self {
        Self {
            total_resources: 0,
            resource_types: HashMap::new(),
            avg_complexity_score: 0.0,
            avg_security_score: 0.0,
            avg_performance_score: 0.0,
            avg_reliability_score: 0.0,
            total_issues: 0,
            high_severity_issues: 0,
        }
    }

    fn update_from_resource(&mut self, resource: &ResourceAnalysis) {
        self.total_resources += 1;
        *self.resource_types.entry(resource.kind.clone()).or_insert(0) += 1;
        
        // Update averages
        let n = self.total_resources as f32;
        self.avg_complexity_score = (self.avg_complexity_score * (n - 1.0) + resource.complexity_score as f32) / n;
        self.avg_security_score = (self.avg_security_score * (n - 1.0) + resource.security_score as f32) / n;
        self.avg_performance_score = (self.avg_performance_score * (n - 1.0) + resource.performance_score as f32) / n;
        self.avg_reliability_score = (self.avg_reliability_score * (n - 1.0) + resource.reliability_score as f32) / n;
        
        self.total_issues += resource.issues.len();
        self.high_severity_issues += resource.issues.iter().filter(|i| i.severity == "High").count();
    }
}

impl OverallStats {
    fn new() -> Self {
        Self {
            total_files: 0,
            total_resources: 0,
            resource_types: HashMap::new(),
            avg_complexity_score: 0.0,
            avg_security_score: 0.0,
            avg_performance_score: 0.0,
            avg_reliability_score: 0.0,
            total_issues: 0,
            high_severity_issues: 0,
            namespaces: HashMap::new(),
        }
    }

    fn update_from_resource(&mut self, resource: &ResourceAnalysis) {
        self.total_resources += 1;
        *self.resource_types.entry(resource.kind.clone()).or_insert(0) += 1;
        *self.namespaces.entry(resource.namespace.clone()).or_insert(0) += 1;
        
        // Update averages
        let n = self.total_resources as f32;
        self.avg_complexity_score = (self.avg_complexity_score * (n - 1.0) + resource.complexity_score as f32) / n;
        self.avg_security_score = (self.avg_security_score * (n - 1.0) + resource.security_score as f32) / n;
        self.avg_performance_score = (self.avg_performance_score * (n - 1.0) + resource.performance_score as f32) / n;
        self.avg_reliability_score = (self.avg_reliability_score * (n - 1.0) + resource.reliability_score as f32) / n;
        
        self.total_issues += resource.issues.len();
        self.high_severity_issues += resource.issues.iter().filter(|i| i.severity == "High").count();
    }
}

fn analyze_resource(doc: &serde_yaml::Value, detailed: bool) -> ResourceAnalysis {
    let (kind, name, namespace) = utils::get_resource_info(doc);
    
    let complexity_score = calculate_complexity_score(doc);
    let security_score = calculate_security_score(doc);
    let performance_score = calculate_performance_score(doc);
    let reliability_score = calculate_reliability_score(doc);
    
    let issues = identify_issues(doc);
    let insights = if detailed {
        generate_insights(doc, &kind)
    } else {
        vec![]
    };
    
    let resource_usage = analyze_resource_usage(doc);

    ResourceAnalysis {
        kind,
        name,
        namespace,
        complexity_score,
        security_score,
        performance_score,
        reliability_score,
        issues,
        insights,
        resource_usage,
    }
}

fn calculate_complexity_score(doc: &serde_yaml::Value) -> u32 {
    let mut score = 50; // Base score

    // Increase complexity based on resource type
    if let Some(kind) = doc.get("kind").and_then(|k| k.as_str()) {
        match kind {
            "Pod" => score += 10,
            "Deployment" => score += 20,
            "StatefulSet" => score += 30,
            "DaemonSet" => score += 25,
            "Service" => score += 15,
            "Ingress" => score += 35,
            _ => score += 5,
        }
    }

    // Increase based on number of containers
    if let Some(containers) = get_containers(doc) {
        score += (containers.len() as u32) * 10;
    }

    // Increase based on number of volumes
    if let Some(spec) = doc.get("spec") {
        if let Some(volumes) = spec.get("volumes").and_then(|v| v.as_sequence()) {
            score += (volumes.len() as u32) * 5;
        }
    }

    score.min(100) // Cap at 100
}

fn calculate_security_score(doc: &serde_yaml::Value) -> u32 {
    let mut score = 100; // Start with perfect score

    if let Some(containers) = get_containers(doc) {
        for container in containers {
            // Check security context
            if let Some(security_context) = container.get("securityContext") {
                if security_context.get("runAsNonRoot").and_then(|v| v.as_bool()) != Some(true) {
                    score -= 15;
                }
                if security_context.get("readOnlyRootFilesystem").and_then(|v| v.as_bool()) != Some(true) {
                    score -= 10;
                }
                if security_context.get("allowPrivilegeEscalation").and_then(|v| v.as_bool()) == Some(true) {
                    score -= 20;
                }
            } else {
                score -= 25; // No security context
            }

            // Check image tags
            if let Some(image) = container.get("image").and_then(|i| i.as_str()) {
                if image.ends_with(":latest") || !image.contains(':') {
                    score -= 15;
                }
            }
        }
    }

    score.max(0)
}

fn calculate_performance_score(doc: &serde_yaml::Value) -> u32 {
    let mut score = 100; // Start with perfect score

    if let Some(containers) = get_containers(doc) {
        for container in containers {
            // Check resource limits
            if let Some(resources) = container.get("resources") {
                if resources.get("requests").is_none() {
                    score -= 20;
                }
                if resources.get("limits").is_none() {
                    score -= 15;
                }
            } else {
                score -= 30; // No resource configuration
            }

            // Check probes
            if container.get("livenessProbe").is_none() {
                score -= 10;
            }
            if container.get("readinessProbe").is_none() {
                score -= 10;
            }
        }
    }

    score.max(0)
}

fn calculate_reliability_score(doc: &serde_yaml::Value) -> u32 {
    let mut score = 100; // Start with perfect score

    // Check for replicas in deployments
    if doc.get("kind").and_then(|k| k.as_str()) == Some("Deployment") {
        if let Some(replicas) = doc.get("spec")
            .and_then(|s| s.get("replicas"))
            .and_then(|r| r.as_u64()) {
            if replicas < 2 {
                score -= 20; // Single replica is not reliable
            }
        }
    }

    // Check for probes
    if let Some(containers) = get_containers(doc) {
        for container in containers {
            if container.get("livenessProbe").is_none() {
                score -= 15;
            }
            if container.get("readinessProbe").is_none() {
                score -= 15;
            }
        }
    }

    // Check for restart policy
    if let Some(spec) = doc.get("spec") {
        if spec.get("restartPolicy").and_then(|p| p.as_str()) != Some("Always") {
            score -= 10;
        }
    }

    score.max(0)
}

fn identify_issues(doc: &serde_yaml::Value) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for missing labels
    if let Some(metadata) = doc.get("metadata") {
        if metadata.get("labels").is_none() {
            issues.push(Issue {
                severity: "Medium".to_string(),
                category: "Best Practices".to_string(),
                message: "Resource has no labels".to_string(),
                recommendation: "Add labels for better organization and selection".to_string(),
            });
        }
    }

    // Check containers for issues
    if let Some(containers) = get_containers(doc) {
        for container in containers {
            let container_name = container.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");

            // Missing resource limits
            if container.get("resources").is_none() {
                issues.push(Issue {
                    severity: "High".to_string(),
                    category: "Performance".to_string(),
                    message: format!("Container '{}' has no resource limits", container_name),
                    recommendation: "Add CPU and memory requests and limits".to_string(),
                });
            }

            // Missing security context
            if container.get("securityContext").is_none() {
                issues.push(Issue {
                    severity: "High".to_string(),
                    category: "Security".to_string(),
                    message: format!("Container '{}' has no security context", container_name),
                    recommendation: "Add security context with runAsNonRoot and other security settings".to_string(),
                });
            }

            // Latest image tag
            if let Some(image) = container.get("image").and_then(|i| i.as_str()) {
                if image.ends_with(":latest") {
                    issues.push(Issue {
                        severity: "Medium".to_string(),
                        category: "Best Practices".to_string(),
                        message: format!("Container '{}' uses 'latest' image tag", container_name),
                        recommendation: "Use specific version tags for reproducible deployments".to_string(),
                    });
                }
            }
        }
    }

    issues
}

fn generate_insights(doc: &serde_yaml::Value, kind: &str) -> Vec<String> {
    let mut insights = Vec::new();

    match kind {
        "Deployment" => {
            if let Some(replicas) = doc.get("spec")
                .and_then(|s| s.get("replicas"))
                .and_then(|r| r.as_u64()) {
                if replicas == 1 {
                    insights.push("Consider increasing replicas for high availability".to_string());
                } else if replicas > 10 {
                    insights.push("High replica count - ensure your cluster can handle the resource requirements".to_string());
                }
            }

            if doc.get("spec")
                .and_then(|s| s.get("strategy"))
                .is_none() {
                insights.push("Consider adding a deployment strategy for controlled rollouts".to_string());
            }
        }
        "Service" => {
            if let Some(service_type) = doc.get("spec")
                .and_then(|s| s.get("type"))
                .and_then(|t| t.as_str()) {
                if service_type == "LoadBalancer" {
                    insights.push("LoadBalancer services incur cloud provider costs - consider using Ingress if appropriate".to_string());
                }
            }
        }
        "Pod" => {
            insights.push("Consider using Deployments instead of bare Pods for better management".to_string());
        }
        _ => {}
    }

    insights
}

fn analyze_resource_usage(doc: &serde_yaml::Value) -> ResourceUsage {
    let mut usage = ResourceUsage {
        cpu_requests: None,
        memory_requests: None,
        cpu_limits: None,
        memory_limits: None,
        has_probes: false,
        has_security_context: false,
    };

    if let Some(containers) = get_containers(doc) {
        for container in containers {
            if let Some(resources) = container.get("resources") {
                if let Some(requests) = resources.get("requests") {
                    if let Some(cpu) = requests.get("cpu").and_then(|c| c.as_str()) {
                        usage.cpu_requests = Some(cpu.to_string());
                    }
                    if let Some(memory) = requests.get("memory").and_then(|m| m.as_str()) {
                        usage.memory_requests = Some(memory.to_string());
                    }
                }
                if let Some(limits) = resources.get("limits") {
                    if let Some(cpu) = limits.get("cpu").and_then(|c| c.as_str()) {
                        usage.cpu_limits = Some(cpu.to_string());
                    }
                    if let Some(memory) = limits.get("memory").and_then(|m| m.as_str()) {
                        usage.memory_limits = Some(memory.to_string());
                    }
                }
            }

            if container.get("livenessProbe").is_some() || container.get("readinessProbe").is_some() {
                usage.has_probes = true;
            }

            if container.get("securityContext").is_some() {
                usage.has_security_context = true;
            }
        }
    }

    usage
}

fn generate_file_recommendations(analysis: &FileAnalysis, detailed: bool) -> Vec<String> {
    let mut recommendations = Vec::new();

    if analysis.summary.avg_security_score < 70.0 {
        recommendations.push("Consider improving security by adding security contexts and avoiding latest image tags".to_string());
    }

    if analysis.summary.avg_performance_score < 70.0 {
        recommendations.push("Add resource requests and limits to improve performance and resource management".to_string());
    }

    if analysis.summary.high_severity_issues > 0 {
        recommendations.push(format!("Address {} high-severity issues to improve overall quality", analysis.summary.high_severity_issues));
    }

    if detailed && analysis.summary.total_resources > 5 {
        recommendations.push("Consider splitting this file into smaller, more focused files for better maintainability".to_string());
    }

    recommendations
}

fn generate_overall_recommendations(stats: &OverallStats, detailed: bool) -> Vec<String> {
    let mut recommendations = Vec::new();

    if stats.avg_security_score < 80.0 {
        recommendations.push("Overall security score is below recommended threshold. Focus on improving security contexts and image management.".to_string());
    }

    if stats.avg_performance_score < 80.0 {
        recommendations.push("Performance can be improved by adding proper resource management across all resources.".to_string());
    }

    if stats.total_issues > stats.total_resources * 2 {
        recommendations.push("High number of issues detected. Consider implementing a systematic review process.".to_string());
    }

    if detailed && stats.resource_types.len() > 5 {
        recommendations.push("Multiple resource types detected. Consider organizing by application or namespace.".to_string());
    }

    recommendations
}

fn print_analysis_report(results: &[FileAnalysis], stats: &OverallStats, detailed: bool) {
    println!("📊 Kubernetes Manifest Analysis Report");
    println!("{}", "=".repeat(60));
    
    // Overall Statistics
    println!("\n📈 Overall Statistics");
    println!("  Total Files: {}", stats.total_files);
    println!("  Total Resources: {}", stats.total_resources);
    println!("  Resource Types: {:?}", stats.resource_types);
    println!("  Namespaces: {:?}", stats.namespaces);
    
    println!("\n📊 Quality Scores (0-100)");
    println!("  Security:     {:.1}", stats.avg_security_score);
    println!("  Performance:  {:.1}", stats.avg_performance_score);
    println!("  Reliability:  {:.1}", stats.avg_reliability_score);
    println!("  Complexity:   {:.1}", stats.avg_complexity_score);
    
    println!("\n⚠️  Issues Summary");
    println!("  Total Issues: {}", stats.total_issues);
    println!("  High Severity: {}", stats.high_severity_issues);
    
    // Per-file analysis
    if detailed {
        println!("\n📁 File-by-File Analysis");
        println!("{}", "-".repeat(60));
        
        for file_analysis in results {
            println!("\n📄 {}", file_analysis.file);
            println!("  Resources: {}", file_analysis.summary.total_resources);
            println!("  Security Score: {:.1}", file_analysis.summary.avg_security_score);
            println!("  Performance Score: {:.1}", file_analysis.summary.avg_performance_score);
            println!("  Issues: {} (High: {})", file_analysis.summary.total_issues, file_analysis.summary.high_severity_issues);
            
            if !file_analysis.recommendations.is_empty() {
                println!("  Recommendations:");
                for rec in &file_analysis.recommendations {
                    println!("    • {}", rec);
                }
            }
        }
    }
    
    // Overall recommendations
    let overall_recommendations = generate_overall_recommendations(stats, detailed);
    if !overall_recommendations.is_empty() {
        println!("\n💡 Overall Recommendations");
        println!("{}", "-".repeat(60));
        for (i, rec) in overall_recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec);
        }
    }
}

fn get_containers(doc: &serde_yaml::Value) -> Option<&Vec<serde_yaml::Value>> {
    doc.get("spec")
        .and_then(|spec| spec.get("template").or_else(|| Some(spec)))
        .and_then(|template_or_spec| template_or_spec.get("spec"))
        .and_then(|spec| spec.get("containers"))
        .and_then(|containers| containers.as_sequence())
}