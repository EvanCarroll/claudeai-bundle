use anyhow::Context;
use clap::{Parser, Subcommand};
use std::{
	fs,
	io::{self, Read},
	path::PathBuf,
};

use claudeai_bundle::{Error, FileSystem, FsNode};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,

	/// Path to a file containing tree output
	#[arg(short, long)]
	file: Option<PathBuf>,

	/// Prefix to strip from each line in the header (e.g. "# ")
	#[arg(long)]
	comment_prefix: Option<String>,

	/// Enable debug mode with verbose output
	#[arg(long, default_value_t = false)]
	debug: bool,
}

#[derive(Subcommand)]
enum Commands {
	/// Find a node by its path
	Find {
		/// The path to find
		path: String,
	},
	/// List all nodes in the filesystem
	List {
		/// Display results in a tree format
		#[arg(short, long)]
		tree: bool,
	},
	/// Display the contents of a file
	Cat {
		/// The path to the file to display
		path: String,
	},
	/// Extract files and directories to the filesystem
	Extract {
		/// Directory to extract files to
		#[arg(long, alias = "output_dir", default_value = ".")]
		output_directory: PathBuf,
	},
}

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	// Get tree output from file or stdin
	let tree_output = if let Some(file_path) = cli.file {
		fs::read_to_string(&file_path).with_context(|| {
			format!(
				"Failed to read tree output from file: {}",
				file_path.display()
			)
		})?
	}
	else {
		let mut buffer = String::new();
		io::stdin()
			.read_to_string(&mut buffer)
			.context("Failed to read tree output from stdin")?;
		buffer
	};

	// For the simple_tree.txt example, we need to detect and apply the comment
	// prefix
	let comment_prefix = if tree_output.starts_with("// ") && cli.comment_prefix.is_none() {
		if cli.debug {
			println!("Auto-detected comment prefix: \"// \"");
		}
		Some("// ")
	}
	else {
		cli.comment_prefix.as_deref()
	};

	// Parse the tree output
	let fs = FileSystem::parse(&tree_output, comment_prefix, cli.debug)
		.context("Failed to parse tree output")?;

	// Process command or default to print
	match cli.command {
		Some(Commands::Find { path }) => println!(
			"{}",
			fs.get_node(&path)?.borrow().relative_location().display()
		),
		Some(Commands::List { tree }) => {
			if tree {
				if cli.debug {
					println!("Printing tree hierarchy");
				}
				// Display in hierarchical tree format
				println!("{}", fs.tree_output());
			}
			else {
				if cli.debug {
					println!("Printing list hierarchy");
				}
				// Display flat list
				for node in fs.nodes() {
					println!("{}", node.borrow().relative_location().display());
				}
			}
		}
		Some(Commands::Cat { path }) => {
			// Display file contents
			let node = match fs.get_node(&path) {
				Ok(node) => node,
				Err(err) => {
					println!("File not found: {}", path);
					return Err(
						anyhow::anyhow!(err).context(format!("Could not find file: {}", path))
					);
				}
			};

			let node_ref = node.borrow();

			if let Some(contents) = node_ref.contents() {
				println!("{}", contents);
			}
			else if node_ref.is_directory() {
				println!("Cannot display contents of directory: {}", path);
				return Err(anyhow::anyhow!(Error::InvalidNodeType)
					.context(format!("Cannot display contents of directory: {}", path)));
			}
			else {
				println!("File has no contents: {}", path);
			}
		}
		Some(Commands::Extract { output_directory }) => {
			// Create the root output directory
			if cli.debug {
				println!("Using output directory: {}", output_directory.display());
			}

			// Ensure the output directory exists before extracting
			fs::create_dir_all(&output_directory).with_context(|| {
				format!(
					"Failed to create output directory: {}",
					output_directory.display()
				)
			})?;

			if cli.debug {
				println!(
					"Ensured output directory exists: {}",
					output_directory.display()
				);
			}

			// Process all nodes using the write_to_disk method
			for node in fs.nodes() {
				let node_ref = node.borrow();

				// Skip the root node
				if matches!(*node_ref, FsNode::Root) {
					continue;
				}

				if cli.debug {
					let path = node_ref.relative_location();
					println!("Writing {} to disk", path.display());
				}

				// Write the node to disk
				node_ref.write_to_disk(&output_directory).with_context(|| {
					format!(
						"Failed to write {} to disk",
						node_ref.relative_location().display()
					)
				})?;

				if cli.debug {
					println!(
						"Successfully wrote: {}",
						node_ref.relative_location().display()
					);
				}
			}

			println!("Extracted to: {}", output_directory.display());
		}
		None => {
			// Default: just output the tree
			println!("{}", fs.tree_output());
		}
	}

	Ok(())
}
