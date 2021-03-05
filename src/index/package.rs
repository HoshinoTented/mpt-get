use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use anyhow::Result;

use std::{collections::HashMap, convert::TryFrom, fs::File, hash::Hash, io::BufReader, path::{Path, PathBuf}};

use crate::error::{index_err, io_err};

macro_rules! make_err {
    ( $file:literal , $reason:literal ) => {{
        let msg = if !$reason.is_empty() {
            format!("failed to parse {}: {}", $file, $reason)
        } else {
            format!("failed to parse {}", $file)
        };

        index_err(msg)
    }};

    ( $file:literal ) => {
        make_err!($file, "")
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct PackageID {
    pub domain: String,
    pub name: String
}

impl PackageID {
    pub fn to_path_str(&self) -> String {
        let domain_path = self.domain.replace(".", "/");

        format!("{}/{}", domain_path, self.name)
    }

    pub fn resolve_path(&self, mut index_dir: PathBuf) -> PathBuf {
        // for dir in self.domain.split(".") {
        //     index_dir.push(dir);
        // }

        // index_dir.push(&self.name);
        // index_dir.push("package.json");
    
        // index_dir

        index_dir.push(self.to_path_str());
        index_dir.push("package.json");
        index_dir
    }
}

impl std::hash::Hash for PackageID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.domain.hash(state);
        self.name.hash(state);
    }
}

impl std::fmt::Display for PackageID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.domain, self.name)
    }
}

impl <'de> Deserialize<'de> for PackageID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        use serde::de::Error;

        struct Visitor;

        impl <'t> serde::de::Visitor<'t> for Visitor {
            type Value = PackageID;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                write!(f, "string")
            }

            fn visit_str<E>(self, str: &str) -> std::result::Result<Self::Value, E> where E: Error {
                self.visit_string(str.to_string())
            }

            fn visit_string<E>(self, str: String) -> std::result::Result<Self::Value, E> where E: Error {
                let reg = Regex::new(r#"^([\w\d\.\-]+):([\w\d\.\-]+)$"#).unwrap();
                let matches = reg.captures(&str).ok_or(Error::custom("invalid pid"))?;

                let pid = PackageID {
                    domain: matches[1].to_string(),
                    name: matches[2].to_string()
                };

                Ok(pid)
            }
        }

        deserializer.deserialize_string(Visitor)
    }
}

#[derive(Debug)]
pub struct Packages {
    pub map: HashMap<PackageID, PackageEntry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PackageEntry {
    pub name: String,
    pub description: String,
    pub channels: Vec<String>,
    pub website: String,
}

#[derive(Debug)]
pub struct PackageVersion {
    pub channels: HashMap<String, Versions>,
}

pub type Versions = Vec<String>;

fn value_from_file<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();

    if let Some(name) = path.file_name() {
        let file = File::open(path)
            .map_err( |_| io_err(format!("cannot open {:?}", name)))?;

        let value = serde_json::from_reader(BufReader::new(file))
            .map_err(|_| io_err(format!("failed to parse {:?}", name)))?;

        Ok(value)
    } else {
        Err(io_err(format!("failed to get filename")).into())
    }
}

impl Packages {
    pub fn from_value(value: Value) -> Result<Packages> {
        match value {
            Value::Object(obj) => {
                let map = obj
                    .into_iter()
                    .filter_map(|(pid, entry)| 
                        Some((serde_json::from_value(Value::String(pid)).ok()?, serde_json::from_value(entry).ok()?)))
                    .collect();

                Ok(Packages { map })
            }

            _ => Err(make_err!("packages.json").into()),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Packages> {
        Packages::from_value(value_from_file(path)?)
    }

    pub fn list(&self) -> &HashMap<PackageID, PackageEntry> {
        &self.map
    }

    pub fn pretty_print(&self) -> String {
        self.map.iter().fold(String::new(), |mut acc, (id, entry)| {
            let indented = entry.pretty_print().replace("\n", "\n    ");
            acc.push_str(&format!("{}:\n    {}\n\n", id, indented));

            acc
        })
    }
}

impl PackageEntry {
    pub fn pretty_print(&self) -> String {
        format!(r#"name: {}
description: {},
channels: {:?},
website: {}"#, self.name, self.description, self.channels, self.website)
    }
}

impl PackageVersion {
    pub fn from_value(value: Value) -> Result<PackageVersion> {
        let obj = value.as_object().ok_or(make_err!("package.json"))?;
        let channels = obj
            .get("channels")
            .ok_or(make_err!("package.json", "missing \"channels\" field"))?
            .as_object()
            .ok_or(make_err!("package.json", "expected \"channels\" is an object"))?
            .iter()
            .filter_map(|(channel, value)| {
                let versions: Versions = value
                    .as_array()?
                    .iter()
                    .filter_map(|v| Some(v.as_str()?.to_string()))
                    .collect();

                Some((channel.clone(), versions))
            })
            .collect();

        Ok(PackageVersion { channels })
    }

    pub fn from_pid<P: AsRef<Path>>(pid: &PackageID, dir: P) -> Result<PackageVersion> {
        let path = PathBuf::from(dir.as_ref());
        let path = PackageID::resolve_path(pid, path);

        PackageVersion::from_value(value_from_file(path)?)
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use crate::{index::update, logger::StdioLogger, config::Config};

    use super::*;

    const PACKAGES_JSON: &'static str = r#"{"net.mamoe:mirai-console": {
        "name": "Mirai Console",
        "description": "Mirai Console 后端",
        "channels": [
            "stable",
            "nightly",
            "beta"
        ],
        "website": "https://github.com/mamoe/mirai-console"
    }}"#;

    const PACKAGE_JSON: &'static str = r#"{
        "channels": {
            "stable": [
                "1.9.6",
                "1.9.7",
                "1.9.8"
            ]
        }
    }"#;

    fn mirai_console_id() -> PackageID {
        serde_json::from_value::<PackageID>(Value::String(String::from("net.mamoe:mirai-console"))).unwrap()
    }

    #[test]
    fn parse_packages() {
        let packages = Packages::from_value(serde_json::from_str(PACKAGES_JSON).unwrap()).unwrap();
        let pid = mirai_console_id();
        let entry = &packages.list()[&pid];

        let expect = PackageEntry {
            name: "Mirai Console".to_string(),
            description: "Mirai Console 后端".to_string(),
            channels: vec!["stable", "nightly", "beta"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            website: "https://github.com/mamoe/mirai-console".to_string(),
        };

        assert_eq!(&expect, entry);
    }

    #[test]
    fn parse_package() {
        let package = PackageVersion::from_value(serde_json::from_str(PACKAGE_JSON).unwrap()).unwrap();
        let stable_channel = &package.channels["stable"];

        let expect: Versions = vec!["1.9.6", "1.9.7", "1.9.8"].into_iter().map(ToString::to_string).collect();

        assert_eq!(&expect, stable_channel);
    }

    /**
     * make sure that have updated index
     */
    #[test]
    fn read_package_info() {
        let updater = Config::default().updater::<StdioLogger>();
        let dir = updater.index_dir();
        let package = PackageVersion::from_pid(&mirai_console_id(), dir).unwrap();

        println!("{:?}", package);
    }

    #[test]
    fn deser_pid() {
        let pid = "\"net.mamoe:mirai-console\"";
        let pkgid: PackageID = serde_json::from_str(pid).unwrap();

        assert_eq!(PackageID {
            domain: String::from("net.mamoe"),
            name: String::from("mirai-console")
        }, pkgid);
    }
}
