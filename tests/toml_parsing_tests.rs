use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    commands: Vec<CommandEntry>,
}

#[derive(Debug, Deserialize)]
struct CommandEntry {
    title: String,
    command: String,
}

#[test]
fn test_parse_example_toml() {
    let toml_content = fs::read_to_string("example.toml").expect("Failed to read example.toml");
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse example.toml");
    
    assert_eq!(config.commands.len(), 3);
    assert_eq!(config.commands[0].title, "List files");
    assert_eq!(config.commands[0].command, "ls -la");
}

#[test]
fn test_parse_example_fail_toml() {
    let toml_content = fs::read_to_string("example-fail.toml").expect("Failed to read example-fail.toml");
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse example-fail.toml");
    
    assert_eq!(config.commands.len(), 3);
    assert_eq!(config.commands[0].title, "This will pass");
}

#[test]
fn test_parse_example_input_toml() {
    let toml_content = fs::read_to_string("example-input.toml").expect("Failed to read example-input.toml");
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse example-input.toml");
    
    assert_eq!(config.commands.len(), 4);
    assert_eq!(config.commands[2].title, "User input");
    assert!(config.commands[2].command.contains("read -r -p"));
}

#[test]
fn test_parse_example_multiline_toml() {
    let toml_content = fs::read_to_string("example-multiline.toml").expect("Failed to read example-multiline.toml");
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse example-multiline.toml");
    
    assert_eq!(config.commands.len(), 1);
    assert_eq!(config.commands[0].title, "Slow loop");
    assert!(config.commands[0].command.contains("while [ $i -le 10 ]"));
}

#[test]
fn test_parse_example_slow_toml() {
    let toml_content = fs::read_to_string("example-slow.toml").expect("Failed to read example-slow.toml");
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse example-slow.toml");
    
    assert_eq!(config.commands.len(), 3);
    assert_eq!(config.commands[1].title, "Slow loop");
    assert!(config.commands[1].command.contains("sleep $i"));
}
