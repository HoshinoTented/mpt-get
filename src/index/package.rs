use serde::Deserialize;
use serde_json::Value;

use std::{collections::HashMap, fs::File, io::BufReader};

use crate::error::{AsResult, Result, index_err};

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

#[derive(Debug)]
pub struct Packages {
    map: HashMap<String, PackageEntry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PackageEntry {
    name: String,
    description: String,
    channels: Vec<String>,
    website: String,
}

#[derive(Debug)]
pub struct Package {
    channels: HashMap<String, Versions>,
}

pub type Versions = Vec<String>;

impl Packages {
    pub fn from_value(value: Value) -> Result<Packages> {
        match value {
            Value::Object(obj) => {
                let map = obj
                    .into_iter()
                    .filter_map(|(id, entry)| Some((id, serde_json::from_value(entry).ok()?)))
                    .collect();

                Ok(Packages { map })
            }

            _ => Err(make_err!("packages.json")),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Packages> {
        let file = File::open(path).map_err( |_| make_err!("packages.json", "cannot open packages.json"))?;
        let value = serde_json::from_reader(BufReader::new(file)).map_err(|_| make_err!("packages.json", "cannot parse packages.json"))?;
        
        Packages::from_value(value)
    }

    pub fn list(&self) -> &HashMap<String, PackageEntry> {
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

impl Package {
    pub fn from_value(value: Value) -> Result<Package> {
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

        Ok(Package { channels })
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
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

    #[test]
    fn parse_packages() {
        let packages = Packages::from_value(serde_json::from_str(PACKAGES_JSON).unwrap()).unwrap();
        let entry = &packages.list()["net.mamoe:mirai-console"];

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
        let package = Package::from_value(serde_json::from_str(PACKAGE_JSON).unwrap()).unwrap();
        let stable_channel = &package.channels["stable"];

        let expect: Versions = vec!["1.9.6", "1.9.7", "1.9.8"].into_iter().map(ToString::to_string).collect();

        assert_eq!(&expect, stable_channel);
    }
}
