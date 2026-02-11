use std::path::Path;

use sha2::{Digest, Sha256};

use super::collect_md_files;

const THRESHOLD: usize = 100;

pub fn run(spec_dir: &Path, repo_root: &Path) -> anyhow::Result<Vec<String>> {
	let files = collect_md_files(spec_dir)?;
	let allowlist = load_allowlist(&repo_root.join("tools/check/allowlist.toml"))?;
	let mut errors = Vec::new();

	for file in &files {
		let name = file.file_name().unwrap_or_default().to_string_lossy();
		if name == "CHANGELOG.md" || name == "README.md" {
			continue;
		}

		let content = std::fs::read_to_string(file)?;
		let count = content.lines().count();

		if count > THRESHOLD {
			let rel_path = file.strip_prefix(repo_root).unwrap_or(file);
			let hash = format!("{:x}", Sha256::digest(&content));

			let allowed = allowlist
				.iter()
				.any(|e| e.file == rel_path.to_string_lossy() && e.check == "line-count" && e.hash == hash);

			if !allowed {
				errors.push(format!(
					"OVER {THRESHOLD} LINES: {} ({count} lines)",
					file.display()
				));
			}
		}
	}

	Ok(errors)
}

struct AllowEntry {
	file: String,
	check: String,
	hash: String,
}

fn load_allowlist(path: &Path) -> anyhow::Result<Vec<AllowEntry>> {
	if !path.exists() {
		return Ok(Vec::new());
	}

	let content = std::fs::read_to_string(path)?;
	let mut entries = Vec::new();
	let mut file = String::new();
	let mut check = String::new();
	let mut hash = String::new();

	for line in content.lines() {
		if line == "[[entry]]" {
			if !file.is_empty() {
				entries.push(AllowEntry {
					file: file.clone(),
					check: check.clone(),
					hash: hash.clone(),
				});
			}
			file.clear();
			check.clear();
			hash.clear();
			continue;
		}

		if let Some(val) = line
			.strip_prefix("file = \"")
			.and_then(|s| s.strip_suffix('"'))
		{
			file = val.to_string();
		} else if let Some(val) = line
			.strip_prefix("check = \"")
			.and_then(|s| s.strip_suffix('"'))
		{
			check = val.to_string();
		} else if let Some(val) = line
			.strip_prefix("hash = \"")
			.and_then(|s| s.strip_suffix('"'))
		{
			hash = val.to_string();
		}
	}

	if !file.is_empty() {
		entries.push(AllowEntry { file, check, hash });
	}

	Ok(entries)
}
