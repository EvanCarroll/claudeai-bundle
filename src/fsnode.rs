use std::{
	cell::RefCell,
	path::PathBuf,
	rc::{Rc, Weak},
};

use crate::error::{Error, Result};

// Type aliases to make the code more readable
pub type NodeRef = Rc<RefCell<FsNode>>;
pub type WeakNodeRef = Weak<RefCell<FsNode>>;

/// Represents a node in the filesystem
#[derive(Debug)]
pub enum FsNode {
	Root,
	File {
		name: String,
		parent: WeakNodeRef,
		contents: Option<String>,
	},
	Directory {
		name: String,
		parent: WeakNodeRef,
		children: Vec<NodeRef>,
	},
}

impl FsNode {
	/// Get a clone of the node's weak parent reference
	pub fn parent_ref(&self) -> Option<WeakNodeRef> {
		match self {
			FsNode::Root => None,
			FsNode::File { parent, .. } => Some(parent.clone()),
			FsNode::Directory { parent, .. } => Some(parent.clone()),
		}
	}

	/// Returns the name of this node
	pub fn name(&self) -> String {
		match self {
			FsNode::Root => String::new(),
			FsNode::File { name, .. } => name.clone(),
			FsNode::Directory { name, .. } => name.clone(),
		}
	}

	/// Returns true if this node is a directory
	pub fn is_directory(&self) -> bool {
		matches!(self, FsNode::Directory { .. } | FsNode::Root)
	}

	/// Returns the file contents if this is a file with contents
	pub fn contents(&self) -> Option<String> {
		match self {
			FsNode::File { contents, .. } => contents.clone(),
			_ => None,
		}
	}

	/// Sets the contents of a file
	pub fn set_contents(&mut self, contents: String) -> Result<()> {
		if let FsNode::File {
			contents: file_contents,
			..
		} = self
		{
			*file_contents = Some(contents);
			Ok(())
		}
		else {
			Err(Error::InvalidNodeType)
		}
	}

	/// Writes the node to disk in the specified directory
	pub fn write_to_disk(&self, output_dir: impl AsRef<std::path::Path>) -> Result<()> {
		use std::fs;

		// Get the relative path
		let relative_path = self.relative_location();
		if relative_path.as_os_str().is_empty() {
			return Ok(());
		}

		// Create a PathBuf from the output directory and relative path
		let output_path = PathBuf::from(output_dir.as_ref()).join(relative_path);

		match self {
			FsNode::Root => {
				// Root node doesn't need writing
				Ok(())
			}
			FsNode::Directory { .. } => {
				// Create directory
				fs::create_dir_all(&output_path)?;
				Ok(())
			}
			FsNode::File { .. } => {
				// Ensure parent directory exists
				if let Some(parent) = output_path.parent() {
					fs::create_dir_all(parent)?
				}

				// Write file contents
				let content = self.contents().unwrap_or_default();

				fs::write(&output_path, content)?;
				Ok(())
			}
		}
	}

	/// Returns the relative location of this node as a PathBuf
	pub fn relative_location(&self) -> PathBuf {
		match self {
			FsNode::Root => PathBuf::new(),
			_ => {
				let mut parts = Vec::new();

				// First add this node's name
				parts.push(self.name());

				// Then traverse up the parent chain
				let mut parent_ref = self.parent_ref();

				while let Some(parent_weak) = parent_ref {
					if let Some(parent) = parent_weak.upgrade() {
						let parent_node = parent.borrow();
						if let FsNode::Root = &*parent_node {
							break;
						}

						// Add the parent's name
						parts.push(parent_node.name());

						// Move up to the next parent
						parent_ref = parent_node.parent_ref();
					}
					else {
						break;
					}
				}

				// Reverse the parts and create a PathBuf
				parts.iter().rev().collect::<PathBuf>()
			}
		}
	}
}

