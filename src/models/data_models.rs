use std::{fs::FileType, ops::DerefMut, sync::{Arc, Mutex}, time::SystemTime};



pub struct Directory {
    pub name: Arc<String>,
    pub path: Arc<String>,
    pub no_folders: u64,
    pub no_files: u64,
    pub total_size: u64,
    pub dirents: Vec<Dirent>
}

pub enum Dirent {
    AGGREGATE(Arc<Mutex<AGGREGATOR>>),
    VALUE(Arc<Mutex<DirEntry>>)
}

pub struct AGGREGATOR {
    pub common_name: String,
    pub total_size: u64,
    pub dirents: Vec<Arc<Mutex<DirEntry>>>
}

pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub blksize: u64,
    pub gid: u32,
    pub dev: u64,
    pub nlink: u64,
    pub ino: u64,
    pub file_type: FileType,
    pub modified: SystemTime,
    pub created: SystemTime,
    pub accessed: SystemTime,
    pub mode: String
}