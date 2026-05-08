mod candidate;
mod config;
mod error;
mod output;
mod screening;
mod server;
mod telemetry;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, instrument};

#[derive(Parser)]
#[command(
    name = "candidate-screener",
    version = env!("CARGO_PKG_VERSION"),
    about = "HR candidate screening and ranking CLI",
    long_about = "Screen and rank job candidates based on configurable weighted criteria.\n\nExit codes:\n  0 — success\n  1 — error\n  2 — no candidates passed screening"
)]
struct Cli {
    #[arg(short, long, global = true, value_name = "FILE", help = "Config file (YAML or TOML)")]
    config: Option<PathBuf>,

    #[arg(long, global = true, help = "Disable colored output")]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Screen candidates from a file and output ranked results
    Screen {
        #[arg(short, long, value_name = "FILE", help = "Candidates file (YAML, TOML, or JSON)")]
        input: PathBuf,

        #[arg(short, long, default_value = "table", help = "Output format: table, text, json")]
        format: String,

        #[arg(long, help = "Minimum score threshold (0.0-1.0), overrides config")]
        min_score: Option<f64>,

        #[arg(long, default_value = "10", help = "Number of top candidates to show")]
        top_n: usize,

        #[arg(long, value_name = "FILE", help = "Write JSON results to file")]
        output: Option<PathBuf>,
    },

    /// Start the HTTP API server for continuous screening
    Serve {
        #[arg(short, long, default_value = "8080", help = "Port to listen on")]
        port: u16,

        #[arg(long, default_value = "0.0.0.0", help = "Host to bind")]
        host: String,
    },

    /// Show version and build information
    Version,
}

#[tokio::main]
async fn main() {
    let exit_code = run().await;
    std::process::exit(exit_code);
}

async fn run() -> i32 {
    let cli = Cli::parse();

    let mut cfg = if let Some(path) = &cli.config {
        match config::load(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: {e}");
                return 1;
            }
        }
    } else {
        config::Config::default()
    };

    if cli.no_color {
        cfg.output.color = false;
    }

    let otlp_from_env = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    let otlp_endpoint: Option<String> = if cfg.telemetry.enabled {
        cfg.telemetry.otlp_endpoint.clone().or(otlp_from_env)
    } else {
        None
    };

    let _guard = match telemetry::init(&cfg.telemetry.service_name, otlp_endpoint.as_deref(), &cfg.log_level) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("warn: telemetry init failed: {e}");
            return run_command(cli.command, cfg).await;
        }
    };

    run_command(cli.command, cfg).await
}

#[instrument(skip(command, cfg))]
async fn run_command(command: Commands, mut cfg: config::Config) -> i32 {
    match command {
        Commands::Version => {
            println!("candidate-screener {}", env!("CARGO_PKG_VERSION"));
            println!("built with Rust {}", env!("CARGO_PKG_VERSION"));
            0
        }

        Commands::Screen { input, format, min_score, top_n, output } => {
            if let Some(ms) = min_score {
                cfg.screening.min_score = ms;
            }
            cfg.screening.top_n = top_n;

            let fmt = format.as_str().to_string();
            let use_color = cfg.output.color;

            info!(input = %input.display(), format = %fmt, "screening candidates");

            let list = match candidate::load(&input) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("error loading candidates: {e}");
                    return 1;
                }
            };

            let result = screening::screen(&list, &cfg.screening);
            output::print_results(&result, &fmt, use_color);

            if let Some(out_path) = output {
                match serde_json::to_string_pretty(&result) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(&out_path, json) {
                            eprintln!("error writing output: {e}");
                            return 1;
                        }
                        eprintln!("results written to {}", out_path.display());
                    }
                    Err(e) => {
                        eprintln!("error serializing results: {e}");
                        return 1;
                    }
                }
            }

            if result.passed == 0 { 2 } else { 0 }
        }

        Commands::Serve { port, host } => {
            cfg.server.port = port;
            cfg.server.host = host;
            eprintln!("starting server on {}:{}", cfg.server.host, cfg.server.port);
            eprintln!("  GET  /health");
            eprintln!("  GET  /ready");
            eprintln!("  POST /screen");

            match server::launch(cfg).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("server error: {e}");
                    1
                }
            }
        }
    }
}
