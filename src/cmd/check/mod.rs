pub mod changelog;
pub mod forbidden;
pub mod line_count;
pub mod links;
pub mod naming;
pub mod terminology;

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use clap::Subcommand;

#[derive(clap::Args)]
pub struct Args {
	#[command(subcommand)]
	pub command: Option<CheckCommand>,
}

#[derive(Subcommand)]
pub enum CheckCommand {
	/// Check internal markdown links
	Links,
	/// Check file line counts
	LineCount,
	/// Check file and directory naming
	Naming,
	/// Check for forbidden patterns (emoji, etc.)
	Forbidden,
	/// Check terminology consistency
	Terminology,
	/// Check CHANGELOG format
	Changelog,
}

pub fn execute(args: Args) -> anyhow::Result<()> {
	let spec_dir = find_spec_dir()?;
	let repo_root = spec_dir
		.parent()
		.context("spec dir has no parent")?
		.to_path_buf();

	match args.command {
		None => run_all(&spec_dir, &repo_root),
		Some(CheckCommand::Links) => run_one("links", links::run(&spec_dir)),
		Some(CheckCommand::LineCount) => run_one("line-count", line_count::run(&spec_dir, &repo_root)),
		Some(CheckCommand::Naming) => run_one("naming", naming::run(&spec_dir)),
		Some(CheckCommand::Forbidden) => run_one("forbidden", forbidden::run(&spec_dir)),
		Some(CheckCommand::Terminology) => run_one("terminology", terminology::run(&spec_dir)),
		Some(CheckCommand::Changelog) => run_one("changelog", changelog::run(&spec_dir)),
	}
}

fn run_all(spec_dir: &Path, repo_root: &Path) -> anyhow::Result<()> {
	let checks: Vec<(&str, anyhow::Result<Vec<String>>)> = vec![
		("links", links::run(spec_dir)),
		("line-count", line_count::run(spec_dir, repo_root)),
		("naming", naming::run(spec_dir)),
		("forbidden", forbidden::run(spec_dir)),
		("terminology", terminology::run(spec_dir)),
		("changelog", changelog::run(spec_dir)),
	];

	let mut total_errors = 0;
	for (name, result) in checks {
		match result {
			Ok(errors) if errors.is_empty() => eprintln!("  {name}: ok"),
			Ok(errors) => {
				for e in &errors {
					eprintln!("  {e}");
				}
				total_errors += errors.len();
			}
			Err(e) => {
				eprintln!("  {name}: ERROR — {e}");
				total_errors += 1;
			}
		}
	}

	if total_errors > 0 {
		bail!("{total_errors} check error(s) found");
	}
	eprintln!("all checks passed");
	Ok(())
}

fn run_one(name: &str, result: anyhow::Result<Vec<String>>) -> anyhow::Result<()> {
	match result {
		Ok(errors) if errors.is_empty() => {
			eprintln!("{name}: ok");
			Ok(())
		}
		Ok(errors) => {
			for e in &errors {
				eprintln!("{e}");
			}
			bail!("{} error(s) found", errors.len());
		}
		Err(e) => bail!("{name}: {e}"),
	}
}

pub fn find_spec_dir() -> anyhow::Result<PathBuf> {
	let mut dir = std::env::current_dir().context("cannot determine current directory")?;
	loop {
		let candidate = dir.join("spec").join("VERSION");
		if candidate.exists() {
			return Ok(dir.join("spec"));
		}
		if !dir.pop() {
			bail!("cannot find spec directory (no spec/VERSION found in any parent)");
		}
	}
}

pub fn collect_md_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
	let mut files = Vec::new();
	walk_md(dir, &mut files)?;
	files.sort();
	Ok(files)
}

fn walk_md(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
	let entries =
		std::fs::read_dir(dir).with_context(|| format!("cannot read directory: {}", dir.display()))?;
	for entry in entries {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			walk_md(&path, files)?;
		} else if path.extension().is_some_and(|ext| ext == "md") {
			files.push(path);
		}
	}
	Ok(())
}

pub fn is_in_code_block(line: &str, in_code_block: bool) -> bool {
	if line.starts_with("```") {
		return !in_code_block;
	}
	in_code_block
}
