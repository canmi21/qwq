use std::path::PathBuf;

use anyhow::Context;

const CONFIG_FILE: &str = "qwq.toml";

pub struct FmtConfig {
	pub ignore: Vec<String>,
	pub rust: Vec<(String, String)>,
	pub oxfmt: Vec<(String, String)>,
}

impl Default for FmtConfig {
	fn default() -> Self {
		Self {
			ignore: vec!["target".to_string(), "node_modules".to_string()],
			rust: Vec::new(),
			oxfmt: Vec::new(),
		}
	}
}

enum Section {
	None,
	Fmt,
	FmtRust,
	FmtOxfmt,
}

pub fn load_fmt() -> anyhow::Result<FmtConfig> {
	let Some(path) = find_config() else {
		return Ok(FmtConfig::default());
	};

	let content =
		std::fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;

	let mut config = FmtConfig::default();
	let mut section = Section::None;

	for line in content.lines() {
		let trimmed = line.trim();
		if trimmed.is_empty() || trimmed.starts_with('#') {
			continue;
		}

		if trimmed.starts_with('[') {
			section = match trimmed {
				"[fmt]" => Section::Fmt,
				"[fmt.rust]" => Section::FmtRust,
				"[fmt.oxfmt]" => Section::FmtOxfmt,
				_ => Section::None,
			};
			continue;
		}

		let Some((key, val)) = parse_kv(trimmed) else {
			continue;
		};

		match section {
			Section::Fmt if key == "ignore" => {
				if let Some(arr) = val.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
					config.ignore = arr
						.split(',')
						.map(|s| s.trim().trim_matches('"').to_string())
						.filter(|s| !s.is_empty())
						.collect();
				}
			}
			Section::FmtRust => config.rust.push((key.to_string(), val.to_string())),
			Section::FmtOxfmt => config.oxfmt.push((key.to_string(), val.to_string())),
			_ => {}
		}
	}

	Ok(config)
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
	let (key, val) = line.split_once('=')?;
	Some((key.trim(), unquote(val.trim())))
}

fn unquote(s: &str) -> &str {
	s.strip_prefix('"')
		.and_then(|s| s.strip_suffix('"'))
		.unwrap_or(s)
}

pub fn find_config() -> Option<PathBuf> {
	let mut dir = std::env::current_dir().ok()?;
	loop {
		let candidate = dir.join(CONFIG_FILE);
		if candidate.exists() {
			return Some(candidate);
		}
		if !dir.pop() {
			return None;
		}
	}
}
