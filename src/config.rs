use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub commands: Vec<CommandEntry>,
    pub platformx: Option<PlatformXConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformXConfig {
    pub secret_key: String,
    pub event_namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommandEntry {
    pub title: String,
    pub command: String,
}

impl Config {
    /// Load and parse a TOML configuration file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let toml_content = fs::read_to_string(path_ref)
            .map_err(|e| format!("Error reading file '{}': {}", path_ref.display(), e))?;
        toml_content.parse()
    }
}

impl FromStr for Config {
    type Err = String;

    /// Parse a TOML configuration from a string
    fn from_str(toml_content: &str) -> Result<Self, Self::Err> {
        toml::from_str(toml_content).map_err(|e| format!("Error parsing TOML: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_file() {
        let config = Config::from_file("tests/fixtures/valid_config.toml")
            .expect("Failed to load config from file");

        assert_eq!(config.commands.len(), 2);
        assert_eq!(config.commands[0].title, "Test Command 1");
        assert_eq!(config.commands[0].command, "echo test1");
        assert_eq!(config.commands[1].title, "Test Command 2");
        assert_eq!(config.commands[1].command, "echo test2");
    }

    #[test]
    fn test_load_from_string() {
        let toml_str = r#"
[[commands]]
title = "String Command"
command = "echo from string"
"#;

        let config: Config = toml_str
            .parse()
            .expect("Failed to parse config from string");

        assert_eq!(config.commands.len(), 1);
        assert_eq!(config.commands[0].title, "String Command");
        assert_eq!(config.commands[0].command, "echo from string");
    }

    #[test]
    fn test_valid_parse() {
        let toml_str = r#"
[[commands]]
title = "Valid Command 1"
command = "ls -la"

[[commands]]
title = "Valid Command 2"
command = "pwd"
"#;

        let config: Config = toml_str.parse().expect("Failed to parse valid TOML");

        assert_eq!(config.commands.len(), 2);
        assert_eq!(config.commands[0].title, "Valid Command 1");
        assert_eq!(config.commands[0].command, "ls -la");
        assert_eq!(config.commands[1].title, "Valid Command 2");
        assert_eq!(config.commands[1].command, "pwd");
    }

    #[test]
    fn test_invalid_parse_missing_field() {
        let invalid_toml = r#"
[[commands]]
title = "Missing command field"
"#;

        let result: Result<Config, _> = invalid_toml.parse();
        assert!(result.is_err(), "Should fail when command field is missing");
        assert!(result.unwrap_err().contains("Error parsing TOML"));
    }

    #[test]
    fn test_invalid_parse_malformed_toml() {
        let malformed_toml = r#"
[[commands]
title = "Missing closing bracket"
command = "echo test"
"#;

        let result: Result<Config, _> = malformed_toml.parse();
        assert!(result.is_err(), "Should fail with malformed TOML");
        assert!(result.unwrap_err().contains("Error parsing TOML"));
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = Config::from_file("tests/fixtures/nonexistent.toml");
        assert!(result.is_err(), "Should fail when file doesn't exist");
        assert!(result.unwrap_err().contains("Error reading file"));
    }

    #[test]
    fn test_load_from_invalid_file() {
        let result = Config::from_file("tests/fixtures/invalid_config.toml");
        assert!(result.is_err(), "Should fail with invalid config file");
        assert!(result.unwrap_err().contains("Error parsing TOML"));
    }
}
