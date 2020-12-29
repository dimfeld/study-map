use std::io;
use std::path::Path;
use tantivy::{
  directory::{
    error::{DeleteError, LockError, OpenReadError, OpenWriteError},
    DirectoryLock, Lock, MmapDirectory, ReadOnlySource, WatchCallback, WatchHandle, WritePtr,
  },
  Directory,
};

#[derive(Debug, Clone)]
pub struct ReadOnlyDirectoryWrapper {
  inner: MmapDirectory,
}

impl ReadOnlyDirectoryWrapper {
  pub fn new(dir: MmapDirectory) -> Self {
    ReadOnlyDirectoryWrapper { inner: dir }
  }
}

struct HasDrop;

impl Drop for HasDrop {
  fn drop(&mut self) {
    println!("Dropping!");
  }
}

impl Directory for ReadOnlyDirectoryWrapper {
  fn acquire_lock(&self, _lock: &Lock) -> Result<DirectoryLock, LockError> {
    Ok(DirectoryLock::from(Box::new(HasDrop)))
  }

  fn open_read(&self, path: &Path) -> Result<ReadOnlySource, OpenReadError> {
    MmapDirectory::open_read(&self.inner, path)
  }

  fn delete(&self, path: &Path) -> Result<(), DeleteError> {
    MmapDirectory::delete(&self.inner, path)
  }

  fn exists(&self, path: &Path) -> bool {
    MmapDirectory::exists(&self.inner, path)
  }

  fn open_write(&mut self, path: &Path) -> Result<WritePtr, OpenWriteError> {
    MmapDirectory::open_write(&mut self.inner, path)
  }

  fn atomic_read(&self, path: &Path) -> Result<Vec<u8>, OpenReadError> {
    MmapDirectory::atomic_read(&self.inner, path)
  }

  fn atomic_write(&mut self, path: &Path, data: &[u8]) -> io::Result<()> {
    MmapDirectory::atomic_write(&mut self.inner, path, data)
  }

  fn watch(&self, watch_callback: WatchCallback) -> tantivy::Result<WatchHandle> {
    MmapDirectory::watch(&self.inner, watch_callback)
  }
}
