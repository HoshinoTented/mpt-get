mod error;
mod index;
mod logger;

use std::iter::FromIterator;

use clap::{App, Arg, SubCommand};
use index::{Package, PackageEntry, Packages, Updater};

fn main() {
    let matches = App::new("mpt-get")
        .version("0.1.0")
        .subcommand(SubCommand::with_name("update").about("Update index from remote server"))
        .subcommand(SubCommand::with_name("list").about("List all packages"))
        .subcommand(
            SubCommand::with_name("show")
                .alias("info")
                .about("Show package information")
                .arg(Arg::with_name("PKG").required(true)),
        )
        .get_matches();

    let updater = Updater::<logger::StdioLogger>::default()
        .expect("failed to get updater: cannot get home_dir");

    match matches.subcommand() {
        ("update", _) => {
            updater.update().unwrap();

            println!("Done. Use mpt-get list to get all indexed packages.");
        }

        ("list", _) => {
            let pkgs = updater.index().unwrap();
            
            fn println_entry(entry: &PackageEntry, indent: usize) {
                let PackageEntry {
                    name,
                    description,
                    channels,
                    website
                } = entry;

                let indent = String::from_iter(vec![' '; indent]);

                println!(r#"{0}name: {1}
{0}description: {2}"#, indent, name, description);
            }

            fn println_packages(pkgs: Packages, indent: usize) {
                for (id, entry) in pkgs.map.iter() {
                    println!("{}:", id);
                    println_entry(entry, indent);
                    println!();
                }
            }

            println_packages(pkgs, 4);
        }

        ("show", Some(arg)) => {
            let pid = arg.value_of("PKG").expect("unreachable");
            let pkgs = updater.index().unwrap();
            let pkg = pkgs.map.get(pid).expect(&format!("Package {} not found. Try to update index.", pid));
            let channels = Package::from_pid(pid, &updater.dir).unwrap();
            let indent = String::from_iter(vec![' '; 4]);

            fn newest_version(pkg: Package) -> String {
                let channels = pkg.channels;
                const PRIORITY: [&'static str; 3] = ["stable", "nightly", "beta"];

                let ver_str = PRIORITY.iter().fold(Option::<String>::None, |acc, channel| {
                    if acc.is_none() {
                        let versions = channels.get(*channel)?;
                        let ver = versions.first()?;

                        Some(format!("{}-{}", channel, ver))
                    } else {
                        acc
                    }
                });

                if let Some(ver_str) = ver_str {
                    ver_str
                } else {
                    String::from("invalid")
                }
            }

            println!(r#"{1}:
{0}name: {2}
{0}description: {3}
{0}channels: {4:?}
{0}website: {5}
{0}newest version: {6}"#, indent, pid, pkg.name, pkg.description, pkg.channels, pkg.website, newest_version(channels));
        }

        _ => todo!(),
    }
}
