use std::str::FromStr;
use std::{fs::File, io::stdout, path::Path};
use std::{io::Write, iter::FromIterator, net::TcpListener, path::PathBuf, time::Duration};
use std::convert::TryFrom;

use anyhow::Result;
use hyper::{
    body::{Buf, HttpBody},
    Client, Uri,
};
use tokio::runtime;

use crate::index::PackageID;

#[derive(Debug)]
pub struct SourceRepo {
    url: String,
}

impl SourceRepo {
    pub fn from_url<S: ToString>(url: S) -> Self {
        SourceRepo {
            url: url.to_string()
        }
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn download_url<S: AsRef<str>>(&self, pkg: PackageID, version: S, suffix: S) -> String {
        format!("{repo}/{pkg}/{version}/{name}-{version}{suffix}", 
            repo = self.url, 
            pkg = pkg.to_path_str(), 
            version = version.as_ref(),
            name = pkg.name,
            suffix = suffix.as_ref())
    }
}

#[derive(Debug)]
pub struct Downloader {
    repo: SourceRepo,
    pkg_path: PathBuf,
}

impl Downloader {
    pub fn get_download_url(&self, pkg: PackageID, ext: &'static str) -> String {
        format!("{}/{}.{}", self.repo.url, pkg.to_path_str(), ext)
    }

    /**
     * Download package into file.
     */
    pub async fn download<P: AsRef<Path>>(&self, pkg: PackageID, output: P) -> Result<()> {
        let client = Client::new();
        let url = Uri::from_str(&self.get_download_url(pkg, "jar"))?;
        let mut resp = client.get(url).await?;
        let mut file = File::open(output.as_ref())?;
        let mut received = 0usize;
        let size = resp.size_hint().exact();

        if let Some(data) = resp.data().await {
            let data = data?;

            received += data.len();
            file.write_all(&data[..])?;

            let process = if let Some(size) = size {
                let f_size = f64::from(u32::try_from(size)?);
                let f_received = f64::from(u32::try_from(received)?);
                let process = f_received / f_size * 10.0;

                process.floor() as u8
            } else {
                0
            };

            TerminalDownloadObserver::update(process, (received, usize::try_from(size.unwrap_or(0))?));
        }

        Ok(())
    }
}

pub trait DownloadObserver {
    fn ready();
    fn update(process: u8, size: (usize, usize));
}

pub struct TerminalDownloadObserver;

impl TerminalDownloadObserver {
    const PROCESS_MAX: u8 = 20;

    pub fn make_download_str(process: u8, size: (usize, usize)) -> String {
        if process > Self::PROCESS_MAX {
            panic!("WHAT ARE YOU FUCKING DOING??");
        }

        let processed_str = String::from_iter(vec!['>'; usize::from(process)]);
        let unprocessed_str = String::from_iter(vec!['<'; usize::from(Self::PROCESS_MAX - process)]);

        format!(
            "{}/{} [{}{}]",
            size.0, size.1, processed_str, unprocessed_str
        )
    }
}

impl DownloadObserver for TerminalDownloadObserver {
    fn update(process: u8, size: (usize, usize)) {
        Terminal::recover_cursor();
        Terminal::erase_line();

        print!("{}", Self::make_download_str(process, size));
    }

    fn ready() {
        Terminal::save_cursor();
    }
}

pub struct Terminal;

impl Terminal {
    pub fn save_cursor() {
        write!(stdout(), "\x1B[s").expect("io error");
        stdout().flush().expect("io error");
    }

    pub fn recover_cursor() {
        write!(stdout(), "\x1B[u").expect("io error");
        stdout().flush().expect("io error");
    }

    pub fn erase_line() {
        write!(stdout(), "\x1B[2K").expect("io error");
        stdout().flush().expect("io error");
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // #[test]
    // pub fn test_downloading() {
    //     println!("Downloading...");

    //     Terminal::save_cursor();

    //     let job = std::thread::spawn(|| {
    //         let mut process = 0;

    //         while process < 100 {
    //             Terminal::recover_cursor();
    //             Terminal::erase_line();

    //             process += 1;

    //             print!(
    //                 "{}",
    //                 Downloader::make_download_str(process / 5, (usize::from(process), 100))
    //             );
    //             std::io::stdout().flush().unwrap();

    //             std::thread::sleep(Duration::from_secs(1));
    //         }
    //     });

    //     job.join().unwrap();
    // }

    // #[tokio::test]
    // pub async fn test_fetching() {
    //     let getter = Downloader {
    //         repo: SourceRepo {
    //             url: "http://maven.aliyun.com/repository/public".to_string(),
    //         },
    //         pkg_path: PathBuf::new(),
    //     };

    //     getter
    //         .download(PackageID {
    //             domain: "net.mamoe".to_string(),
    //             name: "mirai-console".to_string(),
    //         })
    //         .await
    //         .unwrap();
    // }

    #[test]
    pub fn download_url() {
        let expected = "https://maven.aliyun.com/repository/public/net/mamoe/mirai-console/2.4.0/mirai-console-2.4.0-all.jar";
        let repo = SourceRepo::from_url("https://maven.aliyun.com/repository/public");
        let pkg = PackageID {
            domain: "net.mamoe".to_string(),
            name: "mirai-console".to_string(),
        };

        assert_eq!(expected, repo.download_url(pkg, "2.4.0", "-all.jar"));
    }

    #[test]
    pub fn do_download() {
        
    }
}
