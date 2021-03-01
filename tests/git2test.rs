
use std::path::{Path, PathBuf};

use git2::{Object, Repository, ResetType};

fn dir() -> PathBuf {
    PathBuf::from(Path::new("./index"))
}

fn open() -> Repository {
    Repository::open(dir()).unwrap()
}

#[test]
fn clone() {
    Repository::clone("https://gitee.com/peratx/mirai-repo.git", dir()).unwrap();
}

#[test]
fn fetch() {
    open().find_remote("origin").unwrap()
    .fetch(&["master"], None, None).unwrap();
}

#[test]
fn update() {
    let repo = open();
    let remote_master = repo.find_reference("refs/remotes/origin/master").unwrap();
    let commit = repo.find_object(remote_master.target().unwrap(), None).unwrap();

    println!("{:?}", commit.id());
    
    repo.reset(&commit, ResetType::Hard, None).unwrap();
}