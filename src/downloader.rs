#[cfg(feature = "blocking")]
pub mod blocking;

use anyhow::{Context, Result};
use futures::{stream, StreamExt, TryStreamExt};
use reqwest::{header, Client};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

use crate::client;
#[cfg(feature = "events")]
use crate::event::{HandlerExt, Handlers};
use crate::file::{ChunkRange, FileRequest};

pub struct DownloaderBuilder {
    chunk_size: u32,
    buffer_size: usize,
    keep_alive: bool,
    #[cfg(feature = "events")]
    handlers: Handlers<File>,
}

impl DownloaderBuilder {
    pub fn new() -> Self {
        DownloaderBuilder {
            chunk_size: 512000,
            buffer_size: 150,
            keep_alive: false,
            #[cfg(feature = "events")]
            handlers: vec![],
        }
    }

    pub fn chunk_size(mut self, size: u32) -> DownloaderBuilder {
        self.chunk_size = size;
        self
    }

    pub fn buffer_size(mut self, size: usize) -> DownloaderBuilder {
        self.buffer_size = size;
        self
    }

    pub fn keep_alive(mut self, enable: bool) -> DownloaderBuilder {
        self.keep_alive = enable;
        self
    }

    #[cfg(feature = "events")]
    pub fn handlers(mut self, handlers: Handlers<File>) -> DownloaderBuilder {
        self.handlers = handlers;
        self
    }

    pub fn build(self) -> Result<Downloader> {
        let client = if self.keep_alive {
            Some(client::get_keep_alive_client()?)
        } else {
            None
        };
        Ok(Downloader {
            client,
            chunk_size: self.chunk_size,
            buffer_size: self.buffer_size,
            #[cfg(feature = "events")]
            handlers: self.handlers,
        })
    }
}

pub struct Downloader {
    client: Option<Client>,
    chunk_size: u32,
    buffer_size: usize,
    #[cfg(feature = "events")]
    handlers: Handlers<File>,
}

impl Downloader {
    pub fn new() -> Result<Downloader> {
        DownloaderBuilder::new().build()
    }

    pub fn builder() -> DownloaderBuilder {
        DownloaderBuilder::new()
    }

    pub async fn concurrent_download(&self, file_request: FileRequest) -> Result<File> {
        let client = &self
            .client
            .clone()
            .unwrap_or(client::get_keep_alive_client()?);
        let url = &file_request.url;
        let file_size = file_request.size.unwrap_or(
            client
                .get(url)
                .send()
                .await?
                .content_length()
                .context("Failed to get content_length header, can't download file!")?,
        );
        #[cfg(feature = "events")]
        self.handlers.on_content_length(&file_size);
        let mut file = File::create(file_request.path).await?;
        let chunk_range = ChunkRange::new(file_size, self.chunk_size)?;
        let mut stream = stream::iter(chunk_range)
            .map(|range| async move {
                let response = client
                    .get(url)
                    .header(header::RANGE, range.to_header())
                    .send()
                    .await?;
                if response.status().is_success() {
                    Ok((range.start, response.bytes().await?))
                } else {
                    Err(anyhow::anyhow!(
                        "Can't download file, status code: {}",
                        response.status()
                    ))
                }
            })
            .buffer_unordered(self.buffer_size);
        while let Some((offset, bytes)) = stream.try_next().await? {
            #[cfg(feature = "events")]
            self.handlers.on_write(&bytes);
            file.seek(SeekFrom::Start(offset)).await?;
            file.write_all(&bytes).await?;
        }
        #[cfg(feature = "events")]
        self.handlers.on_finish(&file);
        Ok(file)
    }
}
