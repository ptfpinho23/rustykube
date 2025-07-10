# 🚀 RustyKube

**Blazing fast CLI tool for Kubernetes resource management through linting, optimization & validation**

RustyKube is a comprehensive Kubernetes manifest analysis tool built in Rust, designed to help developers and DevOps engineers maintain high-quality, secure, and optimized Kubernetes deployments.

## ✨ Features

- 🔍 **Intelligent Linting** - Advanced rule-based analysis with customizable rule sets
- ✅ **Manifest Validation** - Schema validation and syntax checking
- ⚡ **Performance Optimization** - Automated resource optimization with safety checks
- 🔧 **Auto-Fix Capabilities** - Automatically fix common issues with dry-run support
- 📊 **Detailed Analysis** - Comprehensive insights with scoring and recommendations
- 🎯 **Multi-format Output** - Human-readable and JSON output formats
- 🛡️ **Security Focus** - Built-in security best practices validation

## 🚀 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-org/rustykube.git
cd rustykube

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Using Cargo

```bash
cargo install rustykube
```

## 🎯 Quick Start

```bash
# Lint Kubernetes manifests
rustykube lint -p ./k8s-manifests

# Validate syntax and schema
rustykube validate -p ./deployment.yaml

# Optimize manifests
rustykube optimize -p ./manifests --output ./optimized

# Auto-fix common issues
rustykube fix -p ./manifests --dry-run

# Comprehensive analysis
rustykube analyze -p ./manifests --detailed
```

## 📚 Commands

### `lint` - Lint Kubernetes Manifests

Analyze manifests for best practices, security issues, and potential problems.

```bash
rustykube lint [OPTIONS]

OPTIONS:
    -p, --path <PATH>        Path to manifest file or directory
    -j, --json              Output in JSON format
    -s, --strict            Exit with error code on any issues
    -r, --rules <RULES>     Comma-separated list of specific rules to run
    -v, --verbose           Enable verbose output

EXAMPLES:
    rustykube lint -p ./manifests
    rustykube lint -p deployment.yaml --json --strict
    rustykube lint -p ./k8s --rules "security,resource-limits" -v
```

#### Available Lint Rules

- `missing-labels` - Check for recommended Kubernetes labels
- `resource-limits` - Verify CPU and memory resource limits
- `liveness-probe` - Ensure liveness probes are configured
- `readiness-probe` - Ensure readiness probes are configured
- `run-as-non-root` - Verify containers run as non-root user
- `read-only-root-filesystem` - Check for read-only root filesystem
- `latest-image-tag` - Flag usage of 'latest' image tags

### `validate` - Validate Manifest Syntax and Schema

Validate Kubernetes manifests for syntax errors and schema compliance.

```bash
rustykube validate [OPTIONS]

OPTIONS:
    -p, --path <PATH>           Path to manifest file or directory
    --api-version <VERSION>     Kubernetes API version to validate against [default: 1.28]
    -j, --json                  Output in JSON format
    -v, --verbose               Enable verbose output

EXAMPLES:
    rustykube validate -p ./manifests
    rustykube validate -p deployment.yaml --api-version 1.29
```

### `optimize` - Optimize Manifests for Performance

Automatically optimize Kubernetes manifests for better performance and resource usage.

```bash
rustykube optimize [OPTIONS]

OPTIONS:
    -p, --path <PATH>       Path to manifest file or directory
    -o, --output <DIR>      Output directory for optimized manifests
    --in-place              Apply optimizations in-place (overwrites original files)
    --aggressive            Enable aggressive optimizations (may change behavior)
    -v, --verbose           Enable verbose output

EXAMPLES:
    rustykube optimize -p ./manifests -o ./optimized
    rustykube optimize -p deployment.yaml --in-place
    rustykube optimize -p ./k8s --aggressive
```

#### Optimization Features

- **Resource Management** - Add missing resource requests and limits
- **Security Hardening** - Apply security best practices
- **Label Standardization** - Add recommended Kubernetes labels
- **Strategy Configuration** - Add deployment strategies
- **Performance Tuning** - Optimize image pull policies and other settings

### `fix` - Auto-Fix Common Issues

Automatically fix common issues found in Kubernetes manifests.

```bash
rustykube fix [OPTIONS]

OPTIONS:
    -p, --path <PATH>       Path to manifest file or directory
    --in-place              Apply fixes in-place (overwrites original files)
    --dry-run               Show what would be fixed without making changes
    -v, --verbose           Enable verbose output

EXAMPLES:
    rustykube fix -p ./manifests --dry-run
    rustykube fix -p deployment.yaml --in-place
```

#### Auto-Fix Capabilities

- Add missing labels and selectors
- Configure resource requests and limits
- Add health probes (liveness and readiness)
- Configure security contexts
- Fix image tag issues
- Add missing required fields

### `analyze` - Comprehensive Manifest Analysis

