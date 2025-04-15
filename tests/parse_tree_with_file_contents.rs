use claudeai_bundle::{FileSystem, Result};

#[test]
fn test_parse_tree_with_file_contents() -> Result<()> {
	// First line is the tree structure, second part is file contents
	let input = "// File structure
// - Cargo.toml
// - src/
//   - main.rs
//   - lib.rs

// Cargo.toml
[package]
name = \"example\"
version = \"0.1.0\"

// src/main.rs
fn main() {
    println!(\"Hello, world!\");
}";

	let fs = FileSystem::parse(input, Some("// "), false)?;

	// Make sure the files exist
	assert!(
		fs.get_node("src/main.rs").is_ok(),
		"src/main.rs should exist"
	);

	// Check file contents
	if let Ok(main) = fs.get_node("src/main.rs") {
		let main_content = main.borrow().contents().map(|c| c.to_string()).unwrap_or_default();
		assert!(main_content.contains("println!(\"Hello, world!\")"));
	}
	Ok(())
}
