use crate::candidate::{Candidate, CandidateList};
use crate::config::{Config, ScreeningConfig};
use crate::screening;
use rocket::serde::json::Json;
use rocket::{get, post, routes, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

pub struct AppState {
    pub config: Config,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    service: &'static str,
}

#[derive(Deserialize)]
struct ScreenRequest {
    position: Option<String>,
    required_skills: Option<Vec<String>>,
    candidates: Vec<Candidate>,
    screening: Option<ScreeningConfig>,
}

#[get("/health")]
fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        service: "candidate-screener",
    })
}

#[get("/ready")]
fn ready() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ready": true }))
}

#[post("/screen", data = "<req>")]
#[instrument(skip(req, state))]
fn screen(req: Json<ScreenRequest>, state: &State<Arc<AppState>>) -> Json<serde_json::Value> {
    let list = CandidateList {
        position: req.position.clone(),
        required_skills: req.required_skills.clone(),
        candidates: req.candidates.clone(),
    };
    let cfg = req.screening.clone().unwrap_or_else(|| state.config.screening.clone());
    let result = screening::screen(&list, &cfg);
    Json(serde_json::to_value(&result).unwrap_or_default())
}

pub async fn launch(config: Config) -> anyhow::Result<()> {
    let port = config.server.port;
    let host = config.server.host.clone();
    let state = Arc::new(AppState { config });

    let figment = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", host))
        .merge(("log_level", "critical"));

    rocket::custom(figment)
        .manage(state)
        .mount("/", routes![health, ready, screen])
        .launch()
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("rocket error: {}", e))
}
