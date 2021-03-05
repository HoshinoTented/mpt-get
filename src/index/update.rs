use std::{fs::File, io::{BufReader, Write}, marker::PhantomData, path::{Path, PathBuf}, process::{ExitStatus, Stdio}};
use std::process::Command;

use git2::{Repository, ResetType};
use anyhow::Result;

use crate::logger::{Logger, StdioLogger};
use crate::error::{AsResult, index_err};
use crate::index::package::Packages;

#[derive(Debug, Clone)]
pub struct MirrorRepo {
    pub url: String,
    pub branch: String
}

impl MirrorRepo {
    pub fn new<S: ToString>(url: S) -> Self {
        MirrorRepo {
            url: url.to_string(),
            branch: String::from("master")
        }
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn branch(&self) -> &String {
        &self.branch
    }
}

#[derive(Debug)]
pub struct Updater<Log: Logger> {
    repo: MirrorRepo,
    dir: PathBuf,
    _phantom: PhantomData<Log>
}

impl <Log: Logger> Updater<Log> {
    pub fn new(repo: MirrorRepo, dir: PathBuf) -> Updater<Log> {
        Updater {
            repo,
            dir,
            _phantom: PhantomData::default()
        }
    }

    pub fn repo(&self) -> &MirrorRepo {
        &self.repo
    }

    pub fn index_dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn update(&self) -> Result<()> {
        if self.dir.exists() {
            let repo = Repository::open(&self.dir)?;
            let mut remote = repo.find_remote("origin")?;

            remote.fetch(&["master"], None, None)?;
            
            let remote_master = repo.find_reference(&format!("refs/remotes/origin/{}", self.repo.branch))?;
            let commit_oid = remote_master.target().ok_or(index_err("failed to get reference target"))?;
            let commit_object = repo.find_object(commit_oid, None)?;

            repo.reset(&commit_object, ResetType::Hard, None)?;
        } else {
            writeln!(Log::info(), "Index folder not found.")?;

            Repository::clone(&self.repo.url, &self.dir)?;
        }

        Ok(())
    }


    pub fn index(&self) -> Result<Packages> {
        let mut index = self.dir.clone();
        index.push("packages.json");

        Packages::from_file(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, logger::StdioLogger};

    use super::Updater;

    fn updater() -> Updater<StdioLogger> {
        Config::default().updater()
    }

    #[test]
    fn update() {
        let updater = updater();
        
        updater.update().unwrap();
    }

    #[test]
    fn list_pkg() {
        println!("{:?}", updater().index().unwrap());
    }
}