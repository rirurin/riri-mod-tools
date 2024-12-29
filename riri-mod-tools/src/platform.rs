use walkdir::DirEntry;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_family = "unix")]
pub struct Timestamp(i64);

#[cfg(target_family = "windows")]
pub struct Timestamp(u64);

#[cfg(target_family = "windows")]
impl Timestamp {
    pub fn from_buffer<T: AsRef<[u8]>>(b: T) -> Timestamp {
        Timestamp(u64::from_le_bytes(b.as_ref()[..::std::mem::size_of::<u64>()].try_into().unwrap()))
    }
    pub fn check_edited_later(&self, f: &DirEntry) -> bool {
        f.metadata().unwrap().last_access_time() > self.0
    }
}

#[cfg(target_family = "unix")]
impl Timestamp {
    pub fn from_buffer<T: AsRef<[u8]>>(b: T) -> Timestamp {
        Timestamp(i64::from_le_bytes(b.as_ref()[..::std::mem::size_of::<i64>()].try_into().unwrap()))
    }
    pub fn check_edited_later(&self, f: &DirEntry) -> bool {
        f.metadata().unwrap().mtime() > self.0
    }
}
