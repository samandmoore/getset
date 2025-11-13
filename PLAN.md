# Rust Command Runner - Implementation Plan

## Overview
Build a Rust CLI tool that executes commands from a YAML file sequentially with real-time output streaming.

## Steps

1. **Create a new Rust project** with the necessary dependencies (clap, serde, serde_yaml, tokio)

2. **Define the YAML structure** with serde for deserialization
   - Simple format with `title` and `command` fields
   - Support list of commands

3. **Implement the command runner** that:
   - Reads and parses the YAML file
   - Runs each command sequentially
   - Streams output in real-time
   - On success: clears output and shows title with ✓
   - On failure: exits with error code

4. **Set up clap** for CLI argument parsing
   - Accept YAML file path as argument

## YAML Format
```yaml
commands:
  - title: "Run tests"
    command: "cargo test"
  - title: "Build project"
    command: "cargo build --release"
```
