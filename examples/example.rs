use claudeai_bundle::{FileSystem, Result};

fn main() -> Result<()> {
	// Example tree output
	let tree_output = "dir1/
├── file1
└── dir2/
    ├── file2
    └── file3
";

	// Parse the tree output
	let fs = FileSystem::parse(tree_output, None, false)?;

	// Display the entire filesystem
	println!("Filesystem tree:");
	println!("{}", fs.tree_output());

	// Look up a specific node
	if let Ok(file3) = fs.get_node("dir1/dir2/file3") {
		println!(
			"\nFound file at: {}",
			file3.borrow().relative_location().display()
		);
	}

	// List all nodes
	println!("\nAll nodes:");
	for node in fs.nodes() {
		println!("- {}", node.borrow().relative_location().display());
	}

	Ok(())
}
