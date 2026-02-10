use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qwq", about = "Manage the snowball project")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// VCS operations (commit, land, push, ...)
	#[command(alias = "git", alias = "jj")]
	Vcs(VcsArgs),
	/// Format source files
	Fmt(crate::cmd::fmt::Args),
}

#[derive(clap::Args)]
struct VcsArgs {
	#[command(subcommand)]
	command: VcsCommand,
}

#[derive(Subcommand)]
enum VcsCommand {
	/// Create a commit with validated message format
	Commit(crate::cmd::commit::Args),
}

pub fn run() -> anyhow::Result<()> {
	let cli = Cli::parse();
	match cli.command {
		Command::Vcs(vcs) => match vcs.command {
			VcsCommand::Commit(args) => crate::cmd::commit::execute(args),
		},
		Command::Fmt(args) => crate::cmd::fmt::execute(args),
	}
}
