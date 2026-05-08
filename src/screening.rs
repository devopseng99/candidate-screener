use crate::candidate::{Candidate, CandidateList};
use crate::config::ScreeningConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoredCandidate {
    pub candidate: Candidate,
    pub score: f64,
    pub breakdown: ScoreBreakdown,
    pub passed: bool,
    pub rank: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreBreakdown {
    pub experience: f64,
    pub skills: f64,
    pub education: f64,
    pub cultural_fit: f64,
}

#[derive(Debug, Serialize)]
pub struct ScreeningResult {
    pub position: Option<String>,
    pub total_candidates: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<ScoredCandidate>,
}

pub fn screen(list: &CandidateList, cfg: &ScreeningConfig) -> ScreeningResult {
    let required_skills: Vec<String> = list
        .required_skills
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let mut scored: Vec<ScoredCandidate> = list
        .candidates
        .iter()
        .map(|c| score_candidate(c, &required_skills, cfg))
        .collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let top_n = cfg.top_n.min(scored.len());
    scored = scored.into_iter().take(top_n).collect();

    for (i, s) in scored.iter_mut().enumerate() {
        s.rank = i + 1;
    }

    let passed = scored.iter().filter(|s| s.passed).count();
    let failed = scored.len() - passed;

    ScreeningResult {
        position: list.position.clone(),
        total_candidates: list.candidates.len(),
        passed,
        failed,
        results: scored,
    }
}

fn score_candidate(
    candidate: &Candidate,
    required_skills: &[String],
    cfg: &ScreeningConfig,
) -> ScoredCandidate {
    let w = &cfg.weights;

    let experience_score = normalize_experience(candidate.experience_years);
    let skills_score = score_skills(&candidate.skills, required_skills);
    let education_score = candidate.education.score();
    let cultural_fit_score = candidate.cultural_fit_score.unwrap_or(0.5);

    let total = w.experience * experience_score
        + w.skills * skills_score
        + w.education * education_score
        + w.cultural_fit * cultural_fit_score;

    let weight_sum = w.experience + w.skills + w.education + w.cultural_fit;
    let normalized = if weight_sum > 0.0 { total / weight_sum } else { 0.0 };

    ScoredCandidate {
        passed: normalized >= cfg.min_score,
        score: (normalized * 100.0).round() / 100.0,
        breakdown: ScoreBreakdown {
            experience: (experience_score * 100.0).round() / 100.0,
            skills: (skills_score * 100.0).round() / 100.0,
            education: (education_score * 100.0).round() / 100.0,
            cultural_fit: (cultural_fit_score * 100.0).round() / 100.0,
        },
        candidate: candidate.clone(),
        rank: 0,
    }
}

fn normalize_experience(years: f64) -> f64 {
    // Sigmoid-like curve: 0yr=0, 5yr=0.7, 10yr=0.9, 15yr+=1.0
    (years / (years + 3.0)).min(1.0)
}

fn score_skills(candidate_skills: &[String], required: &[String]) -> f64 {
    if required.is_empty() {
        return 0.7;
    }
    let lower: Vec<String> = candidate_skills.iter().map(|s| s.to_lowercase()).collect();
    let matched = required.iter().filter(|r| lower.iter().any(|s| s.contains(r.as_str()))).count();
    matched as f64 / required.len() as f64
}
