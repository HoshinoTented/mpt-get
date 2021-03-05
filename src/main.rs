mod error;
mod index;
mod logger;
mod get;
mod config;

use std::iter::FromIterator;

use clap::{App, Arg, SubCommand};
use index::{PackageVersion, PackageEntry, Packages, Updater};
use serde_json::Value;
use config::Config;
use logger::StdioLogger;

fn main() {
    let matches = App::new("mpt-get")
        .version("0.1.0")
        .subcommand(SubCommand::with_name("update").about("Update index from remote server"))
        .subcommand(SubCommand::with_name("list").about("List all packages"))
        .subcommand(
            SubCommand::with_name("show")
                .alias("info")
                .about("Show package information")
                .arg(Arg::with_name("PKG").required(true))
                .arg(Arg::with_name("All").short("a").help("Print full information")),
        )
        .subcommand(
            SubCommand::with_name("install")
                // .alias("get")
                .about("Download target package")
                .arg(Arg::with_name("PKG").help("Package ID").required(true).index(1))
                .arg(Arg::with_name("VERSION").help("Optional. Install package of target version").index(2))
        )
        .get_matches();

    let config = Config::default();     // read from file;
    let updater = config.updater::<StdioLogger>();

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

        ("info", Some(arg)) | ("show", Some(arg)) => {
            let pid = arg.value_of("PKG").expect("unreachable");
            let pid = serde_json::from_value(Value::String(pid.to_string())).unwrap();
            let pkgs = updater.index().unwrap();
            let pkg = pkgs.map.get(&pid).expect(&format!("Package {} not found. Try to update index.", pid));
            let vers = PackageVersion::from_pid(&pid, &updater.index_dir()).unwrap();
            let indent = String::from_iter(vec![' '; 4]);

            fn newest_version(pkg: PackageVersion) -> String {
                let channels = pkg.channels;
                const PRIORITY: [&'static str; 3] = ["stable", "nightly", "beta"];

                let ver_str = PRIORITY.iter().fold(Option::<String>::None, |acc, channel| {
                    if acc.is_none() {
                        let versions = channels.get(*channel)?;
                        let ver = versions.last()?;

                        Some(format!("[{}] {}", channel, ver))
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

            fn all_versions<S: AsRef<str>>(pkg: PackageVersion, first_level_indent: S, second_level_indent: S) -> String {
                let channels = pkg.channels;
                let mut vers_str = String::with_capacity(1024);

                for (channel, vers) in channels {
                    let vers_list = vers.into_iter().fold(String::with_capacity(128), |mut acc, ver| {
                        acc.push_str(second_level_indent.as_ref());
                        acc.push_str(ver.as_ref());
                        acc.push('\n');
                        acc
                    });

                    vers_str.push_str(
                        &format!{
                            "{indent0}[{channel}]: \n{vers}",
                            channel = channel, vers = vers_list,
                            indent0 = first_level_indent.as_ref()
                        }
                    )
                }

                vers_str
            }

            println!(r#"{1}:
{0}name: {2}
{0}description: {3}
{0}channels: {4:?}
{0}website: {5}"#, indent, pid, pkg.name, pkg.description, pkg.channels, pkg.website);

            if let None = arg.index_of("All") {
                println!("{}newest version: {}", indent, newest_version(vers))
            } else {
                println!("{}all versions: ", indent);
                print!("{}", all_versions(vers, &indent, &format!("{0}{0}", &indent)));
            }
        }

        ("install", arg) => {
            println!("JUST FOR TEST!");

            // get::tests::test_downloading();
        }

        (name, arg) => panic!("Invalid command '{}'. Please use 'mpt-get --help' for more information.", name),
    }
}
