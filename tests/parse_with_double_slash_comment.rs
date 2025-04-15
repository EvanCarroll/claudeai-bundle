use claudeai_bundle::{FileSystem, Result};

#[test]
fn test_with_double_slash_comment() -> Result<()> {
	// Test with the actual file format as in examples/tree_with_contents.txt
	let input = "// File structure
// - Cargo.toml
// - src/
//   - main.rs
//   - routes.rs

// Cargo.toml
[package]
name = \"ping-server\"
version = \"0.1.0\"

// src/main.rs
use std::net::SocketAddr;

fn main() {
    println!(\"Hello\");
}";

	let fs = FileSystem::parse(input, Some("// "), false)?;

	// Make sure at least one file exists
	assert!(
		fs.get_node("src/main.rs").is_ok(),
		"src/main.rs should exist"
	);

	// Check file contents
	if let Ok(main) = fs.get_node("src/main.rs") {
		let main_content = main.borrow().contents().map(|c| c.to_string()).unwrap_or_default();
		assert!(main_content.contains("use std::net::SocketAddr;"));
	}
	Ok(())
}
