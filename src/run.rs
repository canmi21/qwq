use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qwq", about = "Manage the snowball project")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Run compliance checks
	Check(crate::cmd::check::Args),
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
	/// Show changes in working copy
	Diff(crate::cmd::diff::Args),
	/// Fast-forward main bookmark to latest commit
	Land(crate::cmd::land::Args),
	/// Show commit history
	Log(crate::cmd::log::Args),
	/// Push bookmarks to remote
	Push(crate::cmd::push::Args),
	/// Show working copy status
	Status(crate::cmd::status::Args),
}

pub fn run() -> anyhow::Result<()> {
	let cli = Cli::parse();
	match cli.command {
		Command::Check(args) => crate::cmd::check::execute(args),
		Command::Vcs(vcs) => match vcs.command {
			VcsCommand::Commit(args) => crate::cmd::commit::execute(args),
			VcsCommand::Diff(args) => crate::cmd::diff::execute(args),
			VcsCommand::Land(args) => crate::cmd::land::execute(args),
			VcsCommand::Log(args) => crate::cmd::log::execute(args),
			VcsCommand::Push(args) => crate::cmd::push::execute(args),
			VcsCommand::Status(args) => crate::cmd::status::execute(args),
		},
		Command::Fmt(args) => crate::cmd::fmt::execute(args),
	}
}
