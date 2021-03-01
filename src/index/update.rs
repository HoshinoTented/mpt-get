use std::{fs::File, io::{BufReader, Write}, marker::PhantomData, path::{Path, PathBuf}, process::{ExitStatus, Stdio}};
use std::process::Command;

use git2::{Repository, ResetType};

use crate::logger::{Logger, StdioLogger};
use crate::error::{AsResult, Result, index_err};
use crate::index::package::Packages;

#[derive(Debug)]
pub struct MirrorRepo {
    url: String,
    branch: String
}

impl Default for MirrorRepo {
    fn default() -> Self {
        MirrorRepo {
            url: "https://gitee.com/peratx/mirai-repo.git".to_string(),
            branch: "master".to_string()
        }
    }
}

#[derive(Debug)]
pub struct Updater<Log: Logger> {
    repo: MirrorRepo,
    dir: PathBuf,
    _phantom: PhantomData<Log>
}

fn find_git() -> Result<bool> {
    let output = git().output().as_index_err("failed to execute command \"git\"")?;       // TODO: read from config

    Ok(output.status.success())
}

fn git() -> Command {
    Command::new("git")
}

impl <Log: Logger> Updater<Log> {
    pub fn default() -> Option<Updater<StdioLogger>> {
        let mut home_dir = dirs::home_dir()?;

        home_dir.push(".mpt-get");
        home_dir.push("index");

        Some(Updater {
            repo: MirrorRepo::default(),
            dir: home_dir,
            _phantom: PhantomData::default()
        })
    }

    pub fn new(repo: MirrorRepo, dir: PathBuf) -> Updater<StdioLogger> {
        Updater {
            repo,
            dir,
            _phantom: PhantomData::default()
        }
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


    pub fn list(&self) -> Result<Packages> {
        let mut index = self.dir.clone();
        index.push("packages.json");

        Packages::from_file(index)
    }
}

#[cfg(test)]
mod tests {
    use std::process::{Stdio};

    use crate::logger::StdioLogger;

    use super::Updater;

    fn updater() -> Updater<StdioLogger> {
        Updater::<StdioLogger>::default().unwrap()
    }

    #[test]
    fn update() {
        let updater = Updater::<StdioLogger>::default().unwrap();
        
        updater.update().unwrap();
    }

    #[test]
    fn list_pkg() {
        println!("{:?}", updater().list().unwrap());
    }
}