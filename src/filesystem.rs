use std::{
	cell::RefCell,
	collections::HashMap,
	fmt,
	rc::{Rc, Weak},
};

use crate::{
	error::{Error, Result},
	fsnode::{FsNode, NodeRef},
};

/// Represents a filesystem
#[derive(Debug)]
pub struct FileSystem {
	root: NodeRef,
	nodes: Vec<NodeRef>,
	path_map: HashMap<String, NodeRef>,
}

// Helper function to parse tree command format like:
// dir1/
// ├── file1
// └── dir2/
//     ├── file2
//     └── file3
fn parse_tree_format<F>(fs: &mut FileSystem, header: &str, strip_prefix: F, debug: bool)
where
	F: Fn(&str) -> String,
{
	let mut dir_stack = Vec::new();

	if debug {
		println!(
			"Tree format parser: processing {} lines",
			header.lines().count()
		);
	}

	// Parse the header structure line by line for tree command format
	for line in header.lines() {
		// Skip empty lines in the header
		if line.is_empty() {
			continue;
		}

		// Strip the prefix if needed
		let line = strip_prefix(line);

		// Skip empty lines (which could happen after stripping the prefix)
		if line.trim().is_empty() {
			continue;
		}

		// Skip lines that just contain "File structure"
		if line.trim() == "File structure" {
			if debug {
				println!("Skipping 'File structure' line");
			}
			continue;
		}

		// Skip the first line if it's just a directory name with no indentation
		if !line.contains('├') && !line.contains('└') && !line.contains('│') && dir_stack.is_empty()
		{
			// Create root directory with this name
			let name = line.trim_end_matches('/').to_string();

			*fs.root.borrow_mut() = FsNode::Directory {
				name: name.clone(),
				parent: Weak::new(),
				children: Vec::new(),
			};

			continue;
		}

		// Count the depth based on indentation (each level is 4 spaces)
		let indent_count = line
			.chars()
			.take_while(|&c| c == ' ' || c == '│' || c == '├' || c == '└' || c == '─')
			.count();

		let depth = indent_count / 4 + (if indent_count % 4 > 0 { 1 } else { 0 });

		// Adjust the directory stack based on the depth
		while dir_stack.len() >= depth {
			dir_stack.pop();
		}
		let current_dir = dir_stack.last().unwrap_or(&fs.root).clone();

		// Extract the name by removing indentation characters
		let name = line.trim_start_matches([' ', '│', '├', '└', '─']).to_string();

		// Determine if it's a directory or file
		let is_dir = name.ends_with('/');
		let name = name.trim_end_matches('/').to_string();

		if is_dir {
			// Create a new directory
			let new_dir = Rc::new(RefCell::new(FsNode::Directory {
				name: name.clone(),
				parent: Rc::downgrade(&current_dir),
				children: Vec::new(),
			}));

			// Add to the current directory's children - do it in a separate scope to avoid
			// multiple borrows
			{
				let mut current_borrowed = current_dir.borrow_mut();
				match &mut *current_borrowed {
					FsNode::Directory { children, .. } => {
						children.push(new_dir.clone());
					}
					FsNode::Root => {
						// Convert root to a directory if needed
						*current_borrowed = FsNode::Directory {
							name: String::new(),
							parent: Weak::new(),
							children: vec![new_dir.clone()],
						};
					}
					_ => {}
				}
			}

			// Update directory stack
			dir_stack.push(new_dir.clone());

			// Add to nodes list
			fs.nodes.push(new_dir.clone());

			// Add to path map
			let path = new_dir.borrow().relative_location().to_string_lossy().to_string();
			fs.path_map.insert(path, new_dir.clone());
		}
		else {
			// Create a new file
			let new_file = Rc::new(RefCell::new(FsNode::File {
				name,
				parent: Rc::downgrade(&current_dir),
				contents: None,
			}));

			// Add to the current directory's children - do it in a separate scope to avoid
			// multiple borrows
			{
				let mut current_borrowed = current_dir.borrow_mut();
				match &mut *current_borrowed {
					FsNode::Directory { children, .. } => {
						children.push(new_file.clone());
					}
					FsNode::Root => {
						// Convert root to a directory if needed
						*current_borrowed = FsNode::Directory {
							name: String::new(),
							parent: Weak::new(),
							children: vec![new_file.clone()],
						};
					}
					_ => {}
				}
			}

			// Add to nodes list
			fs.nodes.push(new_file.clone());

			// Add to path map
			let path = new_file.borrow().relative_location().to_string_lossy().to_string();
			fs.path_map.insert(path, new_file.clone());
		}
	}
}

