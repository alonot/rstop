use std::{
    fs::FileType,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub struct Directory {
    pub name: Arc<String>,
    pub path: Arc<String>,
    pub no_folders: u64,
    pub no_files: u64,
    pub total_size: u64,
    pub selected: Option<Arc<Mutex<DirEntry>>>,
    pub dirents: Vec<Arc<Dirent>>,
    pub sorted_by_name: bool, // false means sorted in desc order or never sorted
    pub sorted_by_size: bool, // false means sorted in desc order or never sorted
}

#[derive(Clone)]
pub enum Dirent {
    AGGREGATE(Arc<Mutex<AGGREGATOR>>),
    VALUE(Arc<Mutex<DirEntry>>),
}

pub struct AGGREGATOR {
    pub common_name: String,
    pub sorted_by_name: bool,
    pub sorted_by_size: bool,
    pub total_size: u64,
    pub percent: f32,
    pub expanded: bool,
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
