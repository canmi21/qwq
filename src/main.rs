mod config;
mod run;

fn main() {
	let config = config::load();
	if let Err(e) = run::run(config) {
		eprintln!("Error: {e:?}");
		std::process::exit(1);
	}
}
