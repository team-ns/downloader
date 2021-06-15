use anyhow::Result;
use reqwest::header::HeaderValue;

use std::cmp;
use std::path::PathBuf;

pub struct FileRequest {
    pub path: PathBuf,
    pub url: String,
    pub size: Option<u64>,
}

pub struct ChunkRange {
    start: u64,
    end: u64,
    chunk_size: u32,
}

pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn new(start: u64, end: u64) -> Self {
        ByteRange { start, end }
    }

    pub fn to_header(&self) -> HeaderValue {
        HeaderValue::from_str(&format!("bytes={}-{}", self.start, self.end))
            .expect("Failed to create content_range header")
    }
}

impl ChunkRange {
    pub fn new(file_size: u64, chunk_size: u32) -> Result<Self> {
        if chunk_size == 0 {
            return Err(anyhow::anyhow!(
                "Invalid chunk_size, value must be greater than zero!"
            ));
        }
        Ok(ChunkRange {
            start: 0,
            end: file_size - 1,
            chunk_size,
        })
    }
}

impl Iterator for ChunkRange {
    type Item = ByteRange;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += cmp::min(self.chunk_size as u64, self.end - self.start + 1);
            Some(ByteRange::new(prev_start, self.start - 1))
        }
    }
}