// Helper function to parse indented list format like:
// - File1
// - Dir/
//   - SubFile1
fn parse_indented_list<F>(fs: &mut FileSystem, header: &str, strip_prefix: F, debug: bool)
where
	F: Fn(&str) -> String,
{
	let mut dir_stack: Vec<(NodeRef, usize)> = Vec::new(); // (node, depth)

	if debug {
		println!(
			"Indented list parser: processing {} lines",
			header.lines().count()
		);
	}

	// Process each line in the header
	for line in header.lines() {
		// Strip the prefix if needed
		let line = strip_prefix(line);

		// Skip empty lines (which could happen after stripping the prefix)
		if line.trim().is_empty() {
			continue;
		}

		// Skip lines that just contain "File structure"
		if line.trim() == "File structure" {
			if debug {
				println!("Indented list parser: Skipping 'File structure' line");
			}
			continue;
		}

		// Skip other header lines that don't match the pattern
		if !line.contains("- ") {
			continue;
		}

		// Count indentation level (each level is 2 spaces) and extract the name
		let indent_count = line.chars().take_while(|&c| c == ' ').count();
		let depth = indent_count / 2;

		// Extract the file/directory name, removing the "- " prefix
		let item_name = line.trim_start().trim_start_matches("- ").to_string();

		// Adjust the directory stack if we're moving back up
		while !dir_stack.is_empty() && dir_stack.last().unwrap().1 >= depth {
			dir_stack.pop();
		}

		// Get the current directory from the stack or root
		let current_dir = if let Some((parent, _)) = dir_stack.last() {
			parent.clone()
		}
		else {
			fs.root.clone()
		};

		// Determine if it's a directory or file
		let is_dir = item_name.ends_with('/');
		let name = item_name.trim_end_matches('/').to_string();

		if is_dir {
			// Create a new directory and add it to the parent
			{
				// First, check if we need to borrow the parent to get its children
				let mut needs_new_dir = false;

				match &*current_dir.borrow() {
					FsNode::Directory { .. } => {
						// We'll add the child later
						needs_new_dir = true;
					}
					FsNode::Root => {
						needs_new_dir = true;
					}
					_ => {}
				}

				if needs_new_dir {
					// Create the new directory
					let new_dir = Rc::new(RefCell::new(FsNode::Directory {
						name: name.clone(),
						parent: Rc::downgrade(&current_dir),
						children: Vec::new(),
					}));

					// Add to current directory children
					{
						let mut current_dir_mut = current_dir.borrow_mut();

						match &mut *current_dir_mut {
							FsNode::Directory { children, .. } => {
								children.push(new_dir.clone());
							}
							FsNode::Root => {
								*current_dir_mut = FsNode::Directory {
									name: String::new(),
									parent: Weak::new(),
									children: vec![new_dir.clone()],
								};
							}
							_ => {}
						}
					}

					// Update the directory stack
					dir_stack.push((new_dir.clone(), depth));

					// Add to nodes list
					fs.nodes.push(new_dir.clone());

					// Add to path map
					// We need to be careful about borrowing. First calculate the path:
					let mut path_parts = vec![name.clone()];
					let mut current = current_dir.clone();

					loop {
						let borrowed = current.borrow();
						match &*borrowed {
							FsNode::Root => break,
							FsNode::Directory { name, parent, .. } => {
								if !name.is_empty() {
									path_parts.push(name.clone());
								}

								if let Some(p) = parent.upgrade() {
									drop(borrowed); // Release the borrow before getting another
									current = p;
								}
								else {
									break;
								}
							}
							_ => break,
						}
					}

					path_parts.reverse();
					let path = path_parts.join("/");

					fs.path_map.insert(path, new_dir.clone());
				}
			}
		}
		else {
			// Create a new file
			let new_file = Rc::new(RefCell::new(FsNode::File {
				name: name.clone(),
				parent: Rc::downgrade(&current_dir),
				contents: None,
			}));

			// Add to parent's children
			{
				let mut current_dir_mut = current_dir.borrow_mut();

				match &mut *current_dir_mut {
					FsNode::Directory { children, .. } => {
						children.push(new_file.clone());
					}
					FsNode::Root => {
						*current_dir_mut = FsNode::Directory {
							name: String::new(),
							parent: Weak::new(),
							children: vec![new_file.clone()],
						};
					}
					_ => {}
				}
			}

			// Add to nodes list
			fs.nodes.push(new_file.clone());

			// Add to path map - calculate path safely
			let mut path_parts = vec![name.clone()];
			let mut current = current_dir.clone();

			loop {
				let borrowed = current.borrow();
				match &*borrowed {
					FsNode::Root => break,
					FsNode::Directory { name, parent, .. } => {
						if !name.is_empty() {
							path_parts.push(name.clone());
						}

						if let Some(p) = parent.upgrade() {
							drop(borrowed); // Release the borrow before getting another
							current = p;
						}
						else {
							break;
						}
					}
					_ => break,
				}
			}

			path_parts.reverse();
			let path = path_parts.join("/");

			fs.path_map.insert(path, new_file.clone());
		}
	}
}

