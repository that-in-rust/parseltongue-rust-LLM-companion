//! Command line argument parsing for HTTP server
//!
//! # 4-Word Naming: command_line_argument_parser

use std::path::PathBuf;
use anyhow::{Result, Context};

/// Configuration for HTTP server startup
///
/// # 4-Word Name: HttpServerStartupConfig
#[derive(Debug, Clone)]
pub struct HttpServerStartupConfig {
    /// Target directory to analyze
    pub target_directory_path_value: PathBuf,

    /// Database connection string (e.g., "rocksdb:./analysis.db")
    pub database_connection_string_value: String,

    /// HTTP port override (None = default 7777)
    pub http_port_override_option: Option<u16>,

    /// Force fresh ingestion even if DB exists
    pub force_reindex_enabled_flag: bool,

    /// Run in background/daemon mode
    pub daemon_background_mode_flag: bool,

    /// Auto-shutdown after idle period (minutes)
    pub idle_timeout_minutes_option: Option<u32>,

    /// Show verbose query logs
    pub verbose_logging_enabled_flag: bool,

    /// Enable file watching for incremental reindex (PRD-2026-01-28)
    pub file_watching_enabled_flag: bool,

    /// Directory to watch (defaults to target directory)
    pub watch_directory_path_option: Option<PathBuf>,
}

impl Default for HttpServerStartupConfig {
    fn default() -> Self {
        Self {
            target_directory_path_value: PathBuf::from("."),
            database_connection_string_value: String::new(),
            http_port_override_option: None,
            force_reindex_enabled_flag: false,
            daemon_background_mode_flag: false,
            idle_timeout_minutes_option: None,
            verbose_logging_enabled_flag: false,
            file_watching_enabled_flag: false,
            watch_directory_path_option: None,
        }
    }
}

impl HttpServerStartupConfig {
    /// Parse configuration from environment/CLI arguments
    ///
    /// # 4-Word Name: parse_from_environment_args
    pub fn parse_from_environment_args() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        Self::parse_from_argument_vector(&args)
    }

    /// Parse configuration from argument vector
    ///
    /// # 4-Word Name: parse_from_argument_vector
    pub fn parse_from_argument_vector(args: &[String]) -> Result<Self> {
        let mut config = Self::default();
        let mut i = 1; // Skip program name

        while i < args.len() {
            match args[i].as_str() {
                "--port" => {
                    i += 1;
                    config.http_port_override_option = Some(
                        args.get(i)
                            .context("--port requires a value")?
                            .parse()
                            .context("Invalid port number")?
                    );
                }
                "--reindex" => {
                    config.force_reindex_enabled_flag = true;
                }
                "--daemon" => {
                    config.daemon_background_mode_flag = true;
                }
                "--timeout" => {
                    i += 1;
                    config.idle_timeout_minutes_option = Some(
                        args.get(i)
                            .context("--timeout requires a value")?
                            .parse()
                            .context("Invalid timeout value")?
                    );
                }
                "--verbose" => {
                    config.verbose_logging_enabled_flag = true;
                }
                "--watch" => {
                    config.file_watching_enabled_flag = true;
                }
                "--watch-dir" => {
                    i += 1;
                    config.watch_directory_path_option = Some(
                        PathBuf::from(
                            args.get(i)
                                .context("--watch-dir requires a path")?
                        )
                    );
                    // Implicitly enable watching when watch-dir is specified
                    config.file_watching_enabled_flag = true;
                }
                arg if !arg.starts_with('-') => {
                    // Positional argument = target directory
                    config.target_directory_path_value = PathBuf::from(arg);
                }
                _ => {
                    // Unknown argument - skip for now
                }
            }
            i += 1;
        }

        // Generate database path based on target directory
        if config.database_connection_string_value.is_empty() {
            let db_dir = config.target_directory_path_value.join(
                format!("parseltongue_{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
            );
            config.database_connection_string_value = format!(
                "rocksdb:{}/analysis.db",
                db_dir.display()
            );
        }

        Ok(config)
    }
}

/// Find an available port starting from given port
///
/// # 4-Word Name: find_available_port_number
pub fn find_available_port_number(starting_port: u16) -> Result<u16> {
    use std::net::TcpListener;

    for port in starting_port..starting_port + 100 {
        if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
            return Ok(port);
        }
    }

    anyhow::bail!("No available ports found in range {}-{}", starting_port, starting_port + 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_arguments_config() {
        let args = vec!["pt08".to_string(), ".".to_string()];
        let config = HttpServerStartupConfig::parse_from_argument_vector(&args).unwrap();

        assert_eq!(config.target_directory_path_value, PathBuf::from("."));
        assert!(!config.force_reindex_enabled_flag);
        assert!(!config.daemon_background_mode_flag);
    }

    #[test]
    fn test_parse_with_port_override() {
        let args = vec![
            "pt08".to_string(),
            ".".to_string(),
            "--port".to_string(),
            "7777".to_string(),
        ];
        let config = HttpServerStartupConfig::parse_from_argument_vector(&args).unwrap();

        assert_eq!(config.http_port_override_option, Some(7777));
    }

    #[test]
    fn test_find_available_port_number() {
        let port = find_available_port_number(7777).unwrap();
        assert!(port >= 7777);
    }

    #[test]
    fn test_parse_with_watch_flag() {
        let args = vec![
            "pt08".to_string(),
            ".".to_string(),
            "--watch".to_string(),
        ];
        let config = HttpServerStartupConfig::parse_from_argument_vector(&args).unwrap();

        assert!(config.file_watching_enabled_flag);
        assert!(config.watch_directory_path_option.is_none());
    }

    #[test]
    fn test_parse_with_watch_dir_flag() {
        let args = vec![
            "pt08".to_string(),
            ".".to_string(),
            "--watch-dir".to_string(),
            "/tmp/watch".to_string(),
        ];
        let config = HttpServerStartupConfig::parse_from_argument_vector(&args).unwrap();

        assert!(config.file_watching_enabled_flag);
        assert_eq!(
            config.watch_directory_path_option,
            Some(PathBuf::from("/tmp/watch"))
        );
    }
}
