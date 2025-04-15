use claudeai_bundle::{FileSystem, Result};

#[test]
fn test_parse_tree_with_blank_line() -> Result<()> {
	let tree_output = "dir1/
├── file1
└── dir2/
    ├── file2
    └── file3

This is additional content that should be ignored.
";

	let expected_output = "dir1/
├── file1
└── dir2/
    ├── file2
    └── file3
";

	let fs = FileSystem::parse(tree_output, None, false)?;
	assert_eq!(fs.tree_output(), expected_output);
	Ok(())
}
