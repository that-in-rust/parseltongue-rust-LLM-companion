//! Parseltongue: Unified CLI toolkit for code analysis
//!
//! This binary provides subcommands that dispatch to the individual tools:
//! - pt01-folder-to-cozodb-streamer (Tool 1: Ingest)
//! - pt08-http-code-query-server (Tool 8: HTTP Server - primary interface)
//! - diff (Phase 6: Diff visualization between database snapshots)

use clap::{Arg, ArgMatches, Command};
use console::style;
use anyhow::Result;

// Commands module for organized subcommand implementations
mod commands;
use commands::{DiffCommandArgsPayload, execute_diff_analysis_command};

// Import traits to enable trait methods
use pt01_folder_to_cozodb_streamer::streamer::FileStreamer;

// Import HTTP server types
use pt08_http_code_query_server::{HttpServerStartupConfig, http_server_startup_runner};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("pt01-folder-to-cozodb-streamer", sub_matches)) => {
            run_folder_to_cozodb_streamer(sub_matches).await
        }
        Some(("pt08-http-code-query-server", sub_matches)) => {
            run_http_code_query_server(sub_matches).await
        }
        Some(("diff", sub_matches)) => {
            run_diff_visualization_command(sub_matches).await
        }
        _ => {
            println!("{}", style("Parseltongue CLI Toolkit").blue().bold());
            println!("{}", style("Ultra-minimalist code analysis toolkit").blue());
            println!();
            println!("Use --help for more information");
            println!();
            println!("Available commands:");
            println!("  pt01-folder-to-cozodb-streamer  - Index codebase into CozoDB");
            println!("  pt08-http-code-query-server     - HTTP server for REST API (15 endpoints)");
            println!("  diff                            - Compare two database snapshots");
            Ok(())
        }
    }
}

