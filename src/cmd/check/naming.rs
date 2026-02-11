use std::path::Path;

pub fn run(spec_dir: &Path) -> anyhow::Result<Vec<String>> {
	let mut errors = Vec::new();
	walk_check(spec_dir, &mut errors)?;
	Ok(errors)
}

fn is_allowed_uppercase(name: &str) -> bool {
	matches!(name, "CHANGELOG.md" | "README.md" | "VERSION")
}

fn walk_check(dir: &Path, errors: &mut Vec<String>) -> anyhow::Result<()> {
	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		let name = entry.file_name();
		let name_str = name.to_string_lossy();

		if !is_allowed_uppercase(&name_str) {
			let valid = name_str.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
				&& name_str
					.chars()
					.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-');

			if !valid {
				errors.push(format!("BAD NAME: {}", path.display()));
			}
		}

		if path.is_dir() {
			walk_check(&path, errors)?;
		}
	}
	Ok(())
}
