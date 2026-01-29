//! Parseltongue: Unified CLI toolkit for code analysis
//!
//! This binary provides subcommands that dispatch to the individual tools:
//! - pt01-folder-to-cozodb-streamer (Tool 1: Ingest)
//! - pt08-http-code-query-server (Tool 8: HTTP Server - primary interface)

use clap::{Arg, ArgMatches, Command};
use console::style;
use anyhow::Result;

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
        _ => {
            println!("{}", style("Parseltongue CLI Toolkit").blue().bold());
            println!("{}", style("Ultra-minimalist code analysis toolkit").blue());
            println!();
            println!("Use --help for more information");
            println!();
            println!("Available commands:");
            println!("  pt01-folder-to-cozodb-streamer       - Index codebase into CozoDB");
            println!("  pt08-http-code-query-server              - HTTP server for REST API (15 endpoints)");
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
                    parseltongue pt08-http-code-query-server --port 7777 --db rocksdb:analysis.db\n  \
                    parseltongue pt08-http-code-query-server --db rocksdb:analysis.db --watch\n  \
                    parseltongue pt08-http-code-query-server --db rocksdb:analysis.db --watch --watch-dir ./src"
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
                )
                .arg(
                    Arg::new("watch")
                        .long("watch")
                        .short('w')
                        .help("Enable file watching for automatic reindex on changes")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("watch-dir")
                        .long("watch-dir")
                        .value_name("PATH")
                        .help("Directory to watch (defaults to current directory)")
                        .requires("watch"),
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
    let watch = matches.get_flag("watch");
    let watch_dir = matches.get_one::<String>("watch-dir");

    println!("{}", style("Running Tool 8: HTTP Code Query Server").cyan());
    if verbose {
        if let Some(p) = port {
            println!("  Port: {}", p);
        } else {
            println!("  Port: 7777 (default)");
        }
        println!("  Database: {}", db);
        if watch {
            if let Some(dir) = watch_dir {
                println!("  File watching: enabled ({})", dir);
            } else {
                println!("  File watching: enabled (current directory)");
            }
        }
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
        file_watching_enabled_flag: watch,
        watch_directory_path_option: watch_dir.map(|p| std::path::PathBuf::from(p)),
    };

    // Start the server (blocks until shutdown)
    http_server_startup_runner::start_http_server_blocking_loop(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_builds() {
        let cli = build_cli();
        // Verify all subcommands are present (v1.0.3: HTTP-only architecture)
        let subcommands: Vec<&str> = cli.get_subcommands().map(|cmd| cmd.get_name()).collect();
        assert!(subcommands.contains(&"pt01-folder-to-cozodb-streamer")); // Ingest
        assert!(subcommands.contains(&"pt08-http-code-query-server")); // HTTP server (primary)
        // Note: pt02 (JSON export) and pt07 (terminal viz) removed in v1.0.3
        // All visualization available via HTTP endpoints
    }

    #[test]
    fn test_cli_watch_argument_parsing() {
        let cli = build_cli();

        // Test --watch flag is recognized
        let matches = cli.clone().try_get_matches_from([
            "parseltongue",
            "pt08-http-code-query-server",
            "--db", "rocksdb:test.db",
            "--watch",
        ]).expect("CLI should accept --watch flag");

        let sub_matches = matches.subcommand_matches("pt08-http-code-query-server").unwrap();
        assert!(sub_matches.get_flag("watch"));

        // Test --watch-dir requires --watch
        let result = cli.clone().try_get_matches_from([
            "parseltongue",
            "pt08-http-code-query-server",
            "--db", "rocksdb:test.db",
            "--watch-dir", "./src",
        ]);
        assert!(result.is_err(), "--watch-dir should require --watch");

        // Test --watch with --watch-dir works
        let matches = cli.clone().try_get_matches_from([
            "parseltongue",
            "pt08-http-code-query-server",
            "--db", "rocksdb:test.db",
            "--watch",
            "--watch-dir", "./src",
        ]).expect("CLI should accept --watch --watch-dir combination");

        let sub_matches = matches.subcommand_matches("pt08-http-code-query-server").unwrap();
        assert!(sub_matches.get_flag("watch"));
        assert_eq!(sub_matches.get_one::<String>("watch-dir").unwrap(), "./src");
    }
}
