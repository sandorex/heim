use std::fs;
use std::io::{self, BufRead as _};
use std::marker::Unpin;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::io::{RawHandle, AsRawHandle};

use super::THREAD_POOL;
use heim_common::prelude::*;

#[derive(Debug)]
pub struct File(fs::File);

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl File {
    pub fn open<T>(path: T) -> impl Future<Output = Result<File>>
    where
        T: AsRef<Path> + Send + Unpin + 'static,
    {
        THREAD_POOL.spawn(|| fs::File::open(path).map(File).map_err(Error::from))
    }

    #[cfg(target_os = "windows")]
    pub fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

#[derive(Debug)]
pub struct DirEntry(fs::DirEntry);

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.0.path()
    }

    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }
}

pub fn path_exists<T>(path: T) -> impl Future<Output = bool>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    THREAD_POOL.spawn(move || path.as_ref().exists())
}

pub fn read_to_string<T>(path: T) -> impl Future<Output = Result<String>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    THREAD_POOL.spawn(move || fs::read_to_string(path).map_err(Error::from))
}

pub fn read_into<T, R, E>(path: T) -> impl Future<Output = Result<R>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
    R: FromStr<Err = E>,
    Error: From<E>,
{
    read_to_string(path)
        .and_then(|content| future::ready(R::from_str(&content).map_err(Error::from)))
}

pub fn read_lines<T>(path: T) -> impl Stream<Item = Result<String>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    File::open(path)
        .map_ok(|file| {
            let reader = io::BufReader::new(file);
            stream::iter(reader.lines()).map_err(Error::from)
        })
        .try_flatten_stream()
}

pub fn read_lines_into<T, R, E>(path: T) -> impl Stream<Item = Result<R>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
    R: FromStr<Err = E>,
    Error: From<E>,
{
    read_lines(path).into_stream().then(|result| {
        let res = result.and_then(|line| R::from_str(&line).map_err(Error::from));

        future::ready(res)
    })
}

pub fn read_first_line<T>(path: T) -> impl Future<Output = Result<String>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    // TODO: Looks dumb
    read_lines(path)
        .into_stream()
        .into_future()
        .map(|(try_line, _)| match try_line {
            Some(Ok(line)) => Ok(line),
            Some(Err(e)) => Err(e),
            None => Err(Error::missing_entity("line")),
        })
}

pub fn read_dir<T>(path: T) -> impl Stream<Item = Result<DirEntry>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    THREAD_POOL
        .spawn(move || fs::read_dir(path))
        .map_err(Error::from)
        .map_ok(|iter| stream::iter(iter).map_err(Error::from))
        .try_flatten_stream()
        .map_ok(DirEntry)
}

pub fn read_link<T>(path: T) -> impl Future<Output = Result<PathBuf>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
{
    THREAD_POOL.spawn(move || fs::read_link(path).map_err(Error::from))
}
