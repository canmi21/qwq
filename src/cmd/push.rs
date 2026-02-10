use anyhow::Context;

#[derive(clap::Args)]
pub struct Args;

pub fn execute(_args: Args) -> anyhow::Result<()> {
	let status = std::process::Command::new("jj")
		.args(["git", "push"])
		.status()
		.context("failed to execute jj git push")?;

	if !status.success() {
		std::process::exit(status.code().unwrap_or(1));
	}
	Ok(())
}
