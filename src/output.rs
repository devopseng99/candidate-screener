use crate::screening::ScreeningResult;
use colored::Colorize;
use comfy_table::{Table, Row, Cell, Color, Attribute};

pub fn print_results(result: &ScreeningResult, format: &str, use_color: bool) {
    if !use_color {
        colored::control::set_override(false);
    }

    match format {
        "json" => print_json(result),
        "table" => print_table(result),
        _ => print_text(result),
    }
}

fn print_json(result: &ScreeningResult) {
    println!("{}", serde_json::to_string_pretty(result).unwrap_or_default());
}

fn print_text(result: &ScreeningResult) {
    if let Some(pos) = &result.position {
        println!("Position: {}", pos.bold());
    }
    println!(
        "Candidates: {} total, {} passed, {} failed\n",
        result.total_candidates,
        result.passed.to_string().green(),
        result.failed.to_string().red()
    );

    for s in &result.results {
        let status = if s.passed { "PASS".green().bold() } else { "FAIL".red().bold() };
        let score_str = format!("{:.0}%", s.score * 100.0);
        let score_colored = if s.score >= 0.75 {
            score_str.green()
        } else if s.score >= 0.5 {
            score_str.yellow()
        } else {
            score_str.red()
        };
        println!(
            "#{} {} [{}] {} — exp:{:.0}% skills:{:.0}% edu:{:.0}% fit:{:.0}%",
            s.rank,
            s.candidate.name.bold(),
            status,
            score_colored,
            s.breakdown.experience * 100.0,
            s.breakdown.skills * 100.0,
            s.breakdown.education * 100.0,
            s.breakdown.cultural_fit * 100.0,
        );
    }
}

fn print_table(result: &ScreeningResult) {
    if let Some(pos) = &result.position {
        println!("Position: {}", pos.bold());
    }
    println!(
        "Candidates: {} total  {}  {}",
        result.total_candidates,
        format!("{} passed", result.passed).green(),
        format!("{} failed", result.failed).red()
    );
    println!();

    let mut table = Table::new();
    table.set_header(Row::from(vec![
        Cell::new("Rank").add_attribute(Attribute::Bold),
        Cell::new("Name").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("Score").add_attribute(Attribute::Bold),
        Cell::new("Experience").add_attribute(Attribute::Bold),
        Cell::new("Skills").add_attribute(Attribute::Bold),
        Cell::new("Education").add_attribute(Attribute::Bold),
        Cell::new("Cultural Fit").add_attribute(Attribute::Bold),
        Cell::new("Edu Level").add_attribute(Attribute::Bold),
    ]));

    for s in &result.results {
        let score_pct = format!("{:.0}%", s.score * 100.0);
        let status_cell = if s.passed {
            Cell::new("PASS").fg(Color::Green)
        } else {
            Cell::new("FAIL").fg(Color::Red)
        };
        let score_cell = if s.score >= 0.75 {
            Cell::new(&score_pct).fg(Color::Green)
        } else if s.score >= 0.5 {
            Cell::new(&score_pct).fg(Color::Yellow)
        } else {
            Cell::new(&score_pct).fg(Color::Red)
        };

        table.add_row(Row::from(vec![
            Cell::new(s.rank),
            Cell::new(&s.candidate.name),
            status_cell,
            score_cell,
            Cell::new(format!("{:.0}%", s.breakdown.experience * 100.0)),
            Cell::new(format!("{:.0}%", s.breakdown.skills * 100.0)),
            Cell::new(format!("{:.0}%", s.breakdown.education * 100.0)),
            Cell::new(format!("{:.0}%", s.breakdown.cultural_fit * 100.0)),
            Cell::new(s.candidate.education.label()),
        ]));
    }

    println!("{table}");
}
