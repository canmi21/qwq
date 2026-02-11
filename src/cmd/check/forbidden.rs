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

			if contains_emoji(line) {
				errors.push(format!("EMOJI: {}:{}", file.display(), line_num + 1));
			}
		}
	}

	Ok(errors)
}

fn contains_emoji(line: &str) -> bool {
	line.chars().any(|c| {
		let cp = c as u32;
		(0x1F000..=0x1FFFF).contains(&cp)
			|| (0x2600..=0x27BF).contains(&cp)
			|| (0xFE00..=0xFE0F).contains(&cp)
			|| cp == 0x200D
	})
}
