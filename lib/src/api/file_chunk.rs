use crate::api::FileAttachment;
use anyhow::Context;
use glob::glob;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChunk {
    pub offset: usize,
    pub data: Vec<u8>,
}

pub struct AttachedFiles {
    paths: Vec<PathBuf>,
}

pub struct FileChunkResult {
    pub chunks: Vec<FileChunk>,
    pub hash: Vec<u8>,
    pub size: usize,
}

impl From<&FileChunkResult> for FileAttachment {
    fn from(files: &FileChunkResult) -> Self {
        Self {
            hash: files.hash.clone(),
            size: files.size,
        }
    }
}

impl AttachedFiles {
    pub fn from_input(input: Vec<String>) -> Self {
        let paths = {
            let mut result = Vec::new();
            for arg in input {
                let paths = glob(&arg);
                for paths in paths {
                    for path in paths.flatten() {
                        result.push(path);
                    }
                }
            }
            result
        };
        Self { paths }
    }

    fn archive(&self) -> anyhow::Result<File> {
        let file = NamedTempFile::with_suffix(".zip").context("cannot create temporary file")?;
        let mut writer = zip::ZipWriter::new(file);
        for path in &self.paths {
            let Some(file_name) = path.file_name() else {
                continue;
            };
            let file_name = file_name
                .to_str()
                .context("cannot convert file name to string")?;
            writer.start_file(String::from(file_name), SimpleFileOptions::default())?;
            let mut f = File::open(path).context("cannot open file")?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).context("cannot read file")?;
            writer.write_all(&buffer).context("cannot write file")?;
        }
        let output = writer.finish().context("cannot finish writing archive")?;
        tracing::debug!("Archive saved to {output:?}");
        Ok(output.into_file())
    }

    pub fn chunks(&self, chunk_size_bytes: usize) -> anyhow::Result<FileChunkResult> {
        let mut archive = self.archive()?;
        let mut buffer = Vec::new();
        archive
            .seek(std::io::SeekFrom::Start(0))
            .context("cannot seek archive file")?;
        archive
            .read_to_end(&mut buffer)
            .context("cannot read archive file")?;

        let hasher = md5::compute(buffer.as_slice());
        let hash = hasher.0.to_vec();

        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < buffer.len() {
            let end = std::cmp::min(offset + chunk_size_bytes, buffer.len());
            let data = buffer[offset..end].to_vec();

            chunks.push(FileChunk { offset, data });

            offset = end;
        }

        tracing::debug!(
            "Archive chunks created, got {} chunks of {} bytes total",
            chunks.len(),
            buffer.len()
        );

        Ok(FileChunkResult {
            chunks,
            hash,
            size: buffer.len(),
        })
    }
}
