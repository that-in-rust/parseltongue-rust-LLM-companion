//! Parseltongue: Unified CLI toolkit for code analysis
//!
//! This binary provides subcommands that dispatch to the individual tools:
//! - pt01-folder-to-cozodb-streamer (Tool 1: Ingest)
//! - pt02-llm-cozodb-to-context-writer (Tool 2: Read)
//! - pt07-visual-analytics-terminal (Tool 7: Visualize)
//! - pt08-http-code-query-server (Tool 8: HTTP Server)

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
        Some(("pt02-level00", sub_matches)) => {
            run_pt02_level00(sub_matches).await
        }
        Some(("pt02-level01", sub_matches)) => {
            run_pt02_level01(sub_matches).await
        }
        Some(("pt02-level02", sub_matches)) => {
            run_pt02_level02(sub_matches).await
        }
        Some(("pt07", sub_matches)) => {
            run_pt07(sub_matches).await
        }
        Some(("serve-http-code-backend", sub_matches)) => {
            run_serve_http_code_backend(sub_matches).await
        }
        _ => {
            println!("{}", style("Parseltongue CLI Toolkit").blue().bold());
            println!("{}", style("Ultra-minimalist code analysis toolkit").blue());
            println!();
            println!("Use --help for more information");
            println!();
            println!("Available commands:");
            println!("  pt01-folder-to-cozodb-streamer       - Index codebase into CozoDB (Tool 1: Ingest)");
            println!("");
            println!("  PT02: Export from CozoDB (Progressive Disclosure)");
            println!("    pt02-level00                       - Pure edge list (~2-5K tokens) [RECOMMENDED]");
            println!("    pt02-level01                       - Entity + ISG + Temporal (~30K tokens)");
            println!("    pt02-level02                       - + Type system (~60K tokens)");
            println!("");
            println!("  pt07                                 - Visual analytics (Tool 7: Visualize)");
            println!("  serve-http-code-backend              - HTTP server for REST API (Tool 8: Server)");
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
            Command::new("pt02-level00")
                .about("Tool 2a: Export pure edge list (Level 0 - ~2-5K tokens) [RECOMMENDED]")
                .long_about("Export dependency edges only for graph visualization and dependency analysis.\n\nExample:\n  parseltongue pt02-level00 --where-clause \"ALL\" --output edges.json")
                .arg(
                    Arg::new("where-clause")
                        .long("where-clause")
                        .help("Datalog WHERE clause (use 'ALL' for everything)")
                        .long_help("Datalog WHERE clause or 'ALL' (MANDATORY)\n\nExamples:\n  --where-clause \"ALL\"\n  --where-clause \"edge_type = 'depends_on'\"\n  --where-clause \"from_key ~ 'rust:fn'\"\n\nDatalog syntax:\n  - AND: Use comma (,)     NOT &&\n  - OR: Use semicolon (;)  NOT ||\n  - Equality: Use =        NOT ==")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output JSON file path")
                        .default_value("ISGLevel00.json"),
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
                        .help("Show progress and token estimates")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("pt02-level01")
                .about("Tool 2b: Export entities with ISG + temporal (Level 1 - ~30K tokens)")
                .long_about("Export entities with Interface Signature Graph and temporal state.\n\nExamples:\n  # Signatures only (CHEAP - ~30K tokens)\n  parseltongue pt02-level01 --include-code 0 --where-clause \"ALL\" --output entities.json\n\n  # With code (EXPENSIVE - 100× more tokens)\n  parseltongue pt02-level01 --include-code 1 --where-clause \"future_action != null\" --output changes.json")
                .arg(
                    Arg::new("include-code")
                        .long("include-code")
                        .help("Include current_code field: 0=signatures only (cheap), 1=with code (expensive)")
                        .value_parser(["0", "1"])
                        .required(true),
                )
                .arg(
                    Arg::new("where-clause")
                        .long("where-clause")
                        .help("Datalog WHERE clause (use 'ALL' for everything)")
                        .long_help("Datalog WHERE clause or 'ALL' (MANDATORY)\n\nExamples:\n  --where-clause \"ALL\"\n  --where-clause \"is_public = true, entity_type = 'fn'\"\n  --where-clause \"future_action != null\"\n\nDatalog syntax:\n  - AND: Use comma (,)\n  - OR: Use semicolon (;)\n  - Equality: Use =")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output JSON file path")
                        .default_value("ISGLevel01.json"),
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
                        .help("Show progress and token estimates")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("pt02-level02")
                .about("Tool 2c: Export entities with type system (Level 2 - ~60K tokens)")
                .long_about("Export entities with full type system information for type-safe refactoring.\n\nExamples:\n  # Find async functions\n  parseltongue pt02-level02 --include-code 0 --where-clause \"is_async = true\" --output async.json\n\n  # Find unsafe code\n  parseltongue pt02-level02 --include-code 0 --where-clause \"is_unsafe = true\" --output unsafe.json")
                .arg(
                    Arg::new("include-code")
                        .long("include-code")
                        .help("Include current_code field: 0=signatures only (cheap), 1=with code (expensive)")
                        .value_parser(["0", "1"])
                        .required(true),
                )
                .arg(
                    Arg::new("where-clause")
                        .long("where-clause")
                        .help("Datalog WHERE clause (use 'ALL' for everything)")
                        .long_help("Datalog WHERE clause or 'ALL' (MANDATORY)\n\nExamples:\n  --where-clause \"ALL\"\n  --where-clause \"is_async = true\"\n  --where-clause \"is_unsafe = true\"\n  --where-clause \"is_public = true\"\n\nDatalog syntax:\n  - AND: Use comma (,)\n  - OR: Use semicolon (;)\n  - Equality: Use =")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output JSON file path")
                        .default_value("ISGLevel02.json"),
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
                        .help("Show progress and token estimates")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("pt07")
                .about("Tool 7: Visual analytics for code graphs")
                .subcommand_required(true)
                .subcommand(
                    Command::new("entity-count")
                        .about("Entity count bar chart visualization")
                        .arg(
                            Arg::new("db")
                                .long("db")
                                .help("Database file path")
                                .required(true),
                        )
                        .arg(
                            Arg::new("include-tests")
                                .long("include-tests")
                                .help("Include test entities (default: implementation-only)")
                                .action(clap::ArgAction::SetTrue),
                        ),
                )
                .subcommand(
                    Command::new("cycles")
                        .about("Circular dependency detection visualization")
                        .arg(
                            Arg::new("db")
                                .long("db")
                                .help("Database file path")
                                .required(true),
                        )
                        .arg(
                            Arg::new("include-tests")
                                .long("include-tests")
                                .help("Include test entities (default: implementation-only)")
                                .action(clap::ArgAction::SetTrue),
                        ),
                ),
        )
        .subcommand(
            Command::new("serve-http-code-backend")
                .about("Tool 8: HTTP server for code queries (REST API)")
                .long_about(
                    "Start an HTTP server exposing CozoDB queries via REST endpoints.\n\n\
                    Examples:\n  \
                    parseltongue serve-http-code-backend --port 3000\n  \
                    parseltongue serve-http-code-backend --port 8080 --db rocksdb:analysis.db"
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .short('p')
                        .help("Port to listen on (auto-detects from 3333 if not specified)"),
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
        println!("{}", style("Next steps:").cyan());
        println!("  Export edges:    parseltongue pt02-level00 --where-clause \"ALL\" \\");
        println!("                     --output {}/edges.json \\", workspace_dir);
        println!("                     --db \"{}\"", workspace_db_path);
        println!();
        println!("  Export entities: parseltongue pt02-level01 --include-code 0 --where-clause \"ALL\" \\");
        println!("                     --output {}/entities.json \\", workspace_dir);
        println!("                     --db \"{}\"", workspace_db_path);
        if verbose {
            println!("  Duration: {:?}", result.duration);
        }
    }

    Ok(())
}

async fn run_pt02_level00(matches: &ArgMatches) -> Result<()> {
    use pt02_llm_cozodb_to_context_writer::{CozoDbAdapter, Level0Exporter, LevelExporter};

    let where_clause = matches.get_one::<String>("where-clause").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let db = matches.get_one::<String>("db").unwrap();
    let verbose = matches.get_flag("verbose");

    println!("{}", style("Running PT02 Level 0: Pure Edge List Export").cyan());
    if verbose {
        println!("  Database: {}", db);
        println!("  WHERE clause: {}", where_clause);
        println!("  Output: {}", output);
    }

    // Connect to CozoDB
    let db_adapter = CozoDbAdapter::connect(db).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Create exporter
    let exporter = Level0Exporter::new();
    
    // Extract base output name (remove .json extension if present)
    let base_output = if output.ends_with(".json") {
        &output[..output.len() - 5]
    } else {
        output
    };

    if verbose {
        println!("  Estimated tokens: ~{}", exporter.estimated_tokens());
    }

    // Execute dual file export (REQ-V090-004.0: Automatic dual-file export)
    exporter.export_dual_files(
        &db_adapter,
        base_output,
        where_clause
    ).await
        .map_err(|e| anyhow::anyhow!("Export failed: {}", e))?;

    println!("{}", style("✓ PT02 Level 0 export completed").green().bold());
    println!("  Output files: {}.json, {}_test.json", base_output, base_output);
    
    // Load and display edge counts from the main export file
    let main_output_file = format!("{}.json", base_output);
    if let Ok(content) = std::fs::read_to_string(&main_output_file) {
        if let Ok(export_data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(edges) = export_data["edges"].as_array() {
                println!("  Edges exported: {}", edges.len());
            }
        }
    }
    println!("  Token estimate: ~{}", exporter.estimated_tokens());
    println!("  Fields per edge: 3 (from_key, to_key, edge_type)");

    Ok(())
}

async fn run_pt02_level01(matches: &ArgMatches) -> Result<()> {
    use pt02_llm_cozodb_to_context_writer::{CozoDbAdapter, Level1Exporter, LevelExporter};

    let include_code = matches.get_one::<String>("include-code").unwrap();
    let where_clause = matches.get_one::<String>("where-clause").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let db = matches.get_one::<String>("db").unwrap();
    let verbose = matches.get_flag("verbose");

    println!("{}", style("Running PT02 Level 1: Entity + ISG + Temporal Export").cyan());
    if verbose {
        println!("  Database: {}", db);
        println!("  Include code: {}", if include_code == "1" { "YES (expensive)" } else { "NO (cheap)" });
        println!("  WHERE clause: {}", where_clause);
        println!("  Output: {}", output);
    }

    // Connect to CozoDB
    let db_adapter = CozoDbAdapter::connect(db).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Create exporter
    let exporter = Level1Exporter::new();
    
    // Extract base output name (remove .json extension if present)
    let base_output = if output.ends_with(".json") {
        &output[..output.len() - 5]
    } else {
        output
    };

    let base_tokens = exporter.estimated_tokens();
    let estimated = if include_code == "1" { base_tokens * 20 } else { base_tokens };

    if verbose {
        println!("  Estimated tokens: ~{}", estimated);
    }

    // Execute dual file export (REQ-V090-004.0: Automatic dual-file export)
    exporter.export_dual_files(
        &db_adapter,
        base_output,
        include_code == "1",
        where_clause
    ).await
        .map_err(|e| anyhow::anyhow!("Export failed: {}", e))?;

    println!("{}", style("✓ PT02 Level 1 export completed").green().bold());
    println!("  Output files: {}.json, {}_test.json", base_output, base_output);
    
    // Load and display entity counts from the main export file
    let main_output_file = format!("{}.json", base_output);
    if let Ok(content) = std::fs::read_to_string(&main_output_file) {
        if let Ok(export_data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(entities) = export_data["entities"].as_array() {
                println!("  Entities exported: {}", entities.len());
            }
        }
    }
    println!("  Token estimate: ~{} tokens", estimated);
    println!("  Fields per entity: 14 (isgl1_key, forward_deps, reverse_deps, temporal state, etc.)");

    Ok(())
}

async fn run_pt02_level02(matches: &ArgMatches) -> Result<()> {
    use pt02_llm_cozodb_to_context_writer::{CozoDbAdapter, Level2Exporter, LevelExporter};

    let include_code = matches.get_one::<String>("include-code").unwrap();
    let where_clause = matches.get_one::<String>("where-clause").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let db = matches.get_one::<String>("db").unwrap();
    let verbose = matches.get_flag("verbose");

    println!("{}", style("Running PT02 Level 2: Entity + ISG + Temporal + Type System Export").cyan());
    if verbose {
        println!("  Database: {}", db);
        println!("  Include code: {}", if include_code == "1" { "YES (expensive)" } else { "NO (cheap)" });
        println!("  WHERE clause: {}", where_clause);
        println!("  Output: {}", output);
    }

    // Connect to CozoDB
    let db_adapter = CozoDbAdapter::connect(db).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    // Create exporter
    let exporter = Level2Exporter::new();
    
    // Extract base output name (remove .json extension if present)
    let base_output = if output.ends_with(".json") {
        &output[..output.len() - 5]
    } else {
        output
    };

    let base_tokens = exporter.estimated_tokens();
    let estimated = if include_code == "1" { base_tokens * 20 } else { base_tokens };

    if verbose {
        println!("  Estimated tokens: ~{}", estimated);
    }

    // Execute dual file export (REQ-V090-004.0: Automatic dual-file export)
    exporter.export_dual_files(
        &db_adapter,
        base_output,
        include_code == "1",
        where_clause
    ).await
        .map_err(|e| anyhow::anyhow!("Export failed: {}", e))?;

    println!("{}", style("✓ PT02 Level 2 export completed").green().bold());
    println!("  Output files: {}.json, {}_test.json", base_output, base_output);
    
    // Load and display entity counts from the main export file
    let main_output_file = format!("{}.json", base_output);
    if let Ok(content) = std::fs::read_to_string(&main_output_file) {
        if let Ok(export_data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(entities) = export_data["entities"].as_array() {
                println!("  Entities exported: {}", entities.len());
            }
        }
    }
    println!("  Token estimate: ~{} tokens", estimated);
    println!("  Fields per entity: 16 (includes type system information)");

    Ok(())
}

async fn run_pt07(matches: &ArgMatches) -> Result<()> {
    use pt07_visual_analytics_terminal::visualizations::{
        render_entity_count_bar_chart_visualization,
        render_dependency_cycle_warning_list_visualization,
    };

    println!("{}", style("Running Tool 7: Visual Analytics").cyan());

    match matches.subcommand() {
        Some(("entity-count", sub_matches)) => {
            let db = sub_matches.get_one::<String>("db").unwrap();
            let include_tests = sub_matches.get_flag("include-tests");

            println!("📊 Generating entity count visualization...");
            let output = render_entity_count_bar_chart_visualization(db, include_tests).await?;
            println!("{}", output);

            Ok(())
        }
        Some(("cycles", sub_matches)) => {
            let db = sub_matches.get_one::<String>("db").unwrap();
            let include_tests = sub_matches.get_flag("include-tests");

            println!("🔄 Detecting circular dependencies...");
            let output = render_dependency_cycle_warning_list_visualization(db, include_tests).await?;
            println!("{}", output);

            Ok(())
        }
        _ => {
            println!("Usage: parseltongue pt07 <SUBCOMMAND>");
            println!();
            println!("Subcommands:");
            println!("  entity-count  - Entity count bar chart");
            println!("  cycles        - Circular dependency detection");
            Ok(())
        }
    }
}

/// Run the HTTP server for code queries
///
/// # 4-Word Name: run_serve_http_code_backend
async fn run_serve_http_code_backend(matches: &ArgMatches) -> Result<()> {
    let port = matches.get_one::<String>("port");
    let db = matches.get_one::<String>("db").unwrap();
    let verbose = matches.get_flag("verbose");

    println!("{}", style("Running Tool 8: HTTP Code Query Server").cyan());
    if verbose {
        if let Some(p) = port {
            println!("  Port: {}", p);
        } else {
            println!("  Port: auto-detect from 3333");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_builds() {
        let cli = build_cli();
        // Verify all subcommands are present (v0.9.7: Removed editing tools, focus on analysis)
        let subcommands: Vec<&str> = cli.get_subcommands().map(|cmd| cmd.get_name()).collect();
        assert!(subcommands.contains(&"pt01-folder-to-cozodb-streamer"));
        assert!(subcommands.contains(&"pt02-level00")); // Progressive disclosure
        assert!(subcommands.contains(&"pt02-level01")); // Progressive disclosure
        assert!(subcommands.contains(&"pt02-level02")); // Progressive disclosure
        assert!(subcommands.contains(&"pt07")); // Visual analytics
        assert!(subcommands.contains(&"serve-http-code-backend")); // HTTP server
    }
}
