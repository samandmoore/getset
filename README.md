# On your mark, `getset`, go

A simple command line tool for bootstrapping a codebase and getting it ready
for development. Perfect for onboarding new developers or setting up a fresh
clone of a repository.

## Installation

```bash
cargo install --path .
```

## Usage

Create a TOML file with the setup commands you want to run:

```toml
[[commands]]
title = "Install system dependencies"
command = "brew bundle"

[[commands]]
title = "Install Ruby version"
command = "rbenv install"

[[commands]]
title = "Install gems"
command = "bundle install"
```

Then run `getset` with the path to your TOML file:

```bash
getset bootstrap.toml
```

Or, if you name your file `getset.toml`, you can run it without any arguments:

```bash
getset  # Uses getset.toml by default
```

### Command line options

```bash
getset                      # Run commands from getset.toml (default)
getset <file>               # Run commands from a TOML file
getset <file> --verbose     # Show command text while running
getset <file> --report      # Show timing report at the end
getset <file> --step <step> # Run only commands matching <STEP> (case-insensitive)
```

### Keep on top of slow steps

Run with performance report:

```bash
getset setup.toml --report
```

This will show a timing breakdown at the end:

```
📊 Report
├──▶ 45.23s Install Homebrew dependencies
├──▶ 12.45s Install Node.js
├──▶ 8.32s Install Ruby
├──▶ 23.11s Install Node packages
├──▶ 15.67s Install Ruby gems
└─▶ 104.78s
```

## PlatformX Integration

You can optionally integrate with [getdx.com](https://getdx.com)'s PlatformX to track usage metrics. Add a `[platformx]` section to your TOML config:

```toml
[[commands]]
title = "Install dependencies"
command = "npm install"

[platformx]
secret_key = "your_platformx_secret_key"
event_namespace = "myapp"  # Optional: defaults to "getset"
```

When configured, `getset` will automatically send the following events:

- **{namespace}.start**: Sent when the command starts
  - Metadata: `user_shell` (e.g., `/bin/bash`)

- **{namespace}.complete**: Sent when all commands complete successfully
  - Metadata: `user_shell`, `duration` (in seconds)

- **{namespace}.error**: Sent when a command fails
  - Metadata: `user_shell`, `duration` (in seconds), `error_message`

Where `{namespace}` is the value of `event_namespace` (defaults to `"getset"` if not specified).

Note: PlatformX errors will not interrupt the CLI execution. If telemetry fails, the CLI will continue normally.
