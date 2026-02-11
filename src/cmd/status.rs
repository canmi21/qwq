use anyhow::Context;

#[derive(clap::Args)]
pub struct Args {
	/// Additional arguments passed to jj status
	#[arg(trailing_var_arg = true, allow_hyphen_values = true)]
	pub args: Vec<String>,
}

pub fn execute(args: Args) -> anyhow::Result<()> {
	let mut cmd = std::process::Command::new("jj");
	cmd.arg("status");
	for arg in &args.args {
		cmd.arg(arg);
	}

	let status = cmd.status().context("failed to execute jj status")?;
	if !status.success() {
		std::process::exit(status.code().unwrap_or(1));
	}
	Ok(())
}
