use std::io;
use thiserror::Error;

/// Error types for the claudeai-extract library
#[derive(Error, Debug)]
pub enum Error {
	/// IO errors when reading or writing files
	#[error("I/O error: {0}")]
	Io(#[from] io::Error),

	/// Failed to parse tree structure
	#[error("Parse error: {0}")]
	Parse(String),

	/// Error during node path resolution
	#[error("Path resolution error: {0}")]
	PathResolution(String),

	/// Failed to find node at the specified path
	#[error("Node not found: {0}")]
	NodeNotFound(String),

	/// Error when attempting to set contents on a non-file node
	#[error("Cannot set contents on a non-file node")]
	InvalidNodeType,
}

/// Result type shorthand for Error
pub type Result<T> = std::result::Result<T, Error>;

