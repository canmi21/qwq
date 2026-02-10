use std::path::{Path, PathBuf};

use anyhow::Context;

#[derive(clap::Args)]
pub struct Args {
	/// Path to format (file or directory)
	pub path: PathBuf,

	/// Check formatting without modifying files
	#[arg(long)]
	pub check: bool,
}

pub fn execute(args: Args) -> anyhow::Result<()> {
	let path = args
		.path
		.canonicalize()
		.with_context(|| format!("path not found: {}", args.path.display()))?;

	let files = collect_rs_files(&path)?;
	if files.is_empty() {
		anyhow::bail!("no .rs files found in {}", path.display());
	}

	let mut cmd = std::process::Command::new("rustfmt");
	if args.check {
		cmd.arg("--check");
	}
	for file in &files {
		cmd.arg(file);
	}

	let status = cmd.status().context("failed to execute rustfmt")?;
	if !status.success() {
		std::process::exit(status.code().unwrap_or(1));
	}

	Ok(())
}

fn collect_rs_files(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
	if path.is_file() {
		return Ok(vec![path.to_path_buf()]);
	}
	let mut files = Vec::new();
	walk_dir(path, &mut files)?;
	files.sort();
	Ok(files)
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
	let entries =
		std::fs::read_dir(dir).with_context(|| format!("cannot read directory: {}", dir.display()))?;
	for entry in entries {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			let name = entry.file_name();
			let name_str = name.to_string_lossy();
			if !name_str.starts_with('.') && name_str != "target" {
				walk_dir(&path, files)?;
			}
		} else if path.extension().is_some_and(|ext| ext == "rs") {
			files.push(path);
		}
	}
	Ok(())
}
