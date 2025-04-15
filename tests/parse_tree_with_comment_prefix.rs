use claudeai_bundle::{FileSystem, Result};

#[test]
fn test_parse_tree_with_comment_prefix() -> Result<()> {
	let tree_output = "# dir1/
# ├── file1
# └── dir2/
#     ├── file2
#     └── file3
";

	let expected_output = "dir1/
├── file1
└── dir2/
    ├── file2
    └── file3
";

	let fs = FileSystem::parse(tree_output, Some("# "), false)?;
	assert_eq!(fs.tree_output(), expected_output);

	let file3 = fs.get_node("dir1/dir2/file3").expect("file3 should exist");
	assert_eq!(
		file3.borrow().relative_location().to_string_lossy(),
		"dir1/dir2/file3"
	);
	Ok(())
}
