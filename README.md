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

### Command line options

```bash
getset <file>           # Run commands from a TOML file
getset <file> --verbose # Show command text while running
getset <file> --report  # Show timing report at the end
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
