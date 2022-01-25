use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;

#[cfg(feature = "blocking-events")]
use crate::event::{HandlerExt, Handlers};
use crate::FileRequest;

pub struct DownloaderBuilder {
    duration: u64,
    keep_alive: bool,
    chunk_size: usize,
    #[cfg(feature = "blocking-events")]
    handlers: Handlers<File>,
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        DownloaderBuilder {
            duration: 100,
            keep_alive: false,
            chunk_size: 32768,
            #[cfg(feature = "blocking-events")]
            handlers: vec![],
        }
    }
}

impl DownloaderBuilder {
    pub fn time_out(mut self, duration: u64) -> DownloaderBuilder {
        self.duration = duration;
        self
    }

    pub fn keep_alive(mut self, enable: bool) -> DownloaderBuilder {
        self.keep_alive = enable;
        self
    }

    pub fn chunk_size(mut self, size: usize) -> DownloaderBuilder {
        self.chunk_size = size;
        self
    }

    #[cfg(feature = "blocking-events")]
    pub fn handlers(mut self, handlers: Handlers<File>) -> DownloaderBuilder {
        self.handlers = handlers;
        self
    }

    pub fn build(self) -> Result<Downloader> {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(self.duration))
            .build()?;

        Ok(Downloader {
            client,
            chunk_size: self.chunk_size,
            #[cfg(feature = "blocking-events")]
            handlers: self.handlers,
        })
    }
}

#[derive(Clone)]
pub struct Downloader {
    client: Client,
    chunk_size: usize,
    #[cfg(feature = "blocking-events")]
    handlers: Handlers<File>,
}

impl Default for Downloader {
    fn default() -> Self {
        DownloaderBuilder::default()
            .build()
            .expect("Failed to create default blocking downloader")
    }
}

impl Downloader {
    pub fn builder() -> DownloaderBuilder {
        DownloaderBuilder::default()
    }

    pub fn download(&self, file_request: FileRequest) -> Result<File> {
        let mut response = self.client.get(file_request.url).send()?;
        #[cfg(feature = "blocking-events")]
        {
            let content_length = response
                .content_length()
                .context("Failed to get content_length header, can't send event!")?;
            self.handlers.on_content_length(&content_length);
        }
        let mut file = File::create(file_request.path)?;
        let mut buffer = vec![0; self.chunk_size];
        loop {
            match response.read(&mut buffer)? {
                0 => {
                    #[cfg(feature = "blocking-events")]
                    self.handlers.on_finish(&file);
                    return Ok(file);
                }
                len => {
                    #[cfg(feature = "blocking-events")]
                    self.handlers.on_write(&buffer[..len]);
                    file.write_all(&buffer[..len])?;
                }
            }
        }
    }
}