impl FileSystem {
	/// Creates a new filesystem with a root node
	pub fn new() -> Self {
		let root = Rc::new(RefCell::new(FsNode::Root));
		FileSystem {
			root: root.clone(),
			nodes: vec![root],
			path_map: HashMap::new(),
		}
	}

	fn parse_header(&mut self, header: &str, strip_prefix: impl Fn(&str) -> String, debug: bool) {
		// Check if we're parsing a tree format or an indented list format
		let is_list_format = header.contains(" - ");

		if is_list_format {
			// For "File structure" format with indented "-" items
			if debug {
				println!("Parsing with indented list format parser");
			}
			parse_indented_list(self, header, strip_prefix, debug);
		}
		else {
			// For tree command format with indentation characters
			if debug {
				println!("Parsing with tree command format parser");
			}
			parse_tree_format(self, header, strip_prefix, debug);
		}
	}

	fn parse_body(&mut self, parts: &[&str], comment_prefix: Option<&str>, debug: bool) -> Result<()> {
		// Skip if there's no body
		if parts.len() <= 1 {
			return Ok(());
		}

		let mut current_file: Option<NodeRef> = None;
		let mut current_contents = String::new();

		// Process each line of the body
		for (i, body_part) in parts.iter().skip(1).enumerate() {
			let lines = body_part.lines();

			for line in lines {
				// If there's a comment_prefix and the line starts with it,
				// it might be a file path (start of a new file's contents)
				if let Some(prefix) = comment_prefix {
					if let Some(stripped) = line.strip_prefix(prefix) {
						// Save the previous file's contents if there was one
						if let Some(file) = current_file {
							if !current_contents.is_empty() {
								let mut file_ref = file.borrow_mut();
								file_ref.set_contents(current_contents.clone())?;
								current_contents.clear();
							}
						}

						// Get the new file path
						let path = stripped.trim();

						// Look up the file in our filesystem
						current_file = match self.get_node(path) {
							Ok(node) => Some(node),
							Err(_) => {
								if debug {
									println!("Warning: File not found at path: {}", path);
								}
								None
							}
						};
						continue;
					}
				}

				// If we have a current file, add this line to its contents
				if current_file.is_some() {
					if !current_contents.is_empty() {
						current_contents.push('\n');
					}
					current_contents.push_str(line);
				}
			}

			// If this isn't the last body part, add a blank line between parts
			if i < parts.len() - 2 && !current_contents.is_empty() {
				current_contents.push_str("\n\n");
			}
		}

		// Save the last file's contents if there is one
		if let Some(file) = current_file {
			if !current_contents.is_empty() {
				let mut file_ref = file.borrow_mut();
				file_ref.set_contents(current_contents)?;
			}
		}

		Ok(())
	}

	/// Parse the output of the tree command including file contents
	///
	/// Stops parsing the header when it encounters a blank line, treating
	/// everything up to that point as the header. After the header, it parses
	/// file contents where lines starting with the comment prefix (if provided)
	/// and followed by a path indicate the file path, and subsequent lines are
	/// the file contents.
	///
	/// If comment_prefix is provided, it will strip that prefix from all lines
	/// in the header.
	///
	/// Supports both tree command format (with ├── etc.) and simple indented
	/// list format (with - item)
	///
	/// If debug is true, additional information about the parsing process will
	/// be printed.
	pub fn parse(input: &str, comment_prefix: Option<&str>, debug: bool) -> Result<Self> {
		let mut fs = FileSystem::new();

		if debug {
			println!("Debug mode enabled");
			println!("Input length: {} characters", input.len());
			if let Some(prefix) = comment_prefix {
				println!("Comment prefix: '{}'", prefix);
			}
			else {
				println!("No comment prefix provided");
			}
		}

		// Split the input into header and body parts
		let parts: Vec<&str> = input.split("\n\n").collect();
		let header = parts.first().ok_or_else(|| Error::Parse("Input is empty".to_string()))?;

		// Helper function to strip comment prefix if provided
		let strip_prefix = |line: &str| -> String {
			if let Some(prefix) = comment_prefix {
				if let Some(string) = line.strip_prefix(prefix) {
					string.to_string()
				}
				else {
					line.to_string()
				}
			}
			else {
				line.to_string()
			}
		};

		// Parse the header to build the file structure
		fs.parse_header(header, strip_prefix, debug);

		// Parse the body to extract file contents
		fs.parse_body(&parts, comment_prefix, debug)?;

		if debug {
			println!("Parsing complete: {} nodes created", fs.nodes().len());
		}

		Ok(fs)
	}

