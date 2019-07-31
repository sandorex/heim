use std::marker::Unpin;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::ffi::OsString;

use tokio::fs;
use tokio_fs::DirEntry as TokioDirEntry;

#[cfg(target_os = "windows")]
use std::os::windows::io::{RawHandle, AsRawHandle};

use heim_common::prelude::*;

#[derive(Debug)]
pub struct File(fs::File);

impl File {
    pub fn open<T>(path: T) -> impl Future<Output = Result<File>> where T: AsRef<Path> + Send + Unpin + 'static{
        fs::File::open(path)
            .map_err(Error::from)
            .map_ok(File)
    }

    #[cfg(target_os = "windows")]
    pub fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

#[derive(Debug)]
pub struct DirEntry(TokioDirEntry);

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.0.path()
    }

    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }
}

pub fn path_exists<T>(path: T) -> impl Future<Output = bool> where T: AsRef<Path> + Send + Unpin + 'static{
    fs::metadata(path)
        .map(|res| {
            match res {
                Ok(..) => true,
                Err(..) => false,
            }
        })
}

pub fn read_to_string<T>(path: T) -> impl Future<Output = Result<String>> where T: AsRef<Path> + Send + Unpin + 'static{
    fs::read(path)
        .map_err(Error::from)
        .and_then(|bytes| {
            future::ready(String::from_utf8(bytes).map_err(Error::from))
        })
}

pub fn read_into<T, R, E>(path: T) -> impl Future<Output = Result<R>>
where
    T: AsRef<Path> + Send + Unpin + 'static,
    R: FromStr<Err = E>,
    Error: From<E>,
{
    read_to_string(path)
        .and_then(|content| {
            future::ready(R::from_str(&content).map_err(Error::from))
        })
}

pub fn read_lines<T>(path: T) -> impl TryStream<Ok = String, Error = Error> where T: AsRef<Path> + Send + Unpin + 'static{
    read_to_string(path)
        // TODO: Dumb ass implementation, because tokio' `AsyncBufReadExt` is not implemented for File yet
        // https://github.com/tokio-rs/tokio/issues/1256
        .map_ok(|contents| {
            let iter = contents.lines()
                .map(|s| Ok(s.to_string()))
                .collect::<Vec<_>>();

            stream::iter(iter)
        })
        .try_flatten_stream()
        .map_err(Error::from)
}

pub fn read_lines_into<T, R, E>(path: T) -> impl TryStream<Ok = R, Error = Error>
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

pub fn read_first_line<T>(path: T) -> impl TryFuture<Ok = String, Error = Error> where T: AsRef<Path> + Send + Unpin + 'static{
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

pub fn read_dir<T>(path: T) -> impl TryStream<Ok = DirEntry, Error = Error> where T: AsRef<Path> + Send + Unpin + 'static {
    fs::read_dir(path)
        .try_flatten_stream()
        .map_err(Error::from)
        .map_ok(DirEntry)
}
