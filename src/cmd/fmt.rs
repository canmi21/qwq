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

	let rs_files = collect_rs_files(&path, &cfg.ignore)?;
	if !rs_files.is_empty() && !run_rustfmt(&rs_files, &cfg.rust, args.check)? {
		failed = true;
	}

	if path.is_dir() {
		if !run_oxfmt(&path, &cfg.oxfmt, args.check)? {
			failed = true;
		}
	} else {
		let ext = path.extension().unwrap_or_default().to_string_lossy();
		if is_oxfmt_ext(&ext) && !run_oxfmt(&path, &cfg.oxfmt, args.check)? {
			failed = true;
		}
	}

	if failed {
		std::process::exit(1);
	}
	Ok(())
}

fn run_rustfmt(
	files: &[PathBuf],
	rust_cfg: &[(String, String)],
	check: bool,
) -> anyhow::Result<bool> {
	let mut cmd = std::process::Command::new("rustfmt");
	if check {
		cmd.arg("--check");
	}

	if !rust_cfg.is_empty() {
		let config_str: Vec<String> = rust_cfg.iter().map(|(k, v)| format!("{k}={v}")).collect();
		cmd.arg("--config");
		cmd.arg(config_str.join(","));
	}

	for file in files {
		cmd.arg(file);
	}

	let status = cmd.status().context("failed to execute rustfmt")?;
	Ok(status.success())
}

fn run_oxfmt(path: &Path, oxfmt_cfg: &[(String, String)], check: bool) -> anyhow::Result<bool> {
	let config_dir = if path.is_dir() {
		path.to_path_buf()
	} else {
		path.parent().unwrap_or(path).to_path_buf()
	};
	let tmp_config = config_dir.join(".oxfmtrc.json");
	let wrote_config = if !oxfmt_cfg.is_empty() {
		let json = build_oxfmt_json(oxfmt_cfg);
		std::fs::write(&tmp_config, json).context("failed to write temporary .oxfmtrc.json")?;
		true
	} else {
		false
	};

	let mut cmd = std::process::Command::new("oxfmt");
	if check {
		cmd.arg("--check");
	} else {
		cmd.arg("--write");
	}
	cmd.arg(path);

	let result = cmd.status().context("failed to execute oxfmt");

	if wrote_config {
		let _ = std::fs::remove_file(&tmp_config);
	}

	Ok(result?.success())
}

fn build_oxfmt_json(cfg: &[(String, String)]) -> String {
	let mut entries = Vec::new();
	for (key, val) in cfg {
		let camel = to_camel_case(key);
		let json_val = if val == "true" || val == "false" || val.parse::<f64>().is_ok() {
			val.clone()
		} else {
			let unquoted = val.trim_matches('"');
			format!("\"{unquoted}\"")
		};
		entries.push(format!("\t\"{camel}\": {json_val}"));
	}
	format!("{{\n{}\n}}\n", entries.join(",\n"))
}

fn to_camel_case(s: &str) -> String {
	let mut result = String::new();
	let mut upper_next = false;
	for c in s.chars() {
		if c == '_' {
			upper_next = true;
		} else if upper_next {
			result.push(c.to_ascii_uppercase());
			upper_next = false;
		} else {
			result.push(c);
		}
	}
	result
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