Perform detailed analysis with insights, scoring, and recommendations.

```bash
rustykube analyze [OPTIONS]

OPTIONS:
    -p, --path <PATH>       Path to manifest file or directory
    --detailed              Generate detailed report with recommendations
    -j, --json              Output in JSON format
    -v, --verbose           Enable verbose output

EXAMPLES:
    rustykube analyze -p ./manifests --detailed
    rustykube analyze -p deployment.yaml --json
```

#### Analysis Metrics

- **Security Score** (0-100) - Security posture assessment
- **Performance Score** (0-100) - Resource and performance optimization
- **Reliability Score** (0-100) - High availability and resilience
- **Complexity Score** (0-100) - Configuration complexity assessment

## 🛡️ Security Best Practices

RustyKube enforces industry-standard security practices:

- **Non-root containers** - Ensures containers don't run as root
- **Read-only filesystems** - Prevents runtime file modifications
- **Security contexts** - Validates proper security context configuration
- **Image tags** - Flags usage of mutable tags like 'latest'
- **Privilege escalation** - Prevents container privilege escalation
- **Capabilities** - Validates capability dropping

## 📊 Output Formats

### Human-Readable Output

```
🔍 Linting Results
==================================================
📄 Resource: web-app (Deployment)
  ✅ No issues found

📄 Resource: web-app-service (Service)
  ❌ [missing-labels] Service missing recommended labels

==================================================
📊 Summary
  Resources analyzed: 2
  Total issues found: 1
  ⚠️  1 issue(s) need attention
```

### JSON Output

```json
{
  "summary": {
    "total_resources": 2,
    "total_issues": 1,
    "rules_checked": 7
  },
  "results": [
    {
      "resource": "web-app (Deployment)",
      "issues": [],
      "issue_count": 0
    },
    {
      "resource": "web-app-service (Service)",
      "issues": [
        "[missing-labels] Service missing recommended labels"
      ],
      "issue_count": 1
    }
  ]
}
```

## 🎛️ Configuration

### Global Options

- `-v, --verbose` - Enable verbose output for all commands
- `-h, --help` - Show help information
- `-V, --version` - Show version information

### Environment Variables

- `RUSTYKUBE_LOG_LEVEL` - Set logging level (debug, info, warn, error)
- `RUSTYKUBE_CONFIG_PATH` - Path to configuration file

## 📁 Project Structure

```
rustykube/
├── src/
│   ├── commands/          # CLI command implementations
│   │   ├── lint.rs       # Linting functionality
│   │   ├── validate.rs   # Validation functionality
│   │   ├── optimize.rs   # Optimization functionality
│   │   ├── fix.rs        # Auto-fix functionality
│   │   └── analyze.rs    # Analysis functionality
│   ├── lint_rules/       # Linting rule implementations
│   │   ├── security.rs   # Security-related rules
│   │   ├── health_checks.rs # Health probe rules
│   │   ├── resource_limits.rs # Resource management rules
│   │   └── ...
│   ├── utils.rs          # Utility functions
│   └── main.rs           # CLI entry point
├── samples/              # Example Kubernetes manifests
├── tests/               # Test files
└── Cargo.toml          # Rust dependencies
```

## 🧪 Examples

### Example 1: Basic Linting

```bash
# Lint a single deployment file
rustykube lint -p deployment.yaml

# Output:
# 🔍 Linting Results
# ==================================================
# 📄 Resource: web-app (Deployment)
#   ❌ [resource-limits] Container 'web-container' missing CPU limits
#   ❌ [security] Container 'web-container' not running as non-root
# 
# ==================================================
# 📊 Summary
#   Resources analyzed: 1
#   Total issues found: 2
```

### Example 2: Comprehensive Analysis

```bash
# Analyze with detailed insights
rustykube analyze -p ./k8s-manifests --detailed

# Output includes:
# - Security, Performance, Reliability scores
# - Resource usage analysis
# - Detailed recommendations
# - Best practice suggestions
```

### Example 3: Optimization Workflow

```bash
# 1. Analyze current state
rustykube analyze -p ./manifests

# 2. Check what optimizations would be applied
rustykube optimize -p ./manifests --output ./preview

# 3. Apply optimizations
rustykube optimize -p ./manifests --in-place

# 4. Verify improvements
rustykube lint -p ./manifests
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-org/rustykube.git
cd rustykube

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and test
cargo build
cargo test

# Run linting
cargo clippy

# Run formatting
cargo fmt
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Kubernetes community for excellent documentation and best practices
- Rust community for amazing tooling and ecosystem
- All contributors who help make RustyKube better

## 🔗 Links

- [Documentation](https://rustykube.dev/docs)
- [GitHub Repository](https://github.com/your-org/rustykube)
- [Issue Tracker](https://github.com/your-org/rustykube/issues)
- [Discussions](https://github.com/your-org/rustykube/discussions)

---

**Made with ❤️ and ⚡ Rust**