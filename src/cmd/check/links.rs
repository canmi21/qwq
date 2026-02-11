use std::path::Path;

use super::collect_md_files;

pub fn run(spec_dir: &Path) -> anyhow::Result<Vec<String>> {
	let files = collect_md_files(spec_dir)?;
	let mut errors = Vec::new();

	for file in &files {
		let content = std::fs::read_to_string(file)?;
		let dir = file.parent().unwrap_or(spec_dir);
		let mut in_code_block = false;

		for (line_num, line) in content.lines().enumerate() {
			in_code_block = super::is_in_code_block(line, in_code_block);
			if in_code_block || line.starts_with("```") {
				continue;
			}

			let mut rest = line;
			while let Some(start) = rest.find("](") {
				let after = &rest[start + 2..];
				let Some(end) = after.find(')') else {
					break;
				};
				let target = &after[..end];
				rest = &after[end + 1..];

				if target.starts_with("http://")
					|| target.starts_with("https://")
					|| target.starts_with('#')
				{
					continue;
				}

				let target_path = target.split('#').next().unwrap_or(target);
				if target_path.is_empty() {
					continue;
				}

				let resolved = dir.join(target_path);
				if !resolved.exists() {
					errors.push(format!(
						"BROKEN LINK: {}:{} -> {target_path}",
						file.display(),
						line_num + 1
					));
				}
			}
		}
	}

	Ok(errors)
}