	/// Returns the tree output representation of this filesystem
	pub fn tree_output(&self) -> String {
		let mut result = String::new();

		// Get the name of the root directory
		let root_name = self.root.borrow().name();
		if !root_name.is_empty() {
			result.push_str(&format!("{}/\n", root_name));
		}

		// Recursively print all children
		Self::tree_output_recursive(&self.root, &mut result, 0, &[]);

		result
	}

	fn tree_output_recursive(
		node: &NodeRef,
		result: &mut String,
		depth: usize,
		is_last: &[bool],
	) {
		let node_ref = node.borrow();

		match &*node_ref {
			FsNode::Root => {
				// Root node only prints its children
				if let FsNode::Directory { children, .. } = &*node_ref {
					for (i, child) in children.iter().enumerate() {
						let is_last_child = i == children.len() - 1;
						let mut new_is_last = is_last.to_vec();
						new_is_last.push(is_last_child);
						Self::tree_output_recursive(child, result, depth, &new_is_last);
					}
				}
			}
			FsNode::Directory { name, children, .. } => {
				if depth > 0 {
					// Print directory prefix
					for &is_last_item in is_last.iter().take(depth - 1) {
						if is_last_item {
							result.push_str("    ");
						}
						else {
							result.push_str("│   ");
						}
					}

					// Print the branch character
					if is_last[depth - 1] {
						result.push_str("└── ");
					}
					else {
						result.push_str("├── ");
					}

					result.push_str(&format!("{}/\n", name));
				}

				// Print children
				for (i, child) in children.iter().enumerate() {
					let is_last_child = i == children.len() - 1;
					let mut new_is_last = is_last.to_vec();
					new_is_last.push(is_last_child);
					Self::tree_output_recursive(child, result, depth + 1, &new_is_last);
				}
			}
			FsNode::File { name, .. } => {
				// Print file prefix
				for &is_last_item in is_last.iter().take(depth - 1) {
					if is_last_item {
						result.push_str("    ");
					}
					else {
						result.push_str("│   ");
					}
				}

				// Print the branch character
				if is_last[depth - 1] {
					result.push_str("└── ");
				}
				else {
					result.push_str("├── ");
				}

				result.push_str(&format!("{}\n", name));
			}
		}
	}

	/// Returns all nodes in the filesystem
	pub fn nodes(&self) -> &[NodeRef] {
		&self.nodes
	}

	/// Returns a node by its path or an error if not found
	pub fn get_node(&self, path: &str) -> Result<NodeRef> {
		// Directly look up in path map (fast path)
		if let Some(node) = self.path_map.get(path) {
			return Ok(node.clone());
		}

		// Split the path into components
		let components: Vec<&str> = path.split('/').collect();
		if components.is_empty() {
			return Ok(self.root.clone());
		}

		// Start search from the root
		let mut current = self.root.clone();

		// Special case: the first component might be the root directory name
		let mut start_idx = 0;
		let root_name = current.borrow().name();
		if !root_name.is_empty() && components[0] == root_name {
			start_idx = 1;
		}

		// Navigate through each path component
		for &component in &components[start_idx..] {
			let found = match &*current.borrow() {
				FsNode::Directory { children, .. } => {
					children.iter().find(|node| node.borrow().name() == component).cloned()
				}
				_ => None,
			};

			match found {
				Some(node) => current = node,
				None => return Err(Error::NodeNotFound(path.to_string())), // Component not found
			}
		}

		Ok(current)
	}

	/// Write the entire filesystem to disk
	pub fn write_to_disk(&self, output_dir: impl AsRef<std::path::Path>) -> Result<()> {
		let output_dir = output_dir.as_ref();
		for node in &self.nodes {
			node.borrow().write_to_disk(output_dir)?;
		}
		Ok(())
	}
}

impl Default for FileSystem {
	fn default() -> Self {
		Self::new()
	}
}

impl fmt::Display for FileSystem {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.tree_output())
	}
}

