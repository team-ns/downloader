use futures::{stream, StreamExt};

use crate::downloader;
use crate::downloader::blocking;
use crate::FileRequest;

pub struct Manager {
    blocking: Option<blocking::Downloader>,
    concurrent: Option<downloader::Downloader>,
    small_size: u64,
    blocking_buffer_size: usize,
}

type Result = (
    Vec<anyhow::Result<std::fs::File>>,
    Vec<anyhow::Result<tokio::fs::File>>,
);

impl Manager {
    pub fn new(small_size: u64, blocking_buffer_size: usize) -> Self {
        Manager {
            blocking: None,
            concurrent: None,
            small_size,
            blocking_buffer_size,
        }
    }

    pub async fn download(&self, files: Vec<FileRequest>) -> Result {
        let blocking_downloader = &self.blocking.clone().unwrap_or_default();
        let concurrent_downloader = self.concurrent.clone().unwrap_or_default();
        let (blocking, concurrent) = files.into_iter().partition::<Vec<FileRequest>, _>(|file| {
            if let Some(size) = file.size {
                size <= self.small_size
            } else {
                false
            }
        });
        let blocking_files = stream::iter(blocking)
            .map(|file| async { blocking_downloader.download(file) })
            .buffer_unordered(self.blocking_buffer_size)
            .collect()
            .await;

        let concurrent_files = stream::iter(concurrent)
            .then(|file| concurrent_downloader.concurrent_download(file))
            .collect()
            .await;
        (blocking_files, concurrent_files)
    }
}
