use clap::{Parser, Subcommand};
use log;
use plogger;

#[derive(Parser)]
#[clap(about, arg_required_else_help(true))]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    Run(run::RunCommand),
}

mod run;

fn main() {
    let cli = Cli::parse();

    plogger::init(cli.debug);

    log::info!("Welcome to csv-to-plinks");

    match cli.command {
        Some(Commands::Run(command)) => {
            run::command(&command);
        }
        None => {} // Handled by Clap
    }
}
