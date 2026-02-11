use std::path::Path;

use super::collect_md_files;

pub fn run(spec_dir: &Path) -> anyhow::Result<Vec<String>> {
	let files = collect_md_files(spec_dir)?;
	let mut errors = Vec::new();

	for file in &files {
		let content = std::fs::read_to_string(file)?;
		let mut in_code_block = false;

		for (line_num, line) in content.lines().enumerate() {
			in_code_block = super::is_in_code_block(line, in_code_block);
			if in_code_block || line.starts_with("```") {
				continue;
			}

			let prose = strip_inline_code(line);
			let lower = prose.to_lowercase();

			if contains_word_pair(&lower, "lib", "crate") {
				errors.push(format!(
					"TERMINOLOGY: {}:{} — use \"library crate\" instead of \"lib crate\"",
					file.display(),
					line_num + 1
				));
			}

			if contains_word_pair(&lower, "bin", "crate") {
				errors.push(format!(
					"TERMINOLOGY: {}:{} — use \"binary crate\" instead of \"bin crate\"",
					file.display(),
					line_num + 1
				));
			}
		}
	}

	Ok(errors)
}

fn strip_inline_code(line: &str) -> String {
	let mut result = String::new();
	let mut in_code = false;
	for c in line.chars() {
		if c == '`' {
			in_code = !in_code;
		} else if !in_code {
			result.push(c);
		}
	}
	result
}

fn contains_word_pair(text: &str, first: &str, second: &str) -> bool {
	let pattern = format!("{first} {second}");
	for (i, _) in text.match_indices(&pattern) {
		let before_ok = i == 0 || !text.as_bytes()[i - 1].is_ascii_alphanumeric();
		let end = i + pattern.len();
		let after_ok = end >= text.len() || !text.as_bytes()[end].is_ascii_alphanumeric();
		if before_ok && after_ok {
			return true;
		}
	}
	false
}
