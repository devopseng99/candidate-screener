use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub screening: ScreeningConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub color: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScreeningConfig {
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    #[serde(default)]
    pub weights: CriteriaWeights,
    #[serde(default = "default_top_n")]
    pub top_n: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CriteriaWeights {
    #[serde(default = "default_weight")]
    pub experience: f64,
    #[serde(default = "default_weight")]
    pub skills: f64,
    #[serde(default = "default_weight")]
    pub education: f64,
    #[serde(default = "default_weight")]
    pub cultural_fit: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    pub otlp_endpoint: Option<String>,
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

fn default_log_level() -> String { "warn".to_string() }
fn default_format() -> String { "table".to_string() }
fn default_true() -> bool { true }
fn default_min_score() -> f64 { 0.5 }
fn default_top_n() -> usize { 10 }
fn default_weight() -> f64 { 0.25 }
fn default_port() -> u16 { 8080 }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_service_name() -> String { "candidate-screener".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            output: OutputConfig::default(),
            screening: ScreeningConfig::default(),
            server: ServerConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { format: default_format(), color: true }
    }
}

impl Default for ScreeningConfig {
    fn default() -> Self {
        Self {
            min_score: default_min_score(),
            weights: CriteriaWeights::default(),
            top_n: default_top_n(),
        }
    }
}

impl Default for CriteriaWeights {
    fn default() -> Self {
        Self { experience: 0.25, skills: 0.35, education: 0.2, cultural_fit: 0.2 }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: default_port(), host: default_host() }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { enabled: false, otlp_endpoint: None, service_name: default_service_name() }
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file: {}", path.display()))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .with_context(|| "parsing YAML config"),
        "toml" => toml::from_str(&content)
            .with_context(|| "parsing TOML config"),
        _ => anyhow::bail!("unsupported config format: {}", ext),
    }
}
