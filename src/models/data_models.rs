use std::{
    fs::FileType,
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub struct Directory {
    pub name: Arc<String>,
    pub path: Arc<String>,
    pub no_folders: u64,
    pub no_files: u64,
    pub total_size: u64,
    pub selected: Option<Arc<Dirent>>,
    pub dirents: Vec<Arc<Dirent>>,
}

#[derive(Clone)]
pub enum Dirent {
    AGGREGATE(Arc<Mutex<AGGREGATOR>>),
    VALUE(Arc<Mutex<DirEntry>>),
}

pub struct AGGREGATOR {
    pub common_name: String,
    pub total_size: u64,
    pub percent: f32,
    pub dirents: Vec<Arc<Mutex<DirEntry>>>,
}

pub struct DirEntry {
    pub percent: f32,
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
    pub mode: String,
}
