mod commands;
mod exit_codes;
mod project;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// AA Engine command-line interface for agents and developers.
#[derive(Parser)]
#[command(name = "aa", version, about = "AA Engine CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run `cargo check` on a project or workspace package.
    Check {
        /// Project directory (contains `Cargo.toml` or `aa.project.toml`).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Emit structured JSON to stdout (human text still goes to stderr).
        #[arg(long)]
        json: bool,
    },
    /// Validate project manifest, config files, RON assets, and prefab refs.
    Validate {
        /// Game project root (contains `aa.project.toml`).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: `json` writes validation report to stdout.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Run a game binary or print launch instructions.
    Run {
        /// Game project root.
        #[arg(long)]
        project: PathBuf,
        /// Runtime role: client, dedicated_server, or editor.
        #[arg(long, default_value = "client")]
        role: String,
    },
    /// Run an automated headless playtest scenario.
    Playtest {
        /// Game project root.
        #[arg(long, default_value = "examples/demo_game")]
        project: PathBuf,
        /// Scenario id (maps to `assets/playtests/<scenario>.ron`).
        #[arg(long, default_value = "smoke")]
        scenario: String,
        /// Simulated run duration in seconds.
        #[arg(long, default_value_t = 12)]
        duration: u32,
        /// Emit JSON report to stdout.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, clap::ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Check { path, json } => commands::check::run(&path, json),
        Commands::Validate { path, format } => {
            commands::validate::run(&path, format == OutputFormat::Json)
        }
        Commands::Run { project, role } => commands::run::run(&project, &role),
        Commands::Playtest {
            project,
            scenario,
            duration,
            json,
        } => commands::playtest::run(&project, &scenario, duration, json),
    };
    std::process::exit(code.as_i32());
}
