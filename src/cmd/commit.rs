use anyhow::{Context, bail};

#[derive(clap::Args)]
pub struct Args {
	/// Commit message in type(scope): description format
	#[arg(short)]
	pub m: String,

	/// Files to split (triggers jj describe + jj split instead of jj commit)
	#[arg(last = true)]
	pub files: Vec<String>,
}

const VALID_TYPES: &[&str] = &[
	"add", "fix", "change", "rm", "break", "refactor", "doc", "test", "spec", "ci", "chore",
];

pub fn execute(args: Args) -> anyhow::Result<()> {
	validate_message(&args.m)?;

	if args.files.is_empty() {
		jj_commit(&args.m)
	} else {
		jj_split(&args.m, &args.files)
	}
}

fn validate_message(msg: &str) -> anyhow::Result<()> {
	let subject = msg.lines().next().unwrap_or("");

	// format: type(scope): lowercase description, no trailing period
	let valid = match subject.split_once('(') {
		Some((type_part, rest)) => {
			let type_ok = VALID_TYPES.contains(&type_part);
			match rest.split_once("): ") {
				Some((scope, desc)) => {
					let scope_ok = !scope.is_empty()
						&& scope.starts_with(|c: char| c.is_ascii_lowercase())
						&& scope
							.chars()
							.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
					let desc_ok = desc.starts_with(|c: char| c.is_ascii_lowercase()) && !desc.ends_with('.');
					type_ok && scope_ok && desc_ok
				}
				None => false,
			}
		}
		None => false,
	};

	if !valid {
		let types_str = VALID_TYPES.join(", ");
		bail!(
			"subject does not match type(scope): description format.\n  \
			 subject: {subject}\n  \
			 valid types: {types_str}"
		);
	}

	if subject.len() > 72 {
		bail!(
			"subject exceeds 72 characters ({} chars).\n  subject: {subject}",
			subject.len()
		);
	}

	// no footer in body
	let body: String = msg.lines().skip(1).collect::<Vec<_>>().join("\n");
	if !body.is_empty() {
		for line in body.lines() {
			let has_footer = line
				.split_once(": ")
				.is_some_and(|(key, _)| key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
			let has_ref = line
				.split_once(" #")
				.is_some_and(|(key, _)| key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
			if has_footer || has_ref {
				bail!("commit body contains a footer section.");
			}
		}
	}

	Ok(())
}

fn jj_commit(msg: &str) -> anyhow::Result<()> {
	let status = std::process::Command::new("jj")
		.args(["commit", "-m", msg])
		.status()
		.context("failed to execute jj")?;

	if !status.success() {
		std::process::exit(status.code().unwrap_or(1));
	}
	Ok(())
}

fn jj_split(msg: &str, files: &[String]) -> anyhow::Result<()> {
	let describe = std::process::Command::new("jj")
		.args(["describe", "-m", msg])
		.status()
		.context("failed to execute jj describe")?;

	if !describe.success() {
		std::process::exit(describe.code().unwrap_or(1));
	}

	let mut split_cmd = std::process::Command::new("jj");
	split_cmd.args(["split", "--"]);
	for file in files {
		split_cmd.arg(file);
	}
	split_cmd.env("JJ_EDITOR", "true");

	let split = split_cmd.status().context("failed to execute jj split")?;

	if !split.success() {
		std::process::exit(split.code().unwrap_or(1));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn valid_messages() {
		assert!(validate_message("add(qwq): implement commit subcommand").is_ok());
		assert!(validate_message("fix(my-crate): handle empty input").is_ok());
		assert!(validate_message("spec(foundation): add vcs rules").is_ok());
		assert!(validate_message("chore(workspace): update dependencies").is_ok());
	}

	#[test]
	fn invalid_type() {
		assert!(validate_message("init(qwq): first commit").is_err());
	}

	#[test]
	fn trailing_period() {
		assert!(validate_message("add(qwq): implement commit.").is_err());
	}

	#[test]
	fn uppercase_description() {
		assert!(validate_message("add(qwq): Implement commit").is_err());
	}

	#[test]
	fn too_long() {
		let long = format!("add(qwq): {}", "a".repeat(63));
		assert!(validate_message(&long).is_err());
	}

	#[test]
	fn footer_rejected() {
		assert!(validate_message("add(qwq): something\n\nSigned-off-by: someone").is_err());
	}
}
