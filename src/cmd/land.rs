use anyhow::{Context, bail};

#[derive(clap::Args)]
pub struct Args;

pub fn execute(_args: Args) -> anyhow::Result<()> {
	// check if @ has uncommitted changes
	let status = std::process::Command::new("jj")
		.args(["status"])
		.output()
		.context("failed to execute jj status")?;

	let status_str = String::from_utf8_lossy(&status.stdout);
	if status_str.contains("Working copy changes:") {
		bail!("working copy has uncommitted changes — commit first");
	}

	// get @- change id and description
	let log = std::process::Command::new("jj")
		.args([
			"log",
			"--no-graph",
			"-r",
			"@-",
			"-T",
			r#"change_id ++ "\n" ++ description"#,
		])
		.output()
		.context("failed to execute jj log")?;

	let log_str = String::from_utf8_lossy(&log.stdout);
	let mut lines = log_str.lines();
	let change_id = lines.next().unwrap_or("");
	let description = lines.next().unwrap_or("");

	if change_id.is_empty() || description.is_empty() {
		bail!("no described commit found at @-");
	}

	// move main bookmark
	let set = std::process::Command::new("jj")
		.args(["bookmark", "set", "main", "-r", change_id])
		.output()
		.context("failed to execute jj bookmark set")?;

	if !set.status.success() {
		let stderr = String::from_utf8_lossy(&set.stderr);
		bail!("jj bookmark set failed: {stderr}");
	}

	let set_msg = String::from_utf8_lossy(&set.stderr);
	for line in set_msg.lines() {
		if !line.is_empty() {
			eprintln!("{line}");
		}
	}

	eprintln!("landed: {description}");
	Ok(())
}
