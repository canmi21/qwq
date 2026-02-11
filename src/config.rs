use std::path::PathBuf;

use anyhow::Context;

const CONFIG_FILE: &str = "qwq.toml";

pub struct FmtConfig {
	pub ignore: Vec<String>,
}

impl Default for FmtConfig {
	fn default() -> Self {
		Self {
			ignore: vec!["target".to_string(), "node_modules".to_string()],
		}
	}
}

pub fn load_fmt() -> anyhow::Result<FmtConfig> {
	let Some(path) = find_config() else {
		return Ok(FmtConfig::default());
	};

	let content =
		std::fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;

	let mut config = FmtConfig::default();

	let mut in_fmt = false;
	for line in content.lines() {
		let trimmed = line.trim();

		if trimmed.starts_with('[') {
			in_fmt = trimmed == "[fmt]";
			continue;
		}

		if !in_fmt {
			continue;
		}

		if let Some(val) = trimmed.strip_prefix("ignore") {
			let val = val.trim().strip_prefix('=').unwrap_or("").trim();
			if let Some(arr) = val.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
				config.ignore = arr
					.split(',')
					.map(|s| s.trim().trim_matches('"').to_string())
					.filter(|s| !s.is_empty())
					.collect();
			}
		}
	}

	Ok(config)
}

fn find_config() -> Option<PathBuf> {
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