fn build_cli() -> Command {
    Command::new("parseltongue")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Parseltongue Team")
        .about("Ultra-minimalist CLI toolkit for code analysis")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(
            Command::new("pt01-folder-to-cozodb-streamer")
                .about("Tool 1: Stream folder contents to CozoDB with ISGL1 keys")
                .long_about(
                    "Examples:\n  \
                    parseltongue pt01-folder-to-cozodb-streamer .            # Index current directory\n  \
                    parseltongue pt01-folder-to-cozodb-streamer ./src --db rocksdb:analysis.db --verbose"
                )
                .arg(
                    Arg::new("directory")
                        .help("Directory to index [default: current directory]")
                        .default_value(".")
                        .index(1),
                )
                .arg(
                    Arg::new("db")
                        .long("db")
                        .help("Database file path")
                        .default_value("parseltongue.db"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("Enable verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .help("Suppress output")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("pt08-http-code-query-server")
                .about("Tool 8: HTTP server for code queries (REST API)")
                .long_about(
                    "Start an HTTP server exposing CozoDB queries via REST endpoints.\n\n\
                    Examples:\n  \
                    parseltongue pt08-http-code-query-server --db rocksdb:analysis.db\n  \
                    parseltongue pt08-http-code-query-server --port 7777 --db rocksdb:analysis.db"
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .short('p')
                        .help("Port to listen on [default: 7777]"),
                )
                .arg(
                    Arg::new("db")
                        .long("db")
                        .help("Database file path (rocksdb:path or mem)")
                        .default_value("mem"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("Enable verbose logging")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("diff")
                .about("Compare two database snapshots and visualize changes")
                .long_about(
                    "Compare two CozoDB snapshots (e.g., before/after code changes) and produce\n\
                    a visualization of differences and their blast radius.\n\n\
                    Examples:\n  \
                    parseltongue diff --base rocksdb:before.db --live rocksdb:after.db\n  \
                    parseltongue diff --base rocksdb:v1.db --live rocksdb:v2.db --json\n  \
                    parseltongue diff --base rocksdb:old.db --live rocksdb:new.db --max-hops 3"
                )
                .arg(
                    Arg::new("base")
                        .long("base")
                        .short('b')
                        .help("Path to base/before database (e.g., rocksdb:path/to/base.db)")
                        .required(true),
                )
                .arg(
                    Arg::new("live")
                        .long("live")
                        .short('l')
                        .help("Path to live/after database (e.g., rocksdb:path/to/live.db)")
                        .required(true),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .help("Output results as JSON")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("max-hops")
                        .long("max-hops")
                        .help("Maximum hops for blast radius calculation [default: 2]")
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
}

async fn run_folder_to_cozodb_streamer(matches: &ArgMatches) -> Result<()> {
    let directory = matches.get_one::<String>("directory").unwrap();
    let db = matches.get_one::<String>("db").unwrap();
    let verbose = matches.get_flag("verbose");
    let quiet = matches.get_flag("quiet");

    // Create timestamped workspace directory
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let workspace_dir = format!("parseltongue{}", timestamp);
    std::fs::create_dir_all(&workspace_dir)?;

    // Construct database path within workspace
    let workspace_db_path = if db == "mem" {
        "mem".to_string()
    } else {
        // Always use rocksdb with timestamped workspace
        format!("rocksdb:{}/analysis.db", workspace_dir)
    };

    println!("{}", style("Running Tool 1: folder-to-cozodb-streamer").cyan());
    if !quiet {
        println!("  Workspace: {}", style(&workspace_dir).yellow().bold());
        println!("  Database: {}", &workspace_db_path);
    }

    // Create config (S01 ultra-minimalist: let tree-sitter decide what to parse)
    let config = pt01_folder_to_cozodb_streamer::StreamerConfig {
        root_dir: std::path::PathBuf::from(directory),
        db_path: workspace_db_path.clone(),
        max_file_size: 100 * 1024 * 1024,  // 100MB - no artificial limits
        include_patterns: vec!["*".to_string()],  // ALL files - tree-sitter handles it
        exclude_patterns: vec![
            "target".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
            "build".to_string(),
            "dist".to_string(),
            "__pycache__".to_string(),
            ".venv".to_string(),
            "venv".to_string(),
        ],
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    // Create and run streamer
    let streamer = pt01_folder_to_cozodb_streamer::ToolFactory::create_streamer(config.clone()).await?;
    let result = streamer.stream_directory().await?;

    if !quiet {
        println!("{}", style("✓ Indexing completed").green().bold());
        println!("  Files processed: {}", result.processed_files);
        println!("  Entities created: {}", result.entities_created);
        println!();
        println!("{}", style("📁 Workspace location:").green().bold());
        println!("  {}", style(&workspace_dir).yellow().bold());
        println!();
        println!("{}", style("Next step:").cyan());
        println!("  parseltongue pt08-http-code-query-server \\");
        println!("    --db \"{}\"", workspace_db_path);
        println!();
        println!("{}", style("Quick test:").cyan());
        println!("  curl http://localhost:7777/server-health-check-status");
        println!("  curl http://localhost:7777/codebase-statistics-overview-summary");
        if verbose {
            println!("  Duration: {:?}", result.duration);
        }
    }

    Ok(())
}

/// Run the HTTP server for code queries
///
/// # 4-Word Name: run_http_code_query_server
async fn run_http_code_query_server(matches: &ArgMatches) -> Result<()> {
    let port = matches.get_one::<String>("port");
    let db = matches.get_one::<String>("db").unwrap();
    let verbose = matches.get_flag("verbose");

    println!("{}", style("Running Tool 8: HTTP Code Query Server").cyan());
    if verbose {
        if let Some(p) = port {
            println!("  Port: {}", p);
        } else {
            println!("  Port: 7777 (default)");
        }
        println!("  Database: {}", db);
    }

    // Build configuration
    let config = HttpServerStartupConfig {
        target_directory_path_value: std::path::PathBuf::from("."),
        database_connection_string_value: db.clone(),
        http_port_override_option: port.and_then(|p| p.parse().ok()),
        force_reindex_enabled_flag: false,
        daemon_background_mode_flag: false,
        idle_timeout_minutes_option: None,
        verbose_logging_enabled_flag: verbose,
    };

    // Start the server (blocks until shutdown)
    http_server_startup_runner::start_http_server_blocking_loop(config).await
}

/// Run the diff visualization command
///
/// # 4-Word Name: run_diff_visualization_command
async fn run_diff_visualization_command(matches: &ArgMatches) -> Result<()> {
    let base_db = matches.get_one::<String>("base").unwrap();
    let live_db = matches.get_one::<String>("live").unwrap();
    let json_output = matches.get_flag("json");
    let max_hops = matches.get_one::<u32>("max-hops").copied().unwrap_or(2);

    println!("{}", style("Running Diff Visualization Analysis").cyan());
    println!("  Base database: {}", base_db);
    println!("  Live database: {}", live_db);
    println!("  Max hops: {}", max_hops);
    if json_output {
        println!("  Output format: JSON");
    }
    println!();

    let args = DiffCommandArgsPayload {
        base_database_path_value: base_db.clone(),
        live_database_path_value: live_db.clone(),
        json_output_format_flag: json_output,
        max_hops_depth_limit: max_hops,
    };

    execute_diff_analysis_command(args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_builds() {
        let cli = build_cli();
        // Verify all subcommands are present
        let subcommands: Vec<&str> = cli.get_subcommands().map(|cmd| cmd.get_name()).collect();
        assert!(subcommands.contains(&"pt01-folder-to-cozodb-streamer")); // Ingest
        assert!(subcommands.contains(&"pt08-http-code-query-server")); // HTTP server (primary)
        assert!(subcommands.contains(&"diff")); // Phase 6: Diff visualization
    }
}
