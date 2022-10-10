use std::io::Write;
use structopt::StructOpt;

mod cli;

/// Root CLI struct
#[derive(StructOpt, Debug)]
#[structopt(name = "AirDsl", about = "AirDsl CLI")]
pub struct Cli {
    #[structopt(subcommand)]
    action: Actions,
}

/// CLI actions
#[derive(StructOpt, Debug)]
pub enum Actions {
    Transpile(cli::TranspileCmd),
}

impl Cli {
    pub fn execute(&self) -> Result<(), String> {
        match &self.action {
            Actions::Transpile(transpile) => transpile.execute(),
        }
    }
}

pub fn main() {
    // configure logging
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug)
        .init();

    // read command-line args
    let cli = Cli::from_args();

    // execute cli action
    if let Err(error) = cli.execute() {
        println!("{}", error);
    }
}
