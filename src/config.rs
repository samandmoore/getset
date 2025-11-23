use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub commands: Vec<CommandEntry>,
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
        Self::from_str(&toml_content)
    }

    /// Parse a TOML configuration from a string
    pub fn from_str(toml_content: &str) -> Result<Self, String> {
        toml::from_str(toml_content).map_err(|e| format!("Error parsing TOML: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_example_toml() {
        let config = Config::from_file("example.toml").expect("Failed to parse example.toml");

        assert_eq!(config.commands.len(), 3);
        assert_eq!(config.commands[0].title, "List files");
        assert_eq!(config.commands[0].command, "ls -la");
    }

    #[test]
    fn test_parse_example_fail_toml() {
        let config =
            Config::from_file("example-fail.toml").expect("Failed to parse example-fail.toml");

        assert_eq!(config.commands.len(), 3);
        assert_eq!(config.commands[0].title, "This will pass");
    }

    #[test]
    fn test_parse_example_input_toml() {
        let config =
            Config::from_file("example-input.toml").expect("Failed to parse example-input.toml");

        assert_eq!(config.commands.len(), 4);
        assert_eq!(config.commands[2].title, "User input");
        assert!(config.commands[2].command.contains("read -r -p"));
    }

    #[test]
    fn test_parse_example_multiline_toml() {
        let config = Config::from_file("example-multiline.toml")
            .expect("Failed to parse example-multiline.toml");

        assert_eq!(config.commands.len(), 1);
        assert_eq!(config.commands[0].title, "Slow loop");
        assert!(config.commands[0].command.contains("while [ $i -le 10 ]"));
    }

    #[test]
    fn test_parse_example_slow_toml() {
        let config =
            Config::from_file("example-slow.toml").expect("Failed to parse example-slow.toml");

        assert_eq!(config.commands.len(), 3);
        assert_eq!(config.commands[1].title, "Slow loop");
        assert!(config.commands[1].command.contains("sleep $i"));
    }
}
