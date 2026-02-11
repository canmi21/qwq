use std::path::Path;

pub fn run(spec_dir: &Path) -> anyhow::Result<Vec<String>> {
	let changelog = spec_dir.join("CHANGELOG.md");
	let mut errors = Vec::new();

	if !changelog.exists() {
		errors.push("MISSING: spec/CHANGELOG.md".to_string());
		return Ok(errors);
	}

	let content = std::fs::read_to_string(&changelog)?;
	let lines: Vec<&str> = content.lines().collect();

	// check header
	let has_header = lines.iter().take(3).any(|l| l.contains("# Changelog"));
	if !has_header {
		errors.push("FORMAT: spec/CHANGELOG.md missing '# Changelog' header".to_string());
	}

	// check version entries and category order
	let mut last_cat_order: u8 = 0;
	let mut current_version = String::new();

	for line in &lines {
		if line.starts_with("## [") {
			current_version = line.to_string();
			last_cat_order = 0;

			if !is_valid_timestamp_entry(line) {
				errors.push(format!("FORMAT: invalid version entry: {line}"));
			}
			continue;
		}

		if let Some(category) = line.strip_prefix("### ") {
			let order = cat_order(category);

			if order == 0 {
				errors.push(format!(
					"FORMAT: unknown category '{category}' in {current_version}"
				));
			} else if order < last_cat_order {
				errors.push(format!(
					"FORMAT: wrong category order '{category}' in {current_version} \
					 (expected: Breaking, Added, Changed, Fixed, Removed)"
				));
			}
			last_cat_order = order;
		}
	}

	Ok(errors)
}

fn is_valid_timestamp_entry(line: &str) -> bool {
	// ## [2026-02-10T16:16:38Z]
	let Some(rest) = line.strip_prefix("## [") else {
		return false;
	};
	let Some(ts) = rest.strip_suffix(']') else {
		return false;
	};
	// YYYY-MM-DDTHH:MM:SSZ
	ts.len() == 20
		&& ts.as_bytes()[4] == b'-'
		&& ts.as_bytes()[7] == b'-'
		&& ts.as_bytes()[10] == b'T'
		&& ts.as_bytes()[13] == b':'
		&& ts.as_bytes()[16] == b':'
		&& ts.as_bytes()[19] == b'Z'
}

fn cat_order(category: &str) -> u8 {
	match category {
		"Breaking" => 1,
		"Added" => 2,
		"Changed" => 3,
		"Fixed" => 4,
		"Removed" => 5,
		_ => 0,
	}
}
