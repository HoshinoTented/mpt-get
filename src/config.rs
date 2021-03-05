use std::path::PathBuf;

use serde::Deserialize;

use crate::{index::{MirrorRepo, Updater}, logger::Logger};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mirror_repo: String,
    pub source_repo: String,
    pub index_path: PathBuf,
    pub package_path: PathBuf,
    pub proxy: Option<String>
}

impl Config {
    pub fn mirror_repo(&self) -> MirrorRepo {
        MirrorRepo::new(&self.mirror_repo)
    }

    pub fn updater<Log: Logger>(&self) -> Updater<Log> {
        Updater::new(self.mirror_repo(), self.index_path.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut mpt_dir = dirs::home_dir().unwrap_or(PathBuf::from("~"));
        mpt_dir.push(".mpt-get");

        Config {
            mirror_repo: "http://gitee.com/peratx/mirai-repo.git".to_string(),
            source_repo: "http://maven.aliyun.com/repository/public".to_string(),
            index_path: {
                let mut index_path = mpt_dir.clone();
                index_path.push("index");
                index_path
            },
            package_path: {
                let mut package_path = mpt_dir.clone();
                package_path.push("packages");
                package_path
            },
            proxy: None,

        }
    }
}