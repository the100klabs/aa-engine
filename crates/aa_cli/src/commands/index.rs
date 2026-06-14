use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use walkdir::WalkDir;

use crate::exit_codes::ExitCode;
use crate::project::{self, ProjectError};

#[derive(Debug, Serialize)]
struct IndexHit {
    id: String,
    kind: String,
    path: String,
    title: String,
    score: f32,
    summary: String,
    span: IndexSpan,
    relations: Vec<String>,
    tags: Vec<String>,
    stale: bool,
}

#[derive(Debug, Serialize)]
struct IndexSpan {
    line_start: u32,
}

#[derive(Debug, Serialize)]
struct IndexWarning {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Debug, Serialize)]
struct IndexResult {
    ok: bool,
    query: String,
    duration_ms: u64,
    generated_at: String,
    index_version: String,
    hits: Vec<IndexHit>,
    warnings: Vec<IndexWarning>,
}

/// Query project files for agent index results.
pub fn run(path: &Path, query: &str, scope: Option<&Path>, json: bool) -> ExitCode {
    let started = Instant::now();
    let project_root = match project::resolve_project_root(path) {
        Ok(root) => root,
        Err(ProjectError::ManifestMissing(p)) => {
            eprintln!("error: aa.project.toml not found at {}", p.display());
            return ExitCode::InvalidArgs;
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::InternalError;
        }
    };

    let terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

    let mut warnings = Vec::new();
    if terms.is_empty() {
        warnings.push(IndexWarning {
            code: "EMPTY_QUERY".into(),
            message: "Query had no searchable terms".into(),
            path: None,
        });
    }

    let scan_root = scope
        .map(|s| project_root.join(s))
        .unwrap_or_else(|| project_root.clone());

    let mut hits = Vec::new();
    for entry in WalkDir::new(&scan_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "ron" | "toml" | "md" | "json" | "rs") {
            continue;
        }
        let rel = file_path
            .strip_prefix(&project_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(file_path) else {
            continue;
        };
        let lowered_path = rel.to_lowercase();
        let lowered_text = text.to_lowercase();
        let path_score: usize = terms.iter().filter(|t| lowered_path.contains(t.as_str())).count();
        let text_score: usize = terms
            .iter()
            .map(|t| lowered_text.matches(t.as_str()).count())
            .sum();
        if !terms.is_empty() && path_score == 0 && text_score == 0 {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        let mut best_line_number = 1u32;
        let mut best_line = lines.first().copied().unwrap_or(&rel);
        let mut best_line_score = 0usize;
        for (index, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            let line_score: usize = terms
                .iter()
                .map(|t| line_lower.matches(t.as_str()).count())
                .sum();
            if line_score > best_line_score {
                best_line_score = line_score;
                best_line_number = (index + 1) as u32;
                best_line = line;
            }
        }
        let normalized = (text_score + path_score * 2) as f32 / terms.len().max(1) as f32;
        let score = (normalized / 8.0).min(1.0);
        if score <= 0.0 {
            continue;
        }
        let title = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("asset")
            .to_string();
        hits.push(IndexHit {
            id: format!("{rel}:{best_line_number}"),
            kind: classify_hit(&rel),
            path: rel.clone(),
            title,
            score: (score * 10000.0).round() / 10000.0,
            summary: summarize_line(best_line),
            span: IndexSpan {
                line_start: best_line_number,
            },
            relations: extract_relations(best_line),
            tags: terms
                .iter()
                .filter(|t| lowered_text.contains(t.as_str()) || lowered_path.contains(t.as_str()))
                .cloned()
                .collect(),
            stale: false,
        });
    }

    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    hits.truncate(20);

    let result = IndexResult {
        ok: true,
        query: query.to_string(),
        duration_ms: started.elapsed().as_millis() as u64,
        generated_at: chrono_lite_now(),
        index_version: "rust-1".into(),
        hits,
        warnings,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
        eprintln!("index: {} hits for '{query}'", result.hits.len());
        for hit in &result.hits {
            eprintln!("  {:.2} {} — {}", hit.score, hit.path, hit.summary);
        }
    }
    ExitCode::Success
}

fn classify_hit(path: &str) -> String {
    if path.contains("/sectors/") {
        "sector".into()
    } else if path.contains("/spawn_tables/") {
        "spawn_table".into()
    } else if path.contains("/worlds/") {
        "world".into()
    } else if path.ends_with(".md") {
        "spec".into()
    } else {
        "asset".into()
    }
}

fn summarize_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.len() > 120 {
        format!("{}…", &trimmed[..120])
    } else {
        trimmed.to_string()
    }
}

fn extract_relations(line: &str) -> Vec<String> {
    let mut relations = Vec::new();
    for token in line.split(|c: char| !c.is_alphanumeric() && c != '/' && c != '_' && c != '-' && c != '.') {
        if token.contains("assets/") {
            relations.push(token.to_string());
        }
    }
    relations.sort();
    relations.dedup();
    relations
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}Z")
}
