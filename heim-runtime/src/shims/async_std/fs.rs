use std::io;
use std::path::Path;
use std::marker::Unpin;

use futures::io::{BufReader, AsyncBufReadExt};
// Few `heim-runtime` functions can be mapped as is to `async-std` functions
pub use async_std::fs::{self, File, DirEntry, read_link, read_to_string};

use heim_common::prelude::*;

/// Returns stream of files and directories contained in the `path` directory.
pub fn read_dir<T>(path: T) -> impl Stream<Item = io::Result<DirEntry>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    fs::read_dir(path)
        .try_flatten_stream()
}

pub fn path_exists<T>(path: T) -> impl Future<Output = bool>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    async {
        match fs::metadata(path).await {
            Ok(..) => true,
            Err(..) => false,
        }
    }
}

pub fn read_lines<T>(path: T) -> impl Stream<Item = io::Result<String>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    File::open(path)
        .map_ok(|file| BufReader::new(file).lines())
        .try_flatten_stream()
}

pub fn read_first_line<T>(path: T) -> impl Future<Output = io::Result<String>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    async {
        let lines = read_lines(path);
        futures::pin_mut!(lines);

        match lines.next().await {
            Some(Ok(line)) => Ok(line),
            Some(Err(e)) => Err(e),
            None => Err(io::Error::from(io::ErrorKind::InvalidData))
        }
    }
}
