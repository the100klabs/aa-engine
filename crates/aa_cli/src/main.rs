mod commands;
mod exit_codes;
mod project;
mod ron_subset;
mod schema_subset;

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
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Validate project manifest, config files, RON assets, and prefab refs.
    Validate {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Query project assets and specs for agent indexing.
    Index {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long)]
        scope: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Run a game binary or print launch instructions.
    Run {
        #[arg(long)]
        project: PathBuf,
        #[arg(long, default_value = "client")]
        role: String,
    },
    /// Run an automated headless playtest scenario.
    Playtest {
        #[arg(long, default_value = "examples/demo_game")]
        project: PathBuf,
        #[arg(long, default_value = "smoke")]
        scenario: String,
        #[arg(long, default_value_t = 12)]
        duration: u32,
        #[arg(long)]
        json: bool,
    },
    /// Open-world authoring and streaming commands.
    World {
        #[command(subcommand)]
        command: WorldCommands,
    },
    /// Scene list/inspect/patch commands for agent authoring.
    Scene {
        #[command(subcommand)]
        command: SceneCommands,
    },
    /// Agent eval suite commands.
    Eval {
        #[command(subcommand)]
        command: EvalCommands,
    },
    /// Profile summarize commands for streaming traces.
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
}

#[derive(Subcommand)]
enum WorldCommands {
    Inspect {
        #[arg(long, default_value = "examples/open_world_studio")]
        project: PathBuf,
        #[arg(long)]
        world: String,
        #[arg(long)]
        live: bool,
        #[arg(long)]
        json: bool,
    },
    Cook {
        #[arg(long, default_value = "examples/open_world_studio")]
        project: PathBuf,
        #[arg(long)]
        world: String,
        #[arg(long)]
        verify: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SceneCommands {
    List {
        #[arg(long)]
        scene: String,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Inspect {
        entity_id: String,
        #[arg(long)]
        scene: String,
        #[arg(long)]
        json: bool,
    },
    Patch {
        #[arg(long)]
        scene: String,
        #[arg(long)]
        patch: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum EvalCommands {
    List {
        #[arg(long)]
        json: bool,
    },
    Run {
        eval_id_or_path: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    Summarize {
        artifact_path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, clap::ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Sarif,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Check { path, json } => commands::check::run(&path, json),
        Commands::Validate { path, format } => commands::validate::run(
            &path,
            format == OutputFormat::Json,
            format == OutputFormat::Sarif,
        ),
        Commands::Index {
            path,
            query,
            scope,
            json,
        } => commands::index::run(&path, &query, scope.as_deref(), json),
        Commands::Run { project, role } => commands::run::run(&project, &role),
        Commands::Playtest {
            project,
            scenario,
            duration,
            json,
        } => commands::playtest::run(&project, &scenario, duration, json),
        Commands::World { command } => match command {
            WorldCommands::Inspect {
                project,
                world,
                live,
                json,
            } => commands::world::inspect(&project, &world, live, json),
            WorldCommands::Cook {
                project,
                world,
                verify,
                json,
            } => commands::world::cook(&project, &world, verify, json),
        },
        Commands::Scene { command } => match command {
            SceneCommands::List {
                scene,
                filter,
                json,
            } => commands::scene::list(&scene, filter.as_deref(), json),
            SceneCommands::Inspect {
                entity_id,
                scene,
                json,
            } => commands::scene::inspect(&scene, &entity_id, json),
            SceneCommands::Patch {
                scene,
                patch,
                dry_run,
                json,
            } => commands::scene::patch(&scene, &patch, dry_run, json),
        },
        Commands::Eval { command } => match command {
            EvalCommands::List { json } => commands::eval::list(json),
            EvalCommands::Run {
                eval_id_or_path,
                json,
            } => commands::eval::run_eval(&eval_id_or_path, json),
        },
        Commands::Profile { command } => match command {
            ProfileCommands::Summarize {
                artifact_path,
                json,
            } => commands::profile::summarize(&artifact_path, json),
        },
    };
    std::process::exit(code.as_i32());
}
