mod cmd;
mod run;

fn main() {
	if let Err(e) = run::run() {
		eprintln!("Error: {e:?}");
		std::process::exit(1);
	}
}
