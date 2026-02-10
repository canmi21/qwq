use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qwq", about = "Manage the snowball project")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Format source files
	Fmt(crate::cmd::fmt::Args),
}

pub fn run() -> anyhow::Result<()> {
	let cli = Cli::parse();
	match cli.command {
		Command::Fmt(args) => crate::cmd::fmt::execute(args),
	}
}
