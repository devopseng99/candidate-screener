use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Candidate {
    pub id: Option<String>,
    pub name: String,
    pub email: Option<String>,
    pub position: Option<String>,
    pub experience_years: f64,
    pub skills: Vec<String>,
    pub education: Education,
    pub cultural_fit_score: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Education {
    None,
    HighSchool,
    Associate,
    Bachelor,
    Master,
    Phd,
}

impl Education {
    pub fn score(&self) -> f64 {
        match self {
            Education::None => 0.1,
            Education::HighSchool => 0.3,
            Education::Associate => 0.5,
            Education::Bachelor => 0.7,
            Education::Master => 0.9,
            Education::Phd => 1.0,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Education::None => "None",
            Education::HighSchool => "High School",
            Education::Associate => "Associate",
            Education::Bachelor => "Bachelor's",
            Education::Master => "Master's",
            Education::Phd => "PhD",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateList {
    pub position: Option<String>,
    pub required_skills: Option<Vec<String>>,
    pub candidates: Vec<Candidate>,
}

pub fn load(path: &Path) -> Result<CandidateList> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading candidates file: {}", path.display()))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .with_context(|| "parsing YAML candidates"),
        "toml" => toml::from_str(&content)
            .with_context(|| "parsing TOML candidates"),
        "json" => serde_json::from_str(&content)
            .with_context(|| "parsing JSON candidates"),
        _ => anyhow::bail!("unsupported candidates format: {}", ext),
    }
}
