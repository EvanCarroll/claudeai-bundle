# claudeai-bundle

A Rust tool for parsing directory tree structures and extracting them into actual filesystem structures.

## Overview

claudeai-bundle parses textual bundles produced by
[claudeai](https://claude.ai). These are (a) representations of directory trees
(like those produced by the `tree` command) and (b) an inline list of files
where the name of the file is in a comment followed by the rest of the file.

## Features

- Parse directory tree structures in different formats (tree command format, indented lists)
- Commands to
	- `extract` recreate the actual files and directories
	- `cat` out a single files content from the bundle
	- `list` out all files in the bundle

## Installation

As a note installation isn't required at all: `claudeai-bundle` can be run in our container.

```bash
mkdir extract;
podman run --rm -ti \
	-v "${PWD}/extract:/extract:rw" \
	ghcr.io/evancarroll/claudeai-bundle:latest \
	extract --output-directory /extract
```

The above uses podman to create a container for our 800k image. It then waits
for you paste the contents of a claudeai-bundle into the terminal. After you
paste the contents, it will extract them into the directory `./extract`. Then
podman will delete the container cleaning up after itself.

However, you can install `claudeai-bundle` from source like this,

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
