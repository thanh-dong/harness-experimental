mod application;
mod domain;
mod events;
mod infrastructure;
mod interface;

use clap::Parser;

fn main() {
    let cli = interface::Cli::parse();
    let output = interface::OutputMode::resolve(cli.json_output());
    if let Err(error) = interface::run(cli, output) {
        let code = error.exit_code();
        match output {
            interface::OutputMode::Json => {
                println!("{}", interface::error_envelope(&error));
            }
            interface::OutputMode::Human => {
                eprintln!("error: {error}");
            }
        }
        std::process::exit(code);
    }
}
