mod error;
mod index;
mod logger;

use clap::{App, SubCommand};
use index::Updater;

fn main() {
    let matches = App::new("mpt-get")
        .version("0.1.0")
        .subcommand(SubCommand::with_name("update").about("Update index from remote server"))
        .subcommand(SubCommand::with_name("list").about("List all packages"))
        .get_matches();

    let updater = Updater::<logger::StdioLogger>::default()
        .expect("failed to get updater: cannot get home_dir");

    match matches.subcommand() {
        ("update", _) => {
            updater.update().unwrap();

            println!("Done. Use mpt-get list to get all indexed packages.");
        }

        ("list", _) => {
            let pkgs = updater.list().unwrap();

            println!("{}", pkgs.pretty_print())
        }

        _ => println!("error: no input, try mpt-get --help"),
    }
}
