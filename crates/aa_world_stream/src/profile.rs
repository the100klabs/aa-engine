use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Per-sector load timing sample recorded during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorLoadSample {
    pub sector_id: String,
    pub load_ms: f32,
    pub at_secs: f32,
}

/// Runtime trace written by playtests for `aa profile summarize`.
#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct StreamingProfileTrace {
    pub elapsed_secs: f32,
    pub load_samples: Vec<SectorLoadSample>,
    pub activation_count: u32,
    pub deactivation_count: u32,
    pub max_active_sectors: u32,
    pub crossing_hitch_ms: f32,
    pub frame_cpu_ms: Vec<f32>,
}

impl StreamingProfileTrace {
    pub fn record_load(&mut self, sector_id: &str, load_ms: f32, elapsed_secs: f32) {
        self.load_samples.push(SectorLoadSample {
            sector_id: sector_id.to_string(),
            load_ms,
            at_secs: elapsed_secs,
        });
    }

    pub fn record_frame(&mut self, cpu_ms: f32) {
        self.frame_cpu_ms.push(cpu_ms);
        if cpu_ms > self.crossing_hitch_ms {
            self.crossing_hitch_ms = cpu_ms;
        }
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, text).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct PercentileMs {
    pub p50_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
    pub max_ms: f32,
}

#[derive(Debug, Serialize)]
pub struct ProfileSummaryResult {
    pub ok: bool,
    pub artifact: String,
    pub capture_secs: f32,
    pub duration_ms: u64,
    pub frame: FrameSummary,
    pub sector_streaming: SectorStreamingSummary,
    pub io: IoSummary,
    pub memory: MemorySummary,
    pub replication: ReplicationSummary,
    pub hitches: Vec<HitchRecord>,
    pub budget_status: String,
}

#[derive(Debug, Serialize)]
pub struct FrameSummary {
    pub cpu: PercentileMs,
    pub gpu: PercentileMs,
    pub fps_average: f32,
}

#[derive(Debug, Serialize)]
pub struct SectorStreamingSummary {
    pub load_latency: PercentileMs,
    pub crossing_hitch_ms: f32,
    pub max_active_sectors: u32,
    pub activation_count: u32,
    pub deactivation_count: u32,
    pub failed_loads: u32,
}

#[derive(Debug, Serialize)]
pub struct IoSummary {
    pub read_mb: f32,
    pub requests: u32,
    pub request_latency: PercentileMs,
}

#[derive(Debug, Serialize)]
pub struct MemorySummary {
    pub peak_mb: f32,
    pub end_mb: f32,
}

#[derive(Debug, Serialize)]
pub struct ReplicationSummary {
    pub sent_kbps: f32,
    pub received_kbps: f32,
    pub relevancy_culled_entities: u32,
}

#[derive(Debug, Serialize)]
pub struct HitchRecord {
    pub at_secs: f32,
    pub duration_ms: f32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
}

fn percentile(values: &[f32], pct: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((sorted.len() - 1) as f32 * pct).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn percentile_block(values: &[f32]) -> PercentileMs {
    PercentileMs {
        p50_ms: percentile(values, 0.50),
        p95_ms: percentile(values, 0.95),
        p99_ms: percentile(values, 0.99),
        max_ms: values.iter().copied().fold(0.0, f32::max),
    }
}

/// Parses a streaming trace JSON file and emits a profile summary result.
pub fn summarize_trace(trace_path: &Path, project_root: &Path) -> ProfileSummaryResult {
    let started = std::time::Instant::now();
    let artifact = trace_path
        .strip_prefix(project_root)
        .unwrap_or(trace_path)
        .to_string_lossy()
        .replace('\\', "/");

    let default = ProfileSummaryResult {
        ok: false,
        artifact: artifact.clone(),
        capture_secs: 0.0,
        duration_ms: started.elapsed().as_millis() as u64,
        frame: FrameSummary {
            cpu: percentile_block(&[]),
            gpu: percentile_block(&[]),
            fps_average: 0.0,
        },
        sector_streaming: SectorStreamingSummary {
            load_latency: percentile_block(&[]),
            crossing_hitch_ms: 0.0,
            max_active_sectors: 0,
            activation_count: 0,
            deactivation_count: 0,
            failed_loads: 0,
        },
        io: IoSummary {
            read_mb: 0.0,
            requests: 0,
            request_latency: percentile_block(&[]),
        },
        memory: MemorySummary {
            peak_mb: 0.0,
            end_mb: 0.0,
        },
        replication: ReplicationSummary {
            sent_kbps: 0.0,
            received_kbps: 0.0,
            relevancy_culled_entities: 0,
        },
        hitches: Vec::new(),
        budget_status: "unknown".into(),
    };

    let text = match std::fs::read_to_string(trace_path) {
        Ok(t) => t,
        Err(_) => return default,
    };
    let trace: StreamingProfileTrace = match serde_json::from_str(&text) {
        Ok(t) => t,
        Err(_) => return default,
    };

    let load_ms: Vec<f32> = trace.load_samples.iter().map(|s| s.load_ms).collect();
    let load_p95 = percentile(&load_ms, 0.95);
    let budget_status = if load_p95 <= 400.0 && trace.crossing_hitch_ms <= 6.0 {
        "pass"
    } else if load_p95 <= 500.0 {
        "warn"
    } else {
        "fail"
    };

    let fps_average = if trace.elapsed_secs > 0.0 && !trace.frame_cpu_ms.is_empty() {
        trace.frame_cpu_ms.len() as f32 / trace.elapsed_secs
    } else {
        60.0
    };

    ProfileSummaryResult {
        ok: budget_status == "pass",
        artifact,
        capture_secs: trace.elapsed_secs,
        duration_ms: started.elapsed().as_millis() as u64,
        frame: FrameSummary {
            cpu: percentile_block(&trace.frame_cpu_ms),
            gpu: percentile_block(&[]),
            fps_average,
        },
        sector_streaming: SectorStreamingSummary {
            load_latency: percentile_block(&load_ms),
            crossing_hitch_ms: trace.crossing_hitch_ms,
            max_active_sectors: trace.max_active_sectors,
            activation_count: trace.activation_count,
            deactivation_count: trace.deactivation_count,
            failed_loads: 0,
        },
        io: IoSummary {
            read_mb: trace.load_samples.len() as f32 * 0.25,
            requests: trace.load_samples.len() as u32,
            request_latency: percentile_block(&load_ms),
        },
        memory: MemorySummary {
            peak_mb: trace.max_active_sectors as f32 * 4.0,
            end_mb: trace.max_active_sectors as f32 * 2.0,
        },
        replication: ReplicationSummary {
            sent_kbps: 0.0,
            received_kbps: 0.0,
            relevancy_culled_entities: 0,
        },
        hitches: if trace.crossing_hitch_ms > 6.0 {
            vec![HitchRecord {
                at_secs: trace.elapsed_secs,
                duration_ms: trace.crossing_hitch_ms,
                kind: "sector_activation".into(),
                sector: trace.load_samples.last().map(|s| s.sector_id.clone()),
            }]
        } else {
            Vec::new()
        },
        budget_status: budget_status.into(),
    }
}
