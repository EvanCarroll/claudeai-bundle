# claudeai-bundle

A Rust tool for parsing directory tree structures and extracting them into actual filesystem structures.

## Overview

claudeai-bundle parses textual bundles produced by claudeai. These are (a) representations of directory trees (like those produced by the `tree` command) and (b) an inline list of files where the name of the file is in a comment followed by the rest of the file.

## Features

- Parse directory tree structures in different formats (tree command format, indented lists)
- Commands to
  - `extract` parsed structures to create actual files and directories
	- `cat` out a single files content from the bundle
	- `list` out all files in the bundle

## Installation

```bash
git clone git@github.com:EvanCarroll/claudeai-bundle.git
cd claudeai-bundle;
cargo install claudeai-bundle --path .
```

## Usage

```bash
# This is how I run it. Then just paste and hit Ctrl+D after you're done
claudeai-bundle extract --output-directory ./output

# Parse and extract a tree structure to disk
claudeai-bundle --file examples/tree_with_contents_rust.txt extract --output-directory ./output

# Find a specific node in the parsed structure
claudeai-bundle --file examples/sample_tree.txt find "src/main.rs"

# List all nodes in the parsed structure
claudeai-bundle --file examples/simple_tree.txt list

# Display file contents
claudeai-bundle --file examples/tree_with_contents_rust.txt cat "src/main.rs"
```

## Input Formats

The tool supports various input formats:

1. Tree command format:
   ```
   ├── src/
   │   ├── main.rs
   │   └── lib.rs
   └── Cargo.toml
   ```

2. Indented list format:
   ```
   - src/
     - main.rs
     - lib.rs
   - Cargo.toml
   ```

3. Tree with file contents:

	 ```
	# - src/
	#   - main.rs
	#   - lib.rs
	# - Cargo.toml
	# Cargo.toml
	contents...
	
	# src/main.rs
	contents...
	 ```
