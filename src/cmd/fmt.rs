use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::config;

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

	let cfg = config::load_fmt()?;
	let mut failed = false;

	// rustfmt: collect .rs files ourselves (rustfmt doesn't walk directories)
	let rs_files = collect_rs_files(&path, &cfg.ignore)?;
	if !rs_files.is_empty() && !run_rustfmt(&rs_files, args.check)? {
		failed = true;
	}

	// oxfmt: pass the path directly, it handles file discovery
	if path.is_dir() {
		if !run_oxfmt(&path, args.check)? {
			failed = true;
		}
	} else {
		let ext = path.extension().unwrap_or_default().to_string_lossy();
		if is_oxfmt_ext(&ext) && !run_oxfmt(&path, args.check)? {
			failed = true;
		}
	}

	if failed {
		std::process::exit(1);
	}
	Ok(())
}

fn run_rustfmt(files: &[PathBuf], check: bool) -> anyhow::Result<bool> {
	let mut cmd = std::process::Command::new("rustfmt");
	if check {
		cmd.arg("--check");
	}
	for file in files {
		cmd.arg(file);
	}
	cmd.stdout(std::process::Stdio::inherit());
	cmd.stderr(std::process::Stdio::inherit());

	let status = cmd.status().context("failed to execute rustfmt")?;
	Ok(status.success())
}

fn run_oxfmt(path: &Path, check: bool) -> anyhow::Result<bool> {
	let mut cmd = std::process::Command::new("oxfmt");
	if check {
		cmd.arg("--check");
	} else {
		cmd.arg("--write");
	}
	cmd.arg(path);
	cmd.stdout(std::process::Stdio::inherit());
	cmd.stderr(std::process::Stdio::inherit());

	let status = cmd.status().context("failed to execute oxfmt")?;
	Ok(status.success())
}

fn is_oxfmt_ext(ext: &str) -> bool {
	matches!(
		ext,
		"js"
			| "jsx"
			| "ts"
			| "tsx"
			| "json"
			| "jsonc"
			| "json5"
			| "md"
			| "mdx"
			| "css"
			| "scss"
			| "less"
			| "html"
			| "vue"
			| "yaml"
			| "yml"
			| "toml"
			| "graphql"
	)
}

fn collect_rs_files(path: &Path, ignore: &[String]) -> anyhow::Result<Vec<PathBuf>> {
	if path.is_file() {
		if path.extension().is_some_and(|ext| ext == "rs") {
			return Ok(vec![path.to_path_buf()]);
		}
		return Ok(Vec::new());
	}
	let mut files = Vec::new();
	walk_rs(path, ignore, &mut files)?;
	files.sort();
	Ok(files)
}

fn walk_rs(dir: &Path, ignore: &[String], files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
	let entries =
		std::fs::read_dir(dir).with_context(|| format!("cannot read directory: {}", dir.display()))?;
	for entry in entries {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			let name = entry.file_name();
			let name_str = name.to_string_lossy();
			if name_str.starts_with('.') || ignore.iter().any(|i| i == name_str.as_ref()) {
				continue;
			}
			walk_rs(&path, ignore, files)?;
		} else if path.extension().is_some_and(|ext| ext == "rs") {
			files.push(path);
		}
	}
	Ok(())
}
