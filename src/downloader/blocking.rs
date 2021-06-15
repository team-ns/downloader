use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

pub struct DownloaderBuilder {
    duration: u64,
    keep_alive: bool,
}

impl DownloaderBuilder {
    pub fn new() -> Self {
        DownloaderBuilder {
            duration: 100,
            keep_alive: false,
        }
    }

    pub fn keep_alive(mut self, enable: bool) -> DownloaderBuilder {
        self.keep_alive = enable;
        self
    }

    pub fn time_out(mut self, duration: u64) -> DownloaderBuilder {
        self.duration = duration;
        self
    }

    pub fn build(self) -> Result<Downloader> {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(self.duration))
            .build()?;

        Ok(Downloader { client })
    }
}

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Result<Downloader> {
        DownloaderBuilder::new().build()
    }

    pub fn builder() -> DownloaderBuilder {
        DownloaderBuilder::new()
    }

    pub fn download<P: AsRef<Path>>(&self, url: String, path: P) -> Result<File> {
        let response = self.client.get(url).send()?;
        let mut file = File::create(path)?;
        file.write_all(&response.bytes()?)?;
        Ok(file)
    }
}
